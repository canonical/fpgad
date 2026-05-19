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

//! DFX Manager command implementation for the FPGA CLI.
//!
//! This module passes commands directly to `dfx-mgr-client` via the fpgad daemon's
//! DBus interface.
//!
//! If the connected daemon does not expose `dfx_mgr` (for example because it was built
//! without `xilinx-dfx-mgr`), this handler returns a clear compatibility error.
//!
//!
//! # Usage
//!
//! ```shell
//! fpgad dfx-mgr "-listPackage"
//! fpgad dfx-mgr "-load 0 my_design"
//! fpgad dfx-mgr "-remove 0"
//! ```
//!
//! The entire command string is passed as a single argument and split by the daemon
//! before forwarding to `dfx-mgr-client`.

use crate::proxies::control_proxy;
use zbus::Connection;

/// Main handler for the dfx-mgr command.
///
/// Forwards the provided command string to the daemon's `dfx_mgr` DBus method,
/// which in turn passes it to `dfx-mgr-client`.
///
/// # Arguments
///
/// * `cmd` - Space-separated arguments to pass to `dfx-mgr-client`
///   (e.g. `"-listPackage"` or `"-load 0 my_design"`)
///
/// # Returns: `Result<String, zbus::Error>`
/// * `Ok(String)` - Exit status, stdout, and stderr from `dfx-mgr-client`
/// * `Err(zbus::Error)` - DBus communication error, missing component, or FpgadError.
///   See [Error Handling](../index.html#error-handling) for details.
pub async fn dfx_mgr_handler(cmd: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    match proxy.dfx_mgr(cmd).await {
        Ok(out) => Ok(out),
        Err(zbus::Error::MethodError(name, _, _))
            if name.as_str() == "org.freedesktop.DBus.Error.UnknownMethod" =>
        {
            Err(zbus::Error::Failure(
                "feature not enabled in daemon, or CLI/daemon out of date (missing dfx_mgr method)"
                    .to_string(),
            ))
        }
        Err(e) => Err(e),
    }
}
