extern crate malh;

use std::env;

use malh::relation::Relation;
use malh::choice_vec::ChoiceVec;
use malh::util::*;

fn main() {
    enable_logging();

    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        error("Usage: create <name> <num attrs> <num pages> <choice vec>");
    }

    let relation_name = &args[1];
    let num_attrs = args[2].parse()
        .unwrap_or_else(|_| error("Error: non-integer number of attributes"));
    let est_num_pages = args[3].parse()
        .unwrap_or_else(|_| error("Error: non-integer number of pages"));
    let choice_vec = ChoiceVec::parse(&args[4], num_attrs)
        .unwrap_or_else(|e| error(format!("Error: invalid choice vector, reason: {:?}", e)));

    Relation::new(relation_name, num_attrs, est_num_pages, choice_vec)
        .unwrap_or_else(|e| error(format!("Error: {}", e)));

    println!("Success!");
}
