#!/usr/bin/bash

# This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
#
# Copyright 2025 Canonical Ltd.
#
# SPDX-License-Identifier: GPL-3.0-only
#
# fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
#
# fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

set -xeu

# Enable llvm-coverage build results
eval "$(cargo llvm-cov show-env --sh)"

export RUSTFLAGS="${RUSTFLAGS:-} -C llvm-args=-runtime-counter-relocation"

# build the daemon only
cargo build --bin fpgad

# build the cli only
cargo build --bin fpgad_cli

set +e

# run daemon unit tests
echo "Running daemon unit tests..."
cargo test -p fpgad --lib
DAEMON_UNIT_EXIT=$?

# run cli unit tests
echo "Running CLI unit tests..."
cargo test -p fpgad_cli --lib
CLI_UNIT_EXIT=$?

set -e

# build the test binaries avoiding cli as well. Also extract the names of the integration test binaries
xilinx_sys_test="$(\
cargo test --no-run -p fpgad --test xilinx_sys 2>&1 |\
  grep 'tests/xilinx_sys.rs' |\
  awk '{gsub(/[()]/,""); print $3}'\
)"
echo "xilinx_sys test binary: $xilinx_sys_test"

xilinx_dfx_mgr_test="$(\
cargo test --no-run -p fpgad --test xilinx_dfx_mgr 2>&1 |\
  grep 'tests/xilinx_dfx_mgr.rs' |\
  awk '{gsub(/[()]/,""); print $3}'\
)"
echo "xilinx_dfx_mgr test binary: $xilinx_dfx_mgr_test"

daemon_bin=${CARGO_LLVM_COV_TARGET_DIR}/debug/fpgad

# Kill any leftover processes or other daemon instances (this will not stop the snap version from spawning due to 'activates-on:`

sudo pkill --signal TERM fpgad || true

sudo cp ./daemon/tests/test_data/com.canonical.fpgad.conf /etc/dbus-1/system.d/com.canonical.fpgad.conf
sudo systemctl reload dbus || sudo systemctl start dbus

mkdir -p artifacts

# Save the current timestamp for later log retrieval using `journalctl --since`
TEST_START_TIME=$(date '+%Y-%m-%d %H:%M:%S')

# Function to cleanup daemon
cleanup_daemon() {
    echo "Cleaning up daemon (PID: ${DAEMON_PID:-unknown})..."
    if [ -n "${DAEMON_PID:-}" ]; then
        # Kill the daemon gracefully with SIGTERM to allow coverage data to flush
        sudo kill -TERM $DAEMON_PID 2>/dev/null || true

        # Wait up to 5 seconds for graceful shutdown, then force kill if needed
        timeout 5 bash -c "while kill -0 $DAEMON_PID 2>/dev/null; do sleep 0.1; done" 2>/dev/null || sudo kill -KILL $DAEMON_PID 2>/dev/null || true
        wait $DAEMON_PID 2>/dev/null || true
    fi

    # Fetch daemon logs from journalctl since test start
    if [ -n "${TEST_START_TIME:-}" ]; then
        sudo journalctl --since "$TEST_START_TIME" -u "fpgad*" > artifacts/daemon.log 2>/dev/null || true
    fi
}

# Run the daemon with continuous log tracing
sudo -E env RUST_LOG=trace LLVM_PROFILE_FILE="${CARGO_LLVM_COV_TARGET_DIR}/daemon-%p%c.profraw" "$daemon_bin" &
DAEMON_PID=$!

# Wait up to 5 seconds for the daemon to be ready (check via dbus)
if ! timeout 5 bash -c '
  while ! busctl status com.canonical.fpgad &>/dev/null; do
    sleep 0.1
  done
'; then
    echo "ERROR: Daemon failed to start within 5 seconds"
    cleanup_daemon
    exit 1
fi

echo "Daemon started successfully (PID: $DAEMON_PID)"

# Temporarily disable exit-on-error to capture test results even on failure
set +e

# run the xilinx_sys test binary
echo "Running xilinx_sys tests..."
timeout 300 sudo env LLVM_PROFILE_FILE="${CARGO_LLVM_COV_TARGET_DIR}/test_xilinx_sys-%p.profraw" "$xilinx_sys_test" --test-threads=1 --no-capture < /dev/null
UNIVERSAL_EXIT=$?

# run the xilinx_dfx_mgr test binary
echo "Running xilinx_dfx_mgr tests..."
timeout 300 sudo env LLVM_PROFILE_FILE="${CARGO_LLVM_COV_TARGET_DIR}/test_xilinx_dfx_mgr-%p.profraw" "$xilinx_dfx_mgr_test" --test-threads=1 --no-capture < /dev/null
DFX_MGR_EXIT=$?

# Re-enable exit-on-error
set -e

# Cleanup will be called by trap, but we call it explicitly here to ensure it happens before generating reports
cleanup_daemon

# create a summary and full log
cargo llvm-cov report &> artifacts/summary.txt
cargo llvm-cov report --lcov --output-path artifacts/coverage.lcov

# Exit with failure if any test suite failed
if [ $DAEMON_UNIT_EXIT -ne 0 ] || [ $CLI_UNIT_EXIT -ne 0 ] || [ $UNIVERSAL_EXIT -ne 0 ] || [ $DFX_MGR_EXIT -ne 0 ]; then
    echo "Tests failed: daemon_unit=$DAEMON_UNIT_EXIT, cli_unit=$CLI_UNIT_EXIT, xilinx_sys=$UNIVERSAL_EXIT, dfx_mgr=$DFX_MGR_EXIT"
    exit 1
fi

echo "All tests passed!"

