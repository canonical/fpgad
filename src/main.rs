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

use std::error::Error;
use std::future::pending;
use std::path::Path;
use zbus::connection;
mod error;
use log::trace;

use platforms::{
    platform::{list_fpga_managers, Fpga, Platform},
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
    trace!("FPGAmanagers scraped.");
    let mut universal_platform = UniversalPlatform::new();
    trace!("{}", universal_platform.fpga("fpga0").name());
    let myfpga = universal_platform.fpga("fpga0");
    match myfpga.state() {
        Err(e) => panic!("{}", e),
        Ok(_) => println!("FPGA detected and loaded as universal_platform."),
    };
    

    // TODO: panic is unnacceptable in daemon so need to handle errors properly. 
    let bitstream_path = Path::new("/lib/firmware/k26-starter-kits.bit.bin");
    let dtbo_path = Path::new("/lib/firmware/k26-starter-kits.dtbo");
    universal_platform.load_package(&bitstream_path, &dtbo_path).expect("TODO: panic message");
    
    // 
    // println!("Attempting to load bitstream from {:?}", bitstream_path);
    // myfpga.load_bitstream(bitstream_name, &bitstream_path, dtbo_name)?;
    // println!("Successfully loaded bitstream and its operational? Waiting 5s.");
    // 
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    println!("The wait is over prepare to be unloaded!");
    universal_platform.unload_package().expect("TODO: panic message");
    println!("Successfully unloaded bitstream? Waiting for dbus messages. (ctrl+C to quit).");

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
