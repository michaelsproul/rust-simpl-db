use choice_vec::ChoiceVec;
use util::*;

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

    pub fn serialise(&self) -> Vec<u8> {
        let mut result = self.values.join(",").into_bytes();
        result.push(0); // NUL byte
        result
    }
}
