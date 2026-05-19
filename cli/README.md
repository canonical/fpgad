# FPGAd's CLI application

## Usage
```
Usage: [snap run] fpgad [OPTIONS] <COMMAND>

OPTIONs:
  -h, --help                    Print help
  -d, --device <DEVICE_HANDLE>  FPGA device handle to be used for operations.
                                Default value is calculated at runtime and picks
                                the first available FPGA device in the system
                                (under `/sys/class/fpga_manager/`)
  -p, --platform <PLATFORM>     Override the platform detection. Format:
                                "universal" or "vendor,device"
                                Examples: --platform=universal
                                          --platform=xlnx,zynqmp-pcap-fpga

COMMANDs:
├── load                Load a bitstream or overlay
│   ├── overlay <FILE> [--name <OVERLAY_NAME>]
│   │       Load overlay (.dtbo) into the system using the default OVERLAY_NAME
│   │           (either the provided DEVICE_HANDLE or "overlay0") or provide
│   │       --name (-n): to name the overlay directory
│   └── bitstream <FILE>
│           Load bitstream (e.g. `.bit.bin` file) into the FPGA
│
├── set <ATTRIBUTE> <VALUE>
│       Set an attribute/flag under `/sys/class/fpga_manager/<DEVICE_HANDLE>/<ATTRIBUTE>`
│
├── status
│       Show FPGA status (all devices and overlays)
│
├── remove              Remove an overlay or bitstream
│   ├── overlay [--name <OVERLAY_NAME>]
│   │       Removes the first overlay found (call repeatedly to remove all) or provide
│   │       --name (-n): to remove overlay previously loaded with given OVERLAY_NAME
│   └── bitstream [--handle <HANDLE>]
│           Remove active bitstream from FPGA (bitstream removal is vendor specific)
│           Optionally specify --handle to name the removal handle
│
├── universal           Low-level read/write access to FPGA manager properties
│   ├── read <SUB_CMD> <PATH>
│   │       Read from sysfs path where SUB_CMD is:
│   │           read_property: read property from sysfs path
│   │           read_flags: read flags from device handle
│   └── write <SUB_CMD> <PATH> <VALUE>
│           Write to sysfs path where SUB_CMD is:
│               write_flags: write flags to device handle
│               write_property: write string property to sysfs path
│               write_property_bytes: write bytes to sysfs path
│
└── dfx-mgr <CMD>       Pass commands to dfx-mgr-client (Xilinx DFX manager)
                        Examples: listPackage, listSlot, load, remove
```

### Loading

```shell
fpgad [--device=<device_handle>] load ( (overlay <file> [--name=<name>]) | (bitstream <file>) )
```

### Removing

```shell
fpgad [--device=<device_handle>] remove ( ( overlay [--name=<NAME>] ) | ( bitstream [--handle=<handle>] ) )
```

### Set

```shell
fpgad [--device=<device_handle>] set ATTRIBUTE VALUE
```

### Status

```shell
fpgad [--device=<device_handle>] status
```

### Universal

```shell
fpgad [--device=<device_handle>] universal ( (read <sub_cmd> <path>) | (write <sub_cmd> <path> <value>) )
```

### DFX Manager

```shell
fpgad dfx-mgr <CMD>
```

## examples (for testing)

### Load

```shell
sudo ./target/debug/fpgad_cli load bitstream /lib/firmware/k26-starter-kits.bit.bin
sudo ./target/debug/fpgad_cli --device=fpga0 load bitstream /lib/firmware/k26-starter-kits.bit.bin

sudo ./target/debug/fpgad_cli load overlay /lib/firmware/k26-starter-kits.dtbo
sudo ./target/debug/fpgad_cli load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
sudo ./target/debug/fpgad_cli --device=fpga0 load overlay /lib/firmware/k26-starter-kits.dtbo --name=overlay_handle
```

### Remove

```shell
sudo ./target/debug/fpgad_cli --device=fpga0 remove overlay
sudo ./target/debug/fpgad_cli --device=fpga0 remove overlay --name=overlay_handle
sudo ./target/debug/fpgad_cli remove bitstream
sudo ./target/debug/fpgad_cli remove bitstream --handle=my_handle
```

### Set

```shell
sudo ./target/debug/fpgad_cli set flags 0
sudo ./target/debug/fpgad_cli --device=fpga0 set flags 0
```

### Status

```shell
./target/debug/fpgad_cli status
./target/debug/fpgad_cli --device=fpga0 status
```

### Universal

```shell
# Read operations
sudo ./target/debug/fpgad_cli universal read read_property /sys/class/fpga_manager/fpga0/name
sudo ./target/debug/fpgad_cli universal read read_flags fpga0

# Write operations
sudo ./target/debug/fpgad_cli universal write write_flags fpga0 0
sudo ./target/debug/fpgad_cli universal write write_property /sys/class/fpga_manager/fpga0/flags 0
sudo ./target/debug/fpgad_cli universal write write_property_bytes /sys/class/fpga_manager/fpga0/firmware BYTES
```

### DFX Manager

```shell
# List packages and slots
sudo ./target/debug/fpgad_cli dfx-mgr listPackage
sudo ./target/debug/fpgad_cli dfx-mgr listSlot

# Load and remove with dfx-mgr
sudo ./target/debug/fpgad_cli dfx-mgr "load slot:0 package:my-package"
sudo ./target/debug/fpgad_cli dfx-mgr "remove slot:0"
```

### Platform Override

```shell
# Force universal platform
sudo ./target/debug/fpgad_cli --platform=universal load bitstream /lib/firmware/bitstream.bin

# Force specific Xilinx platform
sudo ./target/debug/fpgad_cli --platform=xlnx,zynqmp-pcap-fpga load bitstream /lib/firmware/bitstream.bin
```
