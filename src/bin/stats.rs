extern crate malh;

use malh::relation::Relation;
use malh::relation::OpenMode::Reading;
use malh::page::Page;

fn main() {
    let r = Relation::open("test", Reading).unwrap();
    println!("== Information for relation '{}' ==", "test");
    println!("# of attributes: {}", r.num_attrs);
    println!("# of pages: {}", r.num_pages);
    println!("# of tuples: {}", r.num_tuples);
    println!("linear hashing params: d = {}, sp = {}", r.depth, r.split_pointer);

    /*
    println!("page 0 contents");
    let page = Page::read(&r.data_file, 0).unwrap();
    for tuple in page.get_tuples() {
        println!("{:?}", tuple.values);
    }
    */
}
