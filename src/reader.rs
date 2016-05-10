use std::fs::File;
use std::io::{ BufRead, BufReader, Read };
use std::path::Path;

extern crate quick_xml;
use self::quick_xml::{ XmlReader };
use self::quick_xml::namespace::XmlnsReader;

use error::SoapError;

pub struct Reader;

impl Reader {
    pub fn from<R: Read>(buffer: R) -> Reader {
        let buffer = BufReader::new(buffer);
        let xml    = XmlReader::from_reader(buffer);

        Reader
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Reader, SoapError> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => { return Err(SoapError::Io(err)) },
        };

        Ok(Reader::from(file))
    }
}

