use inquire::Select;
use org_roam_fetch::{connection::default_db_pool, node::Node, result::Error, tag::Tag};

extern crate org_roam_fetch;

#[tokio::main]
async fn main() {
    let pool = default_db_pool()
        .await
        .expect("Sory.  can't an open DataBase pool");
    let tags: Vec<String> = Tag::all_tags(&pool)
        .await
        .expect("Can't explore all tags to do hint")
        .iter()
        .map(Tag::name)
        .collect();
    let tag_name = Select::new("Choose a tag, pls: ", tags.clone())
        .prompt()
        .expect("You didn't choose a tag?");
    let mut tag = Tag::by_name(tag_name.trim(), &pool).await;

    while let Err(Error::TagNotFound) = tag {
        let tag_name = Select::new("> Choose a tag, pls", tags.clone())
            .prompt()
            .expect("You didn't choose a tag?");
        tag = Tag::by_name(tag_name.trim(), &pool).await;
    }

    let tag = tag.expect("Can't found your tag, internal error");

    println!("> Nodes of your tag with name \"{}\":", &tag.name());

    let nodes = Node::nodes_of_tag(tag, &pool)
        .await
        .expect("I didn't find nodes of your tag?");

    for (i, node) in nodes.iter().enumerate() {
        println!("{i}. {title}", i = i + 1, title = node.title().unwrap());
    }
}
