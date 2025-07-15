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

use crate::RemoveSubcommand;
use crate::proxies::control_proxy;
use crate::status::{call_get_platform_type, get_first_overlay, get_first_platform};
use zbus::Connection;

async fn remove_bitstream() -> Result<String, zbus::Error> {
    // TODO: so this is confusing because we don't have a way to remove a bitstream but with
    //  softeners we might end up with this functionality.
    Err(zbus::Error::Failure("Not implemented".to_string()))
}

/// Sends the dbus command to remove an overlay
async fn call_remove_overlay(
    device_handle: &str,
    overlay_handle: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy.remove_overlay(device_handle, overlay_handle).await
}

/// Populates the platform and overlay handle appropriately before calling `call_apply_overlay`
async fn remove_overlay(
    device_handle: &Option<String>,
    overlay_handle: &Option<String>,
) -> Result<String, zbus::Error> {
    let dev = match device_handle {
        None => get_first_platform().await?,
        Some(dev) => call_get_platform_type(dev).await?,
    };
    let handle = match overlay_handle {
        Some(handle) => handle.clone(),
        None => get_first_overlay().await?,
    };
    call_remove_overlay(&dev, &handle).await
}

/// Argument parser for the remove command
pub async fn remove_handler(
    dev_handle: &Option<String>,
    sub_command: &RemoveSubcommand,
) -> Result<String, zbus::Error> {
    match sub_command {
        RemoveSubcommand::Overlay { handle } => remove_overlay(dev_handle, handle).await,
        RemoveSubcommand::Bitstream => remove_bitstream().await,
    }
}
