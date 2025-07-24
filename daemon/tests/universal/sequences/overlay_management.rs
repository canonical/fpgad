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

use crate::common::proxies::{control_proxy, status_proxy};
use crate::universal::sequences::{DEVICE_HANDLE, OVERLAY_FILE, OVERLAY_HANDLE, PLATFORM_HANDLE};
use std::path::Path;
use zbus::Connection;

// todo: cases for overlay writing (i.e. bad overlay) and ensure that they are removed after so that another can run.
#[tokio::test]
async fn cases() {
    // TODO: reset key
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let status_proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");
    for overlay_handle in status_proxy
        .get_overlays()
        .await
        .expect("failed to get overlays")
        .split(",")
    {
        if !overlay_handle.is_empty() {
            control_proxy
                .remove_overlay(PLATFORM_HANDLE, overlay_handle)
                .await
                .unwrap_or_else(|_| panic!("failed to remove {overlay_handle}"));
        }
    }

    control_proxy
        .set_fpga_flags(PLATFORM_HANDLE, DEVICE_HANDLE, 0)
        .await
        .expect("failed to set fpga flags");
    assert_eq!(
        status_proxy
            .get_fpga_flags(PLATFORM_HANDLE, DEVICE_HANDLE)
            .await
            .expect("failed to get fpga flags"),
        "0",
        "flags not set properly"
    );

    control_proxy
        .write_property("/sys/class/fpga_manager/fpga0/key", "")
        .await
        .expect("failed to reset the encryption key");

    control_proxy
        .apply_overlay(PLATFORM_HANDLE, OVERLAY_HANDLE, OVERLAY_FILE, "")
        .await
        .expect("failed to apply overlay");
    let s = status_proxy
        .get_overlay_status(PLATFORM_HANDLE, OVERLAY_HANDLE)
        .await
        .expect("failed to read overlay status");
    let file_name = Path::new(OVERLAY_FILE)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    assert!(
        s.contains(&file_name),
        "relative path '{file_name}' not found in overlay status: '{s}'"
    );
    assert!(
        s.contains("applied"),
        "state 'applied' not found in overlay status"
    );
}
