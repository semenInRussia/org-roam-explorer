use rusqlite::{self, Params};

use crate::error::Error;
use crate::prelude::Result;
use crate::row::{FromRow, Row};

pub trait QueryAs {
    fn query_as_one<P: Params, T: FromRow>(&mut self, params: P) -> Result<T>;
    fn query_as<P: Params, T: FromRow>(&mut self, params: P) -> Result<Vec<T>>;
}

impl<'a> QueryAs for rusqlite::Statement<'a> {
    fn query_as_one<P: Params, T: FromRow>(&mut self, params: P) -> Result<T> {
        self.query(params)?
            .next()?
            .ok_or(Error::QueryReturnedNoRows)
            .map(Row::from) // converts rusqlite Row to emacsql Row
            .and_then(|r| T::try_from_row(&r))
    }

    fn query_as<P: Params, T: FromRow>(&mut self, params: P) -> Result<Vec<T>> {
        let mut rows = self.query(params)?;
        let mut objs = Vec::new();
        while let Some(row) = rows.next()? {
            objs.push(T::try_from_row(&row.into())?);
        }
        Ok(objs)
    }
}
