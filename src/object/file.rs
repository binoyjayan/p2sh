use std::cell::RefCell;
use std::fmt;
use std::fs;
use std::io;

#[derive(Debug)]
pub enum FileHandle {
    Reader(RefCell<io::BufReader<fs::File>>),
    Writer(RefCell<io::BufWriter<fs::File>>),
    Stdin,
    Stdout,
    Stderr,
}

impl FileHandle {
    pub fn new_reader(reader: io::BufReader<fs::File>) -> Self {
        Self::Reader(RefCell::new(reader))
    }
    pub fn new_writer(writer: io::BufWriter<fs::File>) -> Self {
        Self::Writer(RefCell::new(writer))
    }
}

impl fmt::Display for FileHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Reader(_) => write!(f, "<file: reader>"),
            Self::Writer(_) => write!(f, "<file: writer>"),
            Self::Stdin => write!(f, "<stdin>"),
            Self::Stdout => write!(f, "<stdout>"),
            Self::Stderr => write!(f, "<stderr>"),
        }
    }
}
