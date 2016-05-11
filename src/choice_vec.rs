use util::HASH_SIZE;
use relation::{Relation};

pub type ChoiceEntry = (u64, u8);

#[derive(Debug)]
pub struct ChoiceVec {
    pub data: [ChoiceEntry; HASH_SIZE],
}

#[derive(Debug)]
enum ParseError {
    NumberUnparsable,
    TooManyTuples,
    MissingNumError(u64),
    InvalidAttr(u8)
}

impl ChoiceVec {
    pub fn new(given_bits: Vec<ChoiceEntry>, attrs: u64) -> ChoiceVec {
        assert!(given_bits.len() <= HASH_SIZE);
        let mut cv = ChoiceVec {
            data: [(0, 0); HASH_SIZE]
        };
        // Copy in the provided bits.
        cv.data[0 .. given_bits.len()].clone_from_slice(&given_bits[..]);

        // TODO: Generate the rest.
        cv
    }

    fn parse_choice_vec(input: &str, attr_count: u8) -> Result<ChoiceVec, ParseError> {
        let mut c_vector = ChoiceVec {
            data: [(0, 0); HASH_SIZE]
        };

        for (i, entry) in input.split(':').enumerate() {
            if i > 32 { return Err(ParseError::TooManyTuples) }
            let mut split = entry.split(',');
            if let (Some(l_str), Some(r_str)) = (split.next(), split.next()) {
                let l = try!(l_str.parse().or_else(|_| Err(ParseError::NumberUnparsable)));
                let r = try!(r_str.parse().or_else(|_| Err(ParseError::NumberUnparsable)));
                if r > attr_count { return Err(ParseError::InvalidAttr(r)) }
                c_vector.data[i] = (l, r);
            }
            else {
                return Err(ParseError::MissingNumError(i as u64))
            }
        }

        Ok(c_vector)
    }
}


#[cfg(test)]
mod tests {
    use super::{ChoiceVec, ChoiceEntry};
    use util::HASH_SIZE;

    #[test]
    fn parse_1_n_2s() {
        match ChoiceVec::parse_choice_vec("1,2:1,2", 2) {
            Ok(result) => {
                let expect = ChoiceVec::new(vec![(1, 2), (1, 2)], 2);
                assert_eq!(result.data, expect.data);
            },
            Err(reason) => {
                panic!("failed to parse, because {:?}", reason);
            }
        }
    }
}

