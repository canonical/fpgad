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

use crate::proxies::status_proxy;
use std::collections::HashMap;
use zbus::Connection;

/// Sends the dbus command to get a list of overlays and parses it
pub async fn call_get_overlays() -> Result<Vec<String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let list_str = proxy.get_overlays().await?;
    let ret_list: Vec<String> = list_str.lines().map(|line| line.to_string()).collect();
    Ok(ret_list)
}

/// Sends the dbus command to get the state from an fpga device
pub async fn call_get_fpga_state(device_handle: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_fpga_state("", device_handle).await
}

/// Sends the dbus command to get the platform_compat_string for a given device
pub async fn call_get_platform_type(device_handle: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_platform_type(device_handle).await
}

/// Sends the dbus command to get the status string for a given overlay
pub async fn call_get_overlay_status(
    platform: &str,
    overlay_handle: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_overlay_status(platform, overlay_handle).await
}

/// parses the string from `get_platform_types` interface into a HashMap of
/// device: platform_compat_string
pub async fn call_get_platform_types() -> Result<HashMap<String, String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let ret_str = proxy.get_platform_types().await?;
    let ret_map = ret_str
        .lines() // split by '\n'
        .filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                _ => None, // ignore lines without a colon
            }
        })
        .collect();
    Ok(ret_map)
}
