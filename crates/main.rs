use std::io::Write;

use org_roam_fetch::{
    node::Node,
    connection::default_db_pool
};

#[tokio::main]
async fn main() {
    let mut id = String::new();
    print!("Enter the ID of a node, pls -> ");
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut id)
        .expect("OS read from the stdin???");
    let id = id.trim().to_string();
    let pool = default_db_pool().await.expect("unable open connection to db");
    let node =
        Node::by_id(id, &pool).await.expect("node not found?");
    let childs = node.refers_to(&pool).await.expect("Can't find childs");

    for ch in childs {
        println!("- {}", ch.title().unwrap());
    }
}
