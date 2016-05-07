extern crate malh;

use malh::relation::{Relation, Writing};
use malh::choice_vec::ChoiceVec;
use malh::tuple::Tuple;

fn main() {
    let num_attrs = 10;
    let depth = 4;
    let num_pages = 16;
    let cv = ChoiceVec::new(vec![(1, 0), (1, 1), (2, 0), (2, 1)], num_attrs);
    Relation::new("test", num_attrs, num_pages, depth, cv).unwrap();

    let t = Tuple {
        values: vec!["hello".to_string(), "world".to_string()]
    };

    let mut r = Relation::open("test", Writing).unwrap();
    r.insert(t).unwrap();

    println!("Success!");
}
