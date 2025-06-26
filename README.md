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

# Configure DBUS

```
sudo cp ./data/dbus/com.canonical.fpgad.conf /etc/dbus-1/system.d/
```

# Configuration File

### To use the provided `config.toml`

```
sudo mkdir -p /etc/fpgad/
sudo cp ./data/config.toml /etc/fpgad/
```

### `config.toml` location

The config file must be stored in `/etc/fpgad` (or `$snap/etc/fpgad/` or similar path adjusted by snap layouts) and
must be called `config.toml`

## `config.toml` syntax

Any unspecified values will default to hardcoded defaults, as described in the table below.

### `[system_paths]` section:

| Key                | Description                                                                                                                       | Default                                      |
|--------------------|-----------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------|
| `config_fs_prefix` | The location to which configfs is mounted. This is used to control device tree overlays                                           | `"/sys/kernel/config/device-tree/overlays/"` |
| `firmware_prefix`  | The directory within which the firmware subsystem and overlayfs subssystem search relative to when loading bitstreams or overlays | `"/lib/firmware/"`                           |
| `sys_fs_prefix`    | The location of the fpga_manager device folder which contains, for example, `fpga0`.                                              | `"/sys/class/fpga_manager/"`                 |

### Example `config.toml`

```toml
[system_paths]
config_fs_prefix = "/sys/kernel/config/device-tree/overlays/"
firmware_prefix = "/lib/firmware/"
sys_fs_prefix = "/sys/class/fpga_manager/"
```

# Typical control sequence

#### FPGA only:

1) control.SetFpgaFlags(fpga_handle, flags)
2) control.WriteBitstreamDirect(fpga_handle, bitstream_path)

#### Overlay only:

1) status.GetOverlayStatus(overlay_handle) <- check doesn't exist
2) control.SetFpgaFlags(device_handle, flags) <- does check for sticking internally
3) control.CreateOverlay(overlay_handle) <- just makes a dir and checks the subsystem created the internal files
4) control.ApplyOverlay(overlay_handle, dtbo_path) <- writes dtbo_path to overlay and asserts overlay status
5) status.GetFpgaState(fpga_handle) <- check it is `operating`

#### Combined:

1) control.SetFpgaFlags(device_handle, flags) <- >does check for sticking internally
2) control.WriteBitstreamDirect
3) control.CreateOverlay(overlay_handle) <- just makes a dir and checks the subsystem created the internal files
4) control.ApplyOverlay(overlay_handle, dtbo_path) <- writes dtbo_path to overlay and asserts overlay status
5) status.GetFpgaState(fpga_handle) <- check it is `operating`

#### Removing:

The FPGA subsystem does not have a way to remove an overlay. Instead, you must write a new one.

To remove an overlay simply call:

1) control.RemoveOverlay(overlay_handle)

# Busctrl Call Examples

### Status (unprivileged)

```
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetFpgaState s "fpga0"

busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetFpgaFlags s "fpga0"

busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetOverlayStatus ss "fpga0" "fpga0"
```

### Control (privileged)

```
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control SetFpgaFlags sx "fpga0" 0

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control ApplyOverlay sss "fpga0" "fpga0" "/lib/firmware/k26-starter-kits.dtbo"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ss "fpga0" "/lib/firmware/k26-starter-kits.bit.bin"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control RemoveOverlay ss "fpga0" "fpga0" 
```

### Configure

```
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure GetConfigFsPrefix
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure GetFirmwarePrefix
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure GetSysFsPrefix

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetConfigFsPrefix s "/sys/kernel/config/device-tree/overlays/"
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetFirmwarePrefix s "/lib/firmware/"
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetSysFsPrefix s "/sys/class/fpga_manager/"
```