use tokio::fs::File;

use sqlx::{self, sqlite::SqliteRow, Row, SqlitePool};

use crate::id::ID;
use crate::result::{Error, Result};
use crate::tag::Tag;
use crate::utils::{add_quotes_around, remove_quotes_around};

// NOTE: I am not use columns from the table Node which for me useless
#[derive(Debug, Clone)]
pub struct Node {
    /// the identifier of a node.  This is the value of the propertry ID in an `org-mode` heading
    id: Option<ID>,
    /// the title of a node.  This is the title of the `org-mode` heading which refered by node
    title: Option<String>,
    /// name of the file in which stored a node
    filename: Option<String>,
    /// list of the node's tags
    tags: Option<Vec<Tag>>,
}

impl<'a> sqlx::FromRow<'a, SqliteRow> for Node {
    fn from_row(row: &'a SqliteRow) -> std::result::Result<Self, sqlx::Error> {
        let node = Self {
            id: row
                .try_get("id")
                .map(|ref s| remove_quotes_around(s).to_string())
                .ok(),
            title: row
                .try_get("title")
                .map(|ref s| remove_quotes_around(s).to_string())
                .ok(),
            filename: row
                .try_get("file")
                .map(|ref s| remove_quotes_around(s).to_string())
                .ok(),
            tags: None,
        };
        Ok(node)
    }
}

impl Node {
    /// create a `Node` instance that referes to the `org-roam` node with a given ID
    pub async fn by_id(id: ID, pool: &SqlitePool) -> Result<Self> {
        let q = r#"
SELECT id, title, file from nodes
where nodes.id = $1"#;
        sqlx::query_as(q)
            .bind(add_quotes_around(id))
            .fetch_one(pool)
            .await
            .map_err(|err| match err {
                sqlx::error::Error::RowNotFound => Error::NodeNotFound,
                _ => Error::DBError(err),
            })
    }

    /// creat e `Node` instance that referes to the `org-roam` node with a given name
    pub async fn by_title<T>(title: T, pool: &SqlitePool) -> Result<Self>
    where
        T: Into<String>,
    {
        let q = format!(
            r#"
SELECT id, title, file FROM nodes
WHERE nodes.title = '"{}"'
"#,
            title.into()
        );
        sqlx::query_as(&q)
            .fetch_one(pool)
            .await
            .map_err(|err| match err {
                sqlx::error::Error::RowNotFound => Error::NodeNotFound,
                _ => Error::DBError(err),
            })
    }

    /// return the opened file in which stored a node
    pub async fn file(&self) -> Result<File> {
        File::open(self.filename()?)
            .await
            .map_err(Error::NodeFileOpenError)
    }

    /// return the path to the file in which stored a node
    ///
    /// if the filename isn't provided, return `Error::NodeFileNameNotFetched`
    pub fn filename(&self) -> Result<String> {
        match &self.filename {
            Some(name) => Ok(name.clone()),
            None => Err(Error::NodeFileNameNotFetched),
        }
    }

    /// returns the vector of tags of a node in `Result` container.
    ///
    /// if the tags didn't fetched, returns `result::Error`
    pub async fn tags(&self, pool: &SqlitePool) -> Result<Vec<Tag>> {
        if let Some(tgs) = &self.tags {
            return Ok(tgs.to_owned());
        }
        let id = self
            .id
            .as_ref()
            .map(add_quotes_around)
            .ok_or(Error::TagNotFound)?;
        let q = format!("SELECT tag FROM tags WHERE node_id = '{id}'");
        sqlx::query_as(&q)
            .fetch_all(pool)
            .await
            .map_err(Error::DBError)
    }

    /// return the ID of a node which consists of 5 parts separated with dash.
    ///
    /// If ID isn't provided, return `Error::NodeIdNotFetched`.
    /// Example of a ID: "5f55037f-3e25-448b-97f2-65efd258265c".
    pub fn id(&self) -> Result<String> {
        match &self.id {
            Some(id) => Ok(id.clone()),
            None => Err(Error::NodeIdNotFetched),
        }
    }

    /// return the title of a node in `Result` container.
    ///
    /// If the title isn't provided, return `Error::NodeTitleNotFetched`
    pub fn title(&self) -> Result<String> {
        match &self.title {
            Some(t) => Ok(t.clone()),
            None => Err(Error::NodeTitleNotFetched),
        }
    }

    /// return all nodes that exists in the database.
    ///
    /// use limit and offset to concretize amount of expected nodes.
    pub async fn all_nodes(limit: usize, offset: usize, pool: &SqlitePool) -> Result<Vec<Node>> {
        sqlx::query_as("SELECT file, title, id FROM nodes LIMIT $1 OFFSET $2")
            .bind(limit as u32)
            .bind(offset as u32)
            .fetch_all(pool)
            .await
            .map_err(Error::DBError)
    }

