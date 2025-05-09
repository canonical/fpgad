#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("an IO error occured: {0}")]
    Io(#[from] std::io::Error),
}
