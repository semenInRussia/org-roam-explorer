use quaint::{connector::ResultRow, prelude::*};

use crate::connection::db_connection;
use crate::result::{Error, Result};

#[derive(Debug)]
pub struct Tag {
    name: String,
}

impl From<ResultRow> for Tag {
    fn from(row: ResultRow) -> Self {
        let mut name = row["tag"]
            .clone()
            .into_string()
            .expect("A given row of the table `tags` hasn't column `tag`");
        // remove quotes around
        name.remove(0);
        name.pop();
        Tag { name }
    }
}

impl Tag {
    pub async fn by_name (name_str: &str) -> Result<Self> {
        let mut name = String::new();
        name.push('"');
        name.push_str(name_str);
        name.push('"');
        let query = Select::from_table("tags")
            .column("tag")
            .and_where(Column::new("tag").equals(name));
        db_connection()
            .await?
            .select(query)
            .await?
            .into_iter()
            .nth(0)
            .map(Tag::from)
            .ok_or(Error::TagNotFound)
    }
}
