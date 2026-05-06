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
use crate::xilinx_dfx_mgr::setup;
use googletest::prelude::*;
use rstest::*;
use tokio;
use zbus::Connection;

#[gtest]
#[tokio::test]
#[rstest]
#[case::bad_command(
    "badCommand",
    err(displays_as(contains_substring("Option not recognized")))
)]
#[case::no_command(
    "",
    err(displays_as(contains_substring("Expects an argument. Use -h to see options")))
)]
#[case::all_good("-listPackage", ok(displays_as(contains_substring("Accel_type"))))]
async fn cases<M: for<'a> Matcher<&'a zbus::Result<String>>>(
    #[case] command: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let control_proxy = control_proxy::ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");

    // Remove overlay - for dfx-mgr this removes slots
    // Using empty string as overlay_handle for removal
    let result = control_proxy.dfx_mgr(command).await;

    expect_that!(&result, condition);
    if let Ok(output) = result {
        println!("dfx-mgr output: {:#?}", output);
    }
}
