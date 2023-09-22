use emacsql::query::QueryAs;

use crate::result::{Error, Result};
use rusqlite::Connection;

#[derive(Debug, PartialEq, Clone)]
pub struct Tag {
    name: String,
}

impl<'a> emacsql::FromRow for Tag {
    fn try_from_row(row: &emacsql::Row) -> emacsql::Result<Self> {
        let name: String = row.get("tag")?;
        Ok(Tag::new(name))
    }
}

impl Tag {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Tag { name: name.into() }
    }

    pub fn by_name(name: &str, conn: &mut Connection) -> Result<Self> {
        conn.prepare("SELECT tag FROM tags WHERE tag = $1")?
            .query_as_one([name])
            .map_err(|err| match err {
                emacsql::Error::QueryReturnedNoRows => Error::TagNotFound,
                _ => Error::DBError(err),
            })
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn all_tags(conn: &mut Connection) -> Result<Vec<Self>> {
        conn.prepare("SELECT DISTINCT tag FROM tags")?
            .query_as([])
            .map_err(Error::DBError)
    }
}

#[cfg(test)]
mod tests {
    use crate::connection::default_db_connection;
    use crate::{result::Error, tag::Tag};

    fn test_tag_name() {
        let mut conn = default_db_connection().unwrap();
        let tag = Tag::by_name("physics", &mut conn).expect("Error when fetch a tag");
        assert_eq!(tag.name(), "physics");
    }

    fn test_tag_not_found() {
        let mut conn = default_db_connection().expect("I can't open the pool");
        let err = Tag::by_name("stupid id that can't be in db", &mut conn);
        assert!(matches!(err, Err(Error::TagNotFound)));
    }
}
