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

//! Remove command implementation for the FPGA CLI.
//!
//! This module handles the removal of FPGA bitstreams and device tree overlays through
//! the fpgad daemon's DBus interface. It provides functionality to:
//! - Remove device tree overlays from the system
//! - Remove FPGA bitstreams (platform-dependent, currently not implemented)
//! - Automatically detect and remove the first overlay when not specified
//!
//! The module communicates with the fpgad daemon via DBus to perform these privileged
//! operations on the FPGA subsystem.
//!
//! For information on [Device Handles], [Overlay Handles], and [Error Handling],
//! see the [Common Concepts](../index.html#common-concepts) section in the main CLI documentation.
//!
//! [Device Handles]: ../index.html#device-handles
//! [Overlay Handles]: ../index.html#overlay-handles
//! [Error Handling]: ../index.html#error-handling

use crate::RemoveSubcommand;
use crate::proxies::control_proxy;
use crate::status::{call_get_platform_type, get_first_overlay, get_first_platform};
use zbus::Connection;

/// Removes a bitstream from an FPGA device.
///
/// # Note
///
/// This functionality is currently not implemented as bitstream removal is
/// vendor-specific and depends on platform capabilities that may be added
/// through softener implementations in the future.
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Err(zbus::Error)` - Always returns "Not implemented" error
async fn remove_bitstream() -> Result<String, zbus::Error> {
    // TODO: so this is confusing because we don't have a way to remove a bitstream but with
    //  softeners we might end up with this functionality.
    Err(zbus::Error::Failure("Not implemented".to_string()))
}

/// Sends the DBus command to remove a device tree overlay.
///
/// Communicates with the fpgad daemon via DBus to remove a previously loaded
/// device tree overlay from the system.
///
/// # Arguments
///
/// * `device_handle` - Platform identifier for the [device](../index.html#device-handles)
/// * `overlay_handle` - [Overlay handle](../index.html#overlay-handles) of the overlay to remove
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error, invalid handle(s), or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
async fn call_remove_overlay(
    device_handle: &str,
    overlay_handle: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy.remove_overlay(device_handle, overlay_handle).await
}

/// Removes a device tree overlay with automatic platform and handle detection.
///
/// This function handles the logic for determining the appropriate platform and overlay
/// handle based on what the user has provided. It supports:
/// - Auto-detecting the first platform if no device handle is provided
/// - Auto-detecting the first overlay if no overlay handle is provided
/// - Using provided handles when available
///
/// # Arguments
///
/// * `device_handle` - Optional [device handle](../index.html#device-handles) for platform detection
/// * `overlay_handle` - Optional [overlay handle](../index.html#overlay-handles) of the specific overlay to remove
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error, detection failure, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
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

/// Main handler for the remove command.
///
/// Dispatches to the appropriate remove function based on the subcommand type
/// (overlay or bitstream). This is the entry point called by the CLI's main
/// function when a remove command is issued.
///
/// # Arguments
///
/// * `dev_handle` - Optional [device handle](../index.html#device-handles)
/// * `sub_command` - The remove subcommand specifying what to remove (overlay or bitstream)
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the operation
/// * `Err(zbus::Error)` - DBus communication error, operation failure, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
pub async fn remove_handler(
    dev_handle: &Option<String>,
    sub_command: &RemoveSubcommand,
) -> Result<String, zbus::Error> {
    match sub_command {
        RemoveSubcommand::Overlay { handle } => remove_overlay(dev_handle, handle).await,
        RemoveSubcommand::Bitstream => remove_bitstream().await,
    }
}
