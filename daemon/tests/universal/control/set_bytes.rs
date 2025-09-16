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
use std::fs;
use tokio;
use zbus::Connection;
use zbus::Result;

fn trim_trailing_vals(mut data: Vec<u8>, val: u8) -> Vec<u8> {
    while data.last() == Some(&val) {
        data.pop();
    }
    data
}

#[gtest]
#[tokio::test]
#[rstest]
#[case::no_path(
    "",
    vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::bad_path(
    "key",
    vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::all_good_abadcode(
    "/sys/class/fpga_manager/fpga0/key",
    vec![0xABu8, 0xADu8, 0xC0u8, 0xDEu8],
    ok(contains_substring(""))
)]
#[case::all_good_reset(
    "/sys/class/fpga_manager/fpga0/key",
    vec![0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8],
    ok(contains_substring(""))
)]
async fn set_key_cases<M: for<'a> Matcher<&'a Result<String>>>(
    #[case] platform_string: &str,
    #[case] data: Vec<u8>,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let res = proxy.write_property_bytes(platform_string, &data).await;
    expect_that!(&res, condition);

    if res.is_ok() {
        println!("{res:?}");
        let file_data = fs::read(platform_string).expect("failed to read back file");
        // trim newlines from file read and trailing nulls from input because of
        // how of the xilinx kernel fpga_mgr.c driver formats key data
        assert_eq!(
            trim_trailing_vals(file_data, 0xA),
            trim_trailing_vals(data, 0x0),
            "file contents do not match expected data"
        );
    }
}
