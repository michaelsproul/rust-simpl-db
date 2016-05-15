use util::{hash as util_hash, bit as ith_bit };
use query::Query;
use choice_vec::ChoiceVec;

pub const FULL_MASK: u32 = 0b11111111_11111111_11111111_11111111;

// Representation of a hash value where some bits might be unknown.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PartialHash {
    pub hash: u32,
    pub mask: u32,
}

#[derive(Debug)]
pub struct Iter {
    current: u32,
    depth: u8,
    init: u32,
    mask: u32,
    max: u32,
}

impl PartialHash {
    pub fn match_hash(&self, other_hash: u32) -> bool {
        return (other_hash & self.mask) == self.hash;
    }

    pub fn is_complete(&self) -> bool {
        return self.mask == FULL_MASK;
    }

    pub fn possible_ids(&self, depth: u8) -> Iter {
        return Iter::new(self, depth);
    }

    pub fn from_query(query: &Query, choice: &ChoiceVec) -> PartialHash {
        let mut query_hash: u32 = 0;
        let mut query_mask: u32 = FULL_MASK;

        let hash_match = |&a| match a { None => 0, Some(a) => util_hash(a) };
        let attr_hashs: Vec<u32> = query.matches.iter().map(hash_match).collect();
        for (q_bit, &(a_index, a_bit)) in choice.data.iter().enumerate() {
            let a_hash: u32 = *unsafe { attr_hashs.get_unchecked(a_index as usize) };
            if a_hash != 0 {
                query_hash |= ith_bit(a_bit, a_hash) << q_bit;
            }
            else {
                query_mask &= !(1 << q_bit);
            }
        }

        return PartialHash { hash: query_hash, mask: query_mask };
    }
}


impl Iterator for Iter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.max == 1 && self.current == 0 {
            self.current += 1;
            return Some(self.init);
        }
        else if self.max == self.current {
            return None;
        }

        let mut page_id = self.init;
        let mut r_cursor = 0u8;
        let mut w_cursor = 0u8;

        while w_cursor < self.depth {
            if ith_bit(w_cursor, self.mask) == 0 {
                page_id |= ith_bit(r_cursor, self.current) << w_cursor;
                r_cursor += 1;
            }
            w_cursor += 1;
        }

        self.current += 1;
        return Some(page_id);
    }
}

impl Iter {
    fn new(ma_hash: &PartialHash, depth: u8) -> Self {
        let mut iterations = 1;
        let mut iter_mask = 0;

        for i in 0..depth {
            let position = 1 << i;
            iter_mask |= position;
            if ma_hash.mask & position == 0 {
                iterations <<= 1;
            }
        }

        return Iter {
            current: 0,
            depth: depth,
            init: ma_hash.hash & iter_mask & ma_hash.mask,
            mask: ma_hash.mask & iter_mask,
            max: iterations,
        };
    }
}


#[cfg(test)]
mod tests {
    use super::{ PartialHash, FULL_MASK, Iter };
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

    #[test]
    fn iter_with_no_ambiguities() {
        let mut iter = Iter::new(&PartialHash { hash: 0, mask: FULL_MASK }, 3);
        assert!(iter.next().is_some());
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_with_n_ambiguity_has_correct_number_iterations() {
        for i in 0..16 {
            let mut mask = FULL_MASK;
            for j in 0..i {
                mask &= !(1 << j);
            }

            let mut iter = Iter::new(&PartialHash { hash: 0, mask: mask }, 32);
            for _ in 0 .. (1 << i) {
                assert!(iter.next().is_some());
            }
            assert!(iter.next().is_none());
        }
    }


    #[test]
    fn iter_yields_correct_values() {
        let mut iter = Iter::new(&PartialHash { hash: FULL_MASK, mask: 0b100 }, 3);
        assert_eq!(iter.next(), Some(0b100));
        assert_eq!(iter.next(), Some(0b101));
        assert_eq!(iter.next(), Some(0b110));
        assert_eq!(iter.next(), Some(0b111));
        assert_eq!(iter.next(), None);
    }
}
