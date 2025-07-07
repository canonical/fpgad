
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
use crate::platforms::platform::{
    Fpga, OverlayHandler, Platform, list_fpga_managers, platform_for_device, read_compatible_string,
};
use crate::system_io::validate_device_handle;
use log::{error, trace};
use std::path::Path;
use zbus::fdo;
use zbus::interface;

pub struct StatusInterface {}
pub struct ControlInterface {}
pub struct ConfigureInterface {}

fn validate_device_handle(device_handle: &str) -> Result<(), FpgadError> {
    if device_handle.is_empty() || !device_handle.is_ascii() {
        return Err(FpgadError::Argument(format!(
            "{} is invalid name for fpga device.\
                fpga name must be compliant with sysfs rules.",
            device_handle
        )));
    }
    if !PathBuf::from(config::OVERLAY_CONTROL_DIR)
        .join(device_handle)
        .exists()
    {
        return Err(FpgadError::Argument(format!(
            "Device {} not found.",
            device_handle
        )));
    };
    Ok(())
}

#[interface(name = "com.canonical.fpgad.status")]
impl StatusInterface {
    async fn get_fpga_state(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_state called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        Ok(platform_for_device(device_handle)?
            .fpga(device_handle)?
            .state()?)
    }

    async fn get_fpga_flags(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_flags called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        Ok(platform_for_device(device_handle)?
            .fpga(device_handle)?
            .flags()
            .map(|flags| flags.to_string())?)
    }

    async fn get_overlay_status(
        &self,
        device_handle: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "get_overlay_status called with device_handle: {device_handle} and overlay_handle:\
             {overlay_handle}"
        );
        validate_device_handle(device_handle)?;
        Ok(platform_for_device(device_handle)?
            .overlay_handler(overlay_handle)?
            .status()?)
    }

    async fn get_platform_type(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_platform_type called with device_handle: {device_handle}");
        validate_device_handle(device_handle)?;
        let ret_string = read_compatible_string(device_handle)?;
        Ok(ret_string.to_string())
    }

    async fn get_platform_types(&self) -> Result<String, fdo::Error> {
        trace!("get_platform_types called");
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

    async fn get_platform_name(&self, _device_handle: &str) -> Result<String, fdo::Error> {
        todo!()
    }
}

#[interface(name = "com.canonical.fpgad.control")]
impl ControlInterface {
    async fn set_fpga_flags(&self, device_handle: &str, flags: u32) -> Result<String, fdo::Error> {
        trace!("set_fpga_flags called with name: {device_handle} and flags: {flags}");
        validate_device_handle(device_handle)?;
        platform_for_device(device_handle)?
            .fpga(device_handle)?
            .set_flags(flags)?;
        Ok(format!("Flags set to {flags} for {device_handle}"))
    }

    async fn write_bitstream_direct(
        &self,
        device_handle: &str,
        bitstream_path_str: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "load_firmware called with name: {device_handle} and path_str: {bitstream_path_str}"
        );
        validate_device_handle(device_handle)?;
        let path = Path::new(bitstream_path_str);
        if !path.exists() || path.is_dir() {
            return Err(fdo::Error::InvalidArgs(format!(
                "{bitstream_path_str} is not a valid path to a bitstream file."
            )));
        }
        platform_for_device(device_handle)?
            .fpga(device_handle)?
            .load_firmware(path)?;
        Ok(format!("{bitstream_path_str} loaded to {device_handle}"))
    }

    async fn apply_overlay(
        &self,
        device_handle: &str,
        overlay_handle: &str,
        overlay_source_path: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "apply_overlay called with device_handle:{device_handle}, overlay_handle: \
            {overlay_handle} and overlay_path: {overlay_source_path}",
        );
        validate_device_handle(device_handle)?;

        let platform = platform_for_device(device_handle)?;
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.apply_overlay(Path::new(overlay_source_path))?;
        Ok(format!(
            "{overlay_source_path} loaded via {overlay_fs_path:?}"
        ))
    }

    async fn remove_overlay(
        &self,
        device_handle: &str,
        overlay_handle: &str,
    ) -> Result<String, fdo::Error> {
        trace!(
            "remove_overlay called with device_handle: {device_handle} and overlay_handle:\
             {overlay_handle}"
        );
        validate_device_handle(device_handle)?;
        let platform = platform_for_device(device_handle)?;
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.remove_overlay()?;
        Ok(format!(
            "{overlay_handle} removed by deleting {overlay_fs_path:?}"
        ))
    }
}

fn write_firmware_source_dir(new_path: &str) -> Result<(), FpgadError> {
    trace!(
        "Writing fw prefix {} to {}",
        new_path,
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_write(fw_lookup_override, false, new_path)
}

fn read_firmware_source_dir() -> Result<(String), FpgadError> {
    trace!(
        "Reading fw prefix from {}",
        config::FIRMWARE_LOC_CONTROL_PATH
    );
    let fw_lookup_override = Path::new(config::FIRMWARE_LOC_CONTROL_PATH);
    fs_read(fw_lookup_override)
}

pub fn set_firmware_source_dir(new_path: &str) -> Result<(), FpgadError> {
    // TODO: checks for exist?
    write_firmware_source_dir(new_path)
}

#[interface(name = "com.canonical.fpgad.configure")]
impl ConfigureInterface {
    async fn get_firmware_source_dir(&self) -> Result<String, fdo::Error> {
        trace!("get_firmware_source_dir called");
        Ok(read_firmware_source_dir()?)
    }
    async fn set_firmware_source_dir(&self, new_path: &str) -> Result<String, fdo::Error> {
        trace!("set_firmware_source_dir called with prefix: {new_path}");
        set_firmware_source_dir(new_path)?;
        Ok(format!("firmware_source_dir set to {new_path}"))
    }
}