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
use crate::universal::setup;
use googletest::prelude::*;
use rstest::*;
use std::path::Path;
use zbus::Connection;

#[tokio::test]
#[gtest]
#[rstest]
async fn expected_good_overlay_process(_setup: ()) {
    let device_handle: &str = "fpga0";
    let overlay_handle: &str = "fpga0";
    let platform_handle: &str = "universal";
    let overlay_file: &str = "/lib/firmware/k26-starter-kits.dtbo";

    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let status_proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");

    // remove any and all overlays
    for overlay_handle in status_proxy
        .get_overlays()
        .await
        .expect("failed to get overlays")
        .split(",")
    {
        if !overlay_handle.is_empty() {
            control_proxy
                .remove_overlay(platform_handle, overlay_handle)
                .await
                .unwrap_or_else(|_| panic!("failed to remove {overlay_handle}"));
        }
    }

    // reset flags
    control_proxy
        .set_fpga_flags(platform_handle, device_handle, 0)
        .await
        .expect("failed to set fpga flags");
    expect_that!(
        status_proxy
            .get_fpga_flags(platform_handle, device_handle)
            .await
            .expect("failed to get fpga flags"),
        eq("0"),
        "flags not set properly"
    );

    // reset encryption key
    control_proxy
        .write_property("/sys/class/fpga_manager/fpga0/key", "")
        .await
        .expect("failed to reset the encryption key");

    // hopefully successfully write an overlay
    control_proxy
        .apply_overlay(platform_handle, overlay_handle, overlay_file, "")
        .await
        .expect("failed to apply overlay");
    let s = status_proxy
        .get_overlay_status(platform_handle, overlay_handle)
        .await
        .expect("failed to read overlay status");
    let file_name = Path::new(overlay_file)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    expect_that!(s, contains_substring(file_name));
    expect_that!(s, contains_substring("applied"));

    // remove if applied
    control_proxy
        .remove_overlay(platform_handle, overlay_handle)
        .await
        .expect("failed to remove overlay");
    let s = status_proxy
        .get_overlay_status(platform_handle, overlay_handle)
        .await
        .expect("failed to read overlay status");
    expect_that!(s, contains_substring("not present"))
}

#[tokio::test]
#[gtest]
#[rstest]
async fn overlay_already_applied(_setup: ()) {
    let device_handle: &str = "fpga0";
    let overlay_handle: &str = "fpga0";
    let platform_handle: &str = "universal";
    let overlay_file: &str = "/lib/firmware/k26-starter-kits.dtbo";

    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let status_proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");

    // remove any and all overlays
    for overlay_handle in status_proxy
        .get_overlays()
        .await
        .expect("failed to get overlays")
        .split(",")
    {
        if !overlay_handle.is_empty() {
            control_proxy
                .remove_overlay(platform_handle, overlay_handle)
                .await
                .unwrap_or_else(|_| panic!("failed to remove {overlay_handle}"));
        }
    }

    // reset flags
    control_proxy
        .set_fpga_flags(platform_handle, device_handle, 0)
        .await
        .expect("failed to set fpga flags");
    expect_that!(
        status_proxy
            .get_fpga_flags(platform_handle, device_handle)
            .await
            .expect("failed to get fpga flags"),
        eq("0"),
        "flags not set properly"
    );

    // reset encryption key
    control_proxy
        .write_property("/sys/class/fpga_manager/fpga0/key", "")
        .await
        .expect("failed to reset the encryption key");

    // hopefully successfully write an overlay
    control_proxy
        .apply_overlay(platform_handle, overlay_handle, overlay_file, "")
        .await
        .expect("failed to apply overlay");
    let s = status_proxy
        .get_overlay_status(platform_handle, overlay_handle)
        .await
        .expect("failed to read overlay status");
    let file_name = Path::new(overlay_file)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    expect_that!(s, contains_substring(file_name.clone()));
    expect_that!(s, contains_substring("applied"));

    // try and write the same overlay with the same handle - should fail
    let r = control_proxy
        .apply_overlay(platform_handle, overlay_handle, overlay_file, "")
        .await;
    assert_that!(
        &r,
        err(displays_as(contains_substring(
            "FpgadError::Argument: Overlay with this handle already exists "
        ))),
    );
    let s = status_proxy
        .get_overlay_status(platform_handle, overlay_handle)
        .await
        .expect("failed to read overlay status");

    // should not break the already applied one.
    expect_that!(s, contains_substring(file_name));
    expect_that!(s, contains_substring("applied"));

    // remove if applied
    control_proxy
        .remove_overlay(platform_handle, overlay_handle)
        .await
        .expect("failed to remove overlay");
    let s = status_proxy
        .get_overlay_status(platform_handle, overlay_handle)
        .await
        .expect("failed to read overlay status");
    expect_that!(s, contains_substring("not present"))
}

