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

# Building and running from source

From a fresh install, you must install build-essential, rustup (includes cargo) and pull the source from the repo then
build. To run locally you must configure the dbus daemon. Our docs assume the use of systemd.

### install rust

Following [the rust documentation](https://www.rust-lang.org/tools/install), installing rust can be done by running

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

and following the instructions in the installer. For FPGAd, no custom settings are required if
building on the target device, so the `standard installation` option is fine.

### install build dependencies

The rust compiler has some dependencies (like `gcc`, `make` and `libc-dev`) so please install these. For debian distros,
this can be done in one go by installing `build-essential` (`sudo` may be necessary):

```
apt update && apt install -y build-essential
```

### build

cargo makes this simple:

```
cargo build --workspace 
```

will build all parts including cli.

```
cargo build --bin fpgad
```

will build fpgad and any direct dependencies in debug mode (no optimisations, no stripping of the binary etc.).
Release mode can be used by running

```
cargo build --release [opts] 
```

### run

Using default build settings, the resulting binaries will be in `./target/debug/`
such as `./target/debug/fpgad` for the daemon and `./target/debug/cli` for the
command line application. If using release, then the directory is  `./target/release/`.

If running as a standalone binary (not snap) then dbus needs to be configured to know about
fpgad. If using systemd then an example configuration is provided in
`./daemon/tests/test_data/com.canonical.fpgad.conf` and can be installed by running

```shell
sudo cp ./daemon/tests/test_data/com.canonical.fpgad.conf /etc/dbus-1/system.d/com.canonical.fpgad.conf
sudo systemctl reload dbus
```

but please note that this configuration is not intended for general use since it allows dbus clients to access
privileged IO operations on files. To mitigate this, adjust the file as necessary, e.g. change

```xml
<!-- Unprivileged interface -->
<policy context="default">
    ...
    <allow send_interface="com.canonical.fpgad.control"/>
</policy>
```

to

```xml
<!-- Unprivileged interface -->
<policy context="default">
    ...
    <deny send_interface="com.canonical.fpgad.control"/>
</policy>
```

to block unprivileged use of the control interface.

fpgad itself requires privileges to access certain system files so run it as sudo (or, better yet, configured it as a
systemd daemon)

```
sudo RUST_LOG=trace ./target/<mode>/fpgad
```
