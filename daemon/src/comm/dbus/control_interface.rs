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

//! D-Bus control interface (`com.canonical.fpgad.control`) for modifying FPGA state.
//!
//! The [`ControlInterface`] provides asynchronous methods to load bitstreams, apply and remove
//! device-tree overlays, and interact with vendor-specific FPGA managers.
//! All methods return `Result<String, fdo::Error>` and communicate with the daemon over D-Bus.
//! Error strings are prefixed with the relevant `FpgadError` type, e.g.
//! `FpgadError::Argument: <message>`. See [`crate::comm::dbus`] for a higher-level overview.
//!
//! # Methods
//!
//! | Method | D-Bus signature summary | Description |
//! |--------|------------------------|-------------|
//! | `write_bitstream_direct` | `(platform_string, device_handle, bitstream_path, firmware_lookup_path)` | Load a bitstream directly to an FPGA (no device-tree changes) |
//! | `apply_overlay` | `(platform_string, overlay_handle, overlay_source_path, firmware_lookup_path)` | Apply a device-tree overlay to trigger a bitstream load and driver probe |
//! | `remove_overlay` | `(platform_string, overlay_handle)` | Remove a previously applied device-tree overlay |
//! | `remove_bitstream` | `(platform_string, device_handle, bitstream_handle)` | Remove the currently loaded bitstream from an FPGA device |
//! | `xilinx_sys` | `(sub_cmd, path_str, value_str)` | Low-level write to FPGA manager sysfs properties — see [`WriteSubCommand`](crate::platforms::xilinx_sys::WriteSubCommand) |
//! | `dfx_mgr` | `(cmd_string)` | Passthrough to `dfx-mgr-client` (requires `dfx-mgr` snap component) |
//!
//! ## `xilinx_sys` control sub-commands
//!
//! The `xilinx_sys` method dispatches on `sub_cmd`; see [`WriteSubCommand`](crate::platforms::xilinx_sys::WriteSubCommand) for the full enum available via this (control) interface.
//!
//! | `sub_cmd` | `path_str` | `value_str` |
//! |-----------|-----------|-------------|
//! | `"write_flags"` | Device handle or full sysfs path to flags, e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags` | Hex `u32` with or without `0x` prefix (e.g. `0x20` or `20`, both = decimal 32) |
//! | `"write_property"` | Full sysfs path under `/sys/class/fpga_manager/` | String payload |
//! | `"write_property_bytes"` | Full sysfs path under `/sys/class/fpga_manager/` | Hex byte string, e.g. `deadbeef` |

use crate::comm::dbus::validate_device_handle;
use crate::error::FpgadError;
use crate::platforms::platform::{platform_for_known_platform, platform_from_compat_or_device};
use crate::platforms::xilinx_sys::xilinx_sys_write_handler;
#[cfg(feature = "xilinx-dfx-mgr")]
use crate::softeners::xilinx_dfx_mgr::xilinx_dfx_mgr_helpers::run_dfx_mgr;
use log::{info, trace};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard, OnceCell};
use zbus::{fdo, interface};

/// A mutex lock which implicitly inhibits asynchronous control of the firmware search path.
/// It does not lock other applications out of controlling the firmware search path, it only stops
/// multiple FPGAd calls from changing it while a load is being attempted.
/// See [get_write_lock_guard] for more information on using this lock.
static WRITE_LOCK: OnceCell<Arc<Mutex<()>>> = OnceCell::const_new();

/// A helper function to gain control of the [WRITE_LOCK] lock.
/// This lock is functional for as long as the returned variable is in scope.
///
/// # Examples
///
/// Drop the lock by returning
/// ```rust
/// async fn foo() -> ... {
/// let _guard = get_write_lock_guard().await;
/// ... // do stuff
/// }  // return drops _guard and therefore unlocks
/// ```
///
/// Reduce the lock’s lifetime by constraining it to an inner scope.
/// ```rust
/// async fn foo() -> ... {
/// ... // outer scope stuff
///
/// {
///     let _guard = get_write_lock_guard().await;
///     ... // do inner scope stuff that needs the lock
/// }  // leaving scope drops _guard and therefore unlocks
///
/// ... // more outer scope stuff
/// }  // return releases lock
/// ```
async fn get_write_lock_guard() -> MutexGuard<'static, ()> {
    let lock = WRITE_LOCK
        .get_or_init(|| async { Arc::new(Mutex::new(())) })
        .await;
    lock.lock().await
}

