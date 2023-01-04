use std::env;
use dotenvy::dotenv;

use crate::result::Result;
use quaint::pooled::{Quaint, PooledConnection};

const DEFAULT_ORG_ROAM_DB_FILE: &str = "~/.emacs.d/org-roam.db";

pub async fn db_connection () -> Result<PooledConnection> {
    dotenv().ok();
    let mut url = env::var("ORG_ROAM_DB_FILE")
        .unwrap_or(DEFAULT_ORG_ROAM_DB_FILE.to_string());
    url.insert_str(0, "file:");
    let conn = Quaint::new(url.as_str()).await?.check_out().await?;
    Ok(conn)
}
