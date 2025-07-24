# inputs:

### platform string:

1) platform string provided
2) platform string not provided
3) platform string is bad

### device handle:

1) provide device handle
2) provide no device handle
3) provide bad device handle

### fw_search:

1) provide good fw_search_path
2) provide bad fw_search_path
3) don't provide fw_search_path

### device handle:

1) provide device handle
2) provide no device handle
3) provide bad device handle

# loading

### for loading bitstream direct there are these cases:

path:

1) file doesn't exist
2) bad file
    - not compatible with device
    - not properly structured
    - other
3) file exists
4) no file provided
5) path in lib firmware and fw_search_path overridden
6) path not in lib firmware and fw_search_path overridden
7) path in lib firmware and fw_search_path not overridden
8) path not in lib firmware and fw_search_path not overridden

flags/other:

1) bad flags
2) encrypted but no key provided

platform string:

1) provide platform string
2) provide no platform string
3) provide bad platform string

### for overlay there are these cases:

overlay itself:

1) overlay exists but cannot find target bitstream
2) overlay doesn't exist
3) overlay doesn't set flags etc so load fails
4) overlay exists but bad `fw_search_path`
5) overlay handle in use
6) device in use by other overlay handle
7) overlay handle couldn't be created? <- bad unicode?
8) path in lib firmware and fw_search_path overridden
9) path not in lib firmware and fw_search_path overridden
10) path in lib firmware and fw_search_path not overridden
11) path not in lib firmware and fw_search_path not overridden

### set flags:

flags:

1) out of range
2) not an int (dbus should catch)
3) fail to write?

# to run:

The integration tests themselves do not start fpgad. You must start that yourself. `sudo` permissions are required in
order to call control dbus service, and rustup typically only installs
locally, so you can build the test executable and run it with sudo by running

```shell
cargo test --workspace --no-run
```

then going to ./target/debug/deps/
and run

```shell
sudo ./integration-tests-<hashed value>
```
