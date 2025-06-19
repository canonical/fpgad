# Components for Universal Platforms

- UniversalFpga
- UniversalOverlayHandler

A UniversalFpga instance is created with a known device handle/name such as `fpga0`. The constructor will return None if
the device is not present in sysfs.

A UniversalOverlayHandler instance is created empty and must be populated with it's overlay_source_path (location of the
overlay which will be used) and overlay_fs_path (the configfs directory associated with the file in
overlay_source_path), however, here, the overlay_fs_path is constructed using a handle/name for the overlay. This is
provided by the user and, if `fpga0` is provided, becomes, for example, `/sys/kernel/config/device-tree/overlays/fpga0`.
This overlayfs base path is used to access the overlay's `path` and `status` virtual files. The implementation specific
methods `get_vfs_status()` and `get_vfs_path()` are used to read from these files. The method `vfs_check_applied()` is a
wrapper to check that both the status and path files show the correct contents after an attempted application.

```mermaid
classDiagram
    class UniversalFPGA {
        + device_handle: String e.g. "fpga0"
        device_handle() <- get
        get_state()
        get_flags()
        set_flags()
        load_firmware()
        + new(device_handle) <- takes e.g. "fpga0"
        + assert_state()
    }
    class UniversalOverlayHandler {
        + overlay_source_path: Option&lt;PathBuf&gt;
        + overlay_fs_path: Option&lt;PathBuf&gt;
        get_required_flags() <- FPGA requires?
        get_status()
        set_source_path(source_path)
        set_overlay_fs_path(overlay_handle)
        get_overlay_fs_path()
        apply_overlay()
        remove_overlay()
        + prepare_for_load()
        + get_vfs_status()
        + get_vfs_path()
        + vfs_check_applied()
    }
```