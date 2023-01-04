use crate::connection::db_connection;
use quaint::{prelude::*, Result};

pub async fn nodes_and_tags () -> Result<Vec<(String, String)>> {
    let conn = db_connection().await?;
    let join = "nodes"
        .alias("n")
        .on(("n","id").equals(Column::from(("tags", "node_id"))));
    let query = Select::from_table("tags")
        .columns(["title", "tag"])
        .group_by("tag")
        .inner_join(join);
    let res = conn
        .select(query)
        .await?
        .into_iter()
        .map(|row| ((&row)[0].clone(), row[1].clone()))
        .map(|(title, tag)| {
            (title
             .into_string()
             .expect("I can't transform title from DB result to string"),
             tag
             .into_string()
             .expect("I can't transform tag from DB result to string"))
        })
        .collect::<Vec<_>>();
    Ok(res)
}
