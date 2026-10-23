#!/usr/bin/env bash

set -euxo pipefail

echo "INFO: Installing dependencies"
sudo env DEBIAN_FRONTEND=noninteractive apt update && sudo env DEBIAN_FRONTEND=noninteractive apt install build-essential -y
# Install the dfx-mgr release that the fpgad snap's dfx-mgr component is built from,
# so CI tests the same client the snap ships. The PPA's noble series still carries an
# older git snapshot, so point the source at the resolute series instead.
DFX_MGR_PPA_VERSION="2026.1-0ubuntu1~xlnx2"
sudo env DEBIAN_FRONTEND=noninteractive apt install software-properties-common -y
sudo add-apt-repository -y --no-update ppa:ubuntu-xilinx/default
sudo sed -i 's/^Suites: noble$/Suites: resolute/' /etc/apt/sources.list.d/ubuntu-xilinx-ubuntu-default-noble.sources
sudo env DEBIAN_FRONTEND=noninteractive apt update
sudo env DEBIAN_FRONTEND=noninteractive apt install -y "dfx-mgr=${DFX_MGR_PPA_VERSION}" "libdfx-mgr2=${DFX_MGR_PPA_VERSION}"
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov


echo "INFO: extracting tarball"
tar -xzvf fpgad.gz

echo "INFO: Running test script"
cd fpgad
mkdir -p artifacts
sudo journalctl -f -n0 > artifacts/journal.log 2>&1 &
JOURNAL_PID=$!

# Temporarily disable -e to capture the test exit status even on failure
set +e
./tests/coverage_test.sh 2>&1 | tee artifacts/coverage_test.log
TEST_EXIT_STATUS=${PIPESTATUS[0]}
set -e

# Cleanup regardless of test result
sudo kill ${JOURNAL_PID} || true

# Exit with the test's exit status
exit "$TEST_EXIT_STATUS"
