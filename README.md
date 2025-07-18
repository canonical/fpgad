# FPGAd

[![fpgad](https://snapcraft.io/fpgad/badge.svg)](https://snapcraft.io/fpgad)

FPGAd is a WIP rust project.
It provides a strictly confined snap which will provide a mechanism to dynamically load and unload bitstreams to FPGA
devices on Ubuntu Core.
It mitigates the need for other snaps containing bitstreams to have super privileges.
Instead, those other snaps will connect to FPGAd and FPGAd will have the necessary privileges.
This is for several reasons, but mostly to:

1. make the snap store team happy (not publishing lots of super privileged snaps).
2. add a place for checking and verifying bitstreams before loading them for additional safety/security.
3. make it easier to port already existing bitstream packages to snap store.

In order to understand why the super privileges are necessary, you must understand that loading a bitstream
fundamentally changes the FPGA device's behaviour including what the connections to the CPU are/do.
To do this, device tree of the system must be changed (overlaid).
This device tree tells Ubuntu a lot of information about the device, such as how much ram the system has.
It should be clear to see, then, that compromising the device tree is very powerful.

# Key considerations

#### fw_search_path:

The kernel system only works with relative paths and searches relative to provided `fw_search_path` and hardcoded
firmware
locations see [the kernel docs](https://docs.kernel.org/driver-api/firmware/fw_search_path.html) for details. When
loading an overlay, only the relative path of the loaded overlay file is retrievable. If there is a file with the same
name in the default search paths (such as `/lib/firmware`), there is no guarantee that the loaded file is the one
provided by the user since the system will successfully load from there even in the event that the file is not present
at the provided path. It is up to the user to verify that the correct overlay is applied, such as by checking that the
appropriate drivers are loaded.

# Anticipated Architecture

![anticipated_architecture.png](docs/assets/anticipated_architecture.png)

# To Run Daemon

```shell
sudo RUST_LOG=trace RUST_BACKTRACE=full ./target/debug/fpgad
```

# Typical control sequence

#### FPGA only:

1. control.SetFpgaFlags(fpga_handle, flags)
2. control.WriteBitstreamDirect(fpga_handle, bitstream_path)

#### Overlay only:

1. status.GetOverlayStatus(overlay_handle) <- check doesn't exist
2. control.SetFpgaFlags(device_handle, flags) <- does check for sticking internally
3. control.CreateOverlay(overlay_handle) <- just makes a dir and checks the subsystem created the internal files
4. control.ApplyOverlay(overlay_handle, dtbo_path) <- writes dtbo_path to overlay and asserts overlay status
5. status.GetFpgaState(fpga_handle) <- check it is `operating`

#### Combined:

1. control.SetFpgaFlags(device_handle, flags) <- >does check for sticking internally
2. control.WriteBitstreamDirect
3. control.CreateOverlay(overlay_handle) <- just makes a dir and checks the subsystem created the internal files
4. control.ApplyOverlay(overlay_handle, dtbo_path) <- writes dtbo_path to overlay and asserts overlay status
5. status.GetFpgaState(fpga_handle) <- check it is `operating`

#### Removing:

The FPGA subsystem does not have a way to remove an overlay. Instead, you must write a new one.

To remove an overlay simply call:

1. control.RemoveOverlay(overlay_handle)

# Busctrl Call Examples

### Status (unprivileged)

To get the state of an FPGA device:

```shell
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetFpgaState ss "" "fpga0"
```

To get the currently set flags for an FPGA device:

```shell
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetFpgaFlags ss "" "fpga0"
```

To get the current status of an overlay with given handle and platform:

```shell
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetOverlayStatus ss "xlnx" "fpga0"
```

To get the compatibility string of a given FPGA device:

```shell
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetPlatformType s "fpga0"
```

To get all platforms for all devices:

```shell
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetPlatformTypes
```

To get all currently present overlay handles:

```shell
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetOverlays
```

### Control (privileged)

#### set flags

To set the flags of an FPGA device:

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control SetFpgaFlags ssu "" "fpga0" 0
```

#### apply an overlay

Using default `fw_search_path` generation:

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control ApplyOverlay ssss "xlnx" "fpga0" "/lib/firmware/k26-starter-kits.dtbo" ""
```

or manually specified `fw_search_path`:

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control ApplyOverlay ssss "xlnx" "fpga0" "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.dtbo" "/lib/firmware/xilinx/k26-starter-kits"
```

#### write a bitstream

Using automated platform detectoin and default `fw_search_path` generation:

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ssss "" "fpga0" "/lib/firmware/k26-starter-kits.bit.bin" ""
```

or using specific platform and specific `fw_search_path`:

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ssss "xlnx" "fpga0" "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin" "/lib/firmware/"
```

#### remove an overlay

To remove an overlay with provided platform and handle:

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control RemoveOverlay ss "xlnx" "fpga0"
```

Consider using `GetOverlays` and/or `GetOverlayStatus` if you don't
know the handle.

#### other properties

The virtual files cotained within `/sys/class/fpga_manager/fpga*/`, which do not have specific interfaces, can be
accessed by using ReadProperty or WriteProperty e.g.

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status ReadProperty s "/sys/class/fpga_manager/fpga0/name"
```

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteProperty ss "/sys/class/fpga_manager/fpga0/key" ""
```

# Snap

```shell
sudo snap install fpgad
sudo snap connect fpgad:fpga
```