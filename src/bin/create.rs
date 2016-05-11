extern crate malh;

use malh::relation::{Relation, Writing};
use malh::choice_vec::ChoiceVec;
use malh::tuple::Tuple;

fn main() {
    let num_attrs = 2;
    let depth = 0;
    let num_pages = 1;
    let cv = ChoiceVec::new(vec![(0, 0), (0, 1), (1, 0), (1, 1)], num_attrs);
    Relation::new("test", num_attrs, num_pages, depth, cv.clone()).unwrap();

    let t1 = Tuple {
        values: vec!["hello".to_string(), "world".to_string()]
    };
    let t2 = Tuple {
        values: vec!["rust".to_string(), "postgres".to_string()]
    };
    println!("hash #1 = {:b}, hash #2 = {:b}", t1.hash(&cv), t2.hash(&cv));

    let mut r = Relation::open("test", Writing).unwrap();

    for _ in 0..1000 {
        r.insert(t1.clone()).unwrap();
        r.insert(t2.clone()).unwrap();
    }
    r.close().unwrap();

    println!("Success!");
}
