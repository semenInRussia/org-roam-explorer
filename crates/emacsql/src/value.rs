use std::str::FromStr;

use crate::{error::FromEmacsqlError, utils::maybe_remove_quotes_around};
// use crate::utils::{add_quotes_around, remove_quotes_around};

// use std::ops::{ControlFlow, FromResidual, Try};

use rusqlite::{
    self,
    types::{FromSql, FromSqlError},
};

pub enum Value {
    Lisp(String),
    Null,
}

impl Value {
    fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

type FromEmacsqlResult<T> = Result<T, FromEmacsqlError>;

// from sqlite::Value to Value
impl FromSql for Value {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        use rusqlite::types::ValueRef::*;

        match value {
            Null => Ok(Self::Null),
            Text(s) => Ok(Self::Lisp(String::from_utf8(s.to_vec()).unwrap())),
            // In EmacSQL value cannot be a integer, it's only a text that represents an Emacs lisp or Null that represents nil")
            Blob(_) | Real(_) | Integer(_) => Err(FromSqlError::InvalidType),
        }
    }
}

impl From<rusqlite::types::Value> for Value {
    fn from(val: rusqlite::types::Value) -> Self {
        use rusqlite::types::Value::*;

        match val {
            Null => Self::Null,
            Text(s) => Self::Lisp(s),
            Real(_) | Integer(_) => {
                unreachable!("in emacsql every value is either string that represents emacs lisp value or Null (nil)")
            }
            _ => unreachable!(),
        }
    }
}

// from Value to any Type
pub trait FromEmacsql: Sized {
    fn from_emacsql(val: Value) -> FromEmacsqlResult<Self>;
}

impl<T: FromEmacsql> FromEmacsql for Option<T> {
    fn from_emacsql(val: Value) -> FromEmacsqlResult<Self> {
        if val.is_null() {
            return Ok(None);
        }
        T::from_emacsql(val).map(Some)
    }
}

pub trait FromLisp: Sized {
    fn from_lisp(s: String) -> FromEmacsqlResult<Self>;
}

impl FromLisp for String {
    fn from_lisp(s: String) -> FromEmacsqlResult<Self> {
        Ok(maybe_remove_quotes_around(s))
    }
}

// impl<I: FromLisp> FromLisp for Vec<I> {}

impl<T: FromLisp> FromEmacsql for T {
    fn from_emacsql(val: Value) -> FromEmacsqlResult<Self> {
        match val {
            Value::Null => Err(FromEmacsqlError::InvalidType),
            Value::Lisp(s) => T::from_lisp(s).or(Err(FromEmacsqlError::InvalidType)),
        }
    }
}

pub trait FromLispAsFromStr: Sized + FromStr {}

impl<T: FromLispAsFromStr> FromLisp for T {
    fn from_lisp(s: String) -> FromEmacsqlResult<Self> {
        s.parse().or(Err(FromEmacsqlError::InvalidType))
    }
}

impl FromLispAsFromStr for u8 {}
impl FromLispAsFromStr for u16 {}
impl FromLispAsFromStr for u32 {}
impl FromLispAsFromStr for u64 {}
impl FromLispAsFromStr for i8 {}
impl FromLispAsFromStr for i16 {}
impl FromLispAsFromStr for i32 {}
impl FromLispAsFromStr for i64 {}
impl FromLispAsFromStr for f32 {}
impl FromLispAsFromStr for f64 {}
impl FromLispAsFromStr for usize {}
