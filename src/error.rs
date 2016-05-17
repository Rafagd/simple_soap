use std::{ io, str };

extern crate hyper;

#[derive(Debug)]
pub enum SoapError {
    NotFound,
    Http(hyper::error::Error),
    Io(io::Error),
    Encoding(str::Utf8Error),
    Malformed(String),
    Unexpected(String),
}
              
impl From<hyper::error::Error> for SoapError {
    fn from(other: hyper::error::Error) -> SoapError {
        SoapError::Http(other)
    }
}

