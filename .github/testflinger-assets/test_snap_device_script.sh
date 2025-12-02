#!/usr/bin/env bash
set -euxo pipefail
#echo "INFO: Installing python dependencies"
#sudo apt update
#sudo apt install python3-

# TODO: move the test_data files around?
echo "INFO: Preparing device"
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
echo "    --- Installing fpgad.snap"
sudo snap install ./fpgad.snap --dangerous
echo "    --- Installing provider snap(s)"
# TODO: detection logic per device? - spread?
echo "    --- Installing k26-default-bitstreams snap"
sudo snap install k26-default-bitstreams --edge # TODO: change track?
echo "INFO: Done preparing device"

echo "INFO: Making necessary connections"
echo "    --- connecting to fpgad interface"
sudo snap connect fpgad:fpga
echo "    --- connecting to hardware-observe interface"
sudo snap connect fpgad:hardware-observe
echo "    --- connecting to kernel-firmware-control interface"
sudo snap connect fpgad:kernel-firmware-control
echo "    --- connecting to device-tree-overlays interface"
sudo snap connect fpgad:device-tree-overlays
echo "    --- connecting dbus interfaces"
sudo snap connect fpgad:cli-dbus fpgad:daemon-dbus
# sudo snap connect fpgad:kernel-module-control # in future
echo "INFO: Done making necessary connections"

echo "INFO: Running snap test script"
# NOTE: tarball contains "k24-starter-kits/..." and "k26-starter-kits/..." at tarball root from daemon/tests/test_data
mkdir -p fpgad/artifacts
tar -xzvf test_data.gz -C fpgad
sudo python3 -m unittest ./snap_tests.py 2>&1 | tee fpgad/artifacts/snap_test.log
echo "INFO: Done running snap test script"
