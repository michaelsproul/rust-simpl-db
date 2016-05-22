use std::mem;

use util::{hash as util_hash, bit as ith_bit, lower_bits};
use query::Query;
use choice_vec::ChoiceVec;

use self::IterStage::*;

pub const FULL_MASK: u32 = 0b11111111_11111111_11111111_11111111;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub struct PartialHash {
    /// The multiattribute hash value, if a bit
    /// is unknown it'll be zero, but should be
    /// ignored either way.
    pub hash: u32,
    /// Used to track which bits are being used
    pub mask: u32,
}

#[derive(Debug)]
pub struct PageIdIter {
    inner_iter: BinaryIterator,
    stage: IterStage,
    depth: u8,
    sp: u32,
    num_pages: u64,
    cached: Option<u32>,
    hash: PartialHash
}

#[derive(Debug, PartialEq, Eq)]
enum IterStage {
    Stage1,
    Stage2,
    Stage3,
    Stage4,
    Finished
}

impl PartialHash {
    pub fn match_hash(&self, other_hash: u32) -> bool {
        return (other_hash & self.mask) == self.hash;
    }

    pub fn is_complete(&self) -> bool {
        return self.mask == FULL_MASK;
    }

    pub fn matching_page_ids(&self, depth: u8, split_pointer: u32, num_pages: u64) -> PageIdIter {
        PageIdIter::new(self, depth, split_pointer, num_pages)
    }

    pub fn from_query(query: &Query, choice: &ChoiceVec) -> PartialHash {
        let mut query_hash: u32 = 0;
        let mut query_mask: u32 = FULL_MASK;

        // hash the queries known attributes
        let attr_hashs: Vec<Option<u32>> = {
            let func = |&attr_match: &Option<&str>| attr_match.map(util_hash);
            query.matches.iter().map(func).collect()
        };

        // generate multiattribute hash, from attr_hashs & choice vector,
        // along with the mask signifying which bits are known & unknown
        for (q_bit, &(a_index, a_bit)) in choice.data.iter().enumerate() {
            if let Some(a_hash) = attr_hashs[a_index as usize] {
                // added a'th bit of attribute to hash
                query_hash |= ith_bit(a_bit, a_hash) << q_bit;
            } else {
                // remove the q'th bit from the query mask
                query_mask &= !(1 << q_bit);
            }
        }

        PartialHash { hash: query_hash, mask: query_mask }
    }
}

impl PageIdIter {
    fn new(ma_hash: &PartialHash, depth: u8, sp: u32, num_pages: u64) -> Self {
        let stage1_iter = BinaryIterator {
            value: lower_bits(depth, ma_hash.hash),
            mask: lower_bits(depth, ma_hash.mask),
            num_bits: depth,
            assignment: 0,
            finished: false
        };
        PageIdIter {
            inner_iter: stage1_iter,
            stage: Stage1,
            depth: depth,
            sp: sp,
            num_pages: num_pages,
            hash: ma_hash.clone(),
            cached: None
        }
    }
}

fn assign_unknown_bits(value: u32, mask: u32, num_bits: u8, assignment: u32) -> u32 {
    let mut result = value;
    let mut j = 0;
    for i in 0..num_bits {
        // If the ith bit is in need of assignment, set it to the jth bit of the assignment.
        if ith_bit(i, mask) == 0 {
            result |= ith_bit(j, assignment) << i;
            j += 1;
        }
    }
    result
}

fn num_unknown_bits(num_bits: u8, mask: u32) -> u8 {
    num_bits - mask.count_ones() as u8
}

#[derive(Debug)]
struct BinaryIterator {
    value: u32,
    mask: u32,
    num_bits: u8,
    assignment: u32,
    finished: bool,
}

impl Iterator for BinaryIterator {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.finished {
            return None;
        }
        let res = assign_unknown_bits(self.value, self.mask, self.num_bits, self.assignment);
        // If all bit assignments have been exhausted, stop.
        // TODO: Fix overflow here.
        let nub = num_unknown_bits(self.num_bits, self.mask);
        println!("nub: {}", nub);
        let max_assignment = ((1 << nub as u64) - 1) as u32;
        println!("{}", max_assignment);
        if self.assignment == max_assignment {
            self.finished = true;
        } else {
            self.assignment += 1;
        }
        Some(res)
    }
}

