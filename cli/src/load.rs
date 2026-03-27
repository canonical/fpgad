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

//! Load command implementation for the FPGA CLI.
//!
//! This module handles the loading of FPGA bitstreams and device tree overlays through
//! the fpgad daemon's DBus interface. It provides functionality to:
//! - Load FPGA bitstreams onto FPGA devices
//! - Apply device tree overlays
//! - Automatically detect and use default FPGA devices when not specified
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

use crate::LoadSubcommand;
use crate::proxies::control_proxy;
use crate::status::{
    call_get_platform_type, call_get_platform_types, get_first_device_handle, get_first_platform,
};
use std::path;
use zbus::Connection;

/// Sanitizes and converts a file path string to an absolute path.
///
/// # Arguments
///
/// * `in_str` - The input path string to sanitize
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - Absolute path string resolved from the input
/// * `zbus::Error::Failure` - If the path cannot be resolved to an absolute path
fn sanitize_path_str(in_str: &str) -> Result<String, zbus::Error> {
    match path::absolute(in_str) {
        Ok(absolute_path) => Ok(absolute_path.to_string_lossy().to_string()),
        Err(e) => Err(zbus::Error::Failure(format!(
            "Failed to resolve path '{}': {}",
            in_str, e
        ))),
    }
}

/// Sends the DBus command to load a bitstream onto an FPGA device.
///
/// Communicates with the fpgad daemon via DBus to write a bitstream file to the
/// specified FPGA device. The bitstream configures the FPGA's logic fabric.
///
/// # Arguments
///
/// * `platform_str` - Platform identifier (empty string for auto-detection)
/// * `device_handle` - The [device handle](../index.html#device-handles) (e.g., "fpga0")
/// * `file_path` - Absolute path to the bitstream file
/// * `firmware_lookup_path` - Optional firmware lookup path (empty string for default)
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
async fn call_load_bitstream(
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

/// Sends the DBus command to apply a device tree overlay.
///
/// Communicates with the fpgad daemon via DBus to load a device tree overlay file.
/// Overlays describe hardware configuration and interfaces for the FPGA peripherals.
///
/// # Arguments
///
/// * `platform` - Platform identifier string
/// * `file_path` - Absolute path to the overlay file (.dtbo)
/// * `overlay_handle` - [Overlay handle](../index.html#overlay-handles) for the overlay directory in configfs
/// * `firmware_lookup_path` - Optional firmware lookup path (empty string for default)
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
async fn call_apply_overlay(
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

/// Applies a device tree overlay with automatic platform and handle detection.
///
/// This function handles the logic for determining the appropriate platform and overlay
/// handle based on what the user has provided. It supports four scenarios:
/// 1. Both device and overlay handles provided - use both as-is
/// 2. Only device handle provided - use device name as overlay handle
/// 3. Only overlay handle provided - auto-detect first platform
/// 4. Neither provided - auto-detect both from first available device
///
/// # Arguments
///
/// * `dev_handle` - Optional [device handle](../index.html#device-handles) (e.g., "fpga0")
/// * `file_path` - Path to the overlay file (.dtbo)
/// * `overlay_handle` - Optional [overlay handle](../index.html#overlay-handles) for the overlay directory
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error, device detection failure, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
async fn apply_overlay(
    dev_handle: &Option<String>,
    file_path: &str,
    overlay_handle: &Option<String>,
) -> Result<String, zbus::Error> {
    // Determine platform and overlay handle based on provided parameters
    let (platform, overlay_handle_to_use) = match (dev_handle, overlay_handle) {
        // Both are provided
        (Some(dev), Some(overlay)) => (call_get_platform_type(dev).await?, overlay.clone()),

        // dev_handle provided, overlay_handle not provided so use device name as overlay handle
        (Some(dev), None) => {
            let platform = call_get_platform_type(dev).await?;
            (platform, dev.clone())
        }
        // dev_handle not provided, so use first platform
        (None, Some(overlay)) => {
            let platform = get_first_platform().await?;
            (platform, overlay.clone())
        }
        // neither provided so get first device to and use its platform as platform and its name as
        // overlay_handle
        (None, None) => {
            // this saves making two dbus calls by getting it all from the hashmap
            let platforms = call_get_platform_types().await?;
            let platform = platforms
                .values()
                .next()
                .unwrap_or(&"universal".to_string())
                .clone();
            let overlay = platforms
                .keys()
                .next()
                .unwrap_or(&"overlay0".to_string())
                .clone();
            (platform, overlay)
        }
    };
    call_apply_overlay(
        &platform,
        &sanitize_path_str(file_path)?,
        &overlay_handle_to_use,
        "",
    )
    .await
}

/// Loads a bitstream onto an FPGA device with automatic device detection.
///
/// If no device handle is provided, this function automatically detects and uses
/// the first available FPGA device in the system.
///
/// # Arguments
///
/// * `device_handle` - Optional [device handle](../index.html#device-handles) (e.g., "fpga0")
/// * `file_path` - Path to the bitstream file
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error, device detection failure, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
async fn load_bitstream(
    device_handle: &Option<String>,
    file_path: &str,
) -> Result<String, zbus::Error> {
    let dev = match device_handle {
        None => get_first_device_handle().await?,
        Some(dev) => dev.clone(),
    };
    call_load_bitstream("", &dev, &sanitize_path_str(file_path)?, "").await
}

/// Main handler for the load command.
///
/// Dispatches to the appropriate load function based on the subcommand type
/// (overlay or bitstream). This is the entry point called by the CLI's main
/// function when a load command is issued.
///
/// # Arguments
///
/// * `dev_handle` - Optional [device handle](../index.html#device-handles)
/// * `sub_command` - The load subcommand specifying what to load (overlay or bitstream)
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the operation
/// * `Err(zbus::Error)` - DBus communication error, operation failure, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
pub async fn load_handler(
    dev_handle: &Option<String>,
    sub_command: &LoadSubcommand,
) -> Result<String, zbus::Error> {
    match sub_command {
        LoadSubcommand::Overlay { file, handle } => {
            apply_overlay(dev_handle, file.as_ref(), handle).await
        }
        LoadSubcommand::Bitstream { file } => load_bitstream(dev_handle, file.as_ref()).await,
    }
}
