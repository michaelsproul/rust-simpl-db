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

impl MAHash {
    pub fn match_hash(&self, other_hash: u32) -> bool {
        return (other_hash & self.mask) == self.hash;
    }

    pub fn is_complete(&self) -> bool {
        return self.mask == FULL_MASK;
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
