extern crate malh;

use malh::relation::Relation;
use malh::choice_vec::ChoiceVec;

fn main() {
    Relation::new("test", 10, 0, 8, ChoiceVec {}).unwrap();
}