/// Instance of a [zbus::object_server::Interface] constructed using the [zbus::interface] macro.
pub struct ControlInterface {}

/// See [crate::comm::dbus] for a summary of this interface's methods, or
/// [crate::comm::dbus::control_interface] for a summary of this interface in general.
#[interface(name = "com.canonical.fpgad.control")]
impl ControlInterface {
    /// Trigger a bitstream-only load of a bitstream to a given FPGA device (i.e. no device-tree changes or driver installation).
    ///
    /// # Arguments
    ///
    /// * `platform_string`: Platform compatibility string.
    /// * `device_handle`: FPGA device handle.
    /// * `bitstream_path_str`: Path to the bitstream file.
    /// * `firmware_lookup_path`: Path to resolve overlay firmware or empty string
    ///      (automatically uses the parent dir of `bitstream_path_str`).
    ///
    /// # Returns: `Result<String, Error>`
    /// * `Ok(String)` – Confirmation message including source and target.
    /// * `Err(fdo::Error)` On device validation, file, or firmware write errors.
    ///
    /// **Notes:**
    ///
    /// Acquires an internal write lock on the firmware search path to ensure that no other write
    /// command redirects the search path before loading is complete.
    /// See [get_write_lock_guard] for more details.
    ///
    /// # Examples
    ///
    /// Specifying both `device_handle` and `firmware_lookup_path`
    /// ```rust
    /// let result = control_interface
    ///     .write_bitstream_direct(
    ///         "xlnx,zynqmp-pcap-fpga",
    ///         "fpga0",
    ///         "/snap/my_snap/data/my_package/my_bitstream.bit.bin",
    ///         "/snap/my_snap/data/my_package/",
    ///     )
    ///     .await?;
    /// assert_eq!(result, "/snap/my_snap/data/my_package/my_bitstream.bit.bin loaded to fpga0 using\
    ///                   firmware lookup path: '/snap/my_snap/data/my_package/'");
    /// ```
    ///
    /// Without specifying `device_handle` or `firmware_lookup_path`
    /// ```rust
    /// let result = control_interface
    ///     .write_bitstream_direct(
    ///         "xlnx,zynqmp-pcap-fpga",
    ///         "",
    ///         "/snap/my_snap/data/my_package/my_bitstream.bit.bin",
    ///         "",
    ///     )
    ///     .await?;
    /// assert_eq!(result, "/snap/my_snap/data/my_package/my_bitstream.bit.bin loaded to fpga0 using\
    ///                   firmware lookup path: '/snap/my_snap/data/my_package/'");
    /// ```
    ///
    async fn write_bitstream_direct(
        &self,
        platform_string: &str,
        device_handle: &str,
        bitstream_path_str: &str,
        firmware_lookup_path: &str,
    ) -> Result<String, fdo::Error> {
        info!("load_firmware called with name: {device_handle} and path_str: {bitstream_path_str}");
        validate_device_handle(device_handle)?;
        let path = Path::new(bitstream_path_str);
        let lookup = Path::new(firmware_lookup_path);
        let _guard = get_write_lock_guard().await;
        trace!("Got write lock.");
        let platform = platform_from_compat_or_device(platform_string, device_handle)?;
        Ok(platform.fpga(device_handle)?.load_firmware(path, lookup)?)
    }

