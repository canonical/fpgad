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

use crate::platforms::platform::Fpga;
use crate::platforms::platform::OverlayHandler;
use crate::platforms::platform::Platform;
use crate::platforms::platform::new_platform;
use crate::system_io::validate_device_handle;
use log::trace;
use zbus::{fdo, interface};

pub struct StatusInterface {}

#[interface(name = "com.canonical.fpgad.status")]
impl StatusInterface {
    async fn get_fpga_state(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_state called with name: {device_handle}");
        validate_device_handle(device_handle)?;
        Ok(new_platform(device_handle).fpga(device_handle)?.state()?)
    }

    async fn get_fpga_flags(&self, device_handle: &str) -> Result<String, fdo::Error> {
        trace!("get_fpga_flags called with name: {device_handle}");
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
