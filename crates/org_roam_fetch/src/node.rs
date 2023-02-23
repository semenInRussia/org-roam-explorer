use std::fs::File;

use crate::connection::db_connection;
use crate::result::{Error, Result};
use crate::tag::Tag;
use crate::utils::{add_quotes_around, remove_quotes_around};
use quaint::prelude::*;

// NOTE: I am not use columns from the table Node which for me useless
#[derive(Debug, Clone)]
pub struct Node {
    /// the identifier of the node.  This is value of propertry ID in the node `org-mode` heading
    id: Option<String>,
    /// the title of the node.  This is a title of the `org-mode` heading
    title: Option<String>,
    /// name of the file in which stored `org-mode` with the node
    filename: Option<String>,
    /// list of the node's tags
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
            tags: None,
        }
    }
}

impl Node {
    pub async fn by_id(id: &str) -> Result<Self> {
        let tags = "tags".alias("t");
        let query = Select::from_table("nodes")
            .inner_join(tags.on(("t", "node_id").equals(col!("nodes", "id"))))
            .and_where(col!("nodes", "id").equals(add_quotes_around(id)))
            .columns(["id", "title", "file", "tag"]);
        let rows: &Vec<ResultRow> = &db_connection()
            .await?
            .select(query)
            .await?
            .into_iter()
            .collect();
        let mut node = rows.first().map(Node::from).ok_or(Error::NodeNotFound)?;
        let tags = rows.iter().map(Tag::from).collect();
        node.tags = Some(tags);
        Ok(node)
    }

    pub fn file(&self) -> Result<File> {
        File::open(self.filename()?).map_err(Error::NodeFileOpenError)
    }

    pub fn filename(&self) -> Result<String> {
        self.filename
            .clone()
            .map(remove_quotes_around)
            .ok_or(Error::NodeFileNameNotFetched)
    }

    pub async fn tags(&self) -> Result<Vec<Tag>> {
        if let Some(tgs) = &self.tags {
            return Ok(tgs.to_vec());
        }
        let query = Select::from_table("tags")
            .column("tag")
            .and_where("node_id".equals(add_quotes_around(&self.id()?)));
        let tags: Vec<Tag> = select_in_db!(query);
        Ok(tags)
    }

    pub fn id(&self) -> Result<String> {
        self.id
            .clone()
            .map(remove_quotes_around)
            .ok_or(Error::NodeIdNotFetched)
    }

    pub fn title(&self) -> Result<String> {
        self.title
            .clone()
            .map(remove_quotes_around)
            .ok_or(Error::NodeTitleNotFetched)
    }
}

pub async fn all_nodes(limit: usize, offset: usize) -> Result<Vec<Node>> {
    let query = Select::from_table("nodes")
        .columns(["file", "title", "id"])
        .offset(offset)
        .limit(limit);
    let nodes = select_in_db!(query);
    Ok(nodes)
}

pub async fn nodes_of_tag(tag: Tag) -> Result<Vec<Node>> {
    let node_ids_of_tag = Select::from_table("tags")
        .and_where("tag".equals(add_quotes_around(tag.name())))
        .column("node_id");
    let query = Select::from_table("nodes")
        .and_where("id".in_selection(node_ids_of_tag))
        .columns(["file", "title", "id"]);
    let nodes: Vec<Node> = select_in_db!(query);
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
        assert_eq!(node.title().unwrap(), "momentum");
    }

    #[tokio::test]
    async fn test_node_filename() {
        let node = Node::by_id("1")
            .await
            .expect("Node with available id not found");
        assert_eq!(node.filename().unwrap(), "org-roam/momentum.org");
    }

    #[tokio::test]
    async fn test_node_tags() {
        use crate::tag::Tag;
        let node = Node::by_id("1").await.expect("Error when fetch a node");
        assert_eq!(node.tags().await.unwrap(), vec![Tag::new("\"physics\"")]);
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
        use crate::tag::Tag;
        let nodes = all_nodes(128, 0).await.expect("Error when fetch all nodes");
        assert_eq!(nodes.len(), 5);
        let titles: Vec<String> = nodes.iter().map(Node::title).map(Result::unwrap).collect();
        assert_eq!(
            titles,
            vec!["momentum", "mass", "si", "Second Law of Newton", "newton"]
        );
        let momentum = nodes.into_iter().nth(0).unwrap();
        assert_eq!(
            momentum.tags().await.expect("error when fetch node tags"),
            vec![Tag::new("\"physics\"")]
        )
    }

    #[tokio::test]
    async fn test_all_nodes_with_offset_and_limit() {
        let second_nodes = all_nodes(1, 1).await.expect("Error when fetch 1 node after first");
        assert_eq!(second_nodes.len(), 1);
        let node = second_nodes.iter().nth(0).expect("Fetched 0 nodes, instead of 1");
        assert_eq!(node.title().unwrap(), "mass");
    }

    #[tokio::test]
    async fn test_nodes_of_tag() {
        use crate::tag::Tag;
        let tag = Tag::by_name("person")
            .await
            .expect("Error when fetch a tag");
        let nodes = nodes_of_tag(tag)
            .await
            .expect("Error when fetch nodes of a tag");
        assert_eq!(nodes.len(), 1);
        let nodes_titles: Vec<String> = nodes.iter()
            .map(Node::title)
            .map(Result::unwrap)
            .collect();
        assert_eq!(nodes_titles, vec!["newton"]);
    }
}
