use rusqlite;

use std::fmt::{Debug, Formatter};

pub enum Error {
    DBError(rusqlite::Error),
    InvalidColumnType,
    RowNotFound,
}

impl Debug for Error {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        use Error::*;

        match self {
            DBError(err) => writeln!(fmt, "{:?}", err),
            InvalidColumnType => writeln!(fmt, "Error(InvalidColumnType)"),
            RowNotFound => writeln!(fmt, "Error(RowNotFound)"),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        match err {
            rusqlite::Error::InvalidColumnType { .. } => Self::InvalidColumnType,
            err => Self::DBError(err),
        }
    }
}
