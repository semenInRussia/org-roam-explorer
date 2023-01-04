use quaint::prelude::*;
use crate::result::Result;
use crate::connection::db_connection;

// NOTE: I am not use columns from the table Node which for me useless
pub struct Node {
    id: String,
    title: String,
    tags: String,
    filename: String,
    text: String,
}

impl Node {
    pub async fn by_id (id: &str) -> Result<Self> {
        let query = Select::from_table("nodes")
            .columns(["id", "title", "text", "file"])
            .and_where(Column::new("id").equals(id));
        let row = db_connection()
            .await?
            .select(query).await?
            .first()
            .ok_or();
        Ok(Node {})
    }
}
