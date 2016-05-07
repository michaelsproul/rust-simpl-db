use std::fs::File;
use std::io;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub fn read_u64(f: &mut File) -> io::Result<u64> {
    f.read_u64::<BigEndian>()
}

pub fn write_u64(f: &mut File, x: u64) -> io::Result<()> {
    f.write_u64::<BigEndian>(x)
}
