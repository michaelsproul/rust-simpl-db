use util::HASH_SIZE;
use relation::{Relation};

pub type ChoiceEntry = (u64, u8);

#[derive(Debug)]
pub struct ChoiceVec {
    pub data: [ChoiceEntry; HASH_SIZE],
    pub attr_number: u64
}

enum ParseError {
    NumberError,
    MissingNumError
}

impl ChoiceVec {
    pub fn new(given_bits: Vec<ChoiceEntry>, attrs: u64) -> ChoiceVec {
        assert!(given_bits.len() <= HASH_SIZE);
        let mut cv = ChoiceVec {
            data: [(0, 0); HASH_SIZE],
            attr_number: attrs
        };
        // Copy in the provided bits.
        cv.data[0 .. given_bits.len()].clone_from_slice(&given_bits[..]);

        // TODO: Generate the rest.
        cv
    }

    fn parse_choice_vec(input: &str, attrs: u64) -> Result<ChoiceVec, ParseError> {
        let mut c_vector = ChoiceVec {
            data: [(0, 0); HASH_SIZE],
            attr_number: attrs
        };

        for (i, entry) in input.split(':').enumerate() {
            let mut split = entry.split(',');
            let lo = split.nth(0);
            let ro = split.nth(1);
            if let (Some(l_str), Some(r_str)) = (lo, ro) {
                let l = try!(l_str.parse::<u64>().or_else(|_| Err(ParseError::NumberError)));
                let r = try!(r_str.parse::<u8>().or_else(|_| Err(ParseError::NumberError)));
                c_vector.data[i] = (l, r);
            }
            else {
                return Err(ParseError::MissingNumError)
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
       if let Ok(result) = ChoiceVec::parse_choice_vec("1,2:1,2", 2) {
         let expect     = ChoiceVec::new(vec![(1, 2), (1, 2)], 2);
         assert_eq!(result.data, expect.data);
       }
       else {
         panic!("failed to parse");
       }
    }
}

