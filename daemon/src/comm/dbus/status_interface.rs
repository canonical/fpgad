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

use crate::config;
use crate::platforms::platform::{list_fpga_managers, read_compatible_string};
use crate::platforms::platform::{platform_for_known_platform, platform_from_compat_or_device};

use crate::comm::dbus::{fs_read_property, validate_device_handle};
use crate::error::FpgadError;
use crate::system_io::fs_read_dir;
use log::{error, info};
use zbus::{fdo, interface};

pub struct StatusInterface {}

#[interface(name = "com.canonical.fpgad.status")]
impl StatusInterface {
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

    async fn get_overlay_status(
        &self,
        platform_compat_str: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        info!(
            "get_overlay_status called with platform_compat_str: {platform_compat_str} and overlay_handle:\
             {overlay_handle}"
        );
        if overlay_handle.is_empty() {
            return Err(FpgadError::Argument(
                "An overlay handle is required. Provided overlay handle is empty.".into(),
            )
            .into());
        }
        Ok(platform_for_known_platform(platform_compat_str)?
            .overlay_handler(overlay_handle)?
            .status()?)
    }

    async fn get_overlays(&self) -> Result<String, fdo::Error> {
        info!("get_overlays called");
        let overlay_handles = fs_read_dir(config::OVERLAY_CONTROL_DIR.as_ref())?;
        Ok(overlay_handles.join("\n"))
    }

    async fn get_platform_type(&self, device_handle: &str) -> Result<String, fdo::Error> {
        info!("get_platform_type called with device_handle: {device_handle}");
        validate_device_handle(device_handle)?;
        let ret_string = read_compatible_string(device_handle)?;
        Ok(ret_string.to_string())
    }

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

    /// use to read a device property from /sys/class/fpga_manager/<device>/** that does not have a specific interface
    async fn read_property(&self, property_path_str: &str) -> Result<String, fdo::Error> {
        info!("read_property called with property_path_str: {property_path_str}");
        Ok(fs_read_property(property_path_str)?)
    }
}
