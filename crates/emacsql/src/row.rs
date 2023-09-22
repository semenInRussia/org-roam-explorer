use rusqlite;

use crate::error::Error;
use crate::prelude::*;
use crate::value::FromEmacsql;

pub struct Row<'a> {
    row: &'a rusqlite::Row<'a>,
}

impl<'a> Row<'a> {
    pub fn get<I: RowIndex + Clone, T: FromEmacsql>(&self, idx: I) -> Result<T> {
        let eql = self.row.get(idx.clone())?;
        T::from_emacsql(eql).map_err(|_| idx.as_invalid())
    }
}

pub trait RowIndex: rusqlite::RowIndex {
    fn as_invalid(&self) -> Error;
}

impl RowIndex for &'_ str {
    fn as_invalid(&self) -> Error {
        Error::InvalidColumnName(self.to_string())
    }
}

impl RowIndex for usize {
    fn as_invalid(&self) -> Error {
        Error::InvalidColumnIndex(*self)
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
