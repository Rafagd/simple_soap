use std::collections::HashMap;

use service::Request as ServiceRequest;

extern crate sxd_document;
use self::sxd_document::dom::Document;

use soap::Part;

pub struct Request {
    pub operation: String,
    pub arguments: HashMap<String, Part>,
}

impl Request {
    pub fn from(request: ServiceRequest) -> Request {
        Request {
            operation: String::new(),
            arguments: hashmap!{},
        }
    }
}

