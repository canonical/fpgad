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
//! The `StatusInterface` provides asynchronous methods to query the state, flags, and overlays of FPGA devices on the system.
//! All methods return a `Result<String, fdo::Error>` and are designed for DBus usage.
//! If FPGAd raises the error, then the `fdo::Error` strings are prepended with the relevant FPGAd error type e.g. `FpgadError::Argument: <error text>`. See [crate::comm::dbus] for a summary of this interface's methods.
//!
use crate::config;
use crate::platforms::platform::{list_fpga_managers, read_compatible_string};
use crate::platforms::platform::{platform_for_known_platform, platform_from_compat_or_device};

use crate::comm::dbus::{fs_read_property, validate_device_handle};
use crate::error::FpgadError;
use crate::system_io::fs_read_dir;
use log::{error, info};
use zbus::{fdo, interface};

/// Instance of a [zbus::object_server::Interface] constructed using the [zbus::interface] macro.
pub struct StatusInterface {}

/// See [crate::comm::dbus] for a summary of this interface's methods, or
/// [crate::comm::dbus::status_interface] for a summary of this interface in general.
#[interface(name = "com.canonical.fpgad.status")]
impl StatusInterface {
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
    /// ```rust
    /// # use crate::comm::dbus::StatusInterface
    /// # let status_interface = StatusInterface()
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

    /// Retrieve the current flags set for a specified FPGA device as a hexadecimal ascii string
    /// (missing `0x` prefix).
    ///
    /// # Arguments
    ///
    /// * `platform_string`: Platform compatibility string.
    /// * `device_handle`: The device handle (e.g., `fpga0`) of the FPGA.
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – hexadecimal representation of flags without `0x` hex prefix (e.g. "20"
    /// for decimal value of 32)
    /// * `Err(fdo::Error)` on invalid handle or platform error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::comm::dbus::StatusInterface
    /// # let status_interface = StatusInterface()
    /// let flags = status_interface.get_fpga_flags("xlnx,zynqmp-pcap-fpga", "fpga0").await?;
    /// assert_eq!(flags, "20");
    /// ```
    async fn get_fpga_flags(
        &self,
        platform_string: &str,
        device_handle: &str,
    ) -> Result<String, fdo::Error> {
        info!("get_fpga_flags called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        let platform = platform_from_compat_or_device(platform_string, device_handle)?;
        Ok(platform
            .fpga(device_handle)?
            .flags()
            .map(|flags| flags.to_string())?)
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
    /// ```
    /// # use crate::comm::dbus::StatusInterface
    /// # let status_interface = StatusInterface()
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
        if overlay_handle.is_empty() {
            return Err(FpgadError::Argument(
                "An overlay handle is required. Provided overlay handle is empty.".into(),
            )
            .into());
        }
        Ok(platform_for_known_platform(platform_string)?
            .overlay_handler(overlay_handle)?
            .status()?)
    }

    /// Retrieve a list of all available device-tree overlay handles (names of the device-tree
    /// overlay directories present on the system)
    ///
    /// # Returns: `Result<String, fdo::Error>`
    /// * `Ok(String)` – List of existing overlay handles, separated by newline (`\n`).
    /// * `Err(fdo::Error)` If the overlay handle is empty or not found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::comm::dbus::StatusInterface
    /// # let status_interface = StatusInterface()
    /// let overlays = status_interface.get_overlays().await?;
    /// ```
    ///
    /// # Example Ok(String) values
    /// ## no overlays present:
    /// ```rust
    /// assert_eq!(overlays, "");
    /// ```
    ///
    /// ## One overlay present:
    /// ```text
    /// assert_eq!(overlays, "my_overlay");
    /// ```
    /// ## Multiple overlays present:
    /// ```rust
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
    /// ```rust
    /// # use crate::comm::dbus::StatusInterface
    /// # let status_interface = StatusInterface()
    /// let platform_string = status_interface.get_platform_type("").await?;
    /// assert_eq!(platform_string, "xlnx,zynqmp-pcap-fpga");
    /// ```
    ///
    /// ```rust
    /// # use crate::comm::dbus::StatusInterface
    /// # let status_interface = StatusInterface()
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
    /// # Arguments
    ///
    /// * `device_handle`: FPGA device handle.
    ///
    /// # Returns: `Result<String, Error>`
    /// * `Ok(String)` – Each line formatted as `<device_handle>:<platform_string>\n`.
    ///    Devices without a valid string appear as
    ///   `<device_handle>:\n`.
    /// * `Err(fdo::Error)` if device validation or reading the compatible string fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use crate::comm::dbus::StatusInterface
    /// # let status_interface = StatusInterface()
    /// let platform_map = status_interface.get_platform_types().await?;
    /// ```
    /// If only one device:
    /// ```
    /// assert_eq!(platform_map, "fpga0:xlnx,zynqmp-pcap-fpga\n");
    /// ```
    /// or for multiple devices
    /// ```rust
    /// assert_eq!(platform_map, "fpga0:xlnx,zynqmp-pcap-fpga\nfpga1:xlnx,versal-fpga\n");
    /// ````
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

    /// Read an arbitrary FPGA device property from `/sys/class/fpga_manager/<device>/`.
    ///
    /// # Arguments
    ///
    /// * `property_path_str`: Full path to the property file.
    ///
    /// # Returns: `Result<String, Error>`
    /// * `Ok(String)` – Contents of the property file.
    /// * `Err(fdo::Error)` If the property cannot be read.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let name = status_interface.
    ///     read_property("/sys/class/fpga_manager/fpga0/name")
    ///     .await?;
    /// assert_eq!(name, "Xilinx ZynqMP FPGA Manager\n");
    /// ```
    ///
    async fn read_property(&self, property_path_str: &str) -> Result<String, fdo::Error> {
        info!("read_property called with property_path_str: {property_path_str}");
        Ok(fs_read_property(property_path_str)?)
    }
}
