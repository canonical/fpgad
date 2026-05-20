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
echo "    --- Installing fpgad"
while sudo snap debug state /var/lib/snapd/state.json | grep -qE 'Doing|Undoing|Waiting'; do
    echo "    --- snapd internal tasks still running... waiting..."
    sleep 10
done

if [[ "${SNAP_TEST_SOURCE}" == "local" ]]; then
    if [[ ! -f ./fpgad.snap ]]; then
        echo "ERROR: SNAP_TEST_SOURCE=local but ./fpgad.snap is not present"
        exit 1
    fi
    echo "    --- SNAP_TEST_SOURCE=local, installing ./fpgad.snap --dangerous"
    sudo snap install ./fpgad.snap --dangerous
else
    echo "    --- SNAP_TEST_SOURCE=${SNAP_TEST_SOURCE}, installing from store channel: ${SNAP_TEST_SOURCE}"
    sudo snap install fpgad --channel="${SNAP_TEST_SOURCE}"
fi
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

