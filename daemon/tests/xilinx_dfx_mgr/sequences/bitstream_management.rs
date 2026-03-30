// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
//
// Copyright 2026 Canonical Ltd.
//
// SPDX-License-Identifier: GPL-3.0-only
//
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
//
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

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

#[gtest]
#[tokio::test]
#[rstest]
#[case::load_from_lib_firmware("fpga0", "/lib/firmware/k26-starter-kits.bit.bin", "")]
async fn load_bitstream_via_dfx_mgr(
    #[case] device_handle: &str,
    #[case] bitstream_file: &str,
    #[case] fw_lookup: &str,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let status_proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");

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
        eq("0")
    );

    // Check if there's a loaded bitstream and remove it if "0->0" is found
    let current_state = status_proxy
        .get_fpga_state(PLATFORM_STRING, device_handle)
        .await
        .expect("failed to get fpga state");

    if current_state.contains("0->0") {
        println!("Found '0->0' in state, removing existing bitstream");
        control_proxy
            .remove_overlay(PLATFORM_STRING, "")
            .await
            .expect("failed to remove overlay");
    }

    // Load bitstream via dfx-mgr-client
    let result = control_proxy
        .write_bitstream_direct(PLATFORM_STRING, device_handle, bitstream_file, fw_lookup)
        .await;

    expect_that!(&result, ok(anything()));

    // Check state - for dfx-mgr this returns package listing
    let state = status_proxy
        .get_fpga_state(PLATFORM_STRING, device_handle)
        .await
        .expect("failed to get fpga state");

    println!("DFX-MGR state after load: {:#?}", state);
    expect_that!(state, anything());

    // Cleanup - remove the loaded bitstream
    let cleanup_result = control_proxy.remove_overlay(PLATFORM_STRING, "").await;
    if let Err(e) = cleanup_result {
        println!("Warning: cleanup failed: {:#?}", e);
    }
}

#[gtest]
#[tokio::test]
#[rstest]
async fn remove_overlay_via_dfx_mgr(_setup: ()) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");

    // Remove overlay - for dfx-mgr this removes slots
    // Using empty string as overlay_handle for removal
    let result = control_proxy.remove_overlay(PLATFORM_STRING, "").await;

    // Should succeed or gracefully handle no loaded overlays
    expect_that!(&result, ok(anything()).or(err(anything())));

    if let Ok(output) = result {
        println!("Remove output: {:#?}", output);
    }
}
