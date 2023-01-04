extern crate org_roam_fetch;

#[tokio::main]
async fn main() {
    let id = "29926ae9-16b8-4c44-b528-a3da4a203191";
    let node = org_roam_fetch::node::Node::by_id(id).await;
    match node {
        Ok(n) => {
            println!("{:?}", n.tags().await.expect("Tags not found"));
        },
        Err(err) => eprintln!("{:?}", err)
    }
    let nodes = org_roam_fetch::node::all_nodes().await
        .expect("An error with found all nodes");
    let titles: Vec<String> = nodes
        .into_iter()
        .map(|n| n.title.expect("A node hasn't title"))
        .collect();
    println!("{:#?}", titles);
    println!("{}", titles.len());
}
