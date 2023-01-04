use org_roam_fetch::{tag::Tag, node::all_nodes};

extern crate org_roam_fetch;

#[tokio::main]
async fn main() {
    let tag_name = "sql";
    let tag = Tag::by_name(tag_name).await.expect("Not found");

    println!("{:?}", tag.name());

    let title = all_nodes()
        .await
        .expect("Nodes not fetched")
        .first()
        .expect("Fetched zero nodes")
        .title();

    println!("{}", title);
}
