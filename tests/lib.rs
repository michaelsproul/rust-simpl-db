extern crate malh;
extern crate uuid;
extern crate quickcheck;
extern crate rand;

use std::iter::repeat;
use std::io;
use uuid::Uuid;
use quickcheck::{Arbitrary, StdGen, Gen};
use rand::thread_rng;

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

fn select_single_() {
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

#[test]
fn select_single() {
    // The test depends on the randomly generated choice vec, so run it a few times.
    for _ in 0..500 {
        select_single_();
    }
}


// NOTE: Truly random UTF8 characters were too brutal and difficult to debug, so we just
// generate random strings of A's.
fn random_tuple<G: Gen>(num_attrs: u32, gen: &mut G) -> Tuple {
    use std::iter::repeat;
    loop {
        let tuple_values: Vec<String> = (0..num_attrs).map(|_| {
            let tuple_len = usize::arbitrary(gen) % (PAGE_DATA_SIZE / num_attrs as usize);
            repeat('A').take(tuple_len).collect()
        }).collect();
        let tuple = Tuple { values: tuple_values };
        if tuple.serialise().len() <= PAGE_DATA_SIZE {
            return tuple;
        }
    }
}

fn append_to_query<'a>(elem: Option<&'a str>, q: Query<'a>) -> Query<'a> {
    let mut matches = q.matches;
    matches.push(elem);
    Query {
        matches: matches
    }
}

fn queries_for_tuple<'a>(tuple: &'a [String]) -> Vec<Query<'a>> {
    match tuple.split_last() {
        None => vec![Query::wildcard(0)],
        Some((last, init)) => {
            let init_queries = queries_for_tuple(init);
            let mut none_queries: Vec<Query<'a>> =
                init_queries.iter().map(|q| append_to_query(None, q.clone())).collect();
            let some_queries: Vec<Query<'a>> =
                init_queries.iter().map(|q| append_to_query(Some(&last), q.clone())).collect();
            none_queries.extend(some_queries);
            none_queries
        }
    }
}

/// Compute the 2^n queries that should match a given tuple.
fn all_queries_for_tuple<'a>(tuple: &'a Tuple) -> Vec<Query<'a>> {
    queries_for_tuple(&tuple.values[..])
}

#[test]
fn all_queries_generation() {
    let tuple = Tuple::parse("hello,world,wow", 3).unwrap();
    let queries = all_queries_for_tuple(&tuple);
    assert_eq!(8, queries.len());
}

fn insert_select(num_attrs: u32, num_tuples: u32) {
    if num_attrs == 0 || num_tuples == 0 {
        return;
    }
    let mut gen = StdGen::new(thread_rng(), PAGE_DATA_SIZE / num_attrs as usize);
    let mut r = TestRelation::new(num_attrs);

    let tuples: Vec<Tuple> = (0..num_tuples).map(|_| random_tuple(num_attrs, &mut gen)).collect();
    println!("BEGIN TUPLE INSERTIONS");

    for t in &tuples {
        println!("{}", t.to_string());
        r.0.insert(t.clone()).expect(&format!("inserting tuple failed, tuple: {:?}", t));
    }
    println!("END TUPLE INSERTIONS");

    r.0.is_sane();

    for t in &tuples {
        for q in all_queries_for_tuple(t) {
            let results: Vec<Tuple> = r.0.select(&q).map(|r| r.expect("IO error")).collect();
            assert!(results.contains(t));
        }
    }

    r.close();
}

#[test]
fn random_insert_select() {
    insert_select(2, 5);
    insert_select(3, 5);
    insert_select(8, 5);
    insert_select(3, 300);
}
