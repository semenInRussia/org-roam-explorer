use sqlx::{self, SqlitePool};

use crate::result::{Result, Error};

use crate::utils::{add_quotes_around, remove_quotes_around};

#[derive(sqlx::FromRow, Debug, PartialEq, Clone)]
pub struct Tag {
    #[sqlx(rename = "tag")]
    name: String,
}

impl Tag {
    pub async fn by_name(name: &str, pool: &SqlitePool) -> Result<Self> {
        sqlx::query_as("SELECT tag FROM tags WHERE tag = $1")
            .bind(add_quotes_around(name))
            .fetch_one(pool).await
            .map_err(Error::DBError)
    }

    pub fn name(&self) -> String {
        remove_quotes_around(&self.name).to_owned()
    }

    pub fn new<T>(name: T) -> Self
    where
        T: Into<String>,
    {
        Tag { name: name.into() }
    }

    pub async fn all_tags(pool: &SqlitePool) -> Result<Vec<Self>> {
        sqlx::query_as("SELECT DISTINCT tag FROM tags")
            .fetch_all(pool).await
            .map_err(Error::DBError)
    }
}

#[cfg(test)]
mod tests {
    use crate::tag::Tag;

    #[tokio::test]
    async fn test_tag_name() {
        let tag = Tag::by_name("physics").await
            .expect("Error when fetch a tag");
        assert_eq!(tag.name(), "physics");
    }
}
