use std::collections::HashMap;

use service::Request as ServiceRequest;

extern crate sxd_document;
use self::sxd_document::parser;

use soap::Part;

macro_rules! next_tag(
    ($node:ident, $tag:expr) => {{
        let mut e = None;

        for child in $node.children() {
            if let Some(elem) = child.element() {
                if elem.name().local_part() == $tag {
                    e = Some(elem);
                    break;
                }
            }
        }

        e
    }};
);

#[derive(Debug)]
pub struct Request {
    pub operation: String,
    pub arguments: HashMap<String, Part>,
}

impl From<ServiceRequest> for Request {
    fn from(request: ServiceRequest) -> Request {
        let package = parser::parse(request.content.as_str())
            .expect("Failed to parse XML");
        
        let document = package.as_document();
        let root     = document.root();
        let envelope = next_tag!(root, "Envelope").unwrap();
        let body     = next_tag!(envelope, "Body").unwrap();
        
        let mut req = Request {
            operation: String::new(),
            arguments: hashmap!{},
        };

        let operation = {
            let mut e = None;

            for child in body.children() {
                if let Some(elem) = child.element() {
                    e = Some(elem);
                    break;
                }
            }

            e
        }.unwrap();

        req.operation = operation.name().local_part().to_string();
        
        for arg in operation.children().iter() {
            if let Some(elem) = arg.element() {
                req.arguments.insert(
                    elem.name().local_part().to_string(),
                    {
                        let mut e = None;

                        for child in elem.children() {
                            if let Some(elem) = child.text() {
                                e = Some(elem)
                            }
                        }

                        Part::String(e.unwrap().text().to_string())
                    }
                );
            }
        }

        req
    }
}