    /// Apply a device-tree overlay to trigger a bitstream load and driver probe events.
    ///
    /// # Arguments
    ///
    /// * `platform_string`: Platform compatibility string.
    /// * `overlay_handle`: Handle (arbitrary name) of the overlay to apply.
    /// * `overlay_source_path`: Path to the overlay source file.
    /// * `firmware_lookup_path`: Path to resolve overlay firmware or empty string
    ///     (automatically uses the parent dir of `overlay_source_path`).
    ///
    /// # Returns: `Result<String, Error>`
    /// -  `Ok(String)` – Confirmation message including applied overlay and firmware prefix.
    /// - `Err(fdo::Error)` if overlay or platform cannot be accessed.
    ///
    /// **Notes:**
    ///
    /// * Acquires an internal write lock on the firmware search path to ensure that no other write
    ///     command redirects the search path before loading is complete.
    ///     See [get_write_lock_guard] for more details.
    /// * Checks both the overlay's `path` and `status` attributes to ensure the overlay was applied.
    ///
    /// # Examples
    ///
    /// Specifying `overlay_handle` and `firmware_lookup_path`
    /// ```rust
    /// let result = control_interface
    ///     .apply_overlay(
    ///         "xlnx,zynqmp-pcap-fpga",
    ///         "my_overlay",
    ///         "/snap/my_snap/data/my_package/my_overlay.dtbo",
    ///         "/snap/my_snap/data/my_package/",
    ///     )
    ///     .await?;
    /// assert_eq!(
    ///     result,
    ///     "/snap/my_snap/data/my_package/my_overlay.dtbo loaded via \
    ///      /sys/kernel/config/device-tree/overlays/my_overlay using firmware lookup path: ' \
    ///      /snap/my_snap/data/my_package/'",
    /// );
    /// ```
    ///
    /// Without specifying `firmware_lookup_path`
    /// ```rust
    /// let result = control_interface
    ///     .apply_overlay(
    ///         "xlnx,zynqmp-pcap-fpga",
    ///         "my_overlay",
    ///         "/snap/my_snap/data/my_package/my_overlay.dtbo",
    ///         "",
    ///     )
    ///     .await?;
    /// assert_eq!(
    ///     result,
    ///     "/snap/my_snap/data/my_package/my_overlay.dtbo loaded via \
    ///      /sys/kernel/config/device-tree/overlays/my_overlay using firmware lookup path: ' \
    ///      /snap/my_snap/data/my_package/'"
    /// );
    /// ```
    async fn apply_overlay(
        &self,
        platform_string: &str,
        overlay_handle: &str,
        overlay_source_path: &str,
        firmware_lookup_path: &str,
    ) -> Result<String, fdo::Error> {
        info!(
            "apply_overlay called with platform_string: {platform_string}, overlay_handle: \
            {overlay_handle} and overlay_path: {overlay_source_path}",
        );
        let _guard = get_write_lock_guard().await;
        trace!("Got write lock.");
        let platform = platform_for_known_platform(platform_string)?;
        let overlay_handler = platform.overlay_handler(overlay_handle)?;

        Ok(overlay_handler.apply_overlay(
            Path::new(overlay_source_path),
            Path::new(firmware_lookup_path),
        )?)
    }

    /// Remove a previously applied overlay, identifiable by its `overlay_handle`.
    ///
    /// # Arguments
    ///
    /// * `platform_string`: Platform compatibility string.
    /// * `overlay_handle`: Handle of the overlay to remove.
    ///
    /// # Returns: `Result<String, Error>`
    /// *  `Ok(String)` – Confirmation message including overlay filesystem path.
    /// * `Err(fdo::Error)` if overlay or platform cannot be accessed.
    ///
    /// # Examples
    ///
    /// ```
    /// assert!(remove_overlay("xlnx,zynqmp-pcap-fpga", "my_overlay").await.is_ok());
    /// ```
    async fn remove_overlay(
        &self,
        platform_string: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        info!(
            "remove_overlay called with platform_string: {platform_string} and overlay_handle:\
             {overlay_handle}"
        );
        let platform = platform_for_known_platform(platform_string)?;
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let handle = match overlay_handle {
            "" => None,
            _ => Some(overlay_handle),
        };
        Ok(overlay_handler.remove_overlay(handle)?)
    }

