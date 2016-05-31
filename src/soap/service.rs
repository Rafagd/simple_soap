use std::collections::HashMap;
use std::ops::DerefMut;
use std::string::ToString;
use std::sync::{ Arc, Mutex };

extern crate sxd_document;
use self::sxd_document::parser as xml_parser;

use error::SoapError;
use service;
use soap::{ Operation, Options, Request };
use soap::wsdl::Wsdl;

pub struct Service {
        service:    service::Service,
    pub options:    Options,
    pub operations: Arc<Mutex<HashMap<String, Operation>>>,
}

impl Service {
    pub fn new(host_name: &str, port: u16) -> Service {
        Service::new_with_opts(host_name, port, Options::default())
    }

    pub fn new_with_opts(host_name: &str, port: u16, opts: Options)
        -> Service
    {
        let service = if opts.bind_addr == "" {
            service::Service::new(host_name, port)
        } else {
            service::Service::new_with_bind(host_name, 
                opts.bind_addr.as_str(), port)
        };
            
        Service {
            service:    service,
            options:    opts,
            operations: Arc::new(Mutex::new(hashmap!{})),
        }
    }

    pub fn add_operation(&mut self, operation: Operation) {
        let mut operations = self.operations.lock().unwrap();
        operations.insert(operation.name.clone(), operation);
    }

    pub fn get_uri(&self) -> String {
        self.service.get_uri()
    }

    pub fn start(&mut self) -> Result<(), SoapError> {
        let wsdl       = Wsdl::from(&self).to_string();
        let operations = self.operations.clone();

        let mut not_found = Operation::not_found();

        self.service.add_route("/", move |request| {
            let req = Request::from(request);

            let mut lock = operations.lock().unwrap();
            let     find = lock.get_mut(req.operation.as_str());

            let mut operation = match find {
                Some(op) => op,
                None => &mut not_found,
            };

            let fun = operation.closure.deref_mut();
            let res = fun(req);

            let mut response = service::Response::default();
            response.content = res.to_xml_string();
            response
        });

        self.service.add_route("/?wsdl", move |_| {
            let mut response = service::Response::default();
            response.content = String::from_utf8_lossy(wsdl.as_bytes()).into_owned();
            response
        });

        self.service.start()
    }
}
