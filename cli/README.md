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

## examples (for testing)

### Status

```shell
./target/debug/fpga status
./target/debug/fpga --handle=fgpa0 status
```