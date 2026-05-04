#!/usr/bin/env bash
set -euxo pipefail

echo "INFO: Preparing device"
echo "    --- Updating with apt"
sudo env DEBIAN_FRONTEND=noninteractive apt update && sudo env DEBIAN_FRONTEND=noninteractive apt install build-essential -y
echo "    --- Removing fpgad that was installed from snap store"
sudo snap remove fpgad --purge
echo "    --- Removing dfx-mgr"
sudo apt remove -y dfx-mgr || true
echo "    --- Ensuring snapd is not too busy"
# Otherwise attempting to make connections will error due to, e.g., `error: snap "snapd" has "auto-refresh" change in progress`
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done
echo "    --- Disabling auto-refresh for 24 hours"
sudo snap refresh --hold=24h
echo "    --- Extracting snap package"
if [[ ! -f ./fpgad-snap-package.tar.gz ]]; then
    echo "ERROR: Snap package tarball not found"
    exit 1
fi
tar -xzvf fpgad-snap-package.tar.gz
echo "    --- Listing extracted snap and component files"
echo "Snap files:"
find . -maxdepth 1 -name "*.snap" -type f
echo ""
echo "Component files:"
find . -maxdepth 1 -name "*.comp" -type f
echo ""
echo "    --- Installing fpgad snap"
SNAP_FILE=$(find . -maxdepth 1 -name "*.snap" -type f | head -n 1)
if [[ -z "$SNAP_FILE" || ! -f "$SNAP_FILE" ]]; then
    echo "ERROR: Snap file not found in tarball"
    exit 1
fi
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done
sudo snap install "$SNAP_FILE" --dangerous
echo "    --- Installing snap components"
COMP_FILES=$(find . -maxdepth 1 -name "*.comp" -type f)
if [[ -z "$COMP_FILES" ]]; then
    echo "ERROR: No component files found in tarball - build may have failed"
    exit 1
fi
for COMP_FILE in $COMP_FILES; do
    echo "    --- Installing component: $COMP_FILE"
    while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
        echo "    --- snapd internal tasks still running... waiting..."
        sleep 10
    done
    sudo snap install --dangerous "$COMP_FILE"
    echo "    --- Component installed successfully: $COMP_FILE"
done
echo "    --- Installing provider snap(s)"
echo "INFO: Done preparing device"

echo "INFO: Making necessary connections"
echo "    --- connecting to fpgad interface"
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done
sudo snap connect fpgad:fpga
echo "    --- connecting to hardware-observe interface"
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done
sudo snap connect fpgad:hardware-observe
echo "    --- connecting to kernel-firmware-control interface"
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done
sudo snap connect fpgad:kernel-firmware-control
echo "    --- connecting to device-tree-overlays interface"
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done
sudo snap connect fpgad:device-tree-overlays
echo "    --- connecting dfx-mgr-socket interface"
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done
sudo snap connect fpgad:dfx-mgr-socket
echo "    --- connecting dbus interfaces"
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done
sudo snap connect fpgad:cli-dbus fpgad:daemon-dbus
# sudo snap connect fpgad:kernel-module-control # in future
echo "INFO: Done making necessary connections"

echo "INFO: Running snap test script"
# NOTE: test_data.gz contains "k24-starter-kits/..." and "k26-starter-kits/..." at tarball root from daemon/tests/test_data
# NOTE: tests.gz contains the test structure (common/, test_universal/, test_xlnx/, test_default/, etc.)
mkdir -p fpgad/artifacts
echo "    --- Extracting test data"
tar -xzvf test_data.gz -C fpgad
echo "    --- Extracting tests"
mkdir -p tests
tar -xzvf tests.gz -C tests
echo "    --- Saving timestamp for journal log retrieval"
TEST_START_TIME=$(date '+%Y-%m-%d %H:%M:%S')

# Disable exit-on-error temporarily to capture logs even on test failure
set +e
echo "    --- Running tests with unittest discovery"
sudo tests/snap_testing/test_snap.sh 2>&1 | tee fpgad/artifacts/snap_test.log
TEST_EXIT_CODE=$?
# Re-enable exit-on-error
set -e

echo "    --- Collecting journal logs since test start"
sudo journalctl --since "$TEST_START_TIME" -u "snap.fpgad*" 2>&1 | tee fpgad/artifacts/journal.log || true
echo "INFO: Done running snap test script"

# Exit with the test exit code to signal success/failure
exit $TEST_EXIT_CODE

