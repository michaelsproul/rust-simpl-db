use std::fs::File;
use std::io::{self, Read, Write};
use std::env;
use std::process::exit;
use std::error::Error;
use std::hash::{Hash, SipHasher, Hasher};

use env_logger::LogBuilder;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub const HASH_SIZE: usize = 32;

pub type BoxError = Box<Error + Send + Sync>;

// Binary IO functions for fixed size integers.
pub fn read_u8(mut f: &File) -> io::Result<u8> {
    f.read_u8()
}

pub fn write_u8(mut f: &File, x: u8) -> io::Result<()> {
    f.write_u8(x)
}

pub fn read_u32<R: Read>(mut f: R) -> io::Result<u32> {
    f.read_u32::<BigEndian>()
}

pub fn write_u32<W: Write>(mut f: W, x: u32) -> io::Result<()> {
    f.write_u32::<BigEndian>(x)
}

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

/// determines the highest set bit of u32
pub fn highest_set_bit(number: u32) -> u8 {
    let mut result = 0;
    let mut iter = number;
    while iter != 0 {
        result += 1;
        iter  >>= 1;
    }
    return result;
}


/// Enable logging.
pub fn enable_logging() {
    let mut builder = LogBuilder::new();
    builder.format(|rec| format!("{}", rec.args()));
    if let Ok(log_options) = env::var("RUST_LOG") {
        builder.parse(&log_options);
    }
    builder.init().unwrap();
}

// Round a user-supplied number of pages up to the nearest 2^n.
// Return (n, 2^n).
pub fn get_depth_and_num_pages(num_pages: u64) -> (u8, u64) {
    if num_pages == 0 {
        return (0, 1);
    }
    let n = (num_pages as f64).log2().ceil() as u8;
    (n, 1 << n)
}

/// Run a main function that returns a Result.
pub fn run_main(real_main: fn() -> Result<(), BoxError>) -> ! {
    enable_logging();
    let mut exit_code = 0;
    if let Err(e) = real_main() {
        error!("{}", e);
        exit_code = 1;
    }
    exit(exit_code);
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
    fn test_hsb() {
        assert_eq!(highest_set_bit(0b0), 0);
        assert_eq!(highest_set_bit(0b1), 1);
        assert_eq!(highest_set_bit(0b10), 2);
        assert_eq!(highest_set_bit(0b100), 3);
        assert_eq!(highest_set_bit(0b1000), 4);
        assert_eq!(highest_set_bit(0b1011), 4);
        assert_eq!(highest_set_bit(0b101011), 6);
        assert_eq!(highest_set_bit(1 << 31), 32);
    }

    #[test]
    fn test_lower_bits() {
        assert_eq!(lower_bits(2, 0b111101), 0b01);
    }

    #[test]
    fn round_pages() {
        assert_eq!(get_depth_and_num_pages(0), (0, 1));
        assert_eq!(get_depth_and_num_pages(1), (0, 1));
        assert_eq!(get_depth_and_num_pages(2), (1, 2));
        assert_eq!(get_depth_and_num_pages(3), (2, 4));
        assert_eq!(get_depth_and_num_pages(6), (3, 8));
    }
}
