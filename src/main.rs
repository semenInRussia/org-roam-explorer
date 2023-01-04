extern crate org_roam_fetch;

#[tokio::main]
async fn main() {
    let id = "29926ae9-16b8-4c44-b528-a3da4a203191";
    let node = org_roam_fetch::node::Node::by_id(id).await;
    match node {
        Ok(n) => {
            println!("{:?}", n);
        },
        Err(err) => eprintln!("{:?}", err)
    }
}
