extern crate malh;
extern crate uuid;

use std::iter::repeat;
use std::io;
use uuid::Uuid;

use malh::relation::*;
use malh::choice_vec::*;
use malh::tuple::*;
use malh::page::*;
use malh::query::Query;

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

#[test]
fn select_empty() {
    let r = TestRelation::new(10);
    assert_eq!(r.0.select(&Query::wildcard(r.0.num_attrs)).count(), 0);
    r.close();
}

#[test]
fn select_single() {
    let num_attrs = 5;
    let mut r = TestRelation::new(num_attrs);
    let tuple = Tuple::parse("hello,world,this,is,me", num_attrs).unwrap();
    r.0.insert(tuple.clone()).unwrap();
    let matching_queries = [
        "hello,?,?,?,?",
        "?,world,?,?,?",
        "?,?,this,?,?",
        "?,?,?,is,?",
        "?,?,?,?,me",
        "hello,?,?,is,?"
    ];
    for q in matching_queries.iter() {
        let query = Query::parse(q, num_attrs).unwrap();
        let results: Vec<io::Result<Tuple>> = r.0.select(&query).collect();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].as_ref().unwrap(), &tuple);
    }
    let non_matching_queries = [
        "hello,warld,?,?,?",
        "world,hello,this,is,me"
    ];
    for q in non_matching_queries.iter() {
        let query = Query::parse(q, num_attrs).unwrap();
        assert_eq!(r.0.select(&query).count(), 0);
    }
    r.close();
}
