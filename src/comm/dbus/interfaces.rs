use zbus::interface;

pub struct Greeter {
    pub count: u64,
}

#[interface(name = "com.canonical.fpgad.MyGreeter")]
impl Greeter {
    // Can be `async` as well.
    fn say_hello(&mut self, name: &str) -> String {
        self.count += 1;
        format!("Hello {}! I have been called {} times.", name, self.count)
    }
}
