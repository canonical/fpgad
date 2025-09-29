# FPGAd's CLI application

## Usage
```
Usage: [snap run] fpgad [OPTIONS] <COMMAND>

OPTIONs:
  -h, --help            Print help
      --handle <DEVICE_HANDLE>  fpga device `HANDLE` to be used for the operations.
                       Default value for this option is calculated in runtime
                       and the application picks the first available fpga device
                       in the system (under `/sys/class/fpga_manager/`)

COMMANDs:
├── load                Load a bitstream or overlay
│   ├── overlay <FILE> [--handle <OVERLAY_HANDLE>]
│   │       Load overlay (.dtbo) into the system using the default OVERLAY_HANDLE
│   │           (either the provided DEVICE_HANDLE or "overlay0") or provide
│   │       --handle: to name the overlay directory
│   └── bitstream <FILE>
│           Load bitstream (e.g. `.bit.bin` file) into the FPGA
│
├── set <ATTRIBUTE> <VALUE>
│       Set an attribute/flag under `/sys/class/fpga_manager/<DEVICE_HANDLE>/<ATTRIBUTE>`
│
├── status [--handle <DEVICE_HANDLE>]
│       Show FPGA status (all devices and overlays) or provide
│       --handle: for a specific device status
│
└── remove              Remove an overlay or bitstream
    ├── overlay [--handle <OVERLAY_HANDLE>]
    │       Removes the first overlay found (call repeatedly to remove all) or provide
    │       --handle: to remove overlay previously loaded with given OVERLAY_HANDLE
    └── bitstream
            Remove active bitstream from FPGA (bitstream removal is vendor specific)
```

### Loading

```shell
fpgad [--handle=<device_handle>] load ( (overlay <file> [--handle=<handle>]) | (bitstream <file>) )
```

### Removing

```shell
fpgad [--handle=<device_handle>] remove ( ( overlay <HANDLE> ) | ( bitstream ) )
```

### Set

```shell
fpgad [--handle=<device_handle>] set ATTRIBUTE VALUE
```

### Status

```shell
fpgad [--handle=<device_handle>] status
```

## examples (for testing)

### Load

```shell
sudo ./target/debug/cli load bitstream /lib/firmware/k26-starter-kits.bit.bin
sudo ./target/debug/cli --handle=fpga0 load bitstream /lib/firmware/k26-starter-kits.bit.bin

sudo ./target/debug/cli load overlay /lib/firmware/k26-starter-kits.dtbo
sudo ./target/debug/cli load overlay /lib/firmware/k26-starter-kits.dtbo --handle=overlay_handle
sudo ./target/debug/cli --handle=fpga0 load overlay /lib/firmware/k26-starter-kits.dtbo --handle=overlay_handle
```

### Remove

```shell
sudo ./target/debug/cli --handle=fpga0 remove overlay
sudo ./target/debug/cli --handle=fpga0 remove overlay --handle=overlay_handle
```

### Set

```shell
sudo ./target/debug/cli set flags 0
sudo ./target/debug/cli --handle=fpga0 set flags 0
```

### Status

```shell
./target/debug/cli status
./target/debug/cli --handle=fpga0 status
```
