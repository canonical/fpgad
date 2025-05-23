// This file is part of fpgad, an application to manage FPGA subsystem together with devicetree and kernel modules.
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

use std::{error::Error, future::pending};
use zbus::connection;
mod error;

use platforms::{
    platform::{Fpga, Platform, list_fpga_managers},
    universal::UniversalPlatform,
};

mod comm;
use comm::dbus::interfaces::Greeter;
mod platforms;
mod system_io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let greeter = Greeter { count: 0 };

    let _conn = connection::Builder::session()?
        .name("com.canonical.fpgad.MyGreeter")?
        .serve_at("/com/canonical/fpgad/MyGreeter", greeter)?
        .build()
        .await?;

    // client will send a request to load bitstream to fpga
    // if no fpga name specified fpgad will try all fpgas available under /sys/class/fpga_manager/
    // if no platform specified fpgad will use UniversalPlatform for each fpga
    // available, until it successfully loads the bitstream
    //
    for fpga in list_fpga_managers().iter() {
        let mut universal_platform = UniversalPlatform::new();
        println!("{}", universal_platform.fpga(fpga).name());
    }
    let mut universal_platform = UniversalPlatform::new();
    println!("{}", universal_platform.fpga("fpga0").name());
    match universal_platform.fpga("fpga0").state() {
        Err(e) => panic!("{}", e),
        Ok(_) => println!("Everything seems ok"),
    }

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