impl Iterator for PageIdIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        println!("{:?}", self);
        if self.num_pages == 1 && self.stage != Finished {
            self.stage = Finished;
            return Some(0);
        }

        if let Some(cached) = mem::replace(&mut self.cached, None) {
            return Some(cached);
        }

        match self.stage {
            Stage1 => {
                // Ignore the first section if the (d + 1)th bit of the hash is known to be 1.
                if ith_bit(self.depth, self.hash.hash & self.hash.mask) == 1 {
                    while let Some(v) = self.inner_iter.next() {
                        if v >= self.sp {
                            trace!("caching: {}", v);
                            self.cached = Some(v);
                            break;
                        }
                        trace!("ignored: {}", v);
                    }
                }
                self.stage = Stage2;
                self.next()
            }
            Stage2 => {
                match self.inner_iter.next() {
                    Some(v) => {
                        Some(v)
                    }
                    None => {
                        self.stage = Stage3;
                        self.next()
                    }
                }
            }
            Stage3 => {
                if  ith_bit(self.depth, self.hash.hash & self.hash.mask) == 1 ||
                    ith_bit(self.depth, self.hash.mask) == 0
                {
                    let mut hash_with_1 = lower_bits(self.depth + 1, self.hash.hash & self.hash.mask);
                    hash_with_1 |= 1 << self.depth;
                    let mut mask_with_1 = lower_bits(self.depth + 1, self.hash.mask);
                    mask_with_1 |= 1 << self.depth;
                    self.inner_iter = BinaryIterator {
                        value: hash_with_1,
                        mask: mask_with_1,
                        num_bits: self.depth + 1,
                        assignment: 0,
                        finished: false,
                    };
                    self.stage = Stage4;
                    self.next()
                } else {
                    self.stage = Finished;
                    None
                }
            }
            Stage4 => {
                match self.inner_iter.next() {
                    Some(v) if (v as u64) < self.num_pages => {
                        Some(v)
                    }
                    _ => {
                        self.stage = Finished;
                        None
                    }
                }
            }
            Finished => {
                None
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::{ PartialHash, FULL_MASK, PageIdIter };
    use query::{ Query };
    use tuple::Tuple;
    use choice_vec::ChoiceVec;
    use rand::random;

    // hashing

    #[test]
    fn hash_mask_correctness() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let PartialHash { hash, mask } = PartialHash::from_query(&query, &c_vec);
        assert_eq!(hash & mask, hash);
        assert_eq!(hash | mask, mask);
    }

    #[test]
    fn hash_when_known_isnt_0() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let ma_hash = PartialHash::from_query(&query, &c_vec);
        assert!(ma_hash.mask != 0);
    }

    #[test]
    fn hash_with_unknown() {
        let query = Query::parse("?,?,?", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let PartialHash { hash, mask } = PartialHash::from_query(&query, &c_vec);
        assert_eq!(hash, 0);
        assert_eq!(mask, 0);
    }

    #[test]
    fn query_hash_same_as_tuple_hash() {
        let c_vec = ChoiceVec::parse("0,0:0,1:1,0:1,1:2,0:2,1", 3).unwrap();
        let tuple = Tuple::parse("a,b,c", 3).unwrap();
        let query = Query::parse("a,b,c", 3).unwrap();
        let ma_hash = PartialHash::from_query(&query, &c_vec);
        assert_eq!(tuple.hash(&c_vec), ma_hash.hash);
    }

    // matching hashed query

    #[test]
    fn hash_matching_zero() {
        let num_hash: u32 = 0;
        let query_hash = PartialHash { hash: num_hash, mask: FULL_MASK };
        assert!(query_hash.match_hash(num_hash));
    }

    #[test]
    fn hash_matching_non_zero() {
        let num_hash = random::<u32>();
        let query_hash = PartialHash { hash: num_hash, mask: FULL_MASK };
        assert!(query_hash.match_hash(num_hash));
    }

    // hash iter

    /*
    #[test]
    fn iter_with_no_ambiguities() {
        let mut iter = PageIdIter::new(&PartialHash { hash: 0, mask: FULL_MASK }, 1, 1, 3);
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }
    */

    #[test]
    fn iter_yields_correct_values_for_n8() {
        let mut iter = PageIdIter::new(&PartialHash { hash: 0b100, mask: 0b100 }, 3, 0, 8);
        assert_eq!(iter.next(), Some(0b100));
        assert_eq!(iter.next(), Some(0b101));
        assert_eq!(iter.next(), Some(0b110));
        assert_eq!(iter.next(), Some(0b111));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_yields_correct_values_for_n6() {
        let mut iter = PageIdIter::new(&PartialHash { hash: FULL_MASK, mask: 0b100 }, 2, 4, 7);
        assert_eq!(iter.next(), Some(0b100));
        assert_eq!(iter.next(), Some(0b101));
        assert_eq!(iter.next(), Some(0b110));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_single_page() {
        let mut iter = PageIdIter::new(&PartialHash { hash: 1, mask: 1}, 1, 0, 1);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_sp_zero() {
        let mut iter = PageIdIter::new(&PartialHash { hash : 0b10, mask: 0b10 }, 1, 0, 2);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_lel() {
        let mut iter = PageIdIter::new(&PartialHash { hash : 0b10, mask: 0b10 }, 1, 1, 3);
        assert_eq!(iter.next(), Some(0b01));
        assert_eq!(iter.next(), Some(0b10));
        assert_eq!(iter.next(), None);
    }
}
