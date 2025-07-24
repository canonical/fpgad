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
use crate::common::test_functions::compare_result;
use crate::universal::PLATFORM_STRING;
use googletest::prelude::*;
use rstest::*;
use tokio;
use zbus::Result;
use zbus::{Connection, Error};

#[gtest]
#[tokio::test]
#[rstest]
#[case::all_good(PLATFORM_STRING, "fpga0", 0, Ok("Flags set to 0 for fpga0".to_string()))]
#[case::no_device_handle(PLATFORM_STRING, "", 0, Err(Error::Failure("FpgadError::Argument:".into())))]
#[case::bad_device_handle(PLATFORM_STRING, "dev0", 0, Err(Error::Failure("FpgadError::Argument:".into())))]
#[case::no_platform_str("", "fpga0", 0, Ok("Flags set to 0 for fpga0".to_string()))]
#[case::flag_too_large(PLATFORM_STRING, "fpga0", u32::MAX - 1, Err(Error::Failure("FpgadError::Flag:".into())))]
#[case::bad_platform_string("xln", "fpga0", 0, Err(Error::Failure("FpgadError::Argument:".into())))]
async fn cases(
    #[case] platform_string: &str,
    #[case] device_hande: &str,
    #[case] flags: u32,
    #[case] exp: Result<String>,
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
    compare_result(&res, &exp)
}
