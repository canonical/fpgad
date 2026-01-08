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

use zbus::{Result, proxy};
#[proxy(
    default_service = "com.canonical.fpgad",
    interface = "com.canonical.fpgad.status",
    default_path = "/com/canonical/fpgad/status"
)]
pub trait Status {
    async fn get_fpga_state(&self, platform_string: &str, device_handle: &str) -> Result<String>;
    async fn get_fpga_flags(&self, platform_string: &str, device_handle: &str) -> Result<String>;
    async fn get_overlay_status(
        &self,
        platform_string: &str,
        overlay_handle: &str,
    ) -> Result<String>;
    async fn get_overlays(&self) -> Result<String>;
    async fn get_platform_type(&self, device_handle: &str) -> Result<String>;
    async fn get_platform_types(&self) -> Result<String>;
    async fn read_property(&self, property_path_str: &str) -> Result<String>;
}
