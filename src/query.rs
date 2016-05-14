
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Query<'a> {
    segments: Vec<Option<&'a str>>
}


impl<'a> Query<'a> {
    fn parse_query(input: &'a str, attr_count: usize) -> Option<Query<'a>> {
        let segments: Vec<Option<&'a str>> = input
            .split(',')
            .map(|x| if x == "?" { None } else { Some(x) })
            .collect();

        if segments.len() == attr_count {
            return Some(Query { segments: segments });
        }
        else {
            return None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::Query;

    #[test]
    fn parse_with_correct_arg_number() {
        let query = Query::parse_query("a,b,c", 3);
        assert!(query != None);
    }

    #[test]
    fn parse_with_wrong_arg_number() {
        let query_1 = Query::parse_query("a,b,c", 2);
        assert!(query_1 == None);

        let query_2 = Query::parse_query("a,b,c", 4);
        assert!(query_2 == None);
    }

    #[test]
    fn parse_correctly_identify_unknowns() {
        let query = Query::parse_query("a,?,c", 3);
        if let Some(query) = query {
            assert!(query.segments[0] != None);
            assert!(query.segments[1] == None);
            assert!(query.segments[2] != None);
        } else {
            panic!();
        }
    }
}
