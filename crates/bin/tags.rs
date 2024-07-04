use inquire::Select;
use org_roam_fetch::{connection::default_db_connection, node::Node, result::Error, tag::Tag};

extern crate org_roam_fetch;

fn main() {
    let mut conn = default_db_connection().expect("Sorry.  can't open the DataBase pool");

    let tags: Vec<String> = Tag::all_tags(&mut conn)
        .expect("Can't explore all tags to do hint")
        .iter()
        .map(Tag::name)
        .collect();
    let tag_name = Select::new("Choose a tag, pls: ", tags.clone())
        .prompt()
        .expect("You didn't choose a tag?");

    let mut tag = Tag::by_name(tag_name.trim(), &mut conn);

    while let Err(Error::TagNotFound) = tag {
        let tag_name = Select::new("> Choose a tag, pls", tags.clone())
            .prompt()
            .expect("You didn't choose a tag?");
        tag = Tag::by_name(tag_name.trim(), &mut conn);
    }

    let tag = tag.expect("Can't found your tag, internal error");

    println!("> Nodes with the tag \"{}\":", &tag.name());

    let nodes = Node::nodes_of_tag(tag, &mut conn).expect("I didn't find nodes of your tag?");

    for (i, node) in nodes.iter().enumerate() {
        println!("{i}. {title}", i = i + 1, title = node.title().unwrap());
    }
}
