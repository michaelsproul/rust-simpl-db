use std::fs::File;
use std::io;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::hash::{Hash, SipHasher, Hasher};

pub const HASH_SIZE: usize = 32;

pub fn read_u64(mut f: &File) -> io::Result<u64> {
    f.read_u64::<BigEndian>()
}

pub fn write_u64(mut f: &File, x: u64) -> io::Result<()> {
    f.write_u64::<BigEndian>(x)
}

/// Hash a value with the Rust hasher.
/// Note that the cast from u64 to i32 takes only the lower 32-bits of the hash,
/// which is fine for our purposes.
/// FIXME: We might want to use the same hash function as John.
pub fn hash<T: Hash + ?Sized>(t: &T) -> u32 {
    let mut s = SipHasher::new();
    t.hash(&mut s);
    s.finish() as u32
}

/// Grab the ith bit of a value.
#[inline]
pub fn bit(i: u8, val: u32) -> u32 {
    (val >> i) & 1
}

// Grab the lower n bits of a value.
pub fn lower_bits(n: u8, val: u32) -> u32 {
    let mut mask = 0;
    for i in 0..n {
        mask |= 1 << i;
    }
    val & mask
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_bit() {
        assert_eq!(bit(0, 0b1100), 0);
        assert_eq!(bit(1, 0b1100), 0);
        assert_eq!(bit(2, 0b1100), 1);
        assert_eq!(bit(3, 0b1100), 1);
        assert_eq!(bit(4, 0b1100), 0);
    }

    #[test]
    fn test_lower_bits() {
        assert_eq!(lower_bits(2, 0b111101), 0b01);
    }
}
