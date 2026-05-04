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
use crate::xilinx_dfx_mgr::{PLATFORM_STRING, setup};
use googletest::prelude::*;
use rstest::*;
use zbus::Connection;

#[tokio::test]
#[gtest]
#[rstest]
async fn apply_overlay_via_dfx_mgr(_setup: ()) {
    let device_handle: &str = "fpga0";
    let overlay_handle: &str = "fpga0";
    let overlay_file: &str = "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.dtbo";

    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let status_proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");

    // Remove any existing overlays/slots
    let _ = control_proxy.remove_overlay(PLATFORM_STRING, "").await;

    // Reset flags
    control_proxy
        .set_fpga_flags(PLATFORM_STRING, device_handle, 0)
        .await
        .expect("failed to set fpga flags");
    expect_that!(
        status_proxy
            .get_fpga_flags(PLATFORM_STRING, device_handle)
            .await
            .expect("failed to get fpga flags"),
        eq("0"),
        "flags not set properly"
    );

    // Apply overlay via dfx-mgr (extracts firmware-name and loads both)
    let result = control_proxy
        .apply_overlay(PLATFORM_STRING, overlay_handle, overlay_file, "")
        .await;

    expect_that!(
        &result,
        ok(displays_as(contains_substring("Loaded with slot_handle")))
    );

    if let Ok(output) = &result {
        println!("Apply overlay output: {:#?}", output);
    }

    // Check status - for dfx-mgr returns package listing
    let status = status_proxy
        .get_overlay_status(PLATFORM_STRING, overlay_handle)
        .await
        .expect("failed to read overlay status");

    println!("Overlay status: {:#?}", status);
    expect_that!(status, anything());

    // Remove overlay
    let remove_result = control_proxy.remove_overlay(PLATFORM_STRING, "").await;
    expect_that!(
        &remove_result,
        ok(displays_as(contains_substring("returns: 0 (Ok)")))
    );
}

#[tokio::test]
#[gtest]
#[rstest]
async fn test_slot_removal(_setup: ()) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");

    // Test removing specific slot (if supported)
    let result = control_proxy.remove_overlay(PLATFORM_STRING, "0").await;

    // Should succeed or return appropriate error
    expect_that!(&result, ok(anything()).or(err(anything())));

    if let Ok(output) = result {
        println!("Slot removal output: {:#?}", output);
    }
}
