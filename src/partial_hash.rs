use util::*;

pub const UNKNOWN: u8 = 2;

// Representation of a hash value where some bits might be unknown.
pub struct PartialHash {
    pub data: [u8; HASH_SIZE]
}

impl PartialHash {
    pub fn as_integer(&self, num_bits: u64) -> u32 {
        assert!(self.is_complete());
        let mut result = 0u32;
        for i in 0..num_bits {
            result |= (self.data[i as usize] as u32) << i;
        }
        result
    }

    pub fn is_complete(&self) -> bool {
        !self.data.contains(&UNKNOWN)
    }
}
