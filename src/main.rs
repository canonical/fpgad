use std::{error::Error, future::pending};
use zbus::{connection, interface};

struct Greeter {
    count: u64,
}

#[interface(name = "com.canonical.fpgad.MyGreeter")]
impl Greeter {
    // Can be `async` as well.
    fn say_hello(&mut self, name: &str) -> String {
        self.count += 1;
        format!("Hello {}! I have been called {} times.", name, self.count)
    }
}

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
