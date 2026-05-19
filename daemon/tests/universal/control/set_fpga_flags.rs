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
use crate::universal::setup;
use googletest::prelude::*;
use rstest::*;
use tokio;
use zbus::Connection;
use zbus::Result;

#[gtest]
#[tokio::test]
#[rstest]
#[case::no_device_handle("", 0, err(displays_as(contains_substring("FpgadError::Argument:"))))]
#[case::bad_device_handle(
    "dev0",
    0,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::max_u32_val("fpga0", u32::MAX, ok(contains_substring("Flags set to")))]
#[case::bad_typo_path(
    "/sy/class/fpga_manager/",
    0,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_short_path(
    "/sys/class/fpga_manager/",
    0,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_handle_in_path(
    "/sys/class/fpga_manager/no-dev/flags",
    0,
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_not_full_path_no_flags(
    "/sys/class/fpga_manager/fpga0/",
    0,
    ok(contains_substring("Flags set to '0x0' for 'fpga0'"))
)]
#[case::all_good_full_path(
    "/sys/class/fpga_manager/fpga0/flags",
    0,
    ok(contains_substring("Flags set to '0x0' for 'fpga0'"))
)]
#[case::all_good("fpga0", 0, ok(contains_substring("Flags set to '0x0' for 'fpga0'")))]
async fn cases<M: for<'a> Matcher<&'a Result<String>>>(
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
        .universal("write_flags", device_hande, &flags.to_string())
        .await;
    expect_that!(&res, condition)
}
