use std::{fs::OpenOptions, io::Read};

use std::io::Write as _;

pub fn fs_read(file_path: &str) -> Result<String, std::io::Error> {
    let mut buf: String = String::new();
    let result = OpenOptions::new()
        .read(true)
        .open(file_path)
        .and_then(|mut f| f.read_to_string(&mut buf));

    // do checks on the data we got if necessary
    match result {
        Ok(_) => Ok(buf),
        Err(e) => Err(e),
    }
}

pub fn fs_write(
    file_path: &str,
    create: bool,
    value: impl AsRef<str>,
) -> Result<(), std::io::Error> {
    OpenOptions::new()
        .create(create)
        .read(false)
        .write(true)
        .open(file_path)
        .and_then(|mut f| write!(f, "{}", value.as_ref()))
}