#[tokio::test]
#[gtest]
#[rstest]
#[case::no_dtbo_path("fpga0", "universal", "")]
#[case::bad_platform_string("fpga0", "platform", "/lib/firmware/k26-starter-kits.dtbo")]
#[case::no_platform_string("fpga0", "", "/lib/firmware/k26-starter-kits.dtbo")]
#[case::no_overlay_handle("", "universal", "/lib/firmware/k26-starter-kits.dtbo")]
/// fails if an overlay cannot be applied, and will succeed if the overlay is applied, and then
/// trying to apply it again fails. This test made the "default working case" redundant
async fn argument_errors(
    #[case] overlay_handle: &str,
    #[case] platform_handle: &str,
    #[case] overlay_file: &str,
    _setup: (),
) {
    // good variables used to check status etc
    let good_device_handle: &str = "fpga0";
    let good_platform_handle: &str = "universal";

    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let status_proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");

    // remove any and all overlays
    for overlay_handle in status_proxy
        .get_overlays()
        .await
        .expect("failed to get overlays")
        .split(",")
    {
        if !overlay_handle.is_empty() {
            control_proxy
                .remove_overlay(good_platform_handle, overlay_handle)
                .await
                .unwrap_or_else(|_| panic!("failed to remove {overlay_handle}"));
        }
    }

    // reset flags
    control_proxy
        .set_fpga_flags(good_platform_handle, good_device_handle, 0)
        .await
        .expect("failed to set fpga flags");
    expect_that!(
        status_proxy
            .get_fpga_flags(good_platform_handle, good_device_handle)
            .await
            .expect("failed to get fpga flags"),
        eq("0"),
        "flags not set properly"
    );

    // reset encryption key
    control_proxy
        .write_property("/sys/class/fpga_manager/fpga0/key", "")
        .await
        .expect("failed to reset the encryption key");

    // write a bad overlay path
    let r = control_proxy
        .apply_overlay(platform_handle, overlay_handle, overlay_file, "")
        .await;
    assert_that!(
        &r,
        err(displays_as(contains_substring("FpgadError::Argument: "))),
    );
    if !overlay_handle.is_empty() {
        let s = status_proxy
            .get_overlay_status(good_platform_handle, overlay_handle)
            .await
            .expect("failed to read overlay status");
        expect_that!(s, contains_substring("not present"));
    }

    // remove if applied by accident
    let _ = control_proxy
        .remove_overlay(good_platform_handle, overlay_handle)
        .await;
    if !overlay_handle.is_empty() {
        let s = status_proxy
            .get_overlay_status(good_platform_handle, overlay_handle)
            .await
            .expect("failed to read overlay status");
        expect_that!(s, contains_substring("not present"));
    }
}

#[tokio::test]
#[gtest]
#[rstest]
#[case::bad_dtbo_path("fpga0", "universal", "/lib/firmware/bad-dtbo-path")]
async fn io_errors(
    #[case] overlay_handle: &str,
    #[case] platform_handle: &str,
    #[case] overlay_file: &str,
    _setup: (),
) {
    // good variables used to check status etc
    let good_device_handle: &str = "fpga0";
    let good_platform_handle: &str = "universal";

    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let status_proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");

    // remove any and all overlays
    for overlay_handle in status_proxy
        .get_overlays()
        .await
        .expect("failed to get overlays")
        .split(",")
    {
        if !overlay_handle.is_empty() {
            control_proxy
                .remove_overlay(good_platform_handle, overlay_handle)
                .await
                .unwrap_or_else(|_| panic!("failed to remove {overlay_handle}"));
        }
    }

    // reset flags
    control_proxy
        .set_fpga_flags(good_platform_handle, good_device_handle, 0)
        .await
        .expect("failed to set fpga flags");
    expect_that!(
        status_proxy
            .get_fpga_flags(good_platform_handle, good_device_handle)
            .await
            .expect("failed to get fpga flags"),
        eq("0"),
        "flags not set properly"
    );

    // reset encryption key
    control_proxy
        .write_property("/sys/class/fpga_manager/fpga0/key", "")
        .await
        .expect("failed to reset the encryption key");

    // write a bad overlay path
    let r = control_proxy
        .apply_overlay(platform_handle, overlay_handle, overlay_file, "")
        .await;
    assert_that!(
        &r,
        err(displays_as(contains_substring(
            "FpgadError::OverlayStatus: "
        ))),
    );
    if !overlay_handle.is_empty() {
        let s = status_proxy
            .get_overlay_status(good_platform_handle, overlay_handle)
            .await
            .expect("failed to read overlay status");
        expect_that!(s, contains_substring("not present"));
    }

    // remove if applied by accident
    let _ = control_proxy
        .remove_overlay(good_platform_handle, overlay_handle)
        .await;
    if !overlay_handle.is_empty() {
        let s = status_proxy
            .get_overlay_status(good_platform_handle, overlay_handle)
            .await
            .expect("failed to read overlay status");
        expect_that!(s, contains_substring("not present"));
    }
}
