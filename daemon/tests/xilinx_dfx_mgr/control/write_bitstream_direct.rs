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
use crate::xilinx_dfx_mgr::{PLATFORM_STRING, setup};
use googletest::prelude::*;
use rstest::*;
use tokio;
use zbus::Connection;
use zbus::Result;

#[gtest]
#[tokio::test]
#[rstest]
#[case::no_device("", "", err(displays_as(contains_substring("FpgadError::Argument:"))))]
#[case::bad_device(
    "",
    "dev0",
    err(displays_as(contains_substring("FpgadError::Argument:")))
)]
#[case::no_bitstream_path(
    "fpga0",
    "",
    err(displays_as(contains_substring("FpgadSoftenerError::DfxMgr:")))
)]
#[case::bad_bitstream_path(
    "fpga0",
    "/dev/null",
    err(displays_as(contains_substring("FpgadSoftenerError::DfxMgr:")))
)]
#[case::all_good(
    "fpga0",
    "/lib/firmware/k26-starter-kits.bit.bin",
    ok(contains_substring(""))
)]
async fn load_bitstream_cases<M: for<'a> Matcher<&'a Result<String>>>(
    #[case] device_handle: &str,
    #[case] bitstream_file: &str,
    #[case] condition: M,
    _setup: (),
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");

    // Reset flags before test
    if !device_handle.is_empty() {
        let _ = proxy
            .set_fpga_flags(PLATFORM_STRING, device_handle, 0)
            .await;
    }

    let res = proxy
        .write_bitstream_direct(PLATFORM_STRING, device_handle, bitstream_file, "")
        .await;
    expect_that!(&res, condition);

    // Cleanup - remove bitstream if it was successfully loaded
    if res.is_ok() && !device_handle.is_empty() {
        let _ = proxy.remove_overlay(PLATFORM_STRING, "").await;
    }
}
