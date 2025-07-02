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

```shell
sudo RUST_LOG=trace RUST_BACKTRACE=full ./target/debug/fpgad
```

# Configure DBUS

```shell
sudo cp ./data/dbus/com.canonical.fpgad.conf /etc/dbus-1/system.d/
```

# Configuration File

### To use the provided `config.toml`

```shell
sudo mkdir -p /etc/fpgad/
sudo cp ./data/config.toml /etc/fpgad/ 
sudo mkdir -p /usr/lib/fpgad/
sudo cp ./data/config.toml /usr/lib/fpgad/
```

During install, the /etc/fpgad/ version doesn't need to exist, so can be created blank,
not copied in or be a copy of the `/usr/lib` version.

The `/usr/lib/` variant should be created during install and should contain a comment like
"DO NO EDIT THIS FILE USE  `/etc/fpgad/config.toml` FOR USER SPECIFIED OVERRIDES"

### `config.toml` location

The user provided config file must be stored in `/etc/fpgad` (or `$snap/etc/fpgad/` or similar path adjusted by snap
layouts) and
must be called `config.toml`

## `config.toml` syntax

Any unspecified values will default to hardcoded defaults, as described in the table below.

### `[system_paths]` section:

| Key                   | Description                                                                                                                      | Default                                      |
|-----------------------|----------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------|
| `overlay_control_dir` | The location to which configfs is mounted. This is used to control device tree overlays                                          | `"/sys/kernel/config/device-tree/overlays/"` |
| `firmware_source_dir` | The directory within which the firmware subsystem and overlayfs subsystem search relative to when loading bitstreams or overlays | `"/lib/firmware/"`                           |
| `fpga_managers_dir`   | The location of the fpga_manager device folder which contains, for example, `fpga0`.                                             | `"/sys/class/fpga_manager/"`                 |

### `[boot_firmware]` section:

In order to have firmware load during startup, there are two options (which can be combined):

1) Load a bitstreams directly to an fpga device
2) Load a device-tree overlays (which may apply a bitstream to the device)

These are provided as lists of bitstreams and lists of overlays.

For bitstreams, all fields are required.
For overlays, the device_handle is only reqiured in order to set fpga_flags if necessary. This only applies to the

The bitstreams will always be applied before any overlays to avoid CPU misfires. This means that if an overlay is also
configured to apply a bitstream, this will overwrite the previously written bitstreams.

If a user provided config (`/etc/fpgad/config.toml`) contains a `[boot_firmware]` section, this will override the
vendor provided [boot_firmware].

| Key                      | Description                                                                                                                              |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------|
| `default_device_handle`  | The device to control (e.g. fpga0)                                                                                                       |
| `default_overlay_handle` | The overlay handle to use for an overlay if specified                                                                                    |
| `default_bitstream`      | The path relative to `firmware_prefix` (see `[system_paths]` section) of the bitstream to load if specified                              |
| `default_overlay`        | The path relative to `firmware_prefix` (see `[system_paths]` section) of the device-tree overlay to load if specified                    |
| `default_fpga_flags`     | The fpgad flags necessary for writing the bitstream to the fpga device specified by `default_device_handle` (whether direct or via dtbo) |

### Example `config.toml`

```toml
[system_paths]
overlay_control_dir = "/sys/kernel/config/device-tree/overlays/"
firmware_source_dir = "/lib/firmware/"
fpga_managers_dir = "/sys/class/fpga_manager/"

[boot_firmware]
[[boot_firmware.bitstreams]]
device_handle = "fpga0"
bitstream_path = "/lib/firmware/bitstream0"
flags = 0
[[boot_firmware.bitstreams]]
device_handle = "fpga1"
bitstream_path = "/lib/firmware/bitstream1"
flags = 0
[[boot_firmware.overlays]]
platform = "zynqmp"
overlay_handle = "k26-starter-kits"
overlay_path = "/lib/firmware/k26-starter-kits.dtbo"
device_handle = "fpga0"
fpga_flags = 0
```

#### specify none

In order to specify an empty list of overlays or bitstreams, use the following syntax

```toml
[boot_firmware]
bitstreams = []
overlays = []
```

or, for example, to specify a bitstream but no overlays do add something like

```toml
[boot_firmware]
overlays = []

[[boot_firmware.bitstreams]]
device_handle = "fpga0"
bitstream_path = "/lib/firmware/bitstream0"
flags = 0
```

please note that the empty list must come before the list entries.

# To run on startup

Before installing, confirm that `ExecStart=` in the `.service` file points to the correct executable (e.g.
`ExecStart=/home/ubuntu/fpgad/target/debug/fpgad`).

To install the service run

```shell
sudo cp data/systemd/fpgad.service /lib/systemd/system/
```

To run without restarting

```shell
sudo systemctl daemon-reexec
sudo systemctl daemon-reload
sudo systemctl enable fpgad.service
sudo systemctl start fpgad.service
```

To load defaults on startup run

```shell
sudo cp data/systemd/fpgad-load-defaults.service /lib/systemd/system/
sudo systemctl enable fpgad-load-defaults.service 
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

```shell
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetFpgaState s "fpga0"

busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetFpgaFlags s "fpga0"

busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetOverlayStatus ss "fpga0" "fpga0"

busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetPlatformType s "fpga0"
busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status GetPlatformTypes 
```

### Control (privileged)

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control SetFpgaFlags sx "fpga0" 0

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control ApplyOverlay sss "fpga0" "fpga0" "/lib/firmware/k26-starter-kits.dtbo"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ss "fpga0" "/lib/firmware/k26-starter-kits.bit.bin"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control RemoveOverlay ss "fpga0" "fpga0" 

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control LoadDefaults
```

### Configure

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure GetOverlayControlDir
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure GetFirmwareSourceDir
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure GetFpgaManagersDir

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetOverlayControlDir s "/sys/kernel/config/device-tree/overlays/"
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetFirmwareSourceDir s "/lib/firmware/"
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetFpgaManagersDir s "/sys/class/fpga_manager/"
```

### Example changing FW path

```
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetFirmwareSourceDir s "/lib/firmware/"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ss "fpga0" "/lib/firmware/k26-starter-kits.bit.bin"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/configure com.canonical.fpgad.configure SetFirmwareSourceDir s "/lib/firmware/xilinx/k26-starter-kits"

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteBitstreamDirect ss "fpga0" "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin"
```