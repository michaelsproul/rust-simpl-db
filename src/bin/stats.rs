extern crate malh;

use std::env;

use malh::relation::Relation;
use malh::relation::OpenMode::Reading;
use malh::util::*;

fn main() {
    run_main(stats);
}

fn stats() -> Result<(), BoxError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        try!(Err("Usage: stats <relation name>"));
    }
    let relation_name = &args[1];
    let r = try!(Relation::open(relation_name, Reading)
        .map_err(|e| format!("Error: {}", e)));
    println!("== Information for relation '{}' ==", relation_name);
    println!("# of attributes: {}", r.num_attrs);
    println!("# of pages: {}", r.num_pages);
    println!("# of tuples: {}", r.num_tuples);
    println!("linear hashing params: d = {}, sp = {}", r.depth, r.split_pointer);
    println!("choice vector: {:?}", r.choice_vec.data);
    Ok(())
}
