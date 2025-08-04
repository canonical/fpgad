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

use crate::common::proxies::status_proxy;
use crate::integration_tests::universal::setup;
use googletest::prelude::*;
use rstest::*;
use zbus::Connection;

#[gtest]
#[tokio::test]
#[rstest]
#[case::no_platform("", "fpga0", ok(eq("0")))]
#[case::no_device(
    "universal",
    "",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_platform("x", "", err(displays_as(contains_substring("FpgadError::Argument:"))))]
#[case::bad_device(
    "universal",
    "dev0",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::all_good("universal", "fpga0", ok(eq("0")))]
async fn cases<M: for<'a> Matcher<&'a zbus::Result<String>>>(
    #[case] platform_string: &str,
    #[case] device_handle: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to get fpga state");
    let proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let res = proxy.get_fpga_flags(platform_string, device_handle).await;
    expect_that!(&res, condition);
}
