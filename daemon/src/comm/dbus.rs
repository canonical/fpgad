// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
//
// Copyright 2025 Canonical Ltd.
//
// SPDX-License-Identifier: GPL-3.0-only
//
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
//
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

//!
//! # FPGAd dbus interfaces
//!
//! There are two interfaces available: `status`, which contains methods to access read-only properties of the FPGA and overlay subsystems, and `control` which contains methods to access writable properties of the FPGA and overlay subsystems.
//! Directly following this paragraph is a summary of all available methods, with the rest of this document explaining each method in detail.
//! Please note that these interfaces are used in the `cli` part of FPGAd (e.g. in [load.rs](../../../../cli/src/load.rs)), and so those implementations can be used as examples with greater context than those provided here.
//! There is further usage documentation, including the use of these interfaces from cmdline by way of `dbusctl`, in [daemon/README.md](../../../README.md).
//!
//!
//! For both interfaces, all methods are asynchronous and should be awaited immediately to avoid async pollution.
//!
//! ## Custom Errors
//!
//! All functions are shown to return the `fdo::Error` type, which is a wrapper for "FreeDesktopOrg", i.e. these are DBus error types. In the case that FPGAd returns an error, and it is not a DBus communication error, this error will be of type `fdo::Error:Failure` and the error string with begin with the `FpgadError` identifier such as `"FpgadError::Argument:"` i.e. the full error will be reported (from `busctl`) as
//! ```text
//! Call failed: FpgadError::Argument: <error text>
//! ```
//! or in the case of an IOError, the rust error will appear like:
//! ```text
//! [2026-01-08T16:29:05Z ERROR cli] org.freedesktop.DBus.Error.IOError: FpgadError::IOWrite: <fpgad's error text>: No such file or directory (os error 2)
//! Error: MethodError(OwnedErrorName("org.freedesktop.DBus.Error.IOError"), Some("FpgadError::IOWrite: <fpgad's error text>: No such file or directory (os error 2)"), Msg { type: Error, serial: 11, sender: UniqueName(":1.77"), reply-serial: 4, body: Str, fds: [] })
//! ```
//! whereby the `FPGAdError` type could  be mapped to an equivalent `fdo::Error` type.
//!
//! ## FPGA API Summary
//!
//! | Interface | Method                   | Parameters                                                                                                     | Returns / Notes                                                                                                                             |
//! |-----------|--------------------------|----------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------|
//! | [status](status_interface::StatusInterface)   | [`get_fpga_state`](status_interface::StatusInterface::get_fpga_state)           | `platform_string: &str`, `device_handle: &str`                                             | `String` – Current FPGA state (`unknown`, `operating`, etc.)                                                                                |
//! | [status](status_interface::StatusInterface)   | [`get_fpga_flags`](status_interface::StatusInterface::get_fpga_flags)           | `platform_string: &str`, `device_handle: &str`                                             | `String` – Current FPGA flags from hexadecimal integer to string                                                                            |
//! | [status](status_interface::StatusInterface)   | [`get_overlay_status`](status_interface::StatusInterface::get_overlay_status)   | `platform_string: &str`, `overlay_handle: &str`                                            | `String` – Overlay status; errors if handle empty or invalid                                                                                |
//! | [status](status_interface::StatusInterface)   | [`get_overlays`](status_interface::StatusInterface::get_overlays)               | None                                                                                       | `String` – List of available overlay handles (`\n` separated)                                                                               |
//! | [status](status_interface::StatusInterface)   | [`get_platform_type`](status_interface::StatusInterface::get_platform_type)     | `device_handle: &str`                                                                      | `String` – Compatibility string for device                                                                                                  |
//! | [status](status_interface::StatusInterface)   | [`get_platform_types`](status_interface::StatusInterface::get_platform_types)   | None                                                                                       | `String` – List of all fpga devices and their compatibility strings (`<device>:<compat>\n`)                                                 |
//! | [status](status_interface::StatusInterface)   | [`read_property`](status_interface::StatusInterface::read_property)             | `property_path_str: &str`                                                                  | `String` – Contents of arbitrary FPGA attribute value                                                                                       |
//! | [control](control_interface::ControlInterface) | [`set_fpga_flags`](control_interface::ControlInterface::set_fpga_flags)         | `platform_string: &str`, `device_handle: &str`, `flags: u32`                               | `String` – Confirmation with new flags in hex                                                                                               |
//! | [control](control_interface::ControlInterface) | [`write_bitstream_direct`](control_interface::ControlInterface::write_bitstream_direct) | `platform_string: &str`, `device_handle: &str`, `bitstream_path_str: &str`, `firmware_lookup_path: &str` | `String` – Confirmation of bitstream load; acquires write lock                                                                              |
//! | [control](control_interface::ControlInterface) | [`apply_overlay`](control_interface::ControlInterface::apply_overlay)           | `platform_string: &str`, `overlay_handle: &str`, `overlay_source_path: &str`, `firmware_lookup_path: &str` | `String` – Overlay applied; confirmation including firmware prefix; write lock used to protect against firmware search path race conditions |
//! | [control](control_interface::ControlInterface) | [`remove_overlay`](control_interface::ControlInterface::remove_overlay)         | `platform_string: &str`, `overlay_handle: &str`                                            | `String` – Overlay removed; confirmation including overlay filesystem path                                                                  |
//! | [control](control_interface::ControlInterface) | [`write_property`](control_interface::ControlInterface::write_property)         | `property_path_str: &str`, `data: &str`                                                    | `String` – Confirmation of data written; path must be under `/sys/class/fpga_manager/`                                                      |
//! | [control](control_interface::ControlInterface) | [`write_property_bytes`](control_interface::ControlInterface::write_property_bytes) | `property_path_str: &str`, `data: &[u8]`                                                   | `String` – Confirmation of bytes written; path must be under `/sys/class/fpga_manager/`                                                     |

