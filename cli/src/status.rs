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

//! Status command implementation for the FPGA CLI.
//!
//! This module handles querying FPGA device and overlay status through the fpgad
//! daemon's DBus interface. It provides functionality to:
//! - Query the state of FPGA devices (operating, unknown, etc.)
//! - List available device tree overlays
//! - Get platform compatibility strings for devices
//! - Retrieve overlay status information
//! - Format status information into human-readable tables
//!
//! The module communicates with the fpgad daemon via DBus to access these
//! read-only properties from the FPGA subsystem.
//!
//! For information on [Device Handles], [Overlay Handles], and [Error Handling],
//! see the [Common Concepts](../index.html#common-concepts) section in the main CLI documentation.
//!
//! [Device Handles]: ../index.html#device-handles
//! [Overlay Handles]: ../index.html#overlay-handles
//! [Error Handling]: ../index.html#error-handling

use crate::proxies::status_proxy;
use std::collections::HashMap;
use zbus::Connection;

/// Retrieve a list of all loaded device tree overlays from the system.
///
/// Sends the DBus command to get a list of overlays and parses the newline-separated
/// response into a vector of overlay handles.
///
/// # Returns: `Result<Vec<String>, zbus::Error>`
/// * `Vec<String>` - List of overlay handles currently loaded in the system
/// * `zbus::Error` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let overlays = call_get_overlays().await?;
/// for overlay in overlays {
///     println!("Found overlay: {}", overlay);
/// }
/// ```
pub async fn call_get_overlays() -> Result<Vec<String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let list_str = proxy.get_overlays().await?;
    let ret_list: Vec<String> = list_str.lines().map(|line| line.to_string()).collect();
    Ok(ret_list)
}

/// Retrieve the current state of an FPGA device.
///
/// Queries the daemon for the state of the specified FPGA device (e.g., "operating" or
/// "unknown").
///
/// # Arguments
///
/// * `device_handle` - The [device handle](../index.html#device-handles) (e.g., "fpga0")
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - Current state of the FPGA device
/// * `zbus::Error` - DBus communication error, invalid device handle, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let state = call_get_fpga_state("fpga0").await?;
/// println!("FPGA state: {}", state);
/// ```
pub async fn call_get_fpga_state(device_handle: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_fpga_state("", device_handle).await
}

/// Retrieve the platform compatibility string for an FPGA device.
///
/// Queries the daemon for the platform/compatibility string of the specified device,
/// which identifies the hardware platform type (e.g., "xlnx,zynqmp-pcap-fpga").
///
/// # Arguments
///
/// * `device_handle` - The [device handle](../index.html#device-handles) (e.g., "fpga0")
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - Platform compatibility string for the device
/// * `zbus::Error` - DBus communication error, invalid device handle, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let platform = call_get_platform_type("fpga0").await?;
/// println!("Platform: {}", platform);
/// ```
pub async fn call_get_platform_type(device_handle: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_platform_type(device_handle).await
}

/// Retrieve the status of a specific device tree overlay.
///
/// Queries the daemon for the status information of a loaded overlay.
///
/// # Arguments
///
/// * `platform` - Platform identifier string
/// * `overlay_handle` - [Overlay handle](../index.html#overlay-handles) of the overlay to query
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - Status information for the overlay
/// * `zbus::Error` - DBus communication error, invalid overlay handle, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let status = call_get_overlay_status("universal", "overlay0").await?;
/// println!("Overlay status: {}", status);
/// ```
async fn call_get_overlay_status(
    platform: &str,
    overlay_handle: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.get_overlay_status(platform, overlay_handle).await
}

/// Retrieve all FPGA devices and their platform compatibility strings.
///
/// Parses the newline-separated string from the `get_platform_types` DBus interface
/// into a HashMap mapping device handles to their platform strings. Each line in the
/// response is formatted as "device:platform".
///
/// # Returns: `Result<HashMap<String, String>, zbus::Error>`
/// * `HashMap<String, String>` - Map of device handles to platform compatibility strings
/// * `zbus::Error` - DBus communication error or FpgadError. See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let platforms = call_get_platform_types().await?;
/// for (device, platform) in platforms {
///     println!("{}: {}", device, platform);
/// }
/// ```
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

/// Get the platform string of the first available FPGA device.
///
/// Retrieves the platform compatibility string for the first device found in the
/// system. Returns "universal" as a fallback if no devices are found.
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - Platform compatibility string of the first device, or "universal" if none found
/// * `zbus::Error` - DBus communication error or FpgadError. See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let platform = get_first_platform().await?;
/// println!("Using platform: {}", platform);
/// ```
pub async fn get_first_platform() -> Result<String, zbus::Error> {
    let platforms = call_get_platform_types().await?;
    Ok(platforms
        .values()
        .next()
        .unwrap_or(&"universal".to_string())
        .clone())
}

