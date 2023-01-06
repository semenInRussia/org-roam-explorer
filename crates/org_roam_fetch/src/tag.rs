use quaint::{connector::ResultRow, prelude::*};

use crate::connection::db_connection;
use crate::result::{Error, Result};
use crate::utils::{remove_quotes_around, add_quotes_around};

#[derive(Debug, PartialEq)]
pub struct Tag {
    name: String,
}

impl From<ResultRow> for Tag {
    fn from(row: ResultRow) -> Self {
        row["tag"]
            .clone()
            .into_string()
            .map(Tag::new)
            .expect("A given row of the table `tags` hasn't column `tag`")
    }
}

impl Tag {
    pub async fn by_name (name: &str) -> Result<Self> {
        let query = Select::from_table("tags")
            .column("tag")
            .and_where("tag".equals(add_quotes_around(name)));
        db_connection()
            .await?
            .select(query)
            .await?
            .into_iter()
            .nth(0)
            .map(Tag::from)
            .ok_or(Error::TagNotFound)
    }

    pub fn name (&self) -> String {
        remove_quotes_around(self.name.clone())
    }

    pub fn new<T> (name: T) -> Self where T: Into<String> {
        Tag { name: name.into() }
    }
}

#[cfg(test)]
mod tests {
    use crate::tag::Tag;

    #[tokio::test]
    async fn test_tag_name () {
        let tag = Tag::by_name("physics").await
            .expect("Error when fetch a tag");
        assert_eq!(tag.name(), "physics");
    }
}
