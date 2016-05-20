#[macro_use]
extern crate macros;

#[macro_use]
pub mod types;
pub mod error;
pub mod reader;
pub mod soap;
pub mod wsdl;
pub mod service;

mod xml;
