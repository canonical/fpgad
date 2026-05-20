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

use crate::common::proxies::control_proxy;
use crate::xilinx_dfx_mgr::{PLATFORM_STRING, setup};
use googletest::prelude::*;
use rstest::*;
use zbus::Connection;

#[gtest]
#[tokio::test]
#[rstest]
#[case::bad_path(
    "fpga0",
    "/bad/path.dtbo",
    "",
    err(displays_as(contains_substring("FpgadSoftenerError::DfxMgr:")))
)]
#[case::invalid_file(
    "fpga0",
    "/lib/firmware/not_an_overlay.dtbo",
    "",
    err(displays_as(contains_substring("Failed to parse dtbo file")))
)]
#[case::invalid_utf8(
    "fpga0",
    "/lib/firmware/overlay_invalid_utf8_firmware.dtbo",
    "",
    err(displays_as(contains_substring("Failed to parse firmware-name as UTF-8 string")))
)]
#[case::valid_but_no_firmware(
    "fpga0",
    "/lib/firmware/overlay_no_firmware_name.dtbo",
    "",
    err(displays_as(contains_substring("property not found in dtbo file")))
)]
#[case::valid_but_missing_firmware(
    "fpga0",
    "/lib/firmware/overlay_nonexistent_firmware.dtbo",
    "",
    err(displays_as(contains_substring("not found in lookup path")))
)]

async fn bad_overlay_cases<M: for<'a> Matcher<&'a zbus::Result<String>>>(
    #[case] overlay_handle: &str,
    #[case] overlay_source_path: &str,
    #[case] firmware_lookup_path: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create status proxy");
    let res = proxy
        .apply_overlay(
            PLATFORM_STRING,
            overlay_handle,
            overlay_source_path,
            firmware_lookup_path,
        )
        .await;

    expect_that!(&res, condition);
}
