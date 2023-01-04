#[derive(Debug)]
pub enum Error {
    /// Error from `quaint`
    DBError(quaint::error::Error),
    /// Error when `Node` isn't found in DB
    NodeNotFound,
    /// Error when `Tag` isn't found in DB
    TagNotFound
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<quaint::error::Error> for Error {
    fn from(value: quaint::error::Error) -> Self {
        Error::DBError(value)
    }
}
