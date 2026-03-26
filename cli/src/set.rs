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
use std::path::Path;
use zbus::Connection;

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
        None => Path::new("/sys/class/fpga_manager/")
            .join(get_first_device_handle().await?)
            .join(attribute),
        Some(dev) => Path::new("/sys/class/fpga_manager/")
            .join(dev)
            .join(attribute),
    };
    call_write_property(property_path.to_string_lossy().to_string().as_ref(), value).await
}
