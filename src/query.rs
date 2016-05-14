use util::{hash as util_hash, bit as ith_bit};
use choice_vec::ChoiceVec;


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Query<'a> {
    matches: Vec<Option<&'a str>>
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct QueryHash {
    pub hash: u32,
    pub mask: u32,
}


impl<'a> Query<'a> {
    fn parse(input: &'a str, attr_count: usize) -> Option<Query<'a>> {
        let matches: Vec<Option<&'a str>> = input
            .split(',')
            .map(|x| if x == "?" { None } else { Some(x) })
            .collect();

        if matches.len() == attr_count {
            return Some(Query { matches: matches });
        }
        else {
            return None
        }
    }

    fn ma_hash(&self, choice: &ChoiceVec) -> QueryHash {
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


#[cfg(test)]
mod tests {
    use super::{ Query, QueryHash };
    use choice_vec::ChoiceVec;

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
        let QueryHash { hash, mask } = query.ma_hash(&c_vec);
        assert!(mask != 0);
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
    fn parse_with_correct_arg_number() {
        let query = Query::parse("a,b,c", 3);
        assert!(query != None);
    }

    #[test]
    fn parse_with_wrong_arg_number() {
        let query_1 = Query::parse("a,b,c", 2);
        assert!(query_1 == None);

        let query_2 = Query::parse("a,b,c", 4);
        assert!(query_2 == None);
    }

    #[test]
    fn parse_correctly_identify_unknowns() {
        let query = Query::parse("a,?,c", 3);
        if let Some(query) = query {
            assert!(query.matches[0] != None);
            assert!(query.matches[1] == None);
            assert!(query.matches[2] != None);
        } else {
            panic!();
        }
    }
}

