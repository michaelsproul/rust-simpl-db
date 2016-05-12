use choice_vec::ChoiceVec;
use util::*;
use std::str;

#[derive(Clone)]
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

    pub fn parse(s: &[u8]) -> Tuple {
        let parse_single = |slice| {
            str::from_utf8(slice).expect("Non UTF-8 data in database file.").to_string()
        };

        Tuple {
            values: s.split(|&b| b == ',' as u8)
                     .map(parse_single)
                     .collect()
        }
    }

    pub fn serialise(&self) -> Vec<u8> {
        let mut result = self.values.join(",").into_bytes();
        result.push(0); // NUL byte
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialise_parse() {
        use std::str;
        let data = [
            "hello,world",
            "wow",
            "rust,haskell,c,java,python,lisp"
        ];
        for &tuple in &data {
            let serialised = Tuple::parse(tuple.as_bytes()).serialise();
            let roundtrip = str::from_utf8(&serialised[..serialised.len() - 1]).unwrap();
            assert_eq!(tuple, roundtrip);
        }
    }
}
