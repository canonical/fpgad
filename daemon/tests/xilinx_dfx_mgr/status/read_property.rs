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
use crate::xilinx_dfx_mgr::setup;
use googletest::prelude::*;
use rstest::*;
use std::fs;
use zbus::Connection;
use zbus::Result;

#[gtest]
#[tokio::test]
#[rstest]
#[case::no_path("", err(displays_as(contains_substring("FpgadError::Argument:"))))]
#[case::bad_path("key", err(displays_as(contains_substring("FpgadError::Argument:"))))]
#[case::path_traversal(
    "/sys/class/fpga_manager/../../../usr/bin/evil_file.sh",
    err(displays_as(contains_substring("path traversal")))
)]
#[case::all_good("/sys/class/fpga_manager/fpga0/name", ok(anything()))]
async fn read_property_cases<M: for<'a> Matcher<&'a Result<String>>>(
    #[case] property_path: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");
    let res = proxy.read_property(property_path).await;
    expect_that!(&res, condition);

    // Verify result matches direct file read
    if res.is_ok() && property_path.starts_with("/sys/") {
        let direct_read = fs::read_to_string(property_path)
            .expect("failed to read file directly")
            .to_string();
        let result = res.unwrap();
        expect_that!(result.as_str(), eq(direct_read.as_str()));
    }
}
