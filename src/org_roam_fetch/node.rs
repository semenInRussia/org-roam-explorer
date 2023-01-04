use quaint::prelude::*;
use crate::result::{Result, Error};
use crate::connection::db_connection;

// NOTE: I am not use columns from the table Node which for me useless
#[derive(Debug)]
pub struct Node {
    id: Option<String>,
    title: Option<String>,
    tags: Option<Vec<String>>,
    filename: Option<String>,
}

impl From<ResultRow> for Node {
    fn from(row: ResultRow) -> Self {
        let id = (&row)["id"].clone().into_string();
        let title = (&row)["title"].clone().into_string();
        let filename = (&row)["file"].clone().into_string();
        Node {
            id,
            title,
            tags: None,
            filename,
        }
    }
}

impl Node {
    pub async fn by_id (id_str: &str) -> Result<Self> {
        let mut id = String::new();
        id.push('"');
        id.push_str(id_str);
        id.push('"');
        let query = Select::from_table("nodes")
            .columns(["id", "title", "file"])
            .and_where(Column::new("id").equals(id));
        db_connection()
            .await?
            .select(query)
            .await?
            .into_iter()
            .nth(0)
            .map(Node::from)
            .ok_or(Error::NodeNotFound)
    }
}