    /// Remove a previously loaded bitstream, identifiable by its `bitstream_handle` or `slot`.
    ///
    /// # Arguments
    ///
    /// * `platform_string`: Platform compatibility string.
    /// * `device_handle`: FPGA device handle (e.g., `fpga0`).
    /// * `bitstream_handle`: Handle/slot of the bitstream to remove.
    ///
    /// # Returns: `Result<String, Error>`
    /// *  `Ok(String)` – Confirmation message including device and bitstream handle.
    /// * `Err(fdo::Error)` if device or platform cannot be accessed.
    ///
    /// # Examples
    ///
    /// ```
    /// assert!(remove_bitstream("xlnx,zynqmp-pcap-fpga", "fpga0", "").is_ok());
    /// ```
    async fn remove_bitstream(
        &self,
        platform_string: &str,
        device_handle: &str,
        bitstream_handle: &str,
    ) -> Result<String, fdo::Error> {
        info!(
            "remove_bitstream called with platform_string: {platform_string}, device_handle:\
             {device_handle} and bitstream_handle: {bitstream_handle}"
        );
        let platform = platform_from_compat_or_device(platform_string, device_handle)?;
        let fpga = platform.fpga(device_handle)?;
        let handle = match bitstream_handle {
            "" => None,
            _ => Some(bitstream_handle),
        };
        Ok(fpga.remove_firmware(handle)?)
    }

    /// Entrypoint for xilinx_sys platform specific operations.
    ///
    /// # Arguments
    ///
    /// * `sub_cmd` - The write operation to perform - see [`crate::platforms::xilinx_sys::WriteSubCommand`]
    /// * `path_str` - Device handle for `write_flags`, or sysfs property path for property writes
    /// * `value_str` - Value to write (flags value, string payload for `write_property`,
    ///   or raw byte string for `write_property_bytes`)
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – Confirmation message.
    /// * `Err(fdo::Error)` – If the `sub_cmd` is unrecognised, the path is invalid / outside the
    ///   allowed directory, or the write fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // Write programming flags
    /// control_interface.xlnx_sys("write_flags", "fpga0", "0x20").await?;
    ///
    /// // Write a string property
    /// control_interface.xlnx_sys(
    ///     "write_property",
    ///     "/sys/class/fpga_manager/fpga0/key",
    ///     "BADBADBADBAD",
    /// ).await?;
    ///
    /// // Write raw bytes
    /// control_interface.xlnx_sys(
    ///     "write_property_bytes",
    ///     "/sys/class/fpga_manager/fpga0/key",
    ///     "deadbeef",
    /// ).await?;
    /// ```
    async fn xlnx_sys(
        &self,
        sub_cmd: &str,
        path_str: &str,
        value_str: &str,
    ) -> Result<String, fdo::Error> {
        info!("xlnx_sys (write) called with sub_cmd: {sub_cmd}, path_str: {path_str}");
        xilinx_sys_write_handler(sub_cmd, path_str, value_str)
    }

