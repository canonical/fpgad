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

use crate::xilinx_dfx_mgr::{
    PLATFORM_STRING, find_slot_ids_in_status_message, remove_detected_slots, setup,
};
use fpgad_proxies::proxies::{control_proxy, status_proxy};
use googletest::prelude::*;
use rstest::*;
use std::path::Path;
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
        .xlnx_sys("write_flags", device_handle, "0")
        .await
        .expect("failed to set fpga flags");
    expect_that!(
        status_proxy
            .xlnx_sys("read_flags", device_handle)
            .await
            .expect("failed to get fpga flags"),
        eq("0")
    );

    // Remove any leftover loaded packages so the load below starts from a clean slate
    remove_detected_slots(&control_proxy, &status_proxy, "").await;

    // Load bitstream via dfx-mgr-client
    let result = control_proxy
        .write_bitstream_direct(PLATFORM_STRING, device_handle, bitstream_file, fw_lookup)
        .await;
    expect_that!(
        &result,
        ok(displays_as(contains_substring("Loaded with slot_handle")))
    );

    // Check state - for dfx-mgr this returns package listing
    let state = status_proxy
        .get_fpga_state(PLATFORM_STRING, device_handle)
        .await;

    println!("DFX-MGR state after load: {:#?}", state);
    expect_that!(state, ok(displays_as(contains_substring("accelType"))));

    // Cleanup - find the ID of the loaded .bit.bin row in the status message
    // and remove that slot. Match on the base file name since the Accelerator
    // column of the listing does not contain the full path.
    let file_name = Path::new(bitstream_file)
        .file_name()
        .and_then(|n| n.to_str())
        .expect("bitstream file path has no valid file name");
    let status_message = status_proxy
        .get_status_message(PLATFORM_STRING)
        .await
        .expect("failed to get status message");
    println!("Status message:\n{status_message}");
    let slot_ids = find_slot_ids_in_status_message(&status_message, file_name);
    for slot_id in slot_ids {
        println!("Removing slot with ID: {slot_id}");
        let cleanup_result = control_proxy
            .remove_overlay(PLATFORM_STRING, &slot_id)
            .await;
        if let Err(e) = cleanup_result {
            println!("Warning: cleanup failed: {:#?}", e);
        }
    }
}

#[gtest]
#[tokio::test]
#[rstest]
async fn remove_bitstream_via_dfx_mgr(_setup: ()) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");

    // Remove overlay - for dfx-mgr this removes slots
    // Using empty string as overlay_handle for removal
    let result = control_proxy
        .remove_bitstream(PLATFORM_STRING, "fpga0", "")
        .await;

    // Should succeed or gracefully handle no loaded overlays
    expect_that!(
        &result,
        ok(displays_as(contains_substring("returns: 0 (Ok)")))
            .or(err(displays_as(contains_substring("returns: -1 (Error)"))))
    );

    if let Ok(output) = result {
        println!("Remove output: {:#?}", output);
    }
}
