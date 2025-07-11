#!/usr/bin/bash


NUM_WORKERS=50

cmd1='sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ssss "" "fpga0" "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin" "/lib/firmware/xilinx/k26-starter-kits/"'

cmd2='sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ssss "" "fpga0" "/lib/firmware/k26-starter-kits.bit.bin" ""
            '

# Spawn
for i in $(seq 1 $NUM_WORKERS); do
    eval "$cmd1" &
    eval "$cmd2" &
done

wait

echo "Finished running $((NUM_WORKERS)) concurrent D-Bus calls."