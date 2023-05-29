use emacsql::prelude::*;
use emacsql::query::QueryAs;
use emacsql::row::{FromRow, Row};

use rusqlite::Connection;

use std::env;

struct Node {
    id: String,
    title: String,
}

impl FromRow for Node {
    fn try_from_row(row: &Row) -> Result<Self> {
        let row = Self {
            id: row.get("id")?,
            title: row.get("title")?,
        };
        Ok(row)
    }
}

fn main() {
    let filename = env::var("ORG_ROAM_DB_FILE").unwrap();
    let conn = Connection::open(filename).unwrap();
    let mut nodes_q = conn.prepare("SELECT * from NODES").unwrap();
    let nodes: Vec<Node> = nodes_q.query_as([]).unwrap();
    for n in nodes {
        println!("- {}", n.title);
    }
}