pub mod control_interface;
pub mod status_interface;

use crate::config;
use crate::error::FpgadError;
use crate::system_io::{fs_read, fs_write};
use log::trace;
use std::path::{Component, Path, PathBuf};

/// Read the current contents of an FPGA device property, e.g. "name". The property path must be a subdirectory of the fpga manager directory (typically, /sys/class/fpga_manager/)
///
/// # Arguments
///
/// * `property_path_str`: path to the variable to read e.g. /sys/class/fpga_manager/fpga0/name
///
/// # Returns: `Result<String, FpgadError>`
/// * `String` - the contents of the property path
///
/// * `FpgadError::Argument` if the path is not found within the compile time [config::FPGA_MANAGERS_DIR]
///
/// # Examples
///
/// ```
/// let device_name = fs_read_property("/sys/class/fpga_manager/fpga0/name")?;
/// assert_eq!(device_name, "Xilinx ZynqMP FPGA Manager\n")
/// ```
pub fn fs_read_property(property_path_str: &str) -> Result<String, FpgadError> {
    let property_path = Path::new(property_path_str);
    if !property_path.starts_with(Path::new(config::FPGA_MANAGERS_DIR)) {
        return Err(FpgadError::Argument(format!(
            "Cannot access property {}: does not begin with {}",
            property_path_str,
            config::FPGA_MANAGERS_DIR
        )));
    }
    fs_read(property_path)
}

