use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom};
use std::collections::LinkedList;
use util::*;
use tuple::Tuple;

pub const PAGE_SIZE: usize = 1024;
pub const PAGE_HEADER_SIZE: usize = 8 * 3;
pub const PAGE_DATA_SIZE: usize = PAGE_SIZE - PAGE_HEADER_SIZE;
pub const NO_OVFLOW: u64 = 0xffffffffffffffff;

pub struct Page<'a> {
    /// Page ID for this page - offset within the data file.
    pub id: u64,
    /// Data file for this page.
    file: &'a File,
    /// Offset of free space within data.
    pub free: u64,
    /// Page ID for an associated overflow page, if one exists (or NO_OFFSET).
    pub ovflow: u64,
    /// Number of tuples stored in this page.
    pub num_tuples: u64,
    /// Whether or not this page needs to be written to disk.
    dirty: bool,
    // Actual page data.
    pub data: Box<[u8; PAGE_DATA_SIZE]>
}

impl<'b> Page<'b> {
    pub fn new<'a>(file: &'a File) -> io::Result<Page<'a>> {
        let id = try!(next_page_id(file));
        Ok(Page::empty(file, id))
    }

    pub fn empty<'a>(file: &'a File, page_id: u64) -> Page<'a> {
        Page {
            id: page_id,
            file: file,
            free: 0,
            ovflow: NO_OVFLOW,
            num_tuples: 0,
            dirty: true,
            data: empty_data_block()
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Release the page and write out to disk.
    pub fn close(mut self) -> io::Result<()> {
        if self.dirty {
            self.write()
        } else {
            Ok(())
        }
    }

    pub fn free_space(&self) -> u64 {
        (PAGE_DATA_SIZE as u64) - self.free
    }

    pub fn read<'a>(mut f: &'a File, page_id: u64) -> io::Result<Page<'a>> {
        try!(seek_to_start(f, page_id));
        let free = try!(read_u64(f));
        let ovflow = try!(read_u64(f));
        let num_tuples = try!(read_u64(f));

        let mut page = Page {
            id: page_id,
            file: f,
            free: free,
            ovflow: ovflow,
            num_tuples: num_tuples,
            dirty: false,
            data: empty_data_block()
        };

        // XXX: we might need to call read more than once here.
        let bytes_read = try!(f.read(page.data.as_mut()));
        assert_eq!(bytes_read, PAGE_DATA_SIZE);

        Ok(page)
    }

    pub fn write(&mut self) -> io::Result<()> {
        try!(seek_to_start(&self.file, self.id));
        try!(write_u64(self.file, self.free));
        try!(write_u64(self.file, self.ovflow));
        try!(write_u64(self.file, self.num_tuples));
        try!(self.file.write_all(self.data.as_ref()));
        try!(self.file.flush());
        self.dirty = false;
        Ok(())
    }

    /// Retrieve all the tuples from this page.
    pub fn get_tuple_list(&self) -> LinkedList<Tuple> {
        self.data
            .split(|&b| b == 0)
            .filter(|slice| slice.len() > 0)
            .map(|slice| Tuple::parse_bytes(slice))
            .collect()
    }

    /// Add a tuple if one will fit.
    /// Return true if the tuple was added.
    // FIXME: numeric downcasts, should probably just use usize everywhere.
    pub fn add_tuple(&mut self, tuple: &[u8]) -> bool {
        if (self.free_space() as usize) < tuple.len() {
            return false;
        }
        let new_free = self.free as usize + tuple.len();
        {
            let mut dest = &mut self.data[self.free as usize .. new_free];
            dest.clone_from_slice(tuple);
        }
        self.free = new_free as u64;
        self.num_tuples += 1;
        self.mark_dirty();
        true
    }

    /// Add a tuple to this page's overflow chain, creating any necessary overflow pages.
    pub fn add_to_overflow(&mut self, ovflow_file: &File, tuple: &[u8]) -> io::Result<()> {
        // If the tuple fits in this page, insert it directly.
        if (self.free_space() as usize) >= tuple.len() {
            assert!(self.add_tuple(tuple));
            try!(self.write());
            return Ok(());
        }

        // If the tuple doesn't fit, check for an overflow page for this page.
        // If there isn't one, create one and insert the tuple.
        if self.ovflow == NO_OVFLOW {
            let mut ovflow_page = try!(Page::new(ovflow_file));
            assert!(ovflow_page.add_tuple(tuple));
            self.ovflow = ovflow_page.id;
            try!(self.write());
            try!(ovflow_page.write());
            return Ok(());
        }

        // If there is an overflow page, try the insert there by recursing.
        let mut ovflow_page = try!(Page::read(ovflow_file, self.ovflow));
        ovflow_page.add_to_overflow(ovflow_file, tuple)
    }
}

/// Create an empty data block of all zeroes.
fn empty_data_block() -> Box<[u8; PAGE_DATA_SIZE]> {
    Box::new([0; PAGE_DATA_SIZE])
}

// Fetch the Page ID of the next page to be added to a data file.
fn next_page_id(file: &File) -> io::Result<u64> {
    let file_length = try!(file.metadata().map(|m| m.len()));
    Ok(file_length / (PAGE_SIZE as u64))
}

/// Seek to the start of a page.
fn seek_to_start(mut f: &File, page_id: u64) -> io::Result<u64> {
    f.seek(SeekFrom::Start(page_id * (PAGE_SIZE as u64)))
}
