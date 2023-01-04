use org_roam_fetch::tag::Tag;

extern crate org_roam_fetch;

#[tokio::main]
async fn main() {
    let tag_name = "sql";
    println!("{:?}", Tag::by_name(tag_name).await.expect("Not found"));
}
