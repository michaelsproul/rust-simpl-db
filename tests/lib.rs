extern crate malh;
extern crate uuid;

use std::iter::repeat;
use uuid::Uuid;

use malh::relation::*;
use malh::choice_vec::*;
use malh::tuple::*;
use malh::page::*;

/// A test relation with a random name.
struct TestRelation(pub Relation);

impl TestRelation {
    fn new(num_attrs: u32) -> TestRelation {
        let name = format!("{}", Uuid::new_v4().simple());
        Relation::new(&name, num_attrs, 1, ChoiceVec::new(vec![], 1)).unwrap();
        TestRelation(Relation::open(&name, Writing).unwrap())
    }

    fn close(mut self) {
        self.0.is_sane();
        self.0.delete().unwrap();
    }
}

#[test]
fn insert_oversize_tuple() {
    let mut r = TestRelation::new(1);
    let oversized = Tuple { values: vec![repeat('A').take(PAGE_SIZE as usize).collect()] };
    r.0.insert(oversized).unwrap_err();
    r.close();
}

#[test]
fn insert_largest_tuple() {
    let mut r = TestRelation::new(1);
    let large = Tuple { values: vec![repeat('A').take(PAGE_DATA_SIZE - 1).collect()] };
    r.0.insert(large).unwrap();
    r.close();
}
