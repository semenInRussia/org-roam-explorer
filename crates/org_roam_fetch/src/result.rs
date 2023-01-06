#[derive(Debug)]
pub enum Error {
    /// an error with database
    DBError(quaint::error::Error),
    /// a node (`Node`) isn't found in the database
    NodeNotFound,
    /// opening a node file (`Node`) doesn't work
    NodeFileOpenError(std::io::Error),
    /// a tag (`Tag`) isn't found in the database
    TagNotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<quaint::error::Error> for Error {
    fn from(value: quaint::error::Error) -> Self {
        Error::DBError(value)
    }
}
