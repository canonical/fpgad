use std::{error::Error, future::pending};
use zbus::connection;

mod comm;
use comm::dbus::interfaces::Greeter;

mod platforms;
mod system_io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let greeter = Greeter { count: 0 };

    let _conn = connection::Builder::session()?
        .name("com.canonical.fpgad.MyGreeter")?
        .serve_at("/com/canonical/fpgad/MyGreeter", greeter)?
        .build()
        .await?;

    // Do other things or go to wait forever
    pending::<()>().await;

    Ok(())
}
