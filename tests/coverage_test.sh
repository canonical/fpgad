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
eval "$(cargo llvm-cov show-env --export-prefix)"

export RUSTFLAGS="$RUSTFLAGS -C llvm-args=-runtime-counter-relocation"

# build the daemon only, to avoid getting coverage for cli (no tests written)
cargo build --bin fpgad
# build the test binaries avoiding cli as well
cargo test --bin fpgad --test integration_tests --no-run
# only run fpgad unit tests
cargo test --bin fpgad

# extract the name of the integration test binary
integration_tests="$(\
cargo test --no-run --test integration_tests 2>&1 |\
  grep 'tests/integration_tests' |\
  awk '{gsub(/[()]/,""); print $3}'\
)"
daemon_bin=${CARGO_LLVM_COV_TARGET_DIR}/debug/fpgad

# Kill any leftover processes or other daemon instances (this will not stop the snap version from spawning due to 'activates-on:`

sudo pkill fpgad || true

sudo cp ./daemon/tests/test_data/com.canonical.fpgad.conf /etc/dbus-1/system.d/com.canonical.fpgad.conf
sudo systemctl reload dbus

mkdir -p artifacts
# Run the daemon with continuous log tracing
sudo -E env RUST_LOG=trace LLVM_PROFILE_FILE="${CARGO_LLVM_COV_TARGET_DIR}/fpgad-%p%c.profraw" "$daemon_bin" &> artifacts/daemon.log &
DAEMON_PID=$!

# Wait up to 5 seconds for the daemon to print that it is ready
timeout 5 bash -c '
  while ! grep -q "Started com.canonical.fpgad dbus service" artifacts/daemon.log; do
    sleep 0.1
  done
'

# run the test binary
sudo env LLVM_PROFILE_FILE="${CARGO_LLVM_COV_TARGET_DIR}/test_universal-%p.profraw" "$integration_tests" --test-threads=1 --no-capture


# kill the daemon
sudo kill $DAEMON_PID || true
wait $DAEMON_PID || true


# create a summary and full log
cargo llvm-cov report &> artifacts/summary.txt
cargo llvm-cov report --text &> artifacts/full_report.txt
