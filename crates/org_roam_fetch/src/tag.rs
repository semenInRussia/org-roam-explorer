use sqlx::{self, sqlite::SqliteRow, Row, SqlitePool};

use crate::result::{Error, Result};

use crate::utils::{add_quotes_around, remove_quotes_around};

#[derive(Debug, PartialEq, Clone)]
pub struct Tag {
    name: String,
}

impl<'a> sqlx::FromRow<'a, SqliteRow> for Tag {
    fn from_row(row: &'a SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let name: String = row.try_get("tag")?;
        Ok(Tag::new(remove_quotes_around(&name)))
    }
}

impl Tag {
    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Tag { name: name.into() }
    }

    pub async fn by_name(name: &str, pool: &SqlitePool) -> Result<Self> {
        sqlx::query_as("SELECT tag FROM tags WHERE tag = $1")
            .bind(add_quotes_around(name))
            .fetch_one(pool)
            .await
            .map_err(|err| match err {
                sqlx::error::Error::RowNotFound => Error::TagNotFound,
                _ => Error::DBError(err),
            })
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub async fn all_tags(pool: &SqlitePool) -> Result<Vec<Self>> {
        sqlx::query_as("SELECT DISTINCT tag FROM tags")
            .fetch_all(pool)
            .await
            .map_err(Error::DBError)
    }
}

#[cfg(test)]
mod tests {
    use crate::connection::default_db_pool;
    use crate::{result::Error, tag::Tag};

    #[tokio::test]
    async fn test_tag_name() {
        let pool = default_db_pool().await.expect("I can't open the pool");
        let tag = Tag::by_name("physics", &pool)
            .await
            .expect("Error when fetch a tag");
        assert_eq!(tag.name(), "physics");
    }

    #[tokio::test]
    async fn test_tag_not_found() {
        let pool = default_db_pool().await.expect("I can't open the pool");
        let err = Tag::by_name("stupid id that can't be in db", &pool).await;
        assert!(matches!(err, Err(Error::TagNotFound)));
    }
}
