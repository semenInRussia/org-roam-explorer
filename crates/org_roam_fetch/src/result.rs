#[derive(Debug)]
pub enum Error {
    /// an error with database
    DBError(quaint::error::Error),
    /// a node (`Node`) isn't found in database
    NodeNotFound,
    /// open a file of a node (`Node`) isn't work
    NodeFileOpenError(std::io::Error),
    /// Error when `Tag` isn't found in DB
    TagNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<quaint::error::Error> for Error {
    fn from(value: quaint::error::Error) -> Self {
        Error::DBError(value)
    }
}
