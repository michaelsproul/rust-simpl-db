use util::{hash as util_hash, bit as ith_bit, FULL_MASK };
use choice_vec::ChoiceVec;
use partial_hash::PartialHash;


#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Query<'a> {
    matches: Vec<Option<&'a str>>
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ParseError {
    AttributeMismatch(usize, usize),
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

    pub fn ma_hash(&self, choice: &ChoiceVec) -> PartialHash {


        let mut query_hash: u32 = 0;
        let mut query_mask: u32 = FULL_MASK;

        let hash_match = |&a| match a { None => 0, Some(a) => util_hash(a) };
        let attr_hashs: Vec<u32> = self.matches.iter().map(hash_match).collect();
        for (q_bit, &(a_index, a_bit)) in choice.data.iter().enumerate() {
            let a_hash: u32 = *unsafe { attr_hashs.get_unchecked(a_index as usize) };
            if a_hash != 0 {
                query_hash |= ith_bit(a_bit, a_hash) << q_bit;
            }
            else {
                query_mask |= 1 << q_bit;
            }
        }

        return PartialHash {
            hash: query_hash,
            mask: query_mask,
        };
    }
}


#[cfg(test)]
mod tests {
    use super::{ Query, ParseError };

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