/// Get the handle of the first loaded device tree overlay.
///
/// Retrieves the first overlay from the list of currently loaded overlays in the system.
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - [Overlay handle](../index.html#overlay-handles) of the first loaded overlay
/// * `zbus::Error::Failure` - If no overlays are currently loaded
/// * `zbus::Error` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let overlay = get_first_overlay().await?;
/// println!("First overlay: {}", overlay);
/// ```
pub async fn get_first_overlay() -> Result<String, zbus::Error> {
    let overlays = call_get_overlays().await?;
    let first = overlays.first().ok_or(zbus::Error::Failure(
        "Could not find an overlay to remove".to_string(),
    ))?;
    Ok(first.clone())
}

/// Get the device handle of the first available FPGA device.
///
/// Retrieves the device handle (e.g., "fpga0") of the first FPGA device found in
/// the system by querying the platform types.
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - [Device handle](../index.html#device-handles) of the first available FPGA device
/// * `zbus::Error::Failure` - If no FPGA devices are found in the system
/// * `zbus::Error` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let device = get_first_device_handle().await?;
/// println!("Using device: {}", device);
/// ```
pub async fn get_first_device_handle() -> Result<String, zbus::Error> {
    let platform = match call_get_platform_types().await?.keys().next() {
        Some(p) => p.clone(),
        None => return Err(zbus::Error::Failure("Got no platforms.".to_string())),
    };

    Ok(platform)
}

/// Format the FPGA state for a single device as an ASCII table.
///
/// Retrieves the state and platform information for the specified device and formats
/// it as a simple ASCII table for display.
///
/// # Arguments
///
/// * `device_handle` - The [device handle](../index.html#device-handles) (e.g., "fpga0")
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - Formatted ASCII table with device information
/// * `zbus::Error` - DBus communication error, invalid device handle, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let message = get_fpga_state_message("fpga0").await?;
/// println!("{}", message);
/// ```
async fn get_fpga_state_message(device_handle: &str) -> Result<String, zbus::Error> {
    let state = call_get_fpga_state(device_handle).await?;
    let platform = call_get_platform_type(device_handle).await?;
    Ok(format!(
        "---- DEVICE  ----\n\
        | dev | platform | state |\n\
        {device_handle} | {platform} | {state}"
    ))
}

/// Format comprehensive status information for all FPGA devices and overlays.
///
/// Retrieves and formats status information for all FPGA devices in the system,
/// including their states and platform types, as well as all loaded overlays and
/// their statuses. Returns a multisection ASCII table for display.
///
/// If overlay information cannot be accessed (e.g., overlayfs not enabled), the
/// error is included in the output rather than failing the entire operation.
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - Formatted multisection ASCII table with all device and overlay information
/// * `zbus::Error` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// let status = get_full_status_message().await?;
/// println!("{}", status);
/// ```
async fn get_full_status_message() -> Result<String, zbus::Error> {
    let mut ret_string = String::from(
        "---- DEVICES ----\n\
    | dev | platform | state |\n",
    );

    for (dev, platform) in call_get_platform_types().await? {
        let state = call_get_fpga_state(&dev).await?;
        ret_string += format!("| {dev} | {platform} | {state} |\n").as_str();
    }

    // If overlayfs not enabled, or interface not connected this will be an error.
    let overlays = match call_get_overlays().await {
        Ok(overlays) => {
            ret_string += "\n---- OVERLAYS ----\n\
                   | overlay | status |\n";
            overlays
        }
        Err(e) => {
            ret_string += "\n---- OVERLAYS NOT ACCESSIBLE ----\n\n\
            errors encountered:\n";
            ret_string += e.to_string().as_str();
            Vec::new()
        }
    };

    for overlay in overlays {
        // TODO: overlays do not provide enough information to work out what platform to use.
        //  so maybe the status command can take a platform type instead or something.
        //  This is tricky.
        let p = get_first_platform().await?;
        let status = call_get_overlay_status(&p, &overlay).await?;
        ret_string.push_str(format!("| {overlay} | {status} |\n").as_ref());
    }
    Ok(ret_string)
}

/// Main handler for the status command.
///
/// Dispatches to the appropriate status query function based on whether a specific
/// device handle is provided. If a device handle is given, returns status for that
/// device only; otherwise returns comprehensive status for all devices and overlays.
/// This is the entry point called by the CLI's main function when a status command
/// is issued.
///
/// # Arguments
///
/// * `device_handle` - Optional [device handle](../index.html#device-handles) for querying a specific device
///
/// # Returns: `Result<String, zbus::Error>`
/// * `String` - Formatted status information ready for display
/// * `zbus::Error` - DBus communication error, device query failure, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// ```rust
/// // Get status for all devices
/// let status = status_handler(&None).await?;
///
/// // Get status for a specific device
/// let status = status_handler(&Some("fpga0".to_string())).await?;
/// ```
pub async fn status_handler(device_handle: &Option<String>) -> Result<String, zbus::Error> {
    let ret_string = match device_handle {
        None => get_full_status_message().await?,
        Some(dev) => get_fpga_state_message(dev.as_str()).await?,
    };
    Ok(ret_string)
}
