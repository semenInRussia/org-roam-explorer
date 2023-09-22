use rusqlite;

use crate::lisp;

pub type Error = rusqlite::Error;
pub type FromEmacsqlError = rusqlite::types::FromSqlError;

impl From<lisp::Error> for FromEmacsqlError {
    fn from(_error: lisp::Error) -> Self {
        FromEmacsqlError::InvalidType
    }
}
