use util::{hash as util_hash, bit as ith_bit };
use query::Query;
use choice_vec::ChoiceVec;

pub const FULL_MASK: u32 = 0b11111111_11111111_11111111_11111111;

// Representation of a hash value where some bits might be unknown.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct MAHash {
    pub hash: u32,
    pub mask: u32,
}

pub struct IDIter {
    pub current: u32,
    pub hash: u32,
    pub mask: u32,
    pub max: u32,
}


impl MAHash {
    pub fn match_hash(&self, other_hash: u32) -> bool {
        return (other_hash & self.mask) == self.hash;
    }

    pub fn is_complete(&self) -> bool {
        return self.mask == FULL_MASK;
    }

    pub fn ids_within(&self, depth: u32) -> IDIter {
        return IDIter::new(self, depth);
    }

    pub fn from_query(query: &Query, choice: &ChoiceVec) -> MAHash {
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

        return MAHash { hash: query_hash, mask: query_mask };
    }
}


impl Iterator for IDIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.max == 1 && self.current == 0 {
            self.current += 1;
            return Some(self.hash);
        }
        else if self.max == self.current {
            return None;
        }

        let mut page_id = self.hash;
        let mut r_cursor = 0;
        let mut w_cursor = 0;

        while (1 << w_cursor) < self.current {
            if ith_bit(w_cursor, self.mask) == 1 {
                page_id |= ith_bit(r_cursor, self.current) << w_cursor;
                r_cursor += 1;
            }
            w_cursor += 1;
        }

        self.current += 1;
        return Some(page_id);
    }
}

impl IDIter {
    fn new(ma_hash: &MAHash, depth: u32) -> Self {
        let mut iterations = 1;
        let mut iter_mask = 0;

        for i in 0..depth {
            let position = 1 << i;
            iter_mask |= position;
            if ma_hash.mask & position == 0 {
                iterations *= 2;
            }
        }

        return IDIter {
            current: 0,
            hash: ma_hash.hash | iter_mask,
            mask: !ma_hash.mask | iter_mask,
            max: iterations,
        };
    }
}


#[cfg(test)]
mod tests {
    use super::{ MAHash, FULL_MASK };
    use query::{ Query };
    use tuple::Tuple;
    use choice_vec::ChoiceVec;
    use rand::random;

    // hashing

    #[test]
    fn hash_mask_correctness() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let MAHash { hash, mask } = MAHash::from_query(&query, &c_vec);
        assert_eq!(hash & mask, hash);
        assert_eq!(hash | mask, mask);
    }

    #[test]
    fn hash_when_known_isnt_0() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let ma_hash = MAHash::from_query(&query, &c_vec);
        assert!(ma_hash.mask != 0);
    }

    #[test]
    fn hash_with_unknown() {
        let query = Query::parse("?,?,?", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let MAHash { hash, mask } = MAHash::from_query(&query, &c_vec);
        assert_eq!(hash, 0);
        assert_eq!(mask, 0);
    }

    #[test]
    fn query_hash_same_as_tuple_hash() {
        let c_vec = ChoiceVec::parse("0,0:0,1:1,0:1,1:2,0:2,1", 3).unwrap();
        let tuple = Tuple::parse("a,b,c", 3).unwrap();
        let query = Query::parse("a,b,c", 3).unwrap();
        let ma_hash = MAHash::from_query(&query, &c_vec);
        assert_eq!(tuple.hash(&c_vec), ma_hash.hash);
    }

    // matching hashed query

    #[test]
    fn hash_matching_zero() {
        let num_hash: u32 = 0;
        let query_hash = MAHash { hash: num_hash, mask: FULL_MASK };
        assert!(query_hash.match_hash(num_hash));
    }

    #[test]
    fn hash_matching_non_zero() {
        let num_hash = random::<u32>();
        let query_hash = MAHash { hash: num_hash, mask: FULL_MASK };
        assert!(query_hash.match_hash(num_hash));
    }
}