#[allow(dead_code)]
/// Read the currently specified firmware search path.
/// See [these kernel docs](https://docs.kernel.org/driver-api/firmware/fw_search_path.html)
/// for more information on the process.
///
/// # Returns: `Result<String, FpgadError>`
/// * `String` - The contents of the firmware search path variable.
/// * `FpgadError::IOWRite` (or similar IO error) if writing fails for any reason.
///
/// # Examples
///
/// ```rust
/// let search_path_str = read_firmware_source_dir()?;
/// assert_eq!(search_path_str, "/lib/firmware/my_firmware_dir");
/// ```
pub fn read_firmware_source_dir() -> Result<String, FpgadError> {
    trace!(
        "Reading fw prefix from {}",
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_read(fw_lookup_override)
}

/// Write a specified path to the systems firmware search path.
/// See [these kernel docs](https://docs.kernel.org/driver-api/firmware/fw_search_path.html)
/// for more information on the process.
///
/// # Arguments
///
/// * `new_path`: path inside which firmware can be found
///
/// # Returns: `Result<(), FpgadError>`
/// * `()` on success
/// * `FpgadError::IOWRite` (or similar IO error) if writing fails for any reason.
///
/// # Examples
///
/// ```
/// assert!(write_firmware_source_dir("/lib/firmware/my_firmware_dir").is_ok());
/// ```
pub fn write_firmware_source_dir(new_path: &str) -> Result<(), FpgadError> {
    trace!(
        "Writing fw prefix {} to {}",
        new_path,
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_write(fw_lookup_override, false, new_path)
}

/// Splits a Path object into its parent directory and basename/filename
///
/// # Arguments
///
/// * `path`: path to be split
///
/// # Returns: `Result<(PathBuf, PathBuf), FpgadError>`
/// * `(PathBuf, PathBuf)` - Tuple of parent directory and basename/filename
/// * `FpgadError::Argument` on invalid `path` or `path` is a root directory (no parent)
/// # Examples
///
/// ```rust
/// let (parent, base) = extract_path_and_filename(Path::new("/lib/firmware/file.bin"));
/// assert_eq!(parent.to_string_lossy(), "/lib/firmware");
/// assert_eq!(base.to_string_lossy(), "file.bin");
/// ```
pub fn extract_path_and_filename(path: &Path) -> Result<(PathBuf, PathBuf), FpgadError> {
    // Extract filename
    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or(FpgadError::Argument(format!(
            "Provided bitstream path {path:?} is not a file or a valid directory."
        )))?;

    // Extract parent directory
    let base_path = path
        .parent()
        .and_then(|p| p.to_str())
        .ok_or(FpgadError::Argument(format!(
            "Provided bitstream path {path:?} is missing a parent dir."
        )))?;

    Ok((base_path.into(), filename.into()))
}

/// Helper function to check that a device with given handle does exist.
///
/// # Arguments
///
/// * `device_handle`: name of the device in sysfs, e.g. `fpga0`
///
/// # Returns: `Result<(), FpgadError>`
/// * `()` on success
/// * `FpgadError::Argument` in the case that `device_handle` is not found on the system or
///   device_handle contains invalid characters/is empty.
///
/// # Examples
///
/// ```
/// assert!(validate_device_handle("fpga0").is_ok())
/// assert!(validate_device_handle("").is_err())
/// ```
pub(crate) fn validate_device_handle(device_handle: &str) -> Result<(), FpgadError> {
    if device_handle.is_empty() || !device_handle.is_ascii() {
        return Err(FpgadError::Argument(format!(
            "{device_handle} is invalid name for fpga device.\
                fpga name must be compliant with sysfs rules."
        )));
    }
    let fpga_managers_dir = config::FPGA_MANAGERS_DIR;
    if !PathBuf::from(fpga_managers_dir)
        .join(device_handle)
        .exists()
    {
        return Err(FpgadError::Argument(format!(
            "Device {device_handle} not found."
        )));
    };
    Ok(())
}

/// Helper function to find the overlap between two paths and to return a tuple of the overlap and
/// the difference. Note: the paths must both share the same root otherwise no overlap will be found
///
/// Typically, this is used to create a fw_lookup_path and a corresponding relative path which points
/// to the source file
///
/// # Arguments
///
/// * `source_path`: the full path to the target file (or containing directory?)
/// * `firmware_path`: the root common path for all files to be loaded by the FW subsystem
///
/// # Returns: `Result<(PathBuf, PathBuf), FpgadError>`
/// * `(PathBuf, PathBuf)` - A tuple of (prefix, suffix) where prefix is
///   typically used as the fw_lookup_path and the suffix is remaining relative path from that prefix
/// * `FpgadError::Argument` in case `firmware_path` is not within `source_path`, or for inputs
///   resulting in an empty suffix value
/// # Examples
///
/// ```
/// #use std::path::Path;
/// let (prefix, suffix) = make_firmware_pair(
///      Path::new("/lib/firmware/file.bin"),
///      Path::new("/lib/firmware/"),
/// )?;
/// assert_eq!(prefix, "/lib/firmware");
/// assert_eq!(suffix, "file.bin");
/// ```
pub(crate) fn make_firmware_pair(
    source_path: &Path,
    firmware_path: &Path,
) -> Result<(PathBuf, PathBuf), FpgadError> {
    // No firmware search path provided, so just try to use parent dir
    if firmware_path.as_os_str().is_empty() {
        return extract_path_and_filename(source_path);
    }
    if let Ok(suffix) = source_path.strip_prefix(firmware_path) {
        // Remove leading '/' if present
        let cleaned_suffix_path = suffix
            .components()
            .skip_while(|c| matches!(c, Component::RootDir))
            .collect::<PathBuf>();
        if cleaned_suffix_path.as_os_str().is_empty() {
            return Err(FpgadError::Argument(format!(
                "The resulting filename from stripping {firmware_path:?} from {source_path:?} \
                was empty. Cannot write empty string to fpga."
            )));
        }
        Ok((firmware_path.to_path_buf(), cleaned_suffix_path))
    } else {
        Err(FpgadError::Argument(format!(
            "Could not find {source_path:?} inside {firmware_path:?}"
        )))
    }
}

#[cfg(test)]
mod test_make_firmware_pair {
    use crate::comm::dbus::make_firmware_pair;
    use crate::error::FpgadError;
    use googletest::prelude::*;
    use rstest::*;
    use std::path::PathBuf;

    #[gtest]
    #[rstest]
    #[case::all_good(
        "/lib/firmware/file.bin",
        "/lib/firmware/",
        "/lib/firmware/",
        "file.bin"
    )]
    #[case::no_fw_path("/lib/firmware/file.bin", "", "/lib/firmware/", "file.bin")]
    #[case::no_fw_path_no_file("/lib/firmware/", "", "/lib/", "firmware")]
    fn should_pass(
        #[case] source: &str,
        #[case] fw_path: &str,
        #[case] exp_prefix: &str,
        #[case] exp_suffix: &str,
    ) {
        let result = make_firmware_pair(&PathBuf::from(source), &PathBuf::from(fw_path));
        assert_that!(
            result,
            ok(eq(&(PathBuf::from(exp_prefix), PathBuf::from(exp_suffix))))
        );
    }

    #[gtest]
    #[rstest]
    #[case::no_file(
        "/lib/firmware/",
        "/lib/firmware/",
        err(displays_as(contains_substring("The resulting filename from stripping")))
    )]
    #[case::not_in_dir(
        "/lib/firmware/file.bin",
        "/snap/x1/data/file.bin",
        err(displays_as(contains_substring("Could not find")))
    )]
    fn should_fail<M: for<'a> Matcher<&'a std::result::Result<(PathBuf, PathBuf), FpgadError>>>(
        #[case] source: &str,
        #[case] fw_path: &str,
        #[case] condition: M,
    ) {
        let result = make_firmware_pair(&PathBuf::from(source), &PathBuf::from(fw_path));
        assert_that!(&result, condition);
    }
}
