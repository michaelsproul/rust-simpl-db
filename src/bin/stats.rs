extern crate malh;

use std::env;

use malh::relation::Relation;
use malh::relation::OpenMode::Reading;
use malh::util::*;

fn main() {
    enable_logging();
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error("Usage: stats <relation name>");
    }
    let relation_name = &args[1];
    let r = Relation::open(relation_name, Reading)
        .unwrap_or_else(|e| error(format!("Error: {}", e)));
    println!("== Information for relation '{}' ==", relation_name);
    println!("# of attributes: {}", r.num_attrs);
    println!("# of pages: {}", r.num_pages);
    println!("# of tuples: {}", r.num_tuples);
    println!("linear hashing params: d = {}, sp = {}", r.depth, r.split_pointer);
    println!("choice vector: {:?}", r.choice_vec.data);
}
