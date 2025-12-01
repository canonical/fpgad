#!/usr/bin/env bash
set -euxo pipefail
#echo "INFO: Installing python dependencies"
#sudo apt update
#sudo apt install python3-

# TODO: move the test_data files around?

echo "INFO: Installing fpgad.snap"
sudo snap install ./fpgad.snap --dangerous

echo "INFO: Installing provider snap(s)"
# TODO: detection logic per device? - spread?
echo "INFO: Installing k26-default-bitstreams snap"
sudo snap install k26-default-bitstreams --edge # TODO: change track?


echo "INFO: Making necessary connections"
echo "INFO: connecting to fpgad interface"
sudo snap connect fpgad:fpga
echo "INFO: connecting to hardware-observe interface"
sudo snap connect fpgad:hardware-observe
echo "INFO: connecting to kernel-firmware-control interface"
sudo snap connect fpgad:kernel-firmware-control
echo "INFO: connecting to device-tree-overlays interface"
sudo snap connect fpgad:device-tree-overlays
echo "INFO: connecting dbus interfaces"
sudo snap connect fpgad:cli-dbus fpgad:daemon-dbus
# sudo snap connect fpgad:kernel-module-control # in future
echo "INFO: Done making necessary connections"

echo "INFO: Running snap test script"
# NOTE: tarball contains "k24-starter-kits/..." and "k26-starter-kits/..." at tarball root from daemon/tests/test_data
mkdir -p fpgad/artifacts
tar -xzvf test_data.gz -C fpgad
sudo python3 -m unittest ./snap_tests.py 2>&1 | tee fpgad/artifacts/snap_test.log
