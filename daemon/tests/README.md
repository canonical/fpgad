# to run:

The integration tests themselves do not start fpgad. You must start that yourself. `sudo` permissions are required in
order to call control dbus service, and rustup typically only installs. Please note that in order to provide the dbus
services access, a dbus conf must be added to the system. Details are in `daemon/README.md`.
you can build the test executable and run it with sudo by running

```shell
cargo test --workspace --no-run
```

and run

```shell
sudo ${CARGO_TARGET_DIR:-target}/deps/<binary name>-<hashed value> --test-threads=1
```
