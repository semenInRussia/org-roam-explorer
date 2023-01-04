pub enum Error {
    DBError(quaint::error::Error),
}

pub type Result<T> = std::result::Result<Error, T>;

impl From<quaint::error::Error> for Error {
    fn from(value: quaint::error::Error) -> Self {
        Error::DBError(value)
    }
}
