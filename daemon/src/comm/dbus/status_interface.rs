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

//! D-Bus status interface (`com.canonical.fpgad.status`) for querying FPGA state.
//!
//! The [`StatusInterface`] provides asynchronous read-only methods to query FPGA device state,
//! overlay status, and platform type information.
//! All methods return `Result<String, fdo::Error>` and communicate with the daemon over D-Bus.
//! Error strings are prefixed with the relevant `FpgadError` type, e.g.
//! `FpgadError::Argument: <message>`. See [`crate::comm::dbus`] for a higher-level overview.
//!
//! # Methods
//!
//! | Method | D-Bus signature summary | Description |
//! |--------|------------------------|-------------|
//! | `get_status_message` | `(platform_string)` | Full human-readable status of all FPGA devices and overlays on the platform |
//! | `get_fpga_state` | `(platform_string, device_handle)` | State of a single FPGA device, e.g. `"operating"` |
//! | `get_overlay_status` | `(platform_string, overlay_handle)` | Status of a specific device-tree overlay, e.g. `"applied"` |
//! | `get_overlays` | `()` | Newline-separated list of all overlay handles currently present |
//! | `get_platform_type` | `(device_handle)` | Platform compatibility string for a single FPGA device, e.g. `"xlnx,zynqmp-pcap-fpga"` |
//! | `get_platform_types` | `()` | All FPGA devices and their compat strings, one per line as `device:compat\n` |
//! | `xilinx_sys` | `(sub_cmd, path_str)` | Low-level read from FPGA manager sysfs properties — see [`ReadSubCommand`](crate::platforms::xilinx_sys::ReadSubCommand) |
//!
//! ## `xilinx_sys` status sub-commands
//!
//! The `xilinx_sys` method dispatches on `sub_cmd`; see [`ReadSubCommand`](crate::platforms::xilinx_sys::ReadSubCommand) for the full enum available via this (status) interface.
//!
//! | `sub_cmd` | `path_str` | Returns |
//! |-----------|-----------|---------|
//! | `"read_flags"` | Device handle or full sysfs path to flags, e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags` | Current programming flags as a string |
//! | `"read_property"` | Full sysfs path under `/sys/class/fpga_manager/` | Contents of the sysfs property file |
use crate::comm::dbus::validate_device_handle;
use crate::config;
use crate::error::FpgadError;
use crate::platforms::platform::{list_fpga_managers, read_compatible_string};
use crate::platforms::platform::{platform_for_known_platform, platform_from_compat_or_device};
use crate::platforms::xilinx_sys::xilinx_sys_read_handler;
use crate::system_io::fs_read_dir;
use log::{error, info};
use zbus::{fdo, interface};

/// Instance of a [zbus::object_server::Interface] constructed using the [zbus::interface] macro.
pub struct StatusInterface {}

/// See [crate::comm::dbus] for a summary of this interface's methods, or
/// [crate::comm::dbus::status_interface] for a summary of this interface in general.
#[interface(name = "com.canonical.fpgad.status")]
impl StatusInterface {
    async fn get_status_message(&self, platform_string: &str) -> Result<String, fdo::Error> {
        info!("get_fpga_state called with platform_string: {platform_string}");
        if platform_string.is_empty() {
            return Err(FpgadError::Argument("Empty platform string - cannot determine how to get status message because cannot determine platform to use without platform string".to_string()).into());
        }
        let platform = platform_from_compat_or_device(platform_string, "")?;
        Ok(platform.status_message()?)
    }

