extern crate malh;

use std::env;

use malh::util::*;
use malh::relation::*;
use malh::query::*;

fn main() {
    run_main(select);
}

fn select() -> Result<(), BoxError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        try!(Err("Usage: select <relation name> <query>"));
    }

    let relation_name = &args[1];
    let query_string = &args[2];

    let relation = try!(Relation::open(relation_name, Reading));
    let query = try!(Query::parse(query_string, relation.num_attrs)
        .map_err(|e| format!("Error: unable to parse query, reason: {:?}", e)));

    for item in relation.select(&query) {
        let tuple = try!(item);
        println!("{}", tuple.to_string());
    }

    Ok(())
}
