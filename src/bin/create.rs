extern crate malh;

use std::env;

use malh::relation::Relation;
use malh::choice_vec::ChoiceVec;
use malh::util::*;

fn main() {
    run_main(create);
}

fn create() -> Result<(), BoxError> {
    enable_logging();

    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        try!(Err("Usage: create <name> <num attrs> <num pages> <choice vec>"));
    }

    let relation_name = &args[1];
    let num_attrs = try!(args[2].parse()
        .map_err(|_| "Error: non-integer number of attributes"));
    let est_num_pages = try!(args[3].parse()
        .map_err(|_| "Error: non-integer number of pages"));
    let choice_vec = try!(ChoiceVec::parse(&args[4], num_attrs)
        .map_err(|e| format!("Error: invalid choice vector, reason: {:?}", e)));

    try!(
        Relation::new(relation_name, num_attrs, est_num_pages, choice_vec)
        .map_err(|e| format!("Error: {}", e))
    );

    println!("Success!");
    Ok(())
}
