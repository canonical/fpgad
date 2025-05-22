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
