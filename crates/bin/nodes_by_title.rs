extern crate inquire;
use inquire::Select;
use org_roam_fetch::{connection::default_db_connection, node::Node, tag::Tag};

fn main() {
    let mut conn = default_db_connection().expect("couldn't open a pool for db connection");
    let names: Vec<String> = Node::all_nodes(1024, 0, &mut conn)
        .expect("Couldn't fetch all nodes to do auto complete")
        .iter()
        .filter_map(|n| n.title().ok())
        .collect();

    let node_title = Select::new("Choose the name of a node -> ", names.clone())
        .prompt()
        .expect("You didn't choose a node?");
    let node = Node::by_title(node_title, &mut conn).expect("internal error when search a node");

    let childs = node
        .refers_to(&mut conn)
        .expect("couldn't find ndoes which refers to a given node");

    println!("> Links inside this node");

    if childs.is_empty() {
        println!("  these nodes didn't found");
    }

    for ch in childs {
        let tags: Vec<String> = ch
            .tags(&mut conn)
            .unwrap_or(vec![])
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

    let backlinks = node.backlinks(&mut conn).expect("Couldn't fetch backlinks");

    if backlinks.is_empty() {
        println!("  these nodes didn't found");
    }

    for b in backlinks {
        let tags: Vec<String> = b
            .tags(&mut conn)
            .unwrap_or(vec![])
            .iter()
            .map(Tag::name)
            .collect();
        println!(
            "  - {title} [{tags}]",
            title = b.title().unwrap(),
            tags = tags.join(", ")
        );
    }
}
