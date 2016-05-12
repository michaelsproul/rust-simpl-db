use util::HASH_SIZE;

pub type ChoiceEntry = (u64, u8);

#[derive(Debug)]
pub struct ChoiceVec {
    pub data: [ChoiceEntry; HASH_SIZE],
}

#[derive(Debug)]
pub enum ParseError {
    NumberUnparsable,
    TooManyTuples,
    MissingNumError(u64),
    InvalidAttr(u8),
    InvalidEntry
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

    pub fn parse_choice_vec(input: &str, attr_count: u8) -> Result<ChoiceVec, ParseError> {
        let mut c_vector = ChoiceVec {
            data: [(0, 0); HASH_SIZE]
        };

        for (i, entry) in input.split(':').enumerate() {
            if i >= HASH_SIZE { return Err(ParseError::TooManyTuples) }
            let split: Vec<&str> = entry.split(',').collect();
            if split.len() != 2 {
                return Err(ParseError::InvalidEntry);
            }
            let l = try!(split[0].parse().or_else(|_| Err(ParseError::NumberUnparsable)));
            let r = try!(split[1].parse().or_else(|_| Err(ParseError::NumberUnparsable)));
            if r > attr_count { return Err(ParseError::InvalidAttr(r)) }
            c_vector.data[i] = (l, r);
        }

        Ok(c_vector)
    }
}

#[cfg(test)]
mod tests {
    use super::ChoiceVec;
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

    #[test]
    fn parse_three() {
        ChoiceVec::parse_choice_vec("1,2,3", 3).unwrap_err();
    }

    #[test]
    fn parse_too_long() {
        use std::iter::repeat;
        let raw_vec: Vec<&str> = repeat("1,1").take(HASH_SIZE + 1).collect();
        let too_long: String = raw_vec.join(":");
        ChoiceVec::parse_choice_vec(&too_long, 10).unwrap_err();
    }
}
