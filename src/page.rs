use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom};
use util::*;

pub const PAGE_SIZE: usize = 1024;
pub const PAGE_HEADER_SIZE: usize = 8 * 3;
pub const PAGE_DATA_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE;
pub const NO_OFFSET: u64 = 0xffffffffffffffff;

pub struct Page {
    /// Offset of free space within data.
    pub free: u64,
    /// Page ID for an associated overflow page, if one exists (or NO_OFFSET).
    pub ovflow: u64,
    /// Number of tuples stored in this page.
    pub num_tuples: u64,
    // Actual page data.
    pub data: Box<[u8; PAGE_DATA_SIZE]>
}

fn empty_data_block() -> Box<[u8; PAGE_DATA_SIZE]> {
    Box::new([0; PAGE_DATA_SIZE])
}

impl Page {
    pub fn empty() -> Page {
        Page {
            free: 0,
            ovflow: NO_OFFSET,
            num_tuples: 0,
            data: empty_data_block()
        }
    }

    fn seek_to_start(f: &mut File, page_id: u64) -> io::Result<u64> {
        f.seek(SeekFrom::Start(page_id * (PAGE_SIZE as u64)))
    }

    pub fn read(f: &mut File, page_id: u64) -> io::Result<Page> {
        try!(Page::seek_to_start(f, page_id));
        let free = try!(read_u64(f));
        let ovflow = try!(read_u64(f));
        let num_tuples = try!(read_u64(f));

        let mut page = Page {
            free: free,
            ovflow: ovflow,
            num_tuples: num_tuples,
            data: empty_data_block()
        };

        // XXX: we might need to call read more than once here.
        let bytes_read = try!(f.read(page.data.as_mut()));
        assert_eq!(bytes_read, PAGE_DATA_SIZE);

        Ok(page)
    }

    pub fn write(&self, f: &mut File, page_id: u64) -> io::Result<()> {
        try!(Page::seek_to_start(f, page_id));
        try!(write_u64(f, self.free));
        try!(write_u64(f, self.ovflow));
        try!(write_u64(f, self.num_tuples));
        try!(f.write_all(self.data.as_ref()));
        f.flush()
    }

    pub fn free_space(&self) -> u64 {
        (PAGE_DATA_SIZE as u64) - self.free
    }

    /// Add a tuple if one will fit.
    /// Return true if the tuple was added.
    // FIXME: numeric downcasts, should probably just use usize everywhere.
    pub fn try_add_tuple(&mut self, tuple: &[u8]) -> bool {
        if (self.free_space() as usize) < tuple.len() {
            return false;
        }
        let new_free = self.free as usize + tuple.len();
        let mut dest = &mut self.data[self.free as usize .. new_free];
        dest.clone_from_slice(tuple);
        self.free = new_free as u64;
        self.num_tuples += 1;
        true
    }
}