    /// Entrypoint for dfx-mgr specific operations
    ///
    /// Allows the user to pass a command string directly to the `dfx-mgr-client` binary for
    /// otherwise unsupported actions
    ///
    /// This is a thin passthrough to the Xilinx DFX Manager client. The `cmd_string` is
    /// split on whitespace and forwarded as arguments to `dfx-mgr-client`.
    ///
    /// # Security
    ///
    /// All arguments are validated to prevent command injection and other security issues.
    /// Arguments containing shell metacharacters (`;`, `&`, `|`, `$`, etc.) are rejected.
    /// This is done despite `Command::new().args()` not invoking a shell.
    ///
    /// # Arguments
    ///
    /// * `cmd_string` - Space-separated arguments to pass to `dfx-mgr-client`
    ///   (e.g. `"-listPackage"` or `"-b my_bitstream.bit.bin -o my_overlay.dtbo"`)
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – stdout from `dfx-mgr-client` on success
    /// * `Err(fdo::Error)` – If arguments are invalid, `dfx-mgr-client` is not found,
    ///   the process fails, or the `xilinx-dfx-mgr` feature was not compiled in
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // List available DFX packages
    /// let result = control_interface.dfx_mgr("-listPackage").await?;
    ///
    /// // Load a bitstream into slot 0
    /// let result = control_interface.dfx_mgr("-load 0 my_design").await?;
    /// ```
    async fn dfx_mgr(&self, cmd_string: &str) -> Result<String, fdo::Error> {
        #[cfg(feature = "xilinx-dfx-mgr")]
        {
            use tokio::task;
            // Avoid borrowing from `cmd_string` across an await point by creating
            // owned Strings and moving them into the blocking task.
            let cmd_owned = cmd_string.to_string();
            let res = task::spawn_blocking(move || {
                let args: Vec<&str> = cmd_owned.split_whitespace().collect();

                if let Err(e) = validate_dfx_mgr_args(&args) {
                    return Err(FpgadError::Argument(format!(
                        "Invalid dfx-mgr arguments: {}",
                        e
                    )));
                }

                run_dfx_mgr(&args).map_err(|e| e.into())
            })
            .await
            .map_err(|e| FpgadError::Internal(format!("dfx-mgr-client subprocess failed: {e}")))?;

            match res {
                Ok(output) => {
                    info!("dfx-mgr command ran successfully!");
                    Ok(output)
                }
                Err(e) => {
                    info!("dfx-mgr command failed: {}", e);
                    Err(e.into())
                }
            }
        }

        #[cfg(not(feature = "xilinx-dfx-mgr"))]
        {
            let _ = cmd_string;
            Err(FpgadError::Feature(
                "Cannot use DfxMgr method - FPGAd was compiled without xilinx-dfx-mgr feature"
                    .into(),
            )
            .into())
        }
    }
}

