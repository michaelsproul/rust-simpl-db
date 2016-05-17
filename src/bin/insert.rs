extern crate malh;

use malh::tuple::Tuple;
use malh::relation::{Relation, Writing};
use malh::util::*;

use std::io::{self, BufRead, BufReader};
use std::env;
use std::fs::File;

fn main() {
    run_main(insert);
}

fn insert() -> Result<(), BoxError> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 && args.len() != 3 {
        try!(Err("Usage: insert <relation> [data file]"));
    }

    let relation_name = &args[1];
    let mut relation = try!(Relation::open(&relation_name, Writing)
        .map_err(|_| format!("Error: unable to open relation: {}", relation_name)));

    // OS X's GUI profiler thinks it's too good for IO redirection, so we allow a filename
    // to be specified as an optional second argument.
    let stdin = io::stdin();
    let input: Box<BufRead> = if args.len() == 3 {
        let f = try!(File::open(&args[2]));
        Box::new(BufReader::new(f))
    } else {
        println!("Reading from standard input.");
        Box::new(stdin.lock())
    };

    for raw_line in input.lines() {
        let line = raw_line.unwrap();
        let tuple = try!(Tuple::parse(&line, relation.num_attrs)
            .ok_or_else(|| format!("Error: invalid tuple: {}", line)));

        try!(relation.insert(tuple).map_err(|e| {
            format!("Error: unable to insert tuple\nReason: {}\nTuple: {}", e, line)
        }));
    }

    println!("All insertions successful.");
    Ok(())
}
