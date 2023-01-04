use quaint::prelude::*;
use crate::result::{Result, Error};
use crate::connection::db_connection;

// NOTE: I am not use columns from the table Node which for me useless
#[derive(Debug)]
pub struct Node {
    pub id: Option<String>,
    pub title: Option<String>,
    pub tags: Option<Vec<String>>,
    pub filename: Option<String>,
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

    pub async fn tags (&self) -> Result<Vec<String>> {
        let id = self.id.clone().expect("id of the `Node` isn't exists");
        let query = Select::from_table("tags")
            .column("tag")
            .and_where(Column::new("node_id").equals(id));
        let tags = db_connection()
            .await?
            .select(query)
            .await?
            .into_iter()
            .map(|row| row[0].clone().into_string()
                 .expect("In `tags.node_id` column not string"))
            .collect();
        Ok(tags)
    }
}

pub async fn all_nodes () -> Result<Vec<Node>> {
    let query = Select::from_table("nodes").columns(["file", "title", "id"]);
    let nodes = db_connection()
        .await?
        .select(query)
        .await?
        .into_iter()
        .map(Node::from)
        .collect();
    Ok(nodes)
}
