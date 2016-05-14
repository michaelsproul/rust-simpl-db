use choice_vec::ChoiceVec;
use util::FULL_MASK;


// Representation of a hash value where some bits might be unknown.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PartialHash {
    pub hash: u32,
    pub mask: u32,
}

impl PartialHash {
    pub fn match_hash(&self, other_hash: u32) -> bool {
        return (other_hash & self.mask) == self.hash;
    }

    pub fn is_complete(&self) -> bool {
        return self.mask == FULL_MASK;
    }
}

#[cfg(test)]
mod tests {
    use super::{ PartialHash, FULL_MASK };
    use query::{ Query };
    use tuple::Tuple;
    use choice_vec::ChoiceVec;
    use rand::random;


    // hashing

    #[test]
    fn hash_mask_correctness() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let PartialHash { hash, mask } = query.ma_hash(&c_vec);
        assert_eq!(hash & mask, hash);
        assert_eq!(hash | mask, mask);
    }

    #[test]
    fn hash_when_known_isnt_0() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let ma_hash = query.ma_hash(&c_vec);
        assert!(hash.mask != 0);
    }

    #[test]
    fn hash_with_unknown() {
        let query = Query::parse("?,?,?", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let PartialHash { hash, mask } = query.ma_hash(&c_vec);
        assert_eq!(hash, 0);
        assert_eq!(mask, 0);
    }

    #[test]
    fn query_hash_same_as_tuple_hash() {
        let c_vec = ChoiceVec::parse("0,0:0,1:1,0:1,1:2,0:2,1", 3).unwrap();
        let tuple = Tuple::parse("a,b,c", 3).unwrap();
        let query = Query::parse("a,b,c", 3).unwrap();
        let ma_hash = query.ma_hash(&c_vec);
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
}
