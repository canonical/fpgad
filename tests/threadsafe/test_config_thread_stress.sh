#!/usr/bin/bash


NUM_WORKERS=50

WRITER_CMD='sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetConfigFsPrefix s "/sys/kernel/config/device-tree/overlays/'

READER_CMD='sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure GetConfigFsPrefix'

# Spawn
for i in $(seq 1 $NUM_WORKERS); do
    eval "$WRITER_CMD$i\"" &
    eval "$READER_CMD" &
done

wait

echo "Finished running $((NUM_WORKERS)) concurrent D-Bus calls."