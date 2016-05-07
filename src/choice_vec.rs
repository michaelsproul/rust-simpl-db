use util::HASH_SIZE;

pub struct ChoiceVec {
    pub data: [(u64, u8); HASH_SIZE]
}

impl ChoiceVec {
    pub fn new(given_bits: Vec<(u64, u8)>, num_attrs: u64) -> ChoiceVec {
        assert!(given_bits.len() <= HASH_SIZE);
        let mut cv = ChoiceVec {
            data: [(0, 0); HASH_SIZE]
        };
        // Copy in the provided bits.
        cv.data[0 .. given_bits.len()].clone_from_slice(&given_bits[..]);

        // TODO: Generate the rest.
        cv
    }
}
