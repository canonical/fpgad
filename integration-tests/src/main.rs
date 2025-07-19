// This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
// Copyright 2025 Canonical Ltd.
// SPDX-License-Identifier: GPL-3.0-only
// fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
// fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
// You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

mod universal;

mod proxies;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    // universal::test_status::test_universal_status().await;
    // universal::test_control::test_universal_control().await;
    // universal::test_sequences::test_universal_sequences().await;
}
