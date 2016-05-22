use choice_vec::ChoiceVec;
use util::*;
use std::str;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Tuple {
    pub values: Vec<String>
}

impl Tuple {
    pub fn hash(&self, choice_vec: &ChoiceVec) -> u32 {
        let value_hashes: Vec<u32> = self.values.iter().map(|v| hash(v)).collect();

        let mut result = 0;

        for (i, &(attr, x)) in choice_vec.data.iter().enumerate() {
            result |= bit(x, value_hashes[attr as usize]) << i;
        }

        result
    }

    /// Parse a string into a tuple, validating that it contains no question marks,
    /// and is the correct length.
    pub fn parse(s: &str, num_attrs: u32) -> Option<Tuple> {
        if s.contains('?') {
            return None;
        }
        let t = Tuple::parse_bytes(s.as_bytes());
        if t.values.len() == num_attrs as usize {
            Some(t)
        } else {
            None
        }
    }

    pub fn parse_bytes(s: &[u8]) -> Tuple {
        let parse_single = |slice| {
            str::from_utf8(slice).expect("Non UTF-8 data in database file.").to_string()
        };

        Tuple {
            values: s.split(|&b| b == ',' as u8)
                     .map(parse_single)
                     .collect()
        }
    }

    pub fn to_string(&self) -> String {
        self.values.join(",")
    }

    pub fn serialise(&self) -> Vec<u8> {
        let mut result = self.to_string().into_bytes();
        result.push(0); // NUL byte
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_question_marks() {
        assert!(Tuple::parse("hello?", 1).is_none());
        assert!(Tuple::parse("hello,?", 2).is_none());
    }

    #[test]
    fn parse() {
        assert_eq!(Tuple::parse("hello,world", 2).unwrap().values, vec!["hello", "world"]);
        assert_eq!(Tuple::parse("hello world,,", 3).unwrap().values, vec!["hello world", "", ""]);
    }

    #[test]
    fn serialise_parse_bytes() {
        use std::str;
        let data = [
            "hello,world",
            "wow",
            "rust,haskell,c,java,python,lisp"
        ];
        for &tuple in &data {
            let serialised = Tuple::parse_bytes(tuple.as_bytes()).serialise();
            let roundtrip = str::from_utf8(&serialised[..serialised.len() - 1]).unwrap();
            assert_eq!(tuple, roundtrip);
        }
    }
}
