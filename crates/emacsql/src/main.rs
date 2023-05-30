use emacsql::{Error, FromRow, QueryAs, Result, Row, Value};
use rusqlite::Connection;

use std::env;

#[derive(Debug)]
struct Node {
    id: String,
    title: String,
    // pos: Option<u32>,
}

impl FromRow for Node {
    fn try_from_row(row: &Row) -> Result<Self> {
        let node = Self {
            id: row.get("id")?,
            title: row.get("title")?,
            // pos: row.get("pos")?,
        };
        println!("i try convert a {} to Node", &node.title);
        Ok(node)
    }
}

fn main() {
    let filename = env::var("ORG_ROAM_DB_FILE").unwrap();
    let conn = Connection::open(filename).unwrap();
    let mut nodes_q = conn
        .prepare("SELECT * FROM nodes WHERE title = ?1")
        .unwrap();
    let node: Result<Node> = nodes_q.query_as_one(["\"Smoodin\""]);

    match node {
        Ok(node) => {
            println!("found! ID is {id}", id = node.id);
        }
        // Err::
        Err(Error::QueryReturnedNoRows) => {
            println!("not found...:()");
        }
        Err(err) => {
            println!("{err:?}");
        }
    }

    let mut nodes = nodes_q.query(["\"Smoodin\""]);
    println!("{:?}", nodes.unwrap().next());
}
