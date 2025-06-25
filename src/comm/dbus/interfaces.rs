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

use crate::error::FpgadError;
use crate::platforms::platform::{Fpga, OverlayHandler, Platform, new_platform};
use log::trace;
use std::path::{Path, PathBuf};
use zbus::fdo;
use zbus::interface;

pub struct StatusInterface {}
pub struct ControlInterface {}

fn validate_device_handle(device_handle: &str) -> Result<(), FpgadError> {
    if device_handle.is_empty() || !device_handle.is_ascii() {
        return Err(FpgadError::Argument(format!(
            "{} is invalid name for fpga device.\
                fpga name must be compliant with sysfs rules.",
            device_handle
        )));
    }
    if !PathBuf::from(format!("/sys/class/fpga_manager/{}/", device_handle)).exists() {
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
        trace!("get_fpga_state called with name: {}", device_handle);
        validate_device_handle(device_handle)?;
        Ok(new_platform(device_handle).fpga(device_handle)?.state()?)
    }

    async fn get_fpga_flags(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_flags called with name: {}", device_handle);
        validate_device_handle(device_handle)?;
        Ok(new_platform(device_handle)
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
        Ok(new_platform(device_handle)
            .overlay_handler(overlay_handle)?
            .status()?)
    }
}

#[interface(name = "com.canonical.fpgad.control")]
impl ControlInterface {
    async fn set_fpga_flags(
        &self,
        device_handle: &str,
        flags: isize,
    ) -> Result<String, fdo::Error> {
        trace!(
            "set_fpga_flags called with name: {} and flags: {}",
            device_handle, flags
        );
        validate_device_handle(device_handle)?;
        new_platform(device_handle)
            .fpga(device_handle)?
            .set_flags(flags)?;
        Ok(format!("Flags set to {} for {}", flags, device_handle))
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
                "{} is not a valid path to a bitstream file.",
                bitstream_path_str
            )));
        }
        new_platform(device_handle)
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

        let platform = new_platform(device_handle);
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
        let platform = new_platform(device_handle);
        let overlay_handler = platform.overlay_handler(overlay_handle)?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.remove_overlay()?;
        Ok(format!(
            "{overlay_handle} removed by deleting {overlay_fs_path:?}"
        ))
    }
}
