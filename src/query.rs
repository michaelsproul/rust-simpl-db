use tuple::Tuple;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Query<'a> {
    pub matches: Vec<Option<&'a str>>
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ParseError {
    AttributeMismatch(usize, usize),
}

impl<'a> Query<'a> {
    /// Construct a Query that matches anything for a given number of attributes.
    pub fn wildcard(num_attrs: u32) -> Query<'a> {
        Query { matches: vec![None; num_attrs as usize] }
    }

    pub fn parse(input: &'a str, num_attrs: u32) -> Result<Query<'a>, ParseError> {
        let matches: Vec<Option<&'a str>> = input
            .split(',')
            .map(|x| if x == "?" { None } else { Some(x) })
            .collect();

        let match_len = matches.len();
        if match_len == num_attrs as usize {
            return Ok(Query { matches: matches });
        }
        else {
            return Err(ParseError::AttributeMismatch(num_attrs as usize, match_len));
        }
    }

    pub fn matches_tuple(&self, tuple: &Tuple) -> bool {
        debug_assert!(self.matches.len() == tuple.values.len());
        for i in 0..tuple.values.len() {
            if let Some(query_attr) = self.matches[i] {
                if query_attr != &tuple.values[i] {
                    return false;
                }
            }
        }
        true
    }
}


#[cfg(test)]
mod tests {
    use super::{ Query, ParseError };
    use tuple::Tuple;

    // query parsing matching

    #[test]
    fn parse_with_correct_arg_number() {
        let query = Query::parse("a,b,c", 3);
        assert!(query.is_ok());
    }

    #[test]
    fn parse_with_wrong_arg_number() {
        let query_1 = Query::parse("a,b,c", 2);
        assert!(query_1 == Err(ParseError::AttributeMismatch(2, 3)));

        let query_2 = Query::parse("a,b,c", 4);
        assert!(query_2 == Err(ParseError::AttributeMismatch(4, 3)));
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

    #[test]
    fn matching() {
        let data = [
            (Query::parse("hello,?", 2), Tuple::parse("hello,world", 2), true),
            (Query::parse("hello,?", 2), Tuple::parse("world,hello", 2), false),
            (Query::parse("?", 1), Tuple::parse("wowzas", 1), true)
        ];
        for &(ref query, ref tuple, exp) in data.iter() {
            assert_eq!(query.as_ref().unwrap().matches_tuple(tuple.as_ref().unwrap()), exp);
        }
    }
}
