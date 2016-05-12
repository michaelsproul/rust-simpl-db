extern crate malh;

use malh::tuple::Tuple;
use malh::relation::{Relation, Writing};
use malh::util::error;

use std::io::{self, BufRead};
use std::env;

fn main() {
    if env::args().len() != 2 {
        error("Usage: insert <relation>");
    }

    let relation_name = env::args().nth(1).unwrap();
    let mut relation = Relation::open(&relation_name, Writing)
        .unwrap_or_else(|_| error(format!("Error: unable to open relation: {}", relation_name)));

    let stdin = io::stdin();
    for raw_line in stdin.lock().lines() {
        let line = raw_line.unwrap();
        let tuple = Tuple::parse(&line, relation.num_attrs)
            .unwrap_or_else(|| error(format!("Error: invalid tuple: {}", line)));

        relation.insert(tuple).unwrap_or_else(|e| {
            error(format!("Error: unable to insert tuple.\nReason: {}\nTuple: {}", e, line))
        });
    }

    relation.close().unwrap();
}
