extern crate inquire;
use inquire::Select;
use org_roam_fetch::{connection::default_db_pool, node::Node, tag::Tag};

#[tokio::main]
async fn main() {
    let pool = default_db_pool()
        .await
        .expect("couldn't open a pool for db connection");
    let names: Vec<String> = Node::all_nodes(1024, 0, &pool)
        .await
        .expect("Couldn't fetch all nodes to do auto complete")
        .iter()
        .map(|n| n.title().expect("given a node without title"))
        .collect();

    let node_title = Select::new("Choose the name of a node -> ", names.clone())
        .prompt()
        .expect("You didn't choose a node?");
    let node = Node::by_title(node_title, &pool)
        .await
        .expect("internal error when search a node");

    let childs = node
        .refers_to(&pool)
        .await
        .expect("couldn't find ndoes which refers to a given node");

    println!("> Links on this node");

    if childs.is_empty() {
        println!("  these nodes didn't found");
    }

    for ch in childs {
        let tags: Vec<String> = ch
            .tags(&pool)
            .await
            .unwrap()
            .iter()
            .map(Tag::name)
            .collect();
        println!(
            "  - {title} [{tags}]",
            title = ch.title().unwrap(),
            tags = tags.join(", ")
        );
    }

    println!("> Backlinks");

    let backlinks = node
        .backlinks(&pool)
        .await
        .expect("Couldn't fetch backlinks");

    if backlinks.is_empty() {
        println!("  these nodes didn't found");
    }

    for b in backlinks {
        let tags: Vec<String> = b.tags(&pool).await.unwrap().iter().map(Tag::name).collect();
        println!(
            "  - {title} [{tags}]",
            title = b.title().unwrap(),
            tags = tags.join(", ")
        );
    }
}
