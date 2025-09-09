# FPGAd's CLI application

## Usage

```
Usage: [snap run] fpgad [OPTIONS] <COMMAND>

Commands:
  load    Load a bitstream or an overlay for the given device handle
  remove  Remove bitstream or an overlay
  set     Write a value to an attribute within the sysfs folder e.g. to edit /sys/class/fpga_manager/fpga0/flags
  status  Get the status information for the given device handle
  help    Print this message or the help of the given subcommand(s)

Options:
      --handle <HANDLE>  fpga device `HANDLE` to be used for the operations. Default value for this option is calculated in runtime and the application picks the first available fpga in the system (under /sys/class/fpga_manager)
  -h, --help             Print help

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
