use std::env;
use std::path::Path;

use dotenvy::dotenv;
use rusqlite::Connection;

use crate::result::{Error, Result};

pub const DEFAULT_ORG_ROAM_DB_FILE: &str = "~/.emacs.d/org-roam.db";
#[cfg(test)]
pub const ORG_ROAM_DB_FILE_ENV_VAR: &str = "TEST_ORG_ROAM_DB_FILE";
#[cfg(not(test))]
pub const ORG_ROAM_DB_FILE_ENV_VAR: &str = "ORG_ROAM_DB_FILE";

pub fn db_connection<T: AsRef<Path>>(filename: T) -> Result<Connection> {
    Connection::open(filename).map_err(Error::DBError)
}

pub fn default_db_connection() -> Result<Connection> {
    dotenv().ok();
    db_connection(
        env::var(ORG_ROAM_DB_FILE_ENV_VAR).unwrap_or(DEFAULT_ORG_ROAM_DB_FILE.to_string()),
    )
}
