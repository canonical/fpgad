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

use crate::proxies::control_proxy;
use zbus::Connection;

/// Sends the dbus command to load a bitstream
pub async fn call_load_bitstream(
    platform_str: &str,
    device_handle: &str,
    file_path: &str,
    firmware_lookup_path: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy
        .write_bitstream_direct(platform_str, device_handle, file_path, firmware_lookup_path)
        .await
}

/// Sends the dbus command to apply an overlay
pub async fn call_apply_overlay(
    platform: &str,
    file_path: &str,
    overlay_handle: &str,
    firmware_lookup_path: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy
        .apply_overlay(platform, overlay_handle, file_path, firmware_lookup_path)
        .await
}

/// Sends the dbus command to remove an overlay
pub async fn call_remove_overlay(
    device_handle: &str,
    overlay_handle: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy.remove_overlay(device_handle, overlay_handle).await
}
