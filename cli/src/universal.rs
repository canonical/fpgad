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

//! Universal command implementation for the FPGA CLI.
//!
//! This module provides direct access to the daemon's `universal` read/write interface,
//! allowing low-level control of FPGA manager sysfs properties and flags without needing
//! a platform-specific subcommand.
//!
//! # Subcommands
//!
//! ## Read
//!
//! ```shell
//! fpgad universal read read_property /sys/class/fpga_manager/fpga0/name
//! fpgad universal read read_flags fpga0
//! ```
//!
//! ## Write
//!
//! ```shell
//! fpgad universal write write_flags fpga0 0x20
//! fpgad universal write write_property /sys/class/fpga_manager/fpga0/key VALUE
//! fpgad universal write write_property_bytes /sys/class/fpga_manager/fpga0/key BYTES
//! ```

use crate::UniversalSubcommand;
use crate::proxies::{control_proxy, status_proxy};
use zbus::Connection;

/// Sends the `universal` read command to the daemon's status interface.
///
/// # Arguments
///
/// * `sub_cmd` - One of `read_property` or `read_flags`
/// * `path` - Sysfs property path for `read_property`, or device handle for `read_flags`
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Result from the daemon
/// * `Err(zbus::Error)` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
async fn call_universal_read(sub_cmd: &str, path: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = status_proxy::StatusProxy::new(&connection).await?;
    proxy.universal(sub_cmd, path).await
}

/// Sends the `universal` write command to the daemon's control interface.
///
/// # Arguments
///
/// * `sub_cmd` - One of `write_flags`, `write_property`, `write_property_bytes`
/// * `path` - Device handle for `write_flags`, or sysfs property path for property writes
/// * `value` - Value to write
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Success message from the daemon
/// * `Err(zbus::Error)` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
async fn call_universal_write(
    sub_cmd: &str,
    path: &str,
    value: &str,
) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy.universal(sub_cmd, path, value).await
}

/// Main handler for the universal command.
///
/// Dispatches to the appropriate read or write call based on the subcommand.
///
/// # Arguments
///
/// * `sub_command` - The universal subcommand (read or write) with its arguments
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Result from the daemon
/// * `Err(zbus::Error)` - DBus communication error or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
pub async fn universal_handler(sub_command: &UniversalSubcommand) -> Result<String, zbus::Error> {
    match sub_command {
        UniversalSubcommand::Read { sub_cmd, path } => call_universal_read(sub_cmd, path).await,
        UniversalSubcommand::Write {
            sub_cmd,
            path,
            value,
        } => call_universal_write(sub_cmd, path, value).await,
    }
}
