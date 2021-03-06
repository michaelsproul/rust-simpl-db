use std::fs::File;
use std::slice;
use std::io;
use rand::{self, Rng};
use byteorder::{ReadBytesExt, WriteBytesExt};

use util::*;

pub type ChoiceEntry = (u32, u8);

/// Choice Vector struct.
/// We index attributes from 0.
#[derive(Debug, Clone)]
pub struct ChoiceVec {
    pub data: [ChoiceEntry; HASH_SIZE],
}

#[derive(Debug)]
pub enum ParseError {
    NumberUnparsable,
    TooManyTuples,
    InvalidEntry,
    InvalidAttr(u32),
    InvalidBit(u8)
}

impl ChoiceVec {
    pub fn new(given_bits: Vec<ChoiceEntry>, num_attrs: u32) -> ChoiceVec {
        assert!(given_bits.len() <= HASH_SIZE);
        let mut cv = ChoiceVec {
            data: [(0, 0); HASH_SIZE]
        };
        // Copy in the provided bits.
        cv.data[0 .. given_bits.len()].clone_from_slice(&given_bits[..]);

        // Generate the rest.
        cv.generate_from(num_attrs, given_bits.len());

        cv
    }

    /// Generate all entries of the choice vector from index `start` onwards.
    fn generate_from(&mut self, num_attrs: u32, start: usize) {
        let mut rng = rand::thread_rng();

        for i in start..HASH_SIZE {
            loop {
                let (attr, bit) = (rng.gen::<u32>() % num_attrs, rng.gen::<u8>() % HASH_SIZE as u8);
                if !(&self.data[..i]).contains(&(attr, bit)) {
                    self.data[i] = (attr, bit);
                    trace!("Generated entry: ({}, {})", attr, bit);
                    break;
                }
            }
        }
    }

    pub fn parse(input: &str, num_attrs: u32) -> Result<ChoiceVec, ParseError> {
        let mut given_bits = vec![];

        for (i, entry) in input.split(':').enumerate() {
            if i >= HASH_SIZE { return Err(ParseError::TooManyTuples) }
            let split: Vec<&str> = entry.split(',').collect();
            if split.len() != 2 {
                return Err(ParseError::InvalidEntry);
            }
            let l: u32 = try!(split[0].parse().or_else(|_| Err(ParseError::NumberUnparsable)));
            let r: u8 = try!(split[1].parse().or_else(|_| Err(ParseError::NumberUnparsable)));
            if l >= num_attrs { return Err(ParseError::InvalidAttr(l)) }
            if r as usize >= HASH_SIZE { return Err(ParseError::InvalidBit(r)) }
            given_bits.push((l, r));
        }

        Ok(ChoiceVec::new(given_bits, num_attrs))
    }

    pub fn write(&self, mut f: &File) -> io::Result<()> {
        for &(attr, val) in self.data.iter() {
            try!(write_u32(f, attr));
            try!(f.write_u8(val));
        }
        Ok(())
    }

    pub fn read(mut f: &File) -> io::Result<ChoiceVec> {
        let mut data = [(0, 0); HASH_SIZE];
        for i in 0..HASH_SIZE {
            let attr = try!(read_u32(f));
            let val = try!(f.read_u8());
            data[i] = (attr, val);
        }
        Ok(ChoiceVec { data: data })
    }

    pub fn iter(&self) -> slice::Iter<ChoiceEntry> {
        return self.data.iter();
    }
}

#[cfg(test)]
mod tests {
    use super::ChoiceVec;
    use util::HASH_SIZE;

    #[test]
    fn parse_1_n_2s() {
        match ChoiceVec::parse("1,2:1,2", 2) {
            Ok(result) => {
                let expect = vec![(1, 2), (1, 2)];
                assert_eq!(&result.data[..2], &expect[..]);
            },
            Err(reason) => {
                panic!("failed to parse, because {:?}", reason);
            }
        }
    }

    #[test]
    fn parse_three() {
        ChoiceVec::parse("1,2,3", 3).unwrap_err();
    }

    #[test]
    fn parse_too_long() {
        use std::iter::repeat;
        let raw_vec: Vec<&str> = repeat("1,1").take(HASH_SIZE + 1).collect();
        let too_long: String = raw_vec.join(":");
        ChoiceVec::parse(&too_long, 10).unwrap_err();
    }

    #[test]
    fn parse_attr_too_high() {
        ChoiceVec::parse("50,1", 1).unwrap_err();
        ChoiceVec::parse("1,1", 1).unwrap_err();
    }

    #[test]
    fn parse_bit_too_high() {
        ChoiceVec::parse("0,31", 1).unwrap();
        ChoiceVec::parse("0,32", 1).unwrap_err();
        ChoiceVec::parse("0,33", 1).unwrap_err();
    }

    // This case should be the slowest for generating a choice vec, as the
    // fraction of the search space that's invalid each iteration is maximised.
    // After 31 bits have been generated, there's a 1/32 chance of a correct guess.
    // It doesn't seem too bad.
    #[test]
    fn pathological_generate() {
        assert_eq!(ChoiceVec::new(vec![], 1).data.len(), HASH_SIZE);
    }
}
