use std::collections::HashMap;

use soap::{ Fault, Part, Request, Response };

pub struct Operation {
    pub doc:     String,
    pub name:    String,
    pub inputs:  HashMap<String, Part>,
    pub outputs: HashMap<String, Part>,
    pub closure: Box<FnMut(Request) -> Response>
}

impl Operation {
    pub fn not_found() -> Operation {
        Operation {
            doc:     String::from("Handler for unknown operations."),
            name:    String::from("not_found"),
            inputs:  hashmap!{},
            outputs: hashmap!{},
            closure: Box::new(|request| {
                let reason = format!(
                    "Operation \"{}\" is not defined in the WSDL for this service",
                    request.operation.as_str()
                );

                let mut response = Response::new();
                response.fault(Fault::client(reason.as_str(), "", ""));
                response
            }),
        }
    }
}

unsafe impl Send for Operation {}
