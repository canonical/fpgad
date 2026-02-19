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

To get the currently set flags for an FPGA device in hex (but missing the `0x` prefix):

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

To set the flags of an FPGA device to a value.
Note, the value string is interpreted by a string parser into hex so, for example, "12" , "0b1100" or "0xB" can be used to set to the value of 12 (in decimal)

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control SetFpgaFlags ssu "" "fpga0" 0

sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control SetFpgaFlags ssu "" "fpga0" 12 (converts to hex so is stored as 0xC
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control SetFpgaFlags ssu "" "fpga0" 0x0C
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control SetFpgaFlags ssu "" "fpga0" 0b1100 (converts to hex so is stored as 0xC)
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

Using automated platform detection and default `fw_search_path` generation:

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

The virtual files contained within `/sys/class/fpga_manager/fpga*/`, which do not have specific interfaces, can be
accessed by using ReadProperty or WriteProperty e.g.

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/status com.canonical.fpgad.status ReadProperty s "/sys/class/fpga_manager/fpga0/name"
```

```shell
sudo busctl call --system com.canonical.fpgad /com/canonical/fpgad/control com.canonical.fpgad.control WriteProperty ss "/sys/class/fpga_manager/fpga0/flags" 0
```

```shell
sudo busctl call --system   com.canonical.fpgad   /com/canonical/fpgad/control   com.canonical.fpgad.control   WritePropertyBytes say "/sys/class/fpga_manager/fpga0/key" 4 0xAB 0xAD 0xC0 0xDE
sudo busctl call --system   com.canonical.fpgad   /com/canonical/fpgad/control   com.canonical.fpgad.control   WritePropertyBytes say "/sys/class/fpga_manager/fpga0/key" 8 0x00 0x00 0x00 0x00 0x00 0x00 0x00 0x00 # requires manual reset on kria xilinx kernels
```

# Snap

```shell
sudo snap install fpgad
sudo snap connect fpgad:fpga
```
