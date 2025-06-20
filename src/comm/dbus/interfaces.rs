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

use crate::platforms::platform::{Fpga, Platform, new_platform};
use log::trace;
use std::path::Path;
use zbus::fdo::Error;
use zbus::interface;

pub struct StatusInterface {}
pub struct ControlInterface {}

#[interface(name = "com.canonical.fpgad.status")]
impl StatusInterface {
    async fn get_fpga_state(&self, device_handle: &str) -> Result<String, Error> {
        trace!("get_fpga_state called with name: {}", device_handle);
        let mut platform = new_platform("");
        let fpga = platform.fpga(device_handle)?;
        Ok(fpga.state()?)
    }

    async fn get_fpga_flags(&self, device_handle: &str) -> Result<String, Error> {
        trace!("get_fpga_flags called with name: {}", device_handle);
        let mut platform = new_platform("");
        let fpga = platform.fpga(device_handle)?;
        Ok(fpga.flags().map(|flags| flags.to_string())?)
    }

    async fn get_overlay_status(&self, overlay_handle: &str) -> Result<String, Error> {
        trace!(
            "get_overlay_status called with overlay_handle: {}",
            overlay_handle
        );
        let mut platform = new_platform("");
        platform
            .overlay_handler()
            .set_overlay_fs_path(overlay_handle)?;
        Ok(platform.overlay_handler().status()?)
    }
}

#[interface(name = "com.canonical.fpgad.control")]
impl ControlInterface {
    async fn set_fpga_flags(&self, device_handle: &str, flags: isize) -> Result<String, Error> {
        trace!(
            "set_fpga_flags called with name: {} and flags: {}",
            device_handle, flags
        );
        let mut platform = new_platform("");
        let fpga = platform.fpga(device_handle)?;
        fpga.set_flags(flags)?;
        Ok(format!("Flags set to {} for {}", flags, device_handle))
    }

    async fn write_bitstream_direct(
        &self,
        device_handle: &str,
        bitstream_path_str: &str,
    ) -> Result<String, Error> {
        trace!(
            "load_firmware called with name: {} and path_str: {}",
            device_handle, bitstream_path_str
        );
        let mut platform = new_platform("");
        let fpga = platform.fpga(device_handle)?;
        let path = Path::new(bitstream_path_str);
        if !path.exists() || path.is_dir() {
            return Err(Error::InvalidArgs(format!(
                "{} is not a valid path to a bitstream file.",
                bitstream_path_str
            )));
        }
        fpga.load_firmware(path)?;
        Ok(format!("{bitstream_path_str} loaded to {device_handle}"))
    }

    async fn apply_overlay(
        &self,
        overlay_handle: &str,
        overlay_source_path: &str,
    ) -> Result<String, Error> {
        // TODO: this doesn't take target fpga device which means that we don't check
        // fpga0/state at all
        trace!(
            "apply_overlay called with overlay_handle: {} and overlay_path: {}",
            overlay_handle, overlay_source_path
        );
        let mut platform = new_platform("");
        let overlay_handler = platform.overlay_handler();
        overlay_handler.set_overlay_fs_path(overlay_handle)?;
        overlay_handler.set_source_path(Path::new(overlay_source_path))?;
        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.apply_overlay()?;
        Ok(format!(
            "{overlay_source_path} loaded via {overlay_fs_path:?}"
        ))
    }

    async fn remove_overlay(&self, overlay_handle: &str) -> Result<String, Error> {
        trace!(
            "remove_overlay called with overlay_handle: {}",
            overlay_handle
        );
        let mut platform = new_platform("");
        let overlay_handler = platform.overlay_handler();
        overlay_handler.set_overlay_fs_path(overlay_handle)?;

        let overlay_fs_path = overlay_handler.overlay_fs_path()?;
        overlay_handler.remove_overlay()?;
        Ok(format!(
            "{overlay_handle} removed by deleting {overlay_fs_path:?}"
        ))
    }
}
