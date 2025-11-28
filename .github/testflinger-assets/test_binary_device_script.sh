#!/usr/bin/env bash

set -x

echo "INFO: Installing dependencies"
sudo env DEBIAN_FRONTEND=noninteractive apt update && sudo env DEBIAN_FRONTEND=noninteractive apt install build-essential -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov


echo "INFO: extracting tarball"
tar -xzvf fpgad.gz

echo "INFO: Running test script"
cd fpgad
mkdir -p artifacts
./tests/coverage_test.sh 2>&1 | tee artifacts/coverage_test.log
