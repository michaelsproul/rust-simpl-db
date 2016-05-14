use util::{hash as util_hash, bit as ith_bit};
use choice_vec::ChoiceVec;


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Query<'a> {
    matches: Vec<Option<&'a str>>
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ParseError {
    AttributeMismatch(usize, usize),
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct QueryHash {
    pub hash: u32,
    pub mask: u32,
}


impl<'a> Query<'a> {
    pub fn parse(input: &'a str, attr_count: u64) -> Result<Query<'a>, ParseError> {
        let matches: Vec<Option<&'a str>> = input
            .split(',')
            .map(|x| if x == "?" { None } else { Some(x) })
            .collect();

        let match_len = matches.len();
        if match_len == attr_count as usize {
            return Ok(Query { matches: matches });
        }
        else {
            return Err(ParseError::AttributeMismatch(attr_count as usize, match_len));
        }
    }

    pub fn ma_hash(&self, choice: &ChoiceVec) -> QueryHash {
        let mut query_hash: u32 = 0;
        let mut query_mask: u32 = 0;

        let hash_match = |&a| match a { None => 0, Some(a) => util_hash(a) };
        let attr_hashs: Vec<u32> = self.matches.iter().map(hash_match).collect();
        for (q_bit, &(a_index, a_bit)) in choice.data.iter().enumerate() {
            let a_hash: u32 = *unsafe { attr_hashs.get_unchecked(a_index as usize) };
            if a_hash != 0 {
                query_hash |= ith_bit(a_bit, a_hash) << q_bit;
                query_mask |= 1 << q_bit;
            }
        }

        return QueryHash {
            hash: query_hash,
            mask: query_mask,
        };
    }
}

impl QueryHash {
    pub fn match_hash(&self, other_hash: u32) -> bool {
        return (other_hash & self.mask) == self.hash;
    }
}


#[cfg(test)]
mod tests {
    use super::{ Query, QueryHash, ParseError };
    use tuple::Tuple;
    use choice_vec::ChoiceVec;
    use rand::random;

    const FULL_MASK: u32 = 0b11111111_11111111_11111111_11111111;

    // hashing

    #[test]
    fn hash_mask_correctness() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let QueryHash { hash, mask } = query.ma_hash(&c_vec);
        assert_eq!(hash & mask, hash);
        assert_eq!(hash | mask, mask);
    }

    #[test]
    fn hash_when_known_isnt_0() {
        let query = Query::parse("a,b,c", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let hash = query.ma_hash(&c_vec);
        assert!(hash.mask != 0);
    }

    #[test]
    fn hash_with_unknown() {
        let query = Query::parse("?,?,?", 3).unwrap();
        let c_vec = ChoiceVec::parse("0,0:1,1:2,2", 3).unwrap();
        let QueryHash { hash, mask } = query.ma_hash(&c_vec);
        assert_eq!(hash, 0);
        assert_eq!(mask, 0);
    }

    #[test]
    fn query_hash_same_as_tuple_hash() {
        let c_vec = ChoiceVec::parse("0,0:0,1:1,0:1,1:2,0:2,1", 3).unwrap();
        let tuple = Tuple::parse("a,b,c", 3).unwrap();
        let query = Query::parse("a,b,c", 3).unwrap();
        assert_eq!(tuple.hash(&c_vec), query.ma_hash(&c_vec).hash);
    }

    // matching hashed query

    #[test]
    fn hash_matching_zero() {
        let num_hash: u32 = 0;
        let query_hash = QueryHash { hash: num_hash, mask: FULL_MASK };
        assert!(query_hash.match_hash(num_hash));
    }

    #[test]
    fn hash_matching_non_zero() {
        let num_hash = random::<u32>();
        let query_hash = QueryHash { hash: num_hash, mask: FULL_MASK };
        assert!(query_hash.match_hash(num_hash));
    }

    // query parsing matching

    #[test]
    fn parse_with_correct_arg_number() {
        let query = Query::parse("a,b,c", 3);
        assert!(query.is_ok());
    }

    #[test]
    fn parse_with_wrong_arg_number() {
        let query_1 = Query::parse("a,b,c", 2);
        assert!(query_1 == Err(ParseError::AttributeMismatch(3)));

        let query_2 = Query::parse("a,b,c", 4);
        assert!(query_2 == Err(ParseError::AttributeMismatch(3)));
    }

    #[test]
    fn parse_correctly_identify_unknowns() {
        let query = Query::parse("a,?,c", 3);
        if let Ok(query) = query {
            assert!(query.matches[0] != None);
            assert!(query.matches[1] == None);
            assert!(query.matches[2] != None);
        } else {
            panic!();
        }
    }
}

