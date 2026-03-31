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

/// Parses a newline-separated string of overlays into a `Vec<String>`
///
/// # Arguments
/// * `list_str` - The string containing overlay names separated by newlines.
///
/// # Returns: `Vec<String>`
/// A `Vec<String>` where each element is an overlay name from the input string.
///
/// # Examples
/// ```rust,no_run
/// let overlays = parse_overlay_lines("overlay0\noverlay1\n");
/// assert_eq!(overlays, vec!["overlay0".to_string(), "overlay1".to_string()]);
/// ```
fn parse_overlay_lines(list_str: &str) -> Vec<String> {
    list_str.lines().map(|line| line.to_string()).collect()
}

/// Parses a newline-separated string of platform types into a HashMap of device to platform string.
/// Each line is expected to be in the format "device:platform_string".
/// Lines that do not conform to this format are ignored.
///
/// # Arguments
/// * `ret_str` - The string containing device-platform pairs separated by newlines.
///
/// # Returns: `HashMap<String, String>`
/// A HashMap<String, String> where the key is the device and the value is the platform string.
///
/// # Examples
/// ```rust,no_run
/// let platforms = parse_platform_types_lines("fpga0:xlnx,zynq\n
/// invalid_line\nfpga1:intel,foo\n");
/// assert_eq!(platforms.get("fpga0").map(String::as_str), Some("xlnx,zynq"));
/// assert_eq!(platforms.get("fpga1").map(String::as_str), Some("intel,foo"));
/// assert!(!platforms.contains_key("invalid_line"));
/// ```
/// ```rust,no_run
/// let platforms = parse_platform_types_lines("fpga0:xilinx:zynq:mp\n");
/// assert_eq!(platforms.get("fpga0").map(String::as_str), Some("xilinx:zynq:mp"));
/// ```
fn parse_platform_types_lines(ret_str: &str) -> HashMap<String, String> {
    ret_str
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) => Some((key.to_string(), value.to_string())),
                _ => None,
            }
        })
        .collect()
}

/// Returns the first overlay from the list or an error if the list is empty.
///
/// # Arguments
/// * `overlays` - A slice of overlay names.
///
/// # Returns: Result<String, zbus::Error>
/// The first overlay name as a String if the list is not empty, or a zbus::Error if the list is empty.
///
/// # Examples
/// ```rust,no_run
/// let overlay = first_overlay_or_error(&vec!["overlay0".to_string(), "overlay1".to_string()]).expect("should return first overlay");
/// assert_eq!(overlay, "overlay0".to_string());
/// ```
/// ```rust,no_run
/// let result = first_overlay_or_error(&vec![]);
/// assert!(result.is_err());
/// ```
fn first_overlay_or_error(overlays: &[String]) -> Result<String, zbus::Error> {
    let first = overlays.first().ok_or(zbus::Error::Failure(
        "Could not find an overlay to remove".to_string(),
    ))?;
    Ok(first.clone())
}

