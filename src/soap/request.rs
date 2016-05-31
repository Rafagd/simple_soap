use std::collections::HashMap;

extern crate sxd_document;
use self::sxd_document::dom::Document;

use soap::Part;

pub struct Request {
    pub operation: String,
    pub arguments: HashMap<String, Part>,
}

impl Request {
    fn from(document: Document) -> Request {
        Request {
            operation: String::new(),
            arguments: hashmap!{},
        }
    }
}

