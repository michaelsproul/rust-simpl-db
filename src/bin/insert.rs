extern crate malh;

use malh::tuple::Tuple;
use malh::relation::{Relation, Writing};
use malh::util::*;

use std::io::{self, BufRead};
use std::env;

fn main() {
    run_main(insert);
}

fn insert() -> Result<(), BoxError> {
    if env::args().len() != 2 {
        try!(Err("Usage: insert <relation>"));
    }

    let relation_name = env::args().nth(1).unwrap();
    let mut relation = try!(Relation::open(&relation_name, Writing)
        .map_err(|_| format!("Error: unable to open relation: {}", relation_name)));

    let stdin = io::stdin();
    for raw_line in stdin.lock().lines() {
        let line = raw_line.unwrap();
        let tuple = try!(Tuple::parse(&line, relation.num_attrs)
            .ok_or_else(|| format!("Error: invalid tuple: {}", line)));

        try!(relation.insert(tuple).map_err(|e| {
            format!("Error: unable to insert tuple.\nReason: {}\nTuple: {}", e, line)
        }));
    }

    println!("All insertions successful.");
    Ok(())
}
