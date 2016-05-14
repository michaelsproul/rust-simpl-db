#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Query<'a> {
    pub matches: Vec<Option<&'a str>>
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

