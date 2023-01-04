use std::env;
use dotenvy::dotenv;

use crate::result::Result;
use quaint::pooled::{Quaint, PooledConnection};

const DEFAULT_ORG_ROAM_DB_FILE: &str = "~/.emacs.d/org-roam.db";
#[cfg(test)]
const ORG_ROAM_DB_FILE_ENV_VAR: &str = "TEST_ORG_ROAM_DB_FILE";
#[cfg(not(test))]
const ORG_ROAM_DB_FILE_ENV_VAR: &str = "ORG_ROAM_DB_FILE";

pub async fn db_connection () -> Result<PooledConnection> {
    dotenv().ok();
    let mut url = env::var(ORG_ROAM_DB_FILE_ENV_VAR)
        .unwrap_or(DEFAULT_ORG_ROAM_DB_FILE.to_string());
    url.insert_str(0, "file:");
    dbg!(&url);
    let conn = Quaint::new(url.as_str()).await?.check_out().await?;
    Ok(conn)
}
