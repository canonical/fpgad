# FPGAd

FPGAd is a WIP rust project.
It provides a strictly confined snap which will provide a mechanism to dynamically load and unload bitstreams to FPGA
devices on Ubuntu Core.
It mitigates the need for other snaps containing bitstreams to have super privileges.
Instead, those other snaps will connect to FPGAd and FPGAd will have the necessary privileges.
This is for several reasons, but mostly to:

1) make the snap store team happy (not publishing lots of super privileged snaps).
2) add a place for checking and verifying bitstreams before loading them for additional safety/security.
3) make it easier to port already existing bitstream packages to snap store.

In order to understand why the super privileges are necessary, you must understand that loading a bitstream
fundamentally changes the FPGA device's behaviour including what the connections to the CPU are/do.
To do this, device tree of the system must be changed (overlaid).
This device tree tells Ubuntu a lot of information about the device, such as how much ram the system has.
It should be clear to see, then, that compromising the device tree is very powerful.

# Anticipated Architecture

![anticipated_architecture.png](docs/assets/anticipated_architecture.png)

# To Run Daemon

```
sudo RUST_LOG=trace RUST_BACKTRACE=full ./target/debug/fpgad
```

# Configure DBUS:

```
sudo cp ./data/dbus/com.canonical.fpgad.conf /etc/dbus-1/system.d/
```

# Busctrl Call Examples

### Status (unprivileged)

```
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetFpgaState s "fpga0"

busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetFpgaFlags s "fpga0"

busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetOverlayStatus s "fpga0"
```

### Control (privileged)

```
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control SetFpgaFlags sx "fpga0" 0

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control ApplyOverlay ss "fpga0" "/lib/firmware/k26-starter-kits.dtbo"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ss "fpga0" "/lib/firmware/k26-starter-kits.bit.bin"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control RemoveOverlay s "fpga0" 
```