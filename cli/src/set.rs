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

//! Set command implementation for the FPGA CLI.
//!
//! This module handles setting FPGA device attributes and properties through
//! the fpgad daemon's DBus interface. It provides functionality to:
//! - Write to FPGA manager attributes in sysfs (e.g., flags)
//! - Automatically detect the first device when not specified
//!
//! Common attributes that can be set include programming flags and other
//! FPGA manager properties exposed through the Linux kernel's FPGA subsystem.
//!
//! For information on [Device Handles] and [Error Handling],
//! see the [Common Concepts](../index.html#common-concepts) section in the main CLI documentation.
//!
//! [Device Handles]: ../index.html#device-handles
//! [Error Handling]: ../index.html#error-handling

use crate::proxies::control_proxy;
use crate::status::get_first_device_handle;
use std::path::{Component, Path, PathBuf};
use zbus::Connection;

/// Sanitizes a path segment by ensuring it does not contain absolute paths,
/// parent directory traversal, or root/prefix components.
///
/// # Arguments
/// * `segment` - The path segment to sanitize.
/// * `field_name` - A descriptive name for the field being sanitized, used in error messages.
///
/// # Returns: Result<String, zbus::Error>
/// A sanitized version of the path segment if valid, or a zbus::Error if invalid
///
/// # Examples
/// ```rust,no_run
/// let safe_segment = sanitize_segment("valid/segment", "attribute").expect("should be
/// valid");
/// assert_eq!(safe_segment, "valid/segment");
/// ```
/// ```rust,no_run
/// let result = sanitize_segment("../invalid", "attribute");
/// assert!(result.is_err());
/// ```
fn sanitize_segment(segment: &str, field_name: &str) -> Result<String, zbus::Error> {
    if Path::new(segment).is_absolute() {
        return Err(zbus::Error::Failure(format!(
            "Invalid {} '{}': absolute paths are not allowed",
            field_name, segment
        )));
    }

    let mut buf = PathBuf::new();
    for comp in Path::new(segment).components() {
        match comp {
            Component::Normal(part) => buf.push(part),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(zbus::Error::Failure(format!(
                    "Invalid {} '{}': parent directory traversal is not allowed",
                    field_name, segment
                )));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(zbus::Error::Failure(format!(
                    "Invalid {} '{}': path root/prefix is not allowed",
                    field_name, segment
                )));
            }
        }
    }

    if buf.as_os_str().is_empty() {
        return Err(zbus::Error::Failure(format!(
            "Invalid {} '{}': empty path segment",
            field_name, segment
        )));
    }

    Ok(buf.to_string_lossy().to_string())
}

/// Builds a property path for the given device handle and attribute,
/// ensuring that the inputs are sanitized to prevent path traversal or absolute paths.
///
/// # Arguments
/// * `device_handle` - The device handle to include in the path.
/// * `attribute` - The attribute to include in the path.
///
/// # Returns: Result<String, zbus::Error>
/// A string representing the full property path if inputs are valid, or a zbus::Error
/// if the inputs are invalid.
///
/// # Examples
/// ```rust,no_run
/// let path = build_property_path("fpga0", "flags").expect("should build path");
/// assert_eq!(path, "/sys/class/fpga_manager/fpga0/flags");
/// ```
/// ```rust,no_run
/// let result = build_property_path("../fpga0", "flags");
/// assert!(result.is_err());
/// ```
fn build_property_path(device_handle: &str, attribute: &str) -> Result<String, zbus::Error> {
    let safe_device = sanitize_segment(device_handle, "device handle")?;
    let safe_attribute = sanitize_segment(attribute, "attribute")?;

    Ok(Path::new("/sys/class/fpga_manager/")
        .join(safe_device)
        .join(safe_attribute)
        .to_string_lossy()
        .to_string())
}

/// Sends the DBus command to write a property value.
///
/// Communicates with the fpgad daemon via DBus to write a value to a
/// specified sysfs property path.
///
/// # Arguments
///
/// * `property` - Full path to the sysfs property to write
/// * `value` - Value to write to the property
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
async fn call_write_property(property: &str, value: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy.write_property(property, value).await
}

/// Main handler for the set command.
///
/// Constructs the full sysfs path for the specified attribute under
/// `/sys/class/fpga_manager/<device>/<attribute>` and writes the provided value.
/// If no device handle is specified, it auto-detects the first available FPGA device.
///
/// # Arguments
///
/// * `device_handle` - Optional [device handle](../index.html#device-handles) (e.g., "fpga0")
/// * `attribute` - Name of the attribute to set (e.g., "flags")
/// * `value` - Value to write to the attribute
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error, device detection failure, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
///
/// # Examples
///
/// Setting the flags attribute:
/// ```bash
/// fpgad set flags 0
/// fpgad --handle=fpga0 set flags 0
/// ```
pub async fn set_handler(
    device_handle: &Option<String>,
    attribute: &str,
    value: &str,
) -> Result<String, zbus::Error> {
    let property_path = match device_handle {
        None => build_property_path(&get_first_device_handle().await?, attribute)?,
        Some(dev) => build_property_path(dev, attribute)?,
    };
    call_write_property(&property_path, value).await
}

#[cfg(test)]
mod tests {
    use super::build_property_path;

    #[test]
    fn build_property_path_for_standard_attribute() {
        let path = build_property_path("fpga0", "flags").expect("path should be valid");
        assert_eq!(path, "/sys/class/fpga_manager/fpga0/flags");
    }

    #[test]
    fn build_property_path_keeps_nested_attribute_segments() {
        let path =
            build_property_path("fpga0", "subdir/attr").expect("nested path should be valid");
        assert_eq!(path, "/sys/class/fpga_manager/fpga0/subdir/attr");
    }

    #[test]
    fn build_property_path_with_absolute_root() {
        let result = build_property_path("/fpga0", "subdir/attr");
        assert!(result.is_err());
    }

    #[test]
    fn build_property_path_with_parent_traversal() {
        let result = build_property_path("fpga0", "../attr");
        assert!(result.is_err());
    }
}
