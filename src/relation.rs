use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use std::collections::LinkedList;

use choice_vec::*;
use page::{Page, PAGE_SIZE, NO_OVFLOW};
use tuple::Tuple;
use util::*;

pub use self::OpenMode::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    Reading,
    Writing
}

pub struct Relation {
    pub num_attrs: u64,
    pub depth: u64,
    pub split_pointer: u64,
    /// Number of main data pages (overflow pages not counted).
    pub num_pages: u64,
    pub num_tuples: u64,
    pub choice_vec: ChoiceVec,
    pub mode: OpenMode,
    pub info_file: File,
    pub data_file: File,
    pub ovflow_file: File
}

fn file_name(name: &str, extension: &str) -> String {
    format!("{}.{}", name, extension)
}

fn info_file_name(name: &str) -> String { file_name(name, "info") }
fn data_file_name(name: &str) -> String { file_name(name, "data") }
fn ovflow_file_name(name: &str) -> String { file_name(name, "ovflow") }

impl OpenMode {
    fn open_options(self) -> OpenOptions {
        let mut o = OpenOptions::new();
        o.read(true).write(self == Writing);
        o
    }
}

impl Relation {
    /// Create a new relation on disk.
    pub fn new(name: &str, num_attrs: u64, num_pages: u64, depth: u64, choice_vec: ChoiceVec)
    -> io::Result<()>
    {
        if Relation::exists(name) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Relation already exists: {}", name)
            ));
        }
        // Create new relation struct and associated files.
        let r = Relation {
            num_attrs: num_attrs,
            depth: depth,
            split_pointer: 0,
            num_pages: num_pages,
            num_tuples: 0,
            choice_vec: choice_vec,
            mode: Writing,
            info_file: try!(File::create(info_file_name(name))),
            data_file: try!(File::create(data_file_name(name))),
            ovflow_file: try!(File::create(ovflow_file_name(name)))
        };

        // Write initial empty pages.
        for _ in 0..num_pages {
            let page = try!(Page::new(&r.data_file));
            try!(page.close());
        }

        // Write metadata.
        r.close()
    }

    /// Open an existing relation for reading or writing.
    pub fn open(name: &str, mode: OpenMode) -> io::Result<Relation> {
        let open_opts = mode.open_options();
        let mut info_file = try!(open_opts.open(info_file_name(name)));
        let num_attrs = try!(read_u64(&mut info_file));
        let depth = try!(read_u64(&info_file));
        let split_pointer = try!(read_u64(&info_file));
        let num_pages = try!(read_u64(&info_file));
        let num_tuples = try!(read_u64(&info_file));
        let choice_vec = try!(ChoiceVec::read(&info_file));

        Ok(Relation {
            num_attrs: num_attrs,
            depth: depth,
            split_pointer: split_pointer,
            num_pages: num_pages,
            num_tuples: num_tuples,
            choice_vec: choice_vec,
            mode: mode,
            info_file: info_file,
            data_file: try!(open_opts.open(data_file_name(name))),
            ovflow_file: try!(open_opts.open(ovflow_file_name(name)))
        })
    }

    pub fn exists(name: &str) -> bool {
        Path::new(&info_file_name(name)).is_file()
    }

    fn resize_threshold(&self) -> u64 {
        (PAGE_SIZE as u64 / (10 * self.num_attrs)) * self.num_pages
    }

    /// Insert a tuple into the relation.
    pub fn insert(&mut self, t: Tuple) -> io::Result<()> {
        // Expand whenever the resize threshold is hit.
        if self.num_tuples == self.resize_threshold() {
            println!("Resizing");
            try!(self.grow());
        }

        let tuple_hash = t.hash(&self.choice_vec);

        let mut page_id = lower_bits(self.depth as u8, tuple_hash);

        // If the d-bit hash is less than the split-pointer, then we have to use
        // d + 1 bits of hash.
        if page_id < (self.split_pointer as u32) {
            page_id = lower_bits(self.depth as u8 + 1, tuple_hash);
        }

        let mut page = try!(Page::read(&self.data_file, page_id as u64));

        let serialised_tuple = t.serialise();

        // If the tuple fits in the main data page, add it and write out.
        if page.add_tuple(&serialised_tuple) {
            try!(page.write());
        }
        // Otherwise, add it to the overflow chain.
        else {
            try!(page.add_to_overflow(&self.ovflow_file, &serialised_tuple));
        }

        self.num_tuples += 1;

        Ok(())
    }

    /// Helper function for growing a relation.
    /// Store `tuple` into `storage_page` if it will fit.
    fn store_tuple_grow<'a>(
        tuple: &[u8],
        storage_page: &mut Page<'a>,
        next_page_id: &mut u64,
        ovflow_file: &'a File,
        tuple_cache: &mut LinkedList<Tuple>,
        spare_pages: &mut LinkedList<u64>
    ) -> io::Result<()>
    {
        // If the tuple fits in the page, store it.
        if storage_page.add_tuple(tuple) {
            return Ok(());
        }
        // Otherwise, we look for an overflow page.
        // If there's a spare one already hanging around, use it.
        println!("  looking for an overflow page");
        let ovflow_page_id = if let Some(spare_page) = spare_pages.pop_front() {
            println!("  using existing spare page: {}", spare_page);
            spare_page
        }
        // If there's another page to be loaded, use that.
        else if *next_page_id != NO_OVFLOW {
            println!("  loading the next page and using that: {}", *next_page_id);
            try!(Relation::load_next_page(next_page_id, ovflow_file, tuple_cache, spare_pages));
            spare_pages.pop_front().expect("Load next page didn't work")
        }
        // Impossible case, as (k_i + 1) <= (k_i + 2) <= (k_i + 2)
        // where k_i is the # of initial overflow pages and k_o is the # of overflow pages
        // after the split.
        else {
            unreachable!("There are pages around, you're just not looking hard enough.")
        };

        // Close the old storage page.
        storage_page.ovflow = ovflow_page_id;
        try!(storage_page.write());

        // Open the new one, and do the insert.
        let mut new_storage_page = Page::empty(ovflow_file, ovflow_page_id);
        assert!(new_storage_page.add_tuple(tuple));
        *storage_page = new_storage_page;

        Ok(())
    }

    /// Helper function for grow.
    fn load_next_page(
        next_page_id: &mut u64,
        ovflow_file: &File,
        tuple_cache: &mut LinkedList<Tuple>,
        spare_pages: &mut LinkedList<u64>
    ) -> io::Result<()>
    {
        assert!(*next_page_id != NO_OVFLOW);
        let next_page = try!(Page::read(ovflow_file, *next_page_id));
        tuple_cache.append(&mut next_page.get_tuple_list());

        // Having loaded the tuples, add the page to the list of spare pages.
        spare_pages.push_back(*next_page_id);
        // Finally, set the overflow page of the new page as the next new page.
        *next_page_id = next_page.ovflow;
        Ok(())
    }

    /// Grow the number of main data pages in the relation.
    pub fn grow(&mut self) -> io::Result<()> {
        let d = self.depth as u8;
        let sp = self.split_pointer;

        // Current low numbered page, initialised to a fresh new page in the old position.
        let mut low_page = Page::empty(&self.data_file, sp);
        // Current high numbered page, initialised to a new page at the end of the file.
        let mut high_page = try!(Page::new(&self.data_file));

        println!("Splitting page {:b} into {:b} and {:b}", sp, sp, high_page.id);

        // First old data page.
        let old_low_page = try!(Page::read(&self.data_file, sp));

        // List of spare overflow page IDs that can be claimed for extra storage.
        let mut spare_pages = LinkedList::new();

        // Cache of tuples to be redistributed.
        let mut tuple_cache = old_low_page.get_tuple_list();

        // Page ID of the next page to redistribute.
        let mut next_page_id = old_low_page.ovflow;

        drop(old_low_page);

        loop {
            println!("Iteration start.");
            println!("  next_page_id = {}", next_page_id);
            println!("  size tuple_cache = {}", tuple_cache.len());
            println!("  spare_pages = {:?}", spare_pages);
            // If there is a tuple in the cache, redistribute it.
            if let Some(tuple) = tuple_cache.pop_front() {
                let full_hash = tuple.hash(&self.choice_vec);
                println!("  full tuple hash = {:b}", full_hash);
                let hash = lower_bits(d + 1, full_hash);
                println!("  lower_bits({}, hash) = {:b}", d + 1, hash);
                let s_tuple = tuple.serialise();

                // Store on the left if the hash bits still match the split pointer.
                let storage_page = if hash == sp as u32 {
                    println!("  Storing tuple {:?} with hash {:b} on the LEFT", tuple.values, hash);
                    &mut low_page
                } else {
                    println!("  Storing tuple {:?} with hash {:b} on the RIGHT", tuple.values, hash);
                    &mut high_page
                };
                try!(Relation::store_tuple_grow(
                    &s_tuple, storage_page, &mut next_page_id,
                    &self.ovflow_file, &mut tuple_cache, &mut spare_pages
                ));
            }
            // Otherwise if the cache is exhausted and there are no further pages, we're done.
            else if next_page_id == NO_OVFLOW {
                break;
            }
            // If the cache is exhausted, but there are more pages to read, load one.
            else {
                println!("  tuple cache exhausted, loading the next file");
                try!(Relation::load_next_page(
                    &mut next_page_id, &self.ovflow_file,
                    &mut tuple_cache, &mut spare_pages)
                );
            }
        }

        try!(low_page.close());
        try!(high_page.close());

        self.num_pages += 1;
        // If the split pointer has hit the cross-over point, reset it to 0.
        // sp = 2^d - 1.
        if sp == ((1 << d) - 1) {
            self.split_pointer = 0;
            self.depth += 1;
        } else {
            self.split_pointer += 1;
        }
        Ok(())
    }

    pub fn write_info_file(&mut self) -> io::Result<()> {
        let mut f = &self.info_file;
        try!(f.seek(SeekFrom::Start(0)));
        try!(write_u64(f, self.num_attrs));
        try!(write_u64(f, self.depth));
        try!(write_u64(f, self.split_pointer));
        try!(write_u64(f, self.num_pages));
        try!(write_u64(f, self.num_tuples));
        try!(self.choice_vec.write(f));
        Ok(())
    }

    pub fn close(mut self) -> io::Result<()> {
        try!(self.write_info_file());
        Ok(())
    }
}
