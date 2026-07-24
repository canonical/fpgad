# fpgad_proxies

This crate provides the [`zbus`](https://docs.rs/zbus) proxy definitions used to call the [`fpgad`](https://docs.rs/fpgad) DBus API.

Fpgad is an FPGA manager daemon for bitstream and device tree overlay management on Linux, but which was specifically designed to provide that functionality on Ubuntu Core. If you are writing a Rust application that calls fpgad DBus methods, such as a provider snap, you can depend on this crate rather than defining the proxies yourself.

## Provided proxies

| Proxy                                                  | Module                     | Interface                     | Object path                    |
|--------------------------------------------------------|----------------------------|-------------------------------|--------------------------------|
| [`ControlProxy`](proxies::control_proxy::ControlProxy) | [`proxies::control_proxy`] | `com.canonical.fpgad.control` | `/com/canonical/fpgad/control` |
| [`StatusProxy`](proxies::status_proxy::StatusProxy)    | [`proxies::status_proxy`]  | `com.canonical.fpgad.status`  | `/com/canonical/fpgad/status`  |

The `control` interface handles all write operations (loading bitstreams, applying or removing overlays, writing FPGA manager properties, driving `dfx-mgr-client`).
The `status` interface handles all read-only queries (device state, programming flags, overlay status, platform compatibility strings).

The `zbus` crate is re-exported as [`zbus`] so that consumers do not need to depend on it directly (and to keep proxy and connection versions in lock-step).

## Usage

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
fpgad_proxies = "x.y.z"
```

Connect to the system bus and call a method on either proxy:

```rust,no_run
use fpgad_proxies::proxies::control_proxy::ControlProxy;
use fpgad_proxies::proxies::status_proxy::StatusProxy;
use fpgad_proxies::zbus::Connection;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let connection = Connection::system().await?;

// Read-only: query the state of an FPGA device.
let status = StatusProxy::new(&connection).await?;
let state = status.get_fpga_state("", "fpga0").await?;
println!("fpga0 state: {state}");

// Privileged: write a bitstream directly to an FPGA device.
let control = ControlProxy::new(&connection).await?;
let result = control
    .write_bitstream_direct("", "fpga0", "/lib/firmware/design.bit.bin", "")
    .await?;
println!("{result}");
# Ok(())
# }
```

## Common Concepts

The following concepts apply to both [`proxies::control_proxy`] and [`proxies::status_proxy`].

### Device Handles

[Device Handles]: #device-handles

A "device handle" refers to the name of an FPGA device as it appears in `/sys/class/fpga_manager/`.
Common examples include:

- `fpga0` - The first FPGA device
- `fpga1` - The second FPGA device (if multiple FPGAs are present)

These handles uniquely identify FPGA devices in the system and are used throughout this crate to specify which device to operate on.

### Overlay Handles

[Overlay Handles]: #overlay-handles

An "overlay handle" refers to the name of a device tree overlay as it appears in `/sys/kernel/config/device-tree/overlays/`.
Common examples include:

- `overlay0` - A generic overlay name
- `fpga-design` - A custom overlay name specified during loading

These handles are used to identify and manage loaded device tree overlays.
When applying an overlay, you can specify a custom handle or let the daemon choose one based on the device handle.

### Error Handling

[Error Handling]: #error-handling

Every proxy method returns `zbus::Result<String>` (i.e. `Result<String, zbus::Error>`).
This means callers must distinguish between two categories of failure:

- **DBus communication errors** - Problems connecting to or communicating with the daemon, service not running, method not found, etc.
  These surface as the various non-`Failure` variants of [`zbus::Error`](https://docs.rs/zbus/latest/zbus/enum.Error.html).
- **Application errors** - Errors raised by the `fpgad` daemon itself.
  These surface as `zbus::Error::Failure`, whose message is prefixed with `FpgadError::<variant>:` followed by the error details, for example:

  ```text
  FpgadError::Argument: Device fpga0 not found.
  FpgadError::IOWrite: Failed to write bitstream: Permission denied
  FpgadError::IORead: Failed to read state: No such file or directory
  ```

See the [daemon's error module documentation](https://docs.rs/fpgad/latest/fpgad/error/enum.FpgadError.html) for the full list of `FpgadError` variants and their meanings.

## Related crates

- [`fpgad`](https://docs.rs/fpgad) - The daemon that implements the DBus interfaces these proxies talk to.
- [`fpgad_cli`](https://docs.rs/fpgad_cli) - A command-line client built on top of this crate.

## License

GPL-3.0
