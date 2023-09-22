use emacsql::QueryAs;

use rusqlite::Connection;
use std::fs::File;

use crate::id::ID;
use crate::result::{Error, Result};
use crate::tag::Tag;

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

impl emacsql::FromRow for Node {
    fn try_from_row(row: &emacsql::Row) -> emacsql::Result<Self> {
        let node = Self {
            id: row.get("id").ok(),
            title: row.get("title").ok(),
            filename: row.get("file").ok(),
            tags: None,
        };
        Ok(node)
    }
}

impl Node {
    /// create a `Node` instance that referes to the `org-roam` node with a given ID
    pub fn by_id(id: ID, conn: &mut Connection) -> Result<Self> {
        let q = r#"SELECT id, title, file from nodes where nodes.id = $1"#;
        conn.prepare(q)?
            .query_as_one([id])
            .map_err(|err| match err {
                emacsql::Error::QueryReturnedNoRows => Error::NodeNotFound,
                _ => Error::DBError(err),
            })
    }

    /// creat e `Node` instance that referes to the `org-roam` node with a given name
    pub fn by_title<T>(title: T, conn: &mut Connection) -> Result<Self>
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
        conn.prepare(&q)?.query_as_one([]).map_err(|err| match err {
            emacsql::Error::QueryReturnedNoRows => Error::NodeNotFound,
            _ => Error::DBError(err),
        })
    }

    /// return the opened file in which stored a node
    pub fn file(&self) -> Result<File> {
        File::open(self.filename()?).map_err(Error::NodeFileOpenError)
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
    pub fn tags(&self, conn: &mut Connection) -> Result<Vec<Tag>> {
        if let Some(tgs) = &self.tags {
            return Ok(tgs.to_owned());
        }
        let id = self.id.as_ref().ok_or(Error::TagNotFound)?;
        let q = format!("SELECT tag FROM tags WHERE node_id = '{id}'");
        conn.prepare(&q)?.query_as([]).map_err(Error::DBError)
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
    pub fn all_nodes(limit: usize, offset: usize, conn: &mut Connection) -> Result<Vec<Node>> {
        conn.prepare("SELECT file, title, id FROM nodes LIMIT $1 OFFSET $2")?
            .query_as([limit, offset])
            .map_err(Error::DBError)
    }

    /// return all nodes, that have a given tag.
    pub fn nodes_of_tag(tag: Tag, conn: &mut Connection) -> Result<Vec<Node>> {
        let q = r#"
SELECT file, title, id
FROM nodes
WHERE id in (SELECT node_id FROM tags WHERE tag = $1)"#;
        conn.prepare(q)?
            .query_as([tag.name()])
            .map_err(Error::DBError)
    }

    pub fn refers_to(&self, conn: &Connection) -> Result<Vec<Node>> {
        let id = self.id.as_ref().ok_or(Error::NodeIdNotFetched)?;
        let q = format!(
            r#"
SELECT id, title, file
FROM links AS l
JOIN nodes AS n
ON l.dest = n.id
WHERE l.source = '"{}"'
"#,
            &id
        );
        conn.prepare(&q)?.query_as([]).map_err(Error::DBError)
    }

    /// returns the vector of nodes that refers to the current node
    pub fn backlinks(&self, conn: &mut Connection) -> Result<Vec<Node>> {
        let id = self.id.as_ref().ok_or(Error::NodeIdNotFetched)?;
        let q = format!(
            r#"
SELECT id, title, file
FROM links AS l
JOIN nodes AS n
ON l.source = n.id
WHERE l.dest = '"{}"'
"#,
            &id
        );
        conn.prepare(&q)?.query_as([]).map_err(Error::DBError)
    }
}

#[cfg(test)]
mod tests {
    use crate::connection::default_db_connection;
    use crate::node::Node;

    #[test]
    fn test_node_title() {
        let mut conn = default_db_connection().expect("I can't open a connection");
        let node =
            Node::by_id("1".to_string(), &mut conn).expect("Node with available id not found");
        assert_eq!(node.title().unwrap(), "momentum");
    }

    #[test]
    fn test_node_filename() {
        let mut conn = default_db_connection().expect("I can't open the connection");
        let node = Node::by_id("1".into(), &mut conn).expect("Node with available id not found");
        assert_eq!(node.filename().unwrap(), "org-roam/momentum.org");
    }

    #[test]
    fn test_node_tags() {
        use crate::tag::Tag;
        let mut conn = default_db_connection().expect("I can't open the conn");
        let node = Node::by_id("1".into(), &mut conn).expect("Error when fetch a node");
        assert_eq!(node.tags(&mut conn).unwrap(), vec![Tag::new("physics")]);
    }

    #[test]
    fn test_node_not_found() {
        let mut conn = default_db_connection().expect("I can't open the conn");
        let err = Node::by_id("undefined id".into(), &mut conn);
        assert!(matches!(err, Err(crate::result::Error::NodeNotFound)));
    }

    #[test]
    fn test_all_nodes() {
        use crate::tag::Tag;

        let mut conn = default_db_connection().expect("I can't open the conn");
        let nodes = Node::all_nodes(128, 0, &mut conn).expect("Error when fetch all nodes");
        assert_eq!(nodes.len(), 5);
        let titles: Vec<String> = nodes.iter().map(Node::title).map(Result::unwrap).collect();
        assert_eq!(
            titles,
            vec!["momentum", "mass", "si", "Second Law of Newton", "newton"]
        );
        let momentum = nodes.into_iter().nth(0).unwrap();

        assert_eq!(
            momentum
                .tags(&mut conn)
                .expect("error when fetch node tags"),
            vec![Tag::new("physics")]
        );
    }

    #[test]
    fn test_all_nodes_with_offset_and_limit() {
        let mut conn = default_db_connection().expect("I can't open the connection");
        let second_nodes =
            Node::all_nodes(1, 1, &mut conn).expect("Error when fetch 1 node after first");
        assert_eq!(second_nodes.len(), 1);
        let node = second_nodes
            .iter()
            .nth(0)
            .expect("Fetched 0 nodes, instead of 1");
        assert_eq!(node.title().unwrap(), "mass");
    }

    #[test]
    fn test_nodes_of_tag() {
        use crate::tag::Tag;
        let mut conn = default_db_connection().expect("I can't open the connection");
        let tag = Tag::by_name("person", &mut conn).expect("Error when fetch a tag");
        let nodes = Node::nodes_of_tag(tag, &mut conn).expect("Error when fetch nodes of a tag");
        assert_eq!(nodes.len(), 1);
        let nodes_titles: Vec<String> = nodes.iter().map(Node::title).map(Result::unwrap).collect();
        assert_eq!(nodes_titles, vec!["newton"]);
    }

    #[test]
    fn test_node_refers_to() {
        let mut conn = default_db_connection().expect("I can't open the conn");
        let newton = Node::by_id("5".to_string(), &mut conn).unwrap();
        let childs = newton.refers_to(&mut conn).unwrap();
        let childs_names: Vec<String> =
            childs.iter().map(Node::title).map(Result::unwrap).collect();
        assert_eq!(childs_names, ["Second Law of Newton"]);
    }

    #[test]
    fn test_node_backlinks() {
        let mut conn = default_db_connection().expect("I can't open the conn");
        let newton = Node::by_id("5".to_string(), &mut conn).unwrap();
        let parents = newton.backlinks(&mut conn).unwrap();
        let parents_names: Vec<String> = parents
            .iter()
            .map(Node::title)
            .map(Result::unwrap)
            .collect();
        assert_eq!(parents_names, ["Second Law of Newton"]);
    }

    #[test]
    fn test_node_by_title() {
        let mut conn = default_db_connection().expect("I can't open the conn");
        let si = Node::by_title("si", &mut conn).expect("I don't see SI");
        assert_eq!(si.id().unwrap(), "3");
    }
}
