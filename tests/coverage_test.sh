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

set -xu

# Enable llvm-coverage build results
eval "$(cargo llvm-cov show-env --export-prefix)"

export RUSTFLAGS="$RUSTFLAGS -C llvm-args=-runtime-counter-relocation"

# build the daemon only, to avoid getting coverage for cli (no tests written)
cargo build --bin daemon
# build the test binaries avoiding cli as well. Also extract the name of the integration test binary
universal_test="$(\
cargo test --no-run --bin daemon --test universal 2>&1 |\
  grep 'tests/universal.rs' |\
  awk '{gsub(/[()]/,""); print $3}'\
)"
echo "universal test binary: $universal_test"

# only run daemon unit tests
cargo test --bin daemon


daemon_bin=${CARGO_LLVM_COV_TARGET_DIR}/debug/daemon

# Kill any leftover processes or other daemon instances (this will not stop the snap version from spawning due to 'activates-on:`

sudo pkill daemon || true

sudo cp ./daemon/tests/test_data/com.canonical.fpgad.conf /etc/dbus-1/system.d/com.canonical.fpgad.conf
sudo systemctl reload dbus || sudo systemctl start dbus

mkdir -p artifacts
# Run the daemon with continuous log tracing
sudo -E env RUST_LOG=trace LLVM_PROFILE_FILE="${CARGO_LLVM_COV_TARGET_DIR}/daemon-%p%c.profraw" "$daemon_bin" &> artifacts/daemon.log &
DAEMON_PID=$!

# Wait up to 5 seconds for the daemon to print that it is ready
timeout 5 bash -c '
  while ! grep -q "Started com.canonical.fpgad dbus service" artifacts/daemon.log; do
    sleep 0.1
  done
'

# run the test binary
sudo env LLVM_PROFILE_FILE="${CARGO_LLVM_COV_TARGET_DIR}/test_universal-%p.profraw" "$universal_test" --test-threads=1 --no-capture


# kill the daemon
sudo kill $DAEMON_PID || true
wait $DAEMON_PID || true


# create a summary and full log
cargo llvm-cov report &> artifacts/summary.txt
cargo llvm-cov report --lcov --output-path artifacts/coverage.lcov
