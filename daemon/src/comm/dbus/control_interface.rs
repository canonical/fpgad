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
//! The `ControlInterface` provides asynchronous methods to modify FPGA state, load bitstreams, and manage device tree overlays.
//! All methods return a `Result<String, fdo::Error>` and are designed for D-Bus usage.
//! If FPGAd raises the error, then the fdo::Error strings are prepended with the relevant FPGAd error type e.g. `FpgadError::Argument: <error text>`. See [crate::comm::dbus] for a summary of this interface's methods.
//!

use crate::comm::dbus::{make_firmware_pair, validate_device_handle, write_firmware_source_dir};
use crate::config::FPGA_MANAGERS_DIR;
use crate::error::FpgadError;
use crate::platforms::platform::{platform_for_known_platform, platform_from_compat_or_device};
use crate::system_io::{fs_write, fs_write_bytes};
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
    /// Set the flags for a specific FPGA device.
    ///
    /// # Arguments
    ///
    /// * `platform_string`: Platform compatibility string.
    /// * `device_handle`: FPGA device handle (e.g., `fpga0`).
    /// * `flags`: Bitmask flags to apply to the device.
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – Confirmation message, including the new flags in hex.
    /// * `Err(fdo::Error)` if device validation or flag setting fails.
    ///
    /// # Examples
    ///
    /// Specify device
    /// ```
    /// let result = control_interface
    ///     .set_fpga_flags("xlnx,zynqmp-pcap-fpga", "fpga0", 0x20)
    ///     .await?;
    /// assert_eq!(result, "Flags set to 0x20 for fpga0");
    /// ```
    ///
    /// Don't specify device (defaults to `"fpga0"`)
    /// ```rust
    /// let result = control_interface
    ///     .set_fpga_flags("xlnx,zynqmp-pcap-fpga", "", 0b100000)
    ///     .await?;
    /// assert_eq!(result, "Flags set to 0x20 for fpga0");
    /// ```
    async fn set_fpga_flags(
        &self,
        platform_string: &str,
        device_handle: &str,
        flags: u32,
    ) -> Result<String, fdo::Error> {
        info!("set_fpga_flags called with name: {device_handle} and flags: {flags}");
        validate_device_handle(device_handle)?;
        let platform = platform_from_compat_or_device(platform_string, device_handle)?;
        platform.fpga(device_handle)?.set_flags(flags)?;
        Ok(format!("Flags set to 0x{flags:X} for {device_handle}"))
    }

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
    /// write_bitstream_direct(
    ///     "xlnx,zynqmp-pcap-fpga",
    ///     "fpga0",
    ///     "/snap/my_snap/data/my_package/my_bitstream.bit.bin",
    ///     "/snap/my_snap/data/my_package/",
    /// )
    /// .await?;
    /// assert_eq!(result, "/snap/my_snap/data/my_package/my_bitstream.bit.bin loaded to fpga0 using\
    ///                   firmware lookup path: '/snap/my_snap/data/my_package/'");
    /// ```
    ///
    /// Without specifying `device_handle` or `firmware_lookup_path`
    /// ```rust
    /// write_bitstream_direct(
    ///     "xlnx,zynqmp-pcap-fpga",
    ///     "",
    ///     "/snap/my_snap/data/my_package/my_bitstream.bit.bin",
    ///     "",
    /// )
    /// .await?;
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
        let _guard = get_write_lock_guard().await;
        trace!("Got write lock.");
        let platform = platform_from_compat_or_device(platform_string, device_handle)?;
        let (prefix, suffix) = make_firmware_pair(path, Path::new(firmware_lookup_path))?;
        write_firmware_source_dir(&prefix.to_string_lossy())?;
        platform.fpga(device_handle)?.load_firmware(&suffix)?;
        Ok(format!(
            "{bitstream_path_str} loaded to {device_handle} using firmware lookup path: '\
         {prefix:?}'"
        ))
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
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        let (prefix, suffix) = make_firmware_pair(
            Path::new(overlay_source_path),
            Path::new(firmware_lookup_path),
        )?;
        write_firmware_source_dir(&prefix.to_string_lossy())?;
        overlay_handler.apply_overlay(&suffix)?;
        Ok(format!(
            "{overlay_source_path} loaded via {overlay_fs_path:?} using firmware lookup path: '\
         {prefix:?}'"
        ))
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
    /// assert!(remove_overlay("xlnx,zynqmp-pcap-fpga", "my_overlay").is_ok());
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
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.remove_overlay()?;
        Ok(format!(
            "{overlay_handle} removed by deleting {overlay_fs_path:?}"
        ))
    }

    /// Write a string value to an arbitrary FPGA device property.
    ///
    /// # Arguments
    ///
    /// * `property_path_str`: Full path under [FPGA_MANAGERS_DIR].
    /// * `data`: String data to write.
    ///
    /// # Returns: `Result<String, Error>`
    ///
    /// * `Ok(String)` – Confirmation of written data.
    /// * `Err(fdo::Error)` if path is outside FPGA managers, or if the writing failed for any
    ///     other reason
    /// **Notes:**
    ///
    /// * Path must be under [FPGA_MANAGERS_DIR] - determined at compile time.
    ///
    /// # Examples
    ///
    /// ```
    /// let result = control_interface
    ///     .write_property(
    ///         "xlnx,zynqmp-pcap-fpga",
    ///         "/sys/class/fpga_manager/fpga0/key",
    ///         "BADBADBADBAD")
    ///     .await?;
    /// assert_eq!(result, "BADBADBADBAD written to /sys/class/fpga_manager/fpga0/key");
    /// ```
    async fn write_property(
        &self,
        property_path_str: &str,
        data: &str,
    ) -> Result<String, fdo::Error> {
        info!("write_property called with property_path_str: {property_path_str} and data: {data}");
        let property_path = Path::new(property_path_str);
        if !property_path.starts_with(Path::new(FPGA_MANAGERS_DIR)) {
            return Err(fdo::Error::from(FpgadError::Argument(format!(
                "Cannot access property {property_path_str}: does not begin with {FPGA_MANAGERS_DIR}"
            ))));
        }
        fs_write(property_path, false, data)?;
        Ok(format!("{data} written to {property_path_str}"))
    }

    /// Write raw bytes to an arbitrary FPGA device property.
    ///
    /// # Arguments
    ///
    /// * `property_path_str`: Full path under [FPGA_MANAGERS_DIR].
    /// * `data`: Byte array to write.
    ///
    /// # Returns: `Result<String, Error>`
    ///
    /// * `Ok(String)` – Confirmation of written data.
    /// * `Err(fdo::Error)` if path is outside FPGA managers, or if the writing failed for any
    ///     other reason
    ///
    /// **Notes:**
    ///
    /// * Path must be under [FPGA_MANAGERS_DIR] - determined at compile time.
    ///
    /// # Examples
    ///
    /// ```
    /// let result = control_interface
    ///     .write_property(
    ///         "xlnx,zynqmp-pcap-fpga",
    ///         "/sys/class/fpga_manager/fpga0/key",
    ///         &[0xBA, 0xDB, 0xAD, 0xBA, 0xDB, 0xAD])
    ///     .await?;
    /// assert_eq!(result, "Byte string successfully written to /sys/class/fpga_manager/fpga0/key");
    /// ```
    async fn write_property_bytes(
        &self,
        property_path_str: &str,
        data: &[u8],
    ) -> Result<String, fdo::Error> {
        info!(
            "write_property called with property_path_str: {property_path_str} and data: {data:?}"
        );
        let property_path = Path::new(property_path_str);
        if !property_path.starts_with(Path::new(FPGA_MANAGERS_DIR)) {
            return Err(fdo::Error::from(FpgadError::Argument(format!(
                "Cannot access property {property_path_str}: does not begin with {FPGA_MANAGERS_DIR}"
            ))));
        }
        fs_write_bytes(property_path, false, data)?;
        Ok(format!(
            "Byte string successfully written to {property_path_str}"
        ))
    }
}

#[cfg(test)]
mod test_get_write_lock_guard {
    use crate::comm::dbus::control_interface::get_write_lock_guard;

    #[tokio::test]
    async fn test_get_write_lock_guard() {
        let _guard = get_write_lock_guard().await;
    }
}
