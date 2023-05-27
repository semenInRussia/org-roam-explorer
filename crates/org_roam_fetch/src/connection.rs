use sqlx::SqlitePool;

use dotenvy::dotenv;
use std::env;

use crate::result::{Result, Error};

pub const DEFAULT_ORG_ROAM_DB_FILE: &str = "~/.emacs.d/org-roam.db";
#[cfg(test)]
pub const ORG_ROAM_DB_FILE_ENV_VAR: &str = "TEST_ORG_ROAM_DB_FILE";
#[cfg(not(test))]
pub const ORG_ROAM_DB_FILE_ENV_VAR: &str = "ORG_ROAM_DB_FILE";

pub async fn db_pool(filename: &str) -> Result<SqlitePool> {
    let url = format!("sqlite://{filename}");
    let pool = SqlitePool::connect(&url).await;
    pool.map_err(Error::DBError)
}

pub async fn default_db_pool() -> Result<SqlitePool> {
    dotenv().ok();
    let file = env::var(ORG_ROAM_DB_FILE_ENV_VAR)
        .unwrap_or(DEFAULT_ORG_ROAM_DB_FILE.to_string());
    db_pool(file.as_str()).await
}
