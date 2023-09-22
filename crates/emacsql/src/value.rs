use std::str::FromStr;

use crate::error::FromEmacsqlError;
use crate::lisp;

use rusqlite::{
    self,
    types::{FromSql, FromSqlError},
};

// look up `emacsql-type-map` in the emacs editor to check available types of
// EmacSQL types.

// In emacsql README.md you could readed that every value in emacsql database will represents as
// string or null, but is myth.  Emacsql also represents numbers and floats with respective types

pub enum Value {
    Lisp(lisp::Value),
    Integer(i64),
    Real(f64),
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
            Text(s) => Ok(Self::Lisp(std::str::from_utf8(s).unwrap().parse()?)),
            // In EmacSQL value cannot be a integer, it's only a text that represents an
            // Emacs lisp or Null that represents nil
            Real(n) => Ok(Self::Real(n)),
            Integer(n) => Ok(Self::Integer(n)),
            Blob(_) => unreachable!("Blob can be represented in EmacSQL database"),
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

impl FromEmacsql for bool {
    fn from_emacsql(val: Value) -> FromEmacsqlResult<Self> {
        match val {
            Value::Null => Ok(false),
            Value::Lisp(lisp::Value::String(s)) if s == "t" => Ok(true),
            _ => {
                if let Value::Lisp(sexp) = val {
                    println!("try convert lisp value: {:?} to bool", sexp);
                }
                Err(FromEmacsqlError::InvalidType)
            }
        }
    }
}

macro_rules! some_integer_impls {
    ($( $for:ident ),*) => {
        $(
            impl FromEmacsql for $for {
                fn from_emacsql(val: Value) -> FromEmacsqlResult<Self> {
                    match val {
                        Value::Integer(n) => Ok(n as $for),
                        _ => Err(FromEmacsqlError::InvalidType),
                    }
                }
            }
        )*
    };
}

some_integer_impls![i8, i16, i32, i64, usize];

macro_rules! some_real_impls {
    ($( $for:ident ),*) => {
        $(
            impl FromEmacsql for $for {
                fn from_emacsql(val: Value) -> FromEmacsqlResult<Self> {
                    match val {
                        Value::Real(n) => Ok(n as $for),
                        _ => Err(FromEmacsqlError::InvalidType),
                    }
                }
            }
        )*
    };
}

some_real_impls![f32, f64];

pub trait FromLisp: Sized {
    fn from_lisp(s: lisp::Value) -> FromEmacsqlResult<Self>;
}

impl<T: FromLisp> FromEmacsql for T {
    fn from_emacsql(val: Value) -> FromEmacsqlResult<Self> {
        match val {
            Value::Lisp(s) => T::from_lisp(s).or(Err(FromEmacsqlError::InvalidType)),
            _ => Err(FromSqlError::InvalidType),
        }
    }
}

impl FromLisp for String {
    fn from_lisp(sexp: lisp::Value) -> FromEmacsqlResult<Self> {
        match sexp {
            lisp::Value::String(s) => Ok(s),
            _ => Err(FromEmacsqlError::InvalidType),
        }
    }
}

pub trait FromLispAsFromStr: Sized + FromStr {}

// TODO: implement `FromLisp` for lisp list and vector
// impl<T: FromLisp> FromLisp for Vec<T> {
//     fn from_lisp(s: String) -> FromEmacsqlResult<Self> {
//         match parse_lisp_program(&s) {
//             Ok(LispObject::)
//         }
//     }
// }

impl<T: FromLispAsFromStr> FromLisp for T {
    fn from_lisp(sexp: lisp::Value) -> FromEmacsqlResult<Self> {
        match sexp {
            lisp::Value::String(s) => s.parse().or(Err(FromEmacsqlError::InvalidType)),
            _ => Err(FromEmacsqlError::InvalidType),
        }
    }
}
