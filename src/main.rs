extern crate org_roam_fetch;

#[tokio::main]
async fn main() {
    let tgs = org_roam_fetch::show_tags::nodes_and_tags().await;
    match tgs {
        Ok(tags) => {
            for (title, tag) in tags {
                println!("{} [{}]", title, tag);
            }
        },
        Err(err) => eprintln!("{:?}", err)
    }
}
