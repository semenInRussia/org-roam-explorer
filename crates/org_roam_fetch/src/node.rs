use std::fs::File;

use crate::connection::db_connection;
use crate::result::{Error, Result};
use crate::tag::Tag;
use crate::utils::{add_quotes_around, remove_quotes_around};
use quaint::prelude::*;

// NOTE: I am not use columns from the table Node which for me useless
#[derive(Debug, Clone)]
pub struct Node {
    id: Option<String>,
    title: Option<String>,
    filename: Option<String>,
    tags: Option<Vec<Tag>>,
}

impl From<&ResultRow> for Node {
    fn from(row: &ResultRow) -> Self {
        let id = row["id"].clone().into_string();
        let title = row["title"].clone().into_string();
        let filename = row["file"].clone().into_string();
        Node {
            id,
            title,
            filename,
            tags: None
        }
    }
}

impl Node {
    pub async fn by_id(id: &str) -> Result<Self> {
        let id = add_quotes_around(id);
        let tags = "tags".alias("t");
        let node_id = Column::from(("nodes", "id"));
        let tag_node_id = ("t", "node_id");
        let query = Select::from_table("nodes")
            .inner_join(tags.on(tag_node_id.equals(node_id.clone())))
            .and_where(node_id.equals(id))
            .columns(["id", "title", "file", "tag"]);
        let rows: &Vec<ResultRow> = &db_connection()
            .await?
            .select(query)
            .await?
            .into_iter()
            .collect();
        let mut node = rows
            .first()
            .map(Node::from)
            .clone()
            .ok_or(Error::NodeNotFound)?;
        let tags = rows.iter().map(Tag::from).collect();
        node.tags = Some(tags);
        Ok(node)
    }


    pub fn file(&self) -> Result<File> {
        let filename = self.filename();
        File::open(filename).map_err(Error::NodeFileOpenError)
    }

    pub fn filename(&self) -> String {
        self.filename
            .clone()
            .map(remove_quotes_around)
            .expect("File of a `Node` isn't given in the instance")
    }

    pub fn tags(&self) -> Vec<Tag> {
        self.tags
            .clone()
            .expect("Tags haven't setted yet, create Node using Node::by_id")
    }

    pub fn title(&self) -> String {
        self.title
            .clone()
            .map(remove_quotes_around)
            .expect("File of a `Node` isn't given in the instance")
    }
}

pub async fn all_nodes() -> Result<Vec<Node>> {
    let query = Select::from_table("nodes").columns(["file", "title", "id"]);
    let nodes = db_connection()
        .await?
        .select(query)
        .await?
        .into_iter()
        .map(|row| (&row).into())
        .collect();
    Ok(nodes)
}

async fn nodes_of_tag(tag: Tag) -> Result<Vec<Node>> {
    let node_ids_of_tag = Select::from_table("tags")
        .and_where("tag".equals(add_quotes_around(tag.name())))
        .column("node_id");
    let query = Select::from_table("nodes")
        .and_where("id".in_selection(node_ids_of_tag))
        .columns(["file", "title", "id"]);
    let nodes = db_connection()
        .await?
        .select(query)
        .await?
        .into_iter()
        .map(|row| (&row).into())
        .collect();
    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use crate::node::{all_nodes, nodes_of_tag, Node};

    #[tokio::test]
    async fn test_node_title() {
        let node = Node::by_id("1")
            .await
            .expect("Node with available id not found");
        assert_eq!(node.title(), "momentum");
    }

    #[tokio::test]
    async fn test_node_filename() {
        let node = Node::by_id("1")
            .await
            .expect("Node with available id not found");
        assert_eq!(node.filename(), "org-roam/momentum.org");
    }

    #[tokio::test]
    async fn test_node_tags() {
        use crate::tag::Tag;
        let node = Node::by_id("1").await.expect("Error when fetch a node");
        assert_eq!(
            node.tags(),
            vec![Tag::new("\"physics\"")]
        );
    }

    #[tokio::test]
    #[should_panic(expected = "given a Error::NoteNotFound")]
    async fn test_node_not_found() {
        Node::by_id("undefined id")
            .await
            .expect("given a Error::NoteNotFound");
    }

    #[tokio::test]
    async fn test_all_nodes() {
        let nodes = all_nodes().await.expect("Error when fetch all nodes");
        assert_eq!(nodes.len(), 5);
        let titles: Vec<String> = nodes.iter().map(Node::title).collect();
        assert_eq!(
            titles,
            vec!["momentum", "mass", "si", "Second Law of Newton", "newton"]
        );
    }

    #[tokio::test]
    async fn test_nodes_of_tag() {
        use crate::tag::Tag;
        let tag = Tag::by_name("person")
            .await
            .expect("Error when fetch a tag");
        let nodes = nodes_of_tag(tag).await
            .expect("Error when fetch nodes of a tag");
        assert_eq!(nodes.len(), 1);
        let nodes_titles: Vec<String> = nodes.iter().map(Node::title).collect();
        assert_eq!(nodes_titles, vec!["newton"]);
    }
}
