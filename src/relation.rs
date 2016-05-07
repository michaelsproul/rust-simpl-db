use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{self, Seek, SeekFrom};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use choice_vec::*;
use page::Page;
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
        let mut r = Relation {
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
        for i in 0..num_pages {
            try!(Page::empty().write(&mut r.data_file, i));
        }

        // Write metadata.
        r.close()
    }

    /// Open an existing relation for reading or writing.
    pub fn open(name: &str, mode: OpenMode) -> io::Result<Relation> {
        let open_opts = mode.open_options();
        let mut info_file = try!(open_opts.open(info_file_name(name)));
        let r = |f: &mut File| f.read_u64::<BigEndian>();
        let num_attrs = try!(r(&mut info_file));
        let depth = try!(r(&mut info_file));
        let split_pointer = try!(r(&mut info_file));
        let num_pages = try!(r(&mut info_file));
        let num_tuples = try!(r(&mut info_file));

        Ok(Relation {
            num_attrs: num_attrs,
            depth: depth,
            split_pointer: split_pointer,
            num_pages: num_pages,
            num_tuples: num_tuples,
            // FIXME
            choice_vec: ChoiceVec::new(vec![], num_attrs),
            mode: mode,
            info_file: info_file,
            data_file: try!(open_opts.open(data_file_name(name))),
            ovflow_file: try!(open_opts.open(ovflow_file_name(name)))
        })
    }

    pub fn exists(name: &str) -> bool {
        Path::new(&info_file_name(name)).is_file()
    }

    /// Insert a tuple into the relation.
    pub fn insert(&mut self, t: Tuple) -> io::Result<()> {
        let tuple_hash = t.hash(&self.choice_vec);

        let mut page_id = lower_bits(self.depth as u8, tuple_hash);

        // If the d-bit hash is less than the split-pointer, then we have to use
        // d + 1 bits of hash.
        if page_id < (self.split_pointer as u32) {
            page_id = lower_bits(self.depth as u8 + 1, tuple_hash);
        }

        let mut page = try!(Page::read(&mut self.data_file, page_id as u64));

        let serialised_tuple = t.serialise();
        if page.try_add_tuple(&serialised_tuple) {
            try!(page.write(&mut self.data_file, page_id as u64));
        }

        Ok(())
    }

    pub fn write_info_file(&mut self) -> io::Result<()> {
        let f = &mut self.info_file;
        let w = |f: &mut File, x: u64| f.write_u64::<BigEndian>(x);
        try!(f.seek(SeekFrom::Start(0)));
        try!(w(f, self.num_attrs));
        try!(w(f, self.depth));
        try!(w(f, self.split_pointer));
        try!(w(f, self.num_pages));
        w(f, self.num_tuples)
    }

    pub fn close(mut self) -> io::Result<()> {
        try!(self.write_info_file());
        Ok(())
    }
}