/// Returns the first device handle from the platforms map or an error if the map is empty.
///
/// # Arguments
/// * `platforms` - A HashMap where keys are device handles and values are platform strings.
///
/// # Returns: Result<String, zbus::Error>
/// The first device handle as a String if the map is not empty, or a zbus::Error if the map is empty.
///
/// # Examples
/// ```rust,no_run
/// let device_handle = first_device_handle_or_error(&[("fpga0".to_string(), "xlnx,zynq".to_string())].iter().cloned().collect()).expect("should return first device handle");
/// assert_eq!(device_handle, "fpga0".to_string());
/// ```
/// ```rust,no_run
/// let result = first_device_handle_or_error(&HashMap::new());
/// assert!(result.is_err());
/// ```
fn first_device_handle_or_error(
    platforms: &HashMap<String, String>,
) -> Result<String, zbus::Error> {
    match platforms.keys().next() {
        Some(p) => Ok(p.clone()),
        None => Err(zbus::Error::Failure("Got no platforms.".to_string())),
    }
}

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
/// ```rust,ignore
/// let overlays = call_get_overlays().await?;
/// for overlay in overlays {
///     println!("Found overlay: {}", overlay);
/// }
/// ```
// Isolate empty-overlay error mapping so unit tests can validate exact error
// variant/message without depending on a live D-Bus service.
pub async fn call_get_overlays() -> Result<Vec<String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let list_str = proxy.get_overlays().await?;
    Ok(parse_overlay_lines(&list_str))
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
/// ```rust,ignore
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
/// ```rust,ignore
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
/// ```rust,ignore
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
/// ```rust,ignore
/// let platforms = call_get_platform_types().await?;
/// for (device, platform) in platforms {
///     println!("{}: {}", device, platform);
/// }
/// ```
pub async fn call_get_platform_types() -> Result<HashMap<String, String>, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    let ret_str = proxy.get_platform_types().await?;
    Ok(parse_platform_types_lines(&ret_str))
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
/// ```rust,ignore
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
/// ```rust,ignore
/// let overlay = get_first_overlay().await?;
/// println!("First overlay: {}", overlay);
/// ```
pub async fn get_first_overlay() -> Result<String, zbus::Error> {
    let overlays = call_get_overlays().await?;
    first_overlay_or_error(&overlays)
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
/// ```rust,ignore
/// let device = get_first_device_handle().await?;
/// println!("Using device: {}", device);
/// ```
pub async fn get_first_device_handle() -> Result<String, zbus::Error> {
    let platforms = call_get_platform_types().await?;
    first_device_handle_or_error(&platforms)
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
/// ```rust,ignore
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
/// ```rust,ignore
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
/// ```rust,ignore
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

#[cfg(test)]
mod tests {
    use super::{
        first_device_handle_or_error, first_overlay_or_error, parse_overlay_lines,
        parse_platform_types_lines,
    };
    use std::collections::HashMap;

    #[test]
    fn parse_platform_types_ignores_invalid_lines() {
        let parsed = parse_platform_types_lines("fpga0:xlnx,zynq\ninvalid_line\nfpga1:intel,foo\n");

        assert_eq!(parsed.get("fpga0").map(String::as_str), Some("xlnx,zynq"));
        assert_eq!(parsed.get("fpga1").map(String::as_str), Some("intel,foo"));
        assert!(!parsed.contains_key("invalid_line"));
    }

    #[test]
    fn parse_platform_types_keeps_value_with_extra_colons() {
        let parsed = parse_platform_types_lines("fpga0:xilinx:zynq:mp\n");
        assert_eq!(
            parsed.get("fpga0").map(String::as_str),
            Some("xilinx:zynq:mp")
        );
    }

    #[test]
    fn parse_overlay_lines_keeps_order() {
        let overlays = parse_overlay_lines("overlay0\noverlay1\n");
        assert_eq!(
            overlays,
            vec!["overlay0".to_string(), "overlay1".to_string()]
        );
    }

    #[test]
    fn first_overlay_returns_failure_error_with_expected_message() {
        let overlays = Vec::<String>::new();
        let err = first_overlay_or_error(&overlays).expect_err("empty overlays should fail");

        match &err {
            zbus::Error::Failure(message) => {
                assert_eq!(message, "Could not find an overlay to remove")
            }
            _ => panic!("expected zbus::Error::Failure, got {err}"),
        }

        assert_eq!(err.to_string(), "Could not find an overlay to remove");
    }

    #[test]
    fn first_device_handle_returns_failure_error_with_expected_message() {
        let platforms = HashMap::<String, String>::new();
        let err =
            first_device_handle_or_error(&platforms).expect_err("empty platforms should fail");

        match &err {
            zbus::Error::Failure(message) => assert_eq!(message, "Got no platforms."),
            _ => panic!("expected zbus::Error::Failure, got {err}"),
        }

        assert_eq!(err.to_string(), "Got no platforms.");
    }
}
