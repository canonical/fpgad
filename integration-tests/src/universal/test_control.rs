// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
// Copyright 2025 Canonical Ltd.
// SPDX-License-Identifier: GPL-3.0-only
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

#[cfg(test)]
mod test_control_proxy {
    use crate::proxies::control_proxy;
    use log::{error, info};
    use tokio;
    use zbus::Connection;
    #[tokio::test]
    async fn test_universal_set_fpga_flags() {
        let connection = Connection::system()
            .await
            .expect("failed to create connection");
        let proxy = control_proxy::ControlProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(
            proxy.set_fpga_flags("universal", "fpga0", 0).await.is_ok(),
            "set_fpga_flags failed"
        );
    }

    #[tokio::test]
    pub async fn test_universal_write_property() {
        let connection = Connection::system()
            .await
            .expect("failed to create connection");
        let proxy = control_proxy::ControlProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        match proxy
            .write_property("/sys/class/fpga_manager/fpga0/key", "fpga0")
            .await
        {
            Ok(s) => {
                info!("test_universal_write_property: Ok, set key: \n{s}")
            }
            Err(e) => error!("test_universal_write_property: Err: {e}"),
        }
        assert!(
            proxy
                .write_property("/sys/class/fpga_manager/fpga0/key", "\n")
                .await
                .is_ok(),
            "Failed to set key using write_property"
        )
    }

    #[tokio::test]
    async fn test_universal_write_bitstream_direct() {
        let connection = Connection::system()
            .await
            .expect("failed to create connection");
        let proxy = control_proxy::ControlProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(
            proxy
                .write_bitstream_direct(
                    "universal",
                    "fpga0",
                    "/lib/firmware/xilinx/k26-starter-kits/k26_starter_kits.bit.bin",
                    "",
                )
                .await
                .is_ok(),
            "failed to write bitstream"
        );
    }
}
