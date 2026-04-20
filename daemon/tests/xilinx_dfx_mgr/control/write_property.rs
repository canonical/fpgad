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
use crate::common::proxies::control_proxy::ControlProxy;
use crate::xilinx_dfx_mgr::setup;
use googletest::prelude::*;
use rstest::*;
use std::fs;
use tokio;
use zbus::Connection;
use zbus::Result;
#[gtest]
#[tokio::test]
#[rstest]
#[case::no_path("", "", err(displays_as(contains_substring("FpgadError::Argument:"))))]
#[case::bad_path(
    "key",
    "",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::path_traversal(
    "/sys/class/fpga_manager/../../../usr/bin/evil_file.sh",
    "",
    err(displays_as(contains_substring("path traversal")))
)]
async fn write_property_cases<M: for<'a> Matcher<&'a Result<String>>>(
    #[case] platform_string: &str,
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
    let res = proxy.write_property(platform_string, data).await;
    expect_that!(&res, condition);
    if res.is_ok() && platform_string.starts_with("/sys/") {
        println!("{res:?}");
        let file_data = fs::read_to_string(platform_string).expect("failed to read back file");
        let trimmed_data = file_data.trim();
        assert_eq!(
            trimmed_data, data,
            "file contents do not match expected data"
        );
    }
}
