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

use crate::xilinx_dfx_mgr::setup;
use fpgad_proxies::proxies::status_proxy;
use googletest::prelude::*;
use rstest::*;
use zbus::Connection;

#[gtest]
#[tokio::test]
#[rstest]
async fn get_platform_types_test(_setup: ()) {
    let connection = Connection::system()
        .await
        .expect("failed to create connection");
    let proxy = status_proxy::StatusProxy::new(&connection)
        .await
        .expect("failed to create status proxy");
    let res = proxy.get_platform_types().await;

    expect_that!(&res, ok(contains_substring("xlnx,zynqmp-pcap-fpga")));

    if let Ok(types) = res {
        println!("Available platform types: {:#?}", types);
        // Should include one of the device types dfx-mgr supports
        expect_that!(
            types,
            any![
                contains_substring("versal-fpga"),
                contains_substring("zynq-devcfg-1.0"),
                contains_substring("zynqmp-pcap-fpga")
            ]
        );
    }
}
