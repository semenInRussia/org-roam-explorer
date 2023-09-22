use emacsql::{Error, FromRow, QueryAs, Result, Row};
use rusqlite::Connection;

#[derive(Debug)]
struct Repository {
    id: String,
    class: String,
    forge: Option<String>,
    forge_id: Option<String>,
    sparse_p: Option<bool>,
    stars: Option<usize>,
}

impl FromRow for Repository {
    fn try_from_row(row: &Row) -> Result<Self> {
        let rep = Self {
            id: row.get("id")?,
            class: row.get("class")?,
            forge: row.get("forge")?,
            forge_id: row.get("forge_id")?,
            sparse_p: row.get("sparse_p")?,
            stars: row.get("stars")?,
        };
        Ok(rep)
    }
}

fn main() {
    let filename = "c:/Users/hrams/AppData/Roaming/.emacs.d/forge-database.sqlite";
    let conn = Connection::open(filename).unwrap();
    let mut repos_q = conn.prepare("SELECT * FROM repository").unwrap();
    let repo: Result<Repository> = repos_q.query_as_one([]);

    match repo {
        Ok(repo) => {
            println!("found! repo is {repo:#?}");
        }
        Err(Error::QueryReturnedNoRows) => {
            println!("not found...:()");
        }
        Err(err) => {
            println!("{err:?}");
        }
    }
}
