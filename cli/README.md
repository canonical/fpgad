# fpga - FPGAd's CLI application

## Usage

```
Usage: fpga [OPTIONS] <COMMAND>

Commands:
  status  Get the status information for the given device handle
  load    Load a bitstream or an overlay for the given device handle
  remove  Remove bitstream or an overlay
  help    Print this message or the help of the given subcommand(s)

Options:
      --handle <HANDLE>  fpga device `HANDLE` to be used for the operations. Default value for this option is calculated in runtime and the application picks the first available fpga in the system (under /sys/class/fpga_manager)
  -h, --help             Print help

```

### Status

```shell
fpga [--handle=<device_handle>] status
```

### Loading

```shell
fpga [--handle=<device_handle>] load ( (overlay <file> [--handle=<handle>]) | (bitstream <file>) )
```

### Removing

```shell
fpga [--handle=<device_handle>] remove ( ( overlay <HANDLE> ) | ( bitstream ) )
```

## examples (for testing)

### Status

```shell
./target/debug/fpga status
./target/debug/fpga --handle=fgpa0 status
```

### Load

```shell
sudo ./target/debug/fpga load bitstream /lib/firmware/k26-starter-kits.bit.bin
sudo ./target/debug/fpga --handle=fpga0 load bitstream /lib/firmware/k26-starter-kits.bit.bin

sudo ./target/debug/fpga load overlay /lib/firmware/k26-starter-kits.dtbo
sudo ./target/debug/fpga load overlay /lib/firmware/k26-starter-kits.dtbo --handle=overlay_handle
sudo ./target/debug/fpga --handle=fpga0 load overlay /lib/firmware/k26-starter-kits.dtbo --handle=overlay_handle
```

### Remove

```shell
sudo ./target/debug/fpga --handle=fpga0 remove overlay
sudo ./target/debug/fpga --handle=fpga0 remove overlay --handle=overlay_handle
```