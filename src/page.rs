use std::fs::File;
use std::io::{self, Read, Write, Seek, SeekFrom, Cursor};
use std::io::ErrorKind::InvalidInput;
use std::collections::LinkedList;
use util::*;
use tuple::Tuple;
use query::Query;

pub const PAGE_SIZE: u64 = 1024;
pub const PAGE_HEADER_SIZE: u64 = 4 * 3;
pub const PAGE_DATA_SIZE: usize = (PAGE_SIZE - PAGE_HEADER_SIZE) as usize;
pub const NO_OVFLOW: u32 = 0xffffffff;

pub struct Page<'a> {
    /// Page ID for this page - offset within the data file.
    pub id: u32,
    /// Data file for this page.
    file: &'a File,
    /// Offset of free space within data.
    pub free: u32,
    /// Page ID for an associated overflow page, if one exists (or NO_OFFSET).
    pub ovflow: u32,
    /// Number of tuples stored in this page.
    pub num_tuples: u32,
    /// Whether or not this page needs to be written to disk.
    dirty: bool,
    // Actual page data.
    pub data: Box<[u8; PAGE_DATA_SIZE]>
}

impl<'b> Page<'b> {
    pub fn new<'a>(file: &'a File) -> io::Result<Page<'a>> {
        let id = try!(get_next_page_id(file));
        Ok(Page::empty(file, id))
    }

    pub fn empty<'a>(file: &'a File, page_id: u32) -> Page<'a> {
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
        self.write()
        /*
        if self.dirty {
            self.write()
        } else {
            Ok(())
        }
        */
    }

    pub fn free_space(&self) -> usize {
        PAGE_DATA_SIZE - (self.free as usize)
    }

    pub fn read<'a>(mut f: &'a File, page_id: u32) -> io::Result<Page<'a>> {
        try!(seek_to_start(f, page_id));

        // Load the whole page.
        let mut buffer = Box::new([0; PAGE_SIZE as usize]);
        try!(f.read_exact(&mut buffer[..]));

        // Parse the page's data.
        let mut cursor = Cursor::new(&buffer.as_ref()[..]);

        let free = try!(read_u32(&mut cursor));
        let ovflow = try!(read_u32(&mut cursor));
        let num_tuples = try!(read_u32(&mut cursor));

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
        let bytes_read = try!(cursor.read(page.data.as_mut()));
        assert_eq!(bytes_read, PAGE_DATA_SIZE);

        Ok(page)
    }

    pub fn write(&mut self) -> io::Result<()> {
        try!(seek_to_start(&self.file, self.id));
        // Write all the data into a buffer.
        let mut buf = Vec::<u8>::with_capacity(PAGE_SIZE as usize);
        try!(write_u32(&mut buf, self.free));
        try!(write_u32(&mut buf, self.ovflow));
        try!(write_u32(&mut buf, self.num_tuples));
        try!(buf.write_all(self.data.as_ref()));
        try!(self.file.write_all(buf.as_ref()));
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

    /// Retrieve all the tuples from this page that match the given query.
    pub fn get_tuples_matching<'a>(&self, query: &'a Query<'a>) -> LinkedList<Tuple> {
        self.data
            .split(|&b| b == 0)
            .filter(|slice| slice.len() > 0)
            .map(|slice| Tuple::parse_bytes(slice))
            .filter(|tuple| query.matches_tuple(tuple))
            .collect()
    }

    /// Retrieve all tuples matching a given query from this page AND its overflow.
    pub fn select<'a>(&self, query: &'a Query<'a>, ovflow_file: &'a File) -> PageQueryIter<'a> {
        PageQueryIter {
            query: query,
            next_page_id: self.ovflow,
            ovflow_file: ovflow_file,
            tuple_cache: self.get_tuples_matching(query),
        }
    }

    /// Add a tuple if one will fit.
    /// Return true if the tuple was added.
    pub fn add_tuple(&mut self, tuple: &[u8]) -> bool {
        if self.free_space() < tuple.len() {
            return false;
        }
        let new_free = self.free + tuple.len() as u32;
        {
            let mut dest = &mut self.data[self.free as usize .. new_free as usize];
            dest.clone_from_slice(tuple);
        }
        self.free = new_free;
        self.num_tuples += 1;
        self.mark_dirty();
        true
    }

    /// Add a tuple to this page's overflow chain, creating any necessary overflow pages.
    pub fn add_to_overflow(&mut self, ovflow_file: &File, tuple: &[u8]) -> io::Result<()> {
        if tuple.len() > PAGE_DATA_SIZE {
            return Err(io::Error::new(InvalidInput, "tuple too large to fit in a page"));
        }

        // If the tuple fits in this page, insert it directly.
        if self.free_space() >= tuple.len() {
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

/// Iterator over all matching tuples in a bucket.
pub struct PageQueryIter<'a> {
    query: &'a Query<'a>,
    /// The ID of the next overflow page to read - initially the first overflow page.
    next_page_id: u32,
    ovflow_file: &'a File,
    /// Tuples read from the bucket that have not yet been yielded.
    /// Initially contains all the matching tuples from the data page.
    tuple_cache: LinkedList<Tuple>
}

impl<'a> Iterator for PageQueryIter<'a> {
    type Item = io::Result<Tuple>;

    fn next(&mut self) -> Option<io::Result<Tuple>> {
        // If there's a tuple ready, yield it.
        if let Some(tuple) = self.tuple_cache.pop_front() {
            return Some(Ok(tuple));
        }
        // If there are no more pages in this bucket, yield None forever.
        if self.next_page_id == NO_OVFLOW {
            return None;
        }
        // Otherwise, load the next page in the chain and recurse.
        let page = match Page::read(self.ovflow_file, self.next_page_id) {
            Ok(p) => p,
            Err(e) => {
                return Some(Err(e));
            }
        };
        self.next_page_id = page.ovflow;
        self.tuple_cache = page.get_tuples_matching(self.query);
        self.next()
    }
}

/// Create an empty data block of all zeroes.
fn empty_data_block() -> Box<[u8; PAGE_DATA_SIZE]> {
    Box::new([0; PAGE_DATA_SIZE])
}

// Fetch the Page ID of the next page to be added to a data file.
pub fn get_next_page_id(file: &File) -> io::Result<u32> {
    let file_length = try!(file.metadata().map(|m| m.len()));
    Ok((file_length / PAGE_SIZE) as u32)
}

/// Seek to the start of a page.
fn seek_to_start(mut f: &File, page_id: u32) -> io::Result<u64> {
    f.seek(SeekFrom::Start((page_id as u64) * PAGE_SIZE))
}