    /// return all nodes, that have a given tag.
    pub async fn nodes_of_tag(tag: Tag, pool: &SqlitePool) -> Result<Vec<Node>> {
        let q = r#"
SELECT file, title, id
FROM nodes
WHERE id in (SELECT node_id FROM tags WHERE tag = $1)"#;
        sqlx::query_as(q)
            .bind(add_quotes_around(tag.name()))
            .fetch_all(pool)
            .await
            .map_err(Error::DBError)
    }

    pub async fn refers_to(&self, pool: &SqlitePool) -> Result<Vec<Node>> {
        let id = self.id.as_ref().ok_or(Error::NodeIdNotFetched)?;
        let q = format!(
            r#"
SELECT id, file, title
FROM nodes AS n
JOIN links AS l
ON n.id = l.source
WHERE l.dest = '"{}"'"#,
            &id
        );
        sqlx::query_as(&q)
            .fetch_all(pool)
            .await
            .map_err(Error::DBError)
    }
}

#[cfg(test)]
mod tests {
    use crate::connection::default_db_pool;
    use crate::node::Node;

    #[tokio::test]
    async fn test_node_title() {
        let pool = default_db_pool().await.expect("I can't open the pool");
        let node = Node::by_id("1".to_string(), &pool)
            .await
            .expect("Node with available id not found");
        assert_eq!(node.title().unwrap(), "momentum");
    }

    #[tokio::test]
    async fn test_node_filename() {
        let pool = default_db_pool().await.expect("I can't open the pool");
        let node = Node::by_id("1".into(), &pool)
            .await
            .expect("Node with available id not found");
        assert_eq!(node.filename().unwrap(), "org-roam/momentum.org");
    }

    #[tokio::test]
    async fn test_node_tags() {
        use crate::tag::Tag;
        let pool = default_db_pool().await.expect("I can't open the pool");
        let node = Node::by_id("1".into(), &pool)
            .await
            .expect("Error when fetch a node");
        assert_eq!(node.tags(&pool).await.unwrap(), vec![Tag::new("physics")]);
    }

    #[tokio::test]
    async fn test_node_not_found() {
        let pool = default_db_pool().await.expect("I can't open the pool");
        let err = Node::by_id("undefined id".into(), &pool).await;
        assert!(matches!(err, Err(crate::result::Error::NodeNotFound)));
    }

    #[tokio::test]
    async fn test_all_nodes() {
        use crate::tag::Tag;

        let pool = default_db_pool().await.expect("I can't open the pool");
        let nodes = Node::all_nodes(128, 0, &pool)
            .await
            .expect("Error when fetch all nodes");
        assert_eq!(nodes.len(), 5);
        let titles: Vec<String> = nodes.iter().map(Node::title).map(Result::unwrap).collect();
        assert_eq!(
            titles,
            vec!["momentum", "mass", "si", "Second Law of Newton", "newton"]
        );
        let momentum = nodes.into_iter().nth(0).unwrap();
        // fails
        assert_eq!(
            momentum
                .tags(&pool)
                .await
                .expect("error when fetch node tags"),
            vec![Tag::new("physics")]
        );
    }

    #[tokio::test]
    async fn test_all_nodes_with_offset_and_limit() {
        let pool = default_db_pool().await.expect("I can't open the pool");
        let second_nodes = Node::all_nodes(1, 1, &pool)
            .await
            .expect("Error when fetch 1 node after first");
        assert_eq!(second_nodes.len(), 1);
        let node = second_nodes
            .iter()
            .nth(0)
            .expect("Fetched 0 nodes, instead of 1");
        assert_eq!(node.title().unwrap(), "mass");
    }

    #[tokio::test]
    async fn test_nodes_of_tag() {
        use crate::tag::Tag;
        let pool = default_db_pool().await.expect("I can't open the pool");
        let tag = Tag::by_name("person", &pool)
            .await
            .expect("Error when fetch a tag");
        let nodes = Node::nodes_of_tag(tag, &pool)
            .await
            .expect("Error when fetch nodes of a tag");
        assert_eq!(nodes.len(), 1);
        let nodes_titles: Vec<String> = nodes.iter().map(Node::title).map(Result::unwrap).collect();
        assert_eq!(nodes_titles, vec!["newton"]);
    }

    #[tokio::test]
    async fn test_node_refers_to() {
        let pool = default_db_pool().await.expect("I can't open the pool");
        let second_newton_law = Node::by_id("4".to_string(), &pool).await.unwrap();
        let childs = second_newton_law.refers_to(&pool).await.unwrap();
        let childs_names: Vec<String> =
            childs.iter().map(Node::title).map(Result::unwrap).collect();
        assert_eq!(childs_names, ["newton"]);
    }

    #[tokio::test]
    async fn test_node_by_title() {
        let pool = default_db_pool().await.expect("I can't open the pool");
        let si = Node::by_title("si", &pool).await.expect("I don't see SI");
        assert_eq!(si.id().unwrap(), "3");
    }
}
