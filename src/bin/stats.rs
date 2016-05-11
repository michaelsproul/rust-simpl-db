extern crate malh;

use malh::relation::Relation;
use malh::relation::OpenMode::Reading;

fn main() {
    let r = Relation::open("test", Reading).unwrap();
    println!("== Information for relation '{}' ==", "test");
    println!("# of attributes: {}", r.num_attrs);
    println!("# of pages: {}", r.num_pages);
    println!("# of tuples: {}", r.num_tuples);
    println!("linear hashing params: d = {}, sp = {}", r.depth, r.split_pointer);
    println!("choice vector: {:?}", r.choice_vec.data);
}
