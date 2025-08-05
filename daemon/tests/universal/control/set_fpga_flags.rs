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

use crate::common::proxies::control_proxy::ControlProxy;
use crate::universal::{PLATFORM_STRING, setup};
use googletest::prelude::*;
use rstest::*;
use tokio;
use zbus::Connection;
use zbus::Result;

#[gtest]
#[tokio::test]
#[rstest]
#[case::no_device_handle(
    PLATFORM_STRING,
    "",
    0,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_device_handle(
    PLATFORM_STRING,
    "dev0",
    0,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::no_platform_str("", "fpga0", 0, ok(contains_substring("Flags set to 0 for fpga0")))]
#[case::max_u32_val(
    PLATFORM_STRING,
    "fpga0",
    u32::MAX,
    ok(contains_substring("Flags set to"))
)]
#[case::bad_platform_string(
    "xln",
    "fpga0",
    0,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::all_good(
    PLATFORM_STRING,
    "fpga0",
    0,
    ok(contains_substring("Flags set to 0 for fpga0"))
)]
async fn cases<M: for<'a> Matcher<&'a Result<String>>>(
    #[case] platform_string: &str,
    #[case] device_hande: &str,
    #[case] flags: u32,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let res = proxy
        .set_fpga_flags(platform_string, device_hande, flags)
        .await;
    expect_that!(&res, condition)
}
