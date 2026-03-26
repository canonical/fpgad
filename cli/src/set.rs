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

use crate::proxies::control_proxy;
use crate::status::get_first_device_handle;
use std::path::Path;
use zbus::Connection;

fn build_property_path(device_handle: &str, attribute: &str) -> String {
    Path::new("/sys/class/fpga_manager/")
        .join(device_handle)
        .join(attribute)
        .to_string_lossy()
        .to_string()
}

async fn call_write_property(property: &str, value: &str) -> Result<String, zbus::Error> {
    let connection = Connection::system().await?;
    let proxy = control_proxy::ControlProxy::new(&connection).await?;
    proxy.write_property(property, value).await
}

/// Argument parser for the load command
pub async fn set_handler(
    device_handle: &Option<String>,
    attribute: &str,
    value: &str,
) -> Result<String, zbus::Error> {
    let property_path = match device_handle {
        None => build_property_path(&get_first_device_handle().await?, attribute),
        Some(dev) => build_property_path(dev, attribute),
    };
    call_write_property(&property_path, value).await
}

#[cfg(test)]
mod tests {
    use super::build_property_path;

    #[test]
    fn build_property_path_for_standard_attribute() {
        let path = build_property_path("fpga0", "flags");
        assert_eq!(path, "/sys/class/fpga_manager/fpga0/flags");
    }

    #[test]
    fn build_property_path_keeps_nested_attribute_segments() {
        let path = build_property_path("fpga0", "subdir/attr");
        assert_eq!(path, "/sys/class/fpga_manager/fpga0/subdir/attr");
    }
}
