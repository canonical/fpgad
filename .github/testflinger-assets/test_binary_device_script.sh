#!/usr/bin/env bash

set -euxo pipefail

echo "INFO: Installing dependencies"
sudo env DEBIAN_FRONTEND=noninteractive apt update && sudo env DEBIAN_FRONTEND=noninteractive apt install build-essential -y
# Install the dfx-mgr release that the fpgad snap's dfx-mgr component is built from,
# so CI tests the same client the snap ships. The key is fetched by fingerprint and
# checked before it is trusted (same key as package-repositories in snap/snapcraft.yaml).
DFX_MGR_PPA_KEY_FPR="803DDF595EA7B6644F9B96B752150A179A9E84C9"
DFX_MGR_PPA_VERSION="2026.1-0ubuntu1~xlnx2"
sudo env DEBIAN_FRONTEND=noninteractive apt install gnupg curl -y
curl -fsSL "https://keyserver.ubuntu.com/pks/lookup?op=get&search=0x${DFX_MGR_PPA_KEY_FPR}" -o /tmp/dfx-mgr-ppa.asc
gpg --batch --with-colons --show-keys /tmp/dfx-mgr-ppa.asc | grep -q "^fpr:::::::::${DFX_MGR_PPA_KEY_FPR}:$"
gpg --batch --yes --dearmor -o /tmp/dfx-mgr-ppa.gpg /tmp/dfx-mgr-ppa.asc
sudo install -m 644 /tmp/dfx-mgr-ppa.gpg /usr/share/keyrings/ubuntu-xilinx-default.gpg
echo "deb [signed-by=/usr/share/keyrings/ubuntu-xilinx-default.gpg] https://ppa.launchpadcontent.net/ubuntu-xilinx/default/ubuntu stonking main" \
  | sudo tee /etc/apt/sources.list.d/ubuntu-xilinx-default.list > /dev/null
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
