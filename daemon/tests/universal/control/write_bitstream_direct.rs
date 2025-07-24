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
use crate::universal;
use googletest::prelude::*;
use rstest::rstest;
use tokio;
use tokio::time::{Duration, timeout};
use zbus::{Connection, Error};

#[gtest]
#[tokio::test]
#[rstest]
#[case::no_firmware_lookup(universal::PLATFORM_STRING, "fpga0","/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin", "", Ok("loaded to".into()))]
#[case::path_with_firmware_lookup(universal::PLATFORM_STRING, "fpga0","/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin", "/lib/firmware/xilinx/k26-starter-kits/", Ok("loaded to".into()))]
#[case::bad_firmware_lookup(universal::PLATFORM_STRING, "fpga0","/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin", "/firmware/xilinx/k26-starter-kits/",Err(Error::Failure("FpgadError::Argument:".into())))]
#[case::bad_path(universal::PLATFORM_STRING, "fpga0","k26_starter_kits.bit.bin", "", Err(Error::Failure("FpgadError::Argument:".into())))]
#[case::bad_bitstream_data(universal::PLATFORM_STRING, "fpga0","/lib/firmware/xilinx/k24-starter-kits/k24_starter_kits.bit.bin", "/lib/firmware/xilinx/k24-starter-kits/", Err(Error::Failure("deadline has elapsed".into())))]
async fn cases(
    #[case] platform_str: &str,
    #[case] device_hande: &str,
    #[case] bitstream_path_str: &str,
    #[case] firmware_lookup_path_str: &str,
    #[case] expected_result: zbus::Result<String>,
) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = ControlProxy::new(&connection)
        .await
        .expect("failed to create control proxy");
    if expected_result.is_ok() {
        proxy
            .set_fpga_flags(platform_str, device_hande, 0)
            .await
            .expect("Failed to reset flags during test");
    }
    /* The timeout is required because the xilinx driver returns an error code with
     *  absolute value larger than 4096, (less than -4096) breaking conventions and
     * causing Rust stdlib to panic because it tries to index to the error code:
     * `range start index 18446744073709545980 out of range for slice of length 24`
     */
    // todo: revert the timeout once the error codes from xilinx driver are fixed
    let timeout_result = timeout(
        Duration::from_secs(10),
        proxy.write_bitstream_direct(
            platform_str,
            device_hande,
            bitstream_path_str,
            firmware_lookup_path_str,
        ),
    )
    .await;

    match timeout_result {
        Ok(res) => compare_result(&res, &expected_result),
        Err(e) => {
            assert_that!(
                e.to_string(),
                contains_substring(expected_result.unwrap_err().to_string())
            );
        }
    }
}
