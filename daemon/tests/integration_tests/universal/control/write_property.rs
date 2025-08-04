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
use crate::integration_tests::universal::setup;
use googletest::prelude::*;
use rstest::rstest;
use tokio;
use zbus::{Connection, Result};

#[gtest]
#[tokio::test]
#[rstest]
#[case::out_of_tree(
    "/lib/firmware/new_file",
    "",
    err(displays_as(contains_substring("Cannot access property")))
)]
#[case::missing_file(
    "/sys/class/fpga_manager/fpga0/write_not_exist.txt",
    "",
    err(displays_as(contains_substring("FpgadError::IOWrite: ")))
)]
#[case::reset_key("/sys/class/fpga_manager/fpga0/key", "", ok(anything()))]
pub async fn cases<M: for<'a> Matcher<&'a Result<String>>>(
    #[case] path: &str,
    #[case] data: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let res = proxy.write_property(path, data).await;
    expect_that!(&res, condition);
}