    /// The device handle (e.g., `fpga0`) of the FPGA.
    ///
    /// # Arguments
    ///
    /// * `platform_string`: Platform compatibility string.
    /// * `device_handle`: The device handle (e.g., `fpga0`) of the FPGA.
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – the current FPGA state (e.g., `"operating"`, `"unknown"`).
    /// * `Err(fdo::Error)` if the device handle is invalid or the platform cannot be determined.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let state = status_interface.get_fpga_state("xlnx,zynqmp-pcap-fpga", "fpga0").await?;
    /// assert_eq!(state, "operating");
    /// ```
    async fn get_fpga_state(
        &self,
        platform_string: &str,
        device_handle: &str,
    ) -> Result<String, fdo::Error> {
        info!("get_fpga_state called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        let platform = platform_from_compat_or_device(platform_string, device_handle)?;
        Ok(platform.fpga(device_handle)?.state()?)
    }

    /// Retrieve the status of a specific device-tree overlay.
    ///
    /// # Arguments
    ///
    /// * `platform_string`: Platform compatibility string.
    /// * `overlay_handle`: The handle (name) of the device-tree overlay to query.
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – Status of the overlay (e.g., `"loaded"`, `"unapplied"`).
    /// * `Err(fdo::Error)`  If the overlay handle is empty or not found.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let status = status_interface.get_overlay_status(
    ///     "xlnx,zynqmp-pcap-fpga",
    ///     "my_overlay"
    /// ).await?;
    /// assert_eq!(status, "applied");
    /// ```
    pub async fn get_overlay_status(
        &self,
        platform_string: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        info!(
            "get_overlay_status called with platform_string: {platform_string} and overlay_handle:\
             {overlay_handle}"
        );
        Ok(platform_for_known_platform(platform_string)?
            .overlay_handler(overlay_handle)?
            .status()?)
    }

    /// Retrieve a list of all available device-tree overlay handles (names of the device-tree
    /// overlay directories present on the system)
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – List of existing overlay handles, separated by newline (`\n`).
    /// * `Err(fdo::Error)` - `FpgadError::IOReadDir` if dir cannot be read
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let overlays = status_interface.get_overlays().await?;
    /// ```
    ///
    /// # Example Ok(String) values
    /// ## no overlays present:
    /// ```rust,no_run
    /// assert_eq!(overlays, "");
    /// ```
    ///
    /// ## One overlay present:
    /// ```rust,no_run
    /// assert_eq!(overlays, "my_overlay");
    /// ```
    /// ## Multiple overlays present:
    /// ```rust,no_run
    /// assert_eq!(overlays, "my_overlay1\nmy_overlay2\n...\nmy_overlayN");
    /// ```
    ///
    async fn get_overlays(&self) -> Result<String, fdo::Error> {
        info!("get_overlays called");
        let overlay_handles = fs_read_dir(config::OVERLAY_CONTROL_DIR.as_ref())?;
        Ok(overlay_handles.join("\n"))
    }

    /// Get the compatibility string for a single FPGA device.
    ///
    /// # Arguments
    ///
    /// * `device_handle`: FPGA device handle.
    ///
    /// # Returns: `Result<String, Error>`
    /// * `Ok(String)` – Device's platform compatibility string.
    /// * `Err(fdo::Error)` if device validation or reading the compatible string fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// let platform_string = status_interface.get_platform_type("fpga0").await?;
    /// assert_eq!(platform_string, "xlnx,zynqmp-pcap-fpga");
    /// ```
    ///
    async fn get_platform_type(&self, device_handle: &str) -> Result<String, fdo::Error> {
        info!("get_platform_type called with device_handle: {device_handle}");
        validate_device_handle(device_handle)?;
        let ret_string = read_compatible_string(device_handle)?;
        Ok(ret_string.to_string())
    }

    /// Get a list of all FPGA devices and their platform types.
    ///
    /// # Returns: `Result<String, Error>`
    /// * `Ok(String)` – Each line formatted as `<device_handle>:<platform_string>\n`.
    ///    Devices without a valid string appear as
    ///   `<device_handle>:\n`.
    /// * `Err(fdo::Error)` if reading FPGA managers directory or compatible strings fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use crate::comm::dbus::StatusInterface
    /// # let status_interface = StatusInterface()
    /// let platform_map = status_interface.get_platform_types().await?;
    /// ```
    /// If only one device:
    /// ```rust,no_run
    /// assert_eq!(platform_map, "fpga0:xlnx,zynqmp-pcap-fpga\n");
    /// ```
    /// or for multiple devices
    /// ```rust,no_run
    /// assert_eq!(platform_map, "fpga0:xlnx,zynqmp-pcap-fpga\nfpga1:xlnx,versal-fpga\n");
    /// ```
    ///
    async fn get_platform_types(&self) -> Result<String, fdo::Error> {
        info!("get_platform_types called");
        let mut ret_string = String::new();
        let devices = list_fpga_managers()?;
        for device_handle in devices {
            if let Ok(compat_string) = read_compatible_string(&device_handle) {
                ret_string += format!("{device_handle}:{compat_string}\n").as_str();
            } else {
                error!("Failed to get string for {device_handle}");
                ret_string += format!("{device_handle}:\n").as_str();
            }
        }
        Ok(ret_string)
    }

    /// Entrypoint for xilinx_sys platform specific operations.
    ///
    /// # Arguments
    ///
    /// * `sub_cmd` - The read operation to perform - see [`crate::platforms::xilinx_sys::ReadSubCommand`]
    /// * `path_str` - Device handle or full sysfs path to flags for `read_flags` (e.g. `fpga0` or `/sys/class/fpga_manager/fpga0/flags`), or sysfs property path for `read_property`.
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – The property value or flags as a string.
    /// * `Err(fdo::Error)` – If the `sub_cmd` is unrecognised or the path is invalid / outside
    ///   the allowed directory.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// // Read the FPGA manager name
    /// let name = status_interface
    ///     .xlnx_sys("read_property", "/sys/class/fpga_manager/fpga0/name")
    ///     .await?;
    ///
    /// // Read the current programming flags for fpga0
    /// let flags = status_interface
    ///     .xlnx_sys("read_flags", "fpga0")
    ///     .await?;
    /// ```
    async fn xlnx_sys(&self, sub_cmd: &str, path_str: &str) -> Result<String, fdo::Error> {
        info!("xlnx_sys (read) called with sub_cmd: {sub_cmd}, path_str: {path_str}");
        xilinx_sys_read_handler(sub_cmd, path_str)
    }
}
