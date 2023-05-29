use std::str::FromStr;

use rusqlite;

use crate::error::Error;
use crate::prelude::*;
use crate::utils::remove_quotes_around;

pub trait FromEmacsql: Sized {
    fn try_from_sqlite_str(s: String) -> Result<Self>;
}

impl<T: FromStr> FromEmacsql for T {
    fn try_from_sqlite_str(s: String) -> Result<Self> {
        s.parse().or(Err(Error::InvalidColumnType))
    }
}

pub struct Row<'a> {
    row: &'a rusqlite::Row<'a>,
}

impl<'a> Row<'a> {
    pub fn get<I: rusqlite::RowIndex, T: FromEmacsql>(&self, idx: I) -> Result<T> {
        self.row
            .get(idx)
            .map_err(Error::from)
            .map(remove_quotes_around::<String, String>)
            .and_then(FromEmacsql::try_from_sqlite_str)
            .or(Err(Error::InvalidColumnType))
    }
}

impl<'a> From<&'a rusqlite::Row<'a>> for Row<'a> {
    fn from(row: &'a rusqlite::Row) -> Self {
        Self { row }
    }
}

pub trait FromRow: Sized {
    fn try_from_row(row: &Row) -> Result<Self>;
}
