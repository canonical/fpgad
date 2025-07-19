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

#[cfg(test)]
mod test_status_proxy {
    use crate::proxies::status_proxy;
    use tokio;
    use zbus::Connection;

    #[tokio::test]
    async fn test_universal_get_fpga_state() {
        let connection = Connection::system()
            .await
            .expect("failed to get fpga state");
        let proxy = status_proxy::StatusProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(
            proxy.get_fpga_state("universal", "fpga0").await.is_ok(),
            "failed to get fpga0's state"
        );
    }

    #[tokio::test]
    async fn test_universal_get_fpga_flags() {
        let connection = Connection::system()
            .await
            .expect("failed to get fpga state");
        let proxy = status_proxy::StatusProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(
            proxy.get_fpga_flags("universal", "fpga0").await.is_ok(),
            "failed to get fpga flags"
        );
    }

    #[tokio::test]
    async fn test_universal_get_overlay_status() {
        // TODO: this should load an overlay first.
        let connection = Connection::system()
            .await
            .expect("failed to get fpga state");
        let proxy = status_proxy::StatusProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(
            proxy.get_overlay_status("universal", "fpga0").await.is_ok(),
            "Failed to get overlay status for overlay 'fpga0'"
        )
    }

    #[tokio::test]
    async fn test_universal_get_overlays() {
        let connection = Connection::system()
            .await
            .expect("failed to get fpga state");
        let proxy = status_proxy::StatusProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(proxy.get_overlays().await.is_ok(), "failed to get overlays")
    }

    #[tokio::test]
    async fn test_universal_get_platform_type() {
        let connection = Connection::system()
            .await
            .expect("failed to get fpga state");
        let proxy = status_proxy::StatusProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(
            proxy.get_platform_type("fpga0").await.is_ok(),
            "failed to get platform type for 'fpga0'"
        )
    }

    #[tokio::test]
    async fn test_universal_get_platform_types() {
        let connection = Connection::system()
            .await
            .expect("failed to get fpga state");
        let proxy = status_proxy::StatusProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(
            proxy.get_platform_types().await.is_ok(),
            "failed to get platform types"
        )
    }

    #[tokio::test]
    async fn test_universal_read_property() {
        let connection = Connection::system()
            .await
            .expect("failed to get fpga state");
        let proxy = status_proxy::StatusProxy::new(&connection)
            .await
            .expect("failed to create control proxy");
        assert!(
            proxy
                .read_property("/sys/class/fpga_manager/fpga0/name")
                .await
                .is_ok(),
            "Failed to read 'name' from 'fpga0'"
        );
    }
}