/// Validate dfx-mgr arguments to prevent command injection and other security issues.
///
/// This function blocks shell metacharacters and overly long arguments to try to
/// prevent command injection attacks and other security issues.
///
/// # Security Considerations
///
/// Even though we use `Command::new().args()` which doesn't invoke a shell,
/// we validate inputs to:
/// 1. Prevent any potential vulnerabilities in dfx-mgr-client itself
/// 2. Give error messages for invalid input
///
/// # Arguments
///
/// * `args` - Slice of argument strings to validate
///
/// # Returns
/// * `Ok(())` - If all arguments are valid
/// * `Err(String)` - Error message describing the validation failure
#[cfg(feature = "xilinx-dfx-mgr")]
fn validate_dfx_mgr_args(args: &[&str]) -> Result<(), String> {
    // Shell metacharacters and other dangerous patterns that should never appear
    const DANGEROUS_CHARS: &[char] = &[
        ';', '&', '|', '$', '`', '\n', '\r', '<', '>', '(', ')', '{', '}', '[', ']', '\\', '\'',
        '"', '*', '?',
    ];

    for (index, arg) in args.iter().enumerate() {
        // Check for dangerous shell metacharacters
        if let Some(dangerous_char) = arg.chars().find(|c| DANGEROUS_CHARS.contains(c)) {
            return Err(format!(
                "Argument at position {} contains dangerous character '{}': \"{}\". \
                Shell metacharacters and special characters are not allowed in dfx-mgr commands.",
                index, dangerous_char, arg
            ));
        }

        // Prevent excessively long arguments (potential buffer overflow attacks)
        if arg.len() > 1024 {
            return Err(format!(
                "Argument at position {} is too long ({} characters). Maximum length is 1024 characters.",
                index,
                arg.len()
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod test_get_write_lock_guard {
    use crate::comm::dbus::control_interface::get_write_lock_guard;

    #[tokio::test]
    async fn test_get_write_lock_guard() {
        let _guard = get_write_lock_guard().await;
    }
}

#[cfg(test)]
#[cfg(feature = "xilinx-dfx-mgr")]
mod test_validate_dfx_mgr_args {
    use super::validate_dfx_mgr_args;

    #[test]
    fn test_validate_valid_commands() {
        // Valid single flag commands
        assert!(validate_dfx_mgr_args(&["-listPackage"]).is_ok());
        assert!(validate_dfx_mgr_args(&["-listSlot"]).is_ok());

        // Valid load command with slot and package name
        assert!(validate_dfx_mgr_args(&["-load", "0", "my_design"]).is_ok());

        // Valid remove command
        assert!(validate_dfx_mgr_args(&["-remove", "0"]).is_ok());

        // Package names with underscores, hyphens, dots
        assert!(validate_dfx_mgr_args(&["-load", "1", "my-package_v2.0"]).is_ok());

        // Paths with forward slashes and colons
        assert!(validate_dfx_mgr_args(&["-load", "0", "/path/to/package"]).is_ok());

        // Complex valid flag names
        assert!(validate_dfx_mgr_args(&["-list_Package"]).is_ok());
        assert!(validate_dfx_mgr_args(&["-loadPackage123"]).is_ok());
    }

    #[test]
    fn test_validate_shell_injection_attempts() {
        // Shell command chaining with semicolon
        let result = validate_dfx_mgr_args(&["-listPackage", ";", "rm", "-rf", "/"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Background command with ampersand
        let result = validate_dfx_mgr_args(&["-listPackage", "&", "sudo", "rm", "-rf", "/"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Pipe to another command
        let result = validate_dfx_mgr_args(&["-listPackage", "|", "grep", "secret"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Variable expansion attempt
        let result = validate_dfx_mgr_args(&["-load", "$HOME"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Command substitution with backticks
        let result = validate_dfx_mgr_args(&["`whoami`"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Redirect attempts
        let result = validate_dfx_mgr_args(&["-listPackage", ">", "/tmp/output"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        let result = validate_dfx_mgr_args(&["-listPackage", "<", "/etc/passwd"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Glob patterns
        let result = validate_dfx_mgr_args(&["-load", "*"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Quotes
        let result = validate_dfx_mgr_args(&["\"malicious\""]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        let result = validate_dfx_mgr_args(&["'malicious'"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));
    }

    #[test]
    fn test_validate_allows_special_characters() {
        // Characters that are not dangerous shell metacharacters should be allowed
        // This supports international use cases and various naming conventions
        assert!(validate_dfx_mgr_args(&["-load", "0", "package@version"]).is_ok());
        assert!(validate_dfx_mgr_args(&["-load", "0", "package+variant"]).is_ok());
        assert!(validate_dfx_mgr_args(&["-load", "0", "design#1"]).is_ok());
    }

    #[test]
    fn test_validate_allows_unicode() {
        // International characters should be allowed for file paths
        assert!(validate_dfx_mgr_args(&["-load", "0", "设计文件"]).is_ok()); // Chinese
        assert!(validate_dfx_mgr_args(&["-load", "0", "ファイル"]).is_ok()); // Japanese
        assert!(validate_dfx_mgr_args(&["-load", "0", "файл"]).is_ok()); // Cyrillic
    }

    #[test]
    fn test_validate_long_arguments() {
        // Argument that's too long (potential buffer overflow attempt)
        let long_arg = "a".repeat(1025);
        let result = validate_dfx_mgr_args(&["-load", "0", &long_arg]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("too long"));

        // Boundary case: exactly 1024 characters should be OK
        let boundary_arg = "a".repeat(1024);
        assert!(validate_dfx_mgr_args(&["-load", "0", &boundary_arg]).is_ok());
    }

    #[test]
    fn test_validate_newlines_and_control_chars() {
        // Newline injection
        let result = validate_dfx_mgr_args(&["-list\nPackage"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Carriage return
        let result = validate_dfx_mgr_args(&["-list\rPackage"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));
    }

    #[test]
    fn test_validate_parentheses_and_brackets() {
        // Subshell attempts
        let result = validate_dfx_mgr_args(&["$(whoami)"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        let result = validate_dfx_mgr_args(&["(ls)"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));

        // Arrays/brackets
        let result = validate_dfx_mgr_args(&["[test]"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("dangerous character"));
    }
}
