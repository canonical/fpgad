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
use crate::common::test_functions::compare_result;
use googletest::prelude::*;
use rstest::*;
use zbus::{Connection, Error};

#[gtest]
#[tokio::test]
#[rstest]
#[case::read_name("/sys/class/fpga_manager/fpga0/name", Ok("Xilinx ZynqMP FPGA Manager\n".to_string()))]
#[case::missing_file("/sys/class/fpga_manager/fpga0/read_not_exist.txt", Err(Error::Failure("FpgadError::IORead".into())))]
async fn cases(#[case] property_path: &str, #[case] expected_result: zbus::Result<String>) {
    let connection = Connection::system()
        .await
        .expect("failed to get fpga state");
    let proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    let res = proxy.read_property(property_path).await;
    compare_result(&res, &expected_result);
}
