use std::fs::File;

use crate::connection::db_connection;
use crate::result::{Error, Result};
use crate::tag::Tag;
use crate::utils::{add_quotes_around, remove_quotes_around};
use quaint::prelude::*;

// NOTE: I am not use columns from the table Node which for me useless
#[derive(Debug)]
pub struct Node {
    id: Option<String>,
    title: Option<String>,
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
            filename,
        }
    }
}

impl Node {
    pub async fn by_id(id: &str) -> Result<Self> {
        let id = add_quotes_around(id);
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

    pub async fn tags(&self) -> Result<Vec<Tag>> {
        let id = self.id.clone().expect("id of the `Node` isn't exists");
        let query = Select::from_table("tags")
            .column("tag")
            .and_where(Column::new("node_id").equals(id));
        let tags = db_connection()
            .await?
            .select(query)
            .await?
            .into_iter()
            .map(Tag::from)
            .collect();
        Ok(tags)
    }

    pub fn title(&self) -> String {
        self.title
            .clone()
            .map(remove_quotes_around)
            .expect("File of a `Node` isn't given in the instance")
    }

    pub fn filename(&self) -> String {
        self.filename
            .clone()
            .map(remove_quotes_around)
            .expect("File of a `Node` isn't given in the instance")
    }

    pub fn file(&self) -> Result<File> {
        let filename = self.filename();
        File::open(filename).map_err(Error::NodeFileOpenError)
    }
}

pub async fn all_nodes() -> Result<Vec<Node>> {
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

#[cfg(test)]
mod tests {
    use crate::node::Node;

    #[tokio::test]
    async fn node_title() {
        let node = Node::by_id("0decd9d4-4029-4c96-9a5a-75f4f449a4fd").await
            .expect("Node with available id not found");
        assert_eq!(node.title(), "Cross SQL Joining");
    }

    #[tokio::test]
    async fn node_filename() {
        let node = Node::by_id("0decd9d4-4029-4c96-9a5a-75f4f449a4fd").await
            .expect("Node with available id not found");
        assert_eq!(
            node.filename(),
            "c:/Users/hrams/AppData/Roaming/org-roam/20221030190542-sql.org");
    }
}
