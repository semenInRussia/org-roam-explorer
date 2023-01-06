use dotenvy::dotenv;
use std::env;

use crate::result::Result;
use quaint::pooled::{PooledConnection, Quaint};

const DEFAULT_ORG_ROAM_DB_FILE: &str = "~/.emacs.d/org-roam.db";
#[cfg(test)]
const ORG_ROAM_DB_FILE_ENV_VAR: &str = "TEST_ORG_ROAM_DB_FILE";
#[cfg(not(test))]
const ORG_ROAM_DB_FILE_ENV_VAR: &str = "ORG_ROAM_DB_FILE";

pub async fn db_connection() -> Result<PooledConnection> {
    dotenv().ok();
    let mut url =
        env::var(ORG_ROAM_DB_FILE_ENV_VAR).unwrap_or(DEFAULT_ORG_ROAM_DB_FILE.to_string());
    url.insert_str(0, "file:");
    let conn = Quaint::new(url.as_str()).await?.check_out().await?;
    Ok(conn)
}

macro_rules! select_first_in_db {
    ($query:expr) => {
        db_connection()
            .await?
            .select($query)
            .await?
            .into_iter()
            .nth(0)
            .as_ref()
            .map(Into::into)
    };
}

macro_rules! select_in_db {
    ($query:expr) => {
        db_connection()
            .await?
            .select($query)
            .await?
            .into_iter()
            .map(|row| (&row).into())
            .collect()
    };
}
