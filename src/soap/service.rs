use std::net::{ ToSocketAddrs, SocketAddr };
use std::str::FromStr;
use std::sync::{ Arc, Mutex };

extern crate sxd_document;
use self::sxd_document::parser as xml_parser;

use types::*;
use error::SoapError;
use service;
use soap::wsdl;

pub struct Service {
    service:   service::Service,
    namespace: String,
    wsdl:      Arc<Mutex<String>>,
    calls:     Arc<Mutex<Vec<RemoteCall>>>,
    changed:   bool,
}

impl Service {
    pub fn new<A: ToSocketAddrs>(address: A, namespace: &str) -> Service {
        let server = address.to_socket_addrs().unwrap().next().unwrap();
        let listen = SocketAddr::new(
            FromStr::from_str("0.0.0.0").unwrap(),
            server.port()
        );

        let mut service = service::Service::new();
        service.server_addr = server;
        service.listen_addr = listen;

        let soap_service = Service {
            service:   service,
            namespace: namespace.to_string(),
            wsdl:      Arc::new(Mutex::new(String::new())),
            calls:     Arc::new(Mutex::new(vec![])),
            changed:   false,
        };

        let buffer = {
            wsdl::from(namespace, &soap_service)
        };

        let     mutex = soap_service.wsdl.clone();
        let mut wsdl  = mutex.lock().unwrap();

        wsdl.clear();
        wsdl.push_str(buffer.as_str());

        soap_service
    }

    pub fn get_calls<'a>(&'a self) -> Arc<Mutex<Vec<RemoteCall>>> {
        self.calls.clone()
    }

    pub fn start(&mut self) -> Result<(), SoapError> {
        let wsdl_mutex = self.wsdl.clone();

        if self.changed {
            let buffer = {
                wsdl::from(self.namespace.as_str(), &self)
            };

            let mut wsdl_lock = wsdl_mutex.lock().unwrap();
            wsdl_lock.clear();
            wsdl_lock.push_str(buffer.as_str());
        }

        self.service.add_route("/", move |request| {
            let document = xml_parser::parse(request.content.as_str())
                .unwrap();

            println!("Request: {:?}", request);
            println!("Document: {:?}", document);
            /*
            let requests = Reader::from(request);
            let mut res  = vec![];

            for request in requests.iter() {
                let callback = {
                    let wsdl = mutex.lock().unwrap();
                    wsdl.get_callback_mut(request.method.as_str())
                };

                match callback {
                    Ok(callback) => {
                        for (name, arg) in callback.arguments.iter_mut() {
                            let req_arg = {
                                request.arguments
                                    .get(name.as_str())
                                    .unwrap()
                            };

                            arg.set_value(req_arg);
                        }

                        res.push((
                            callback.name.clone(),
                            callback.call().unwrap()
                        ));
                    },
                    Err(e) => {
                        println!("error {:?}", request.method);
                    }
                }
            }

            let mut response = service::Response::default();
            response.content = soap_response(res);
            response
            */
            service::Response::default()
        });

        self.service.add_route("/?wsdl", move |_| {
            let wsdl = wsdl_mutex.lock().unwrap();

            let mut response = service::Response::default();
            response.content = String::from_utf8_lossy(wsdl.as_bytes()).into_owned();
            response
        });

        self.service.start()
    }
}

fn soap_response(responses: Vec<(String, Data)>) -> String {
    let mut result = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");

    result.push_str("<SOAP-ENV:Envelope ");
    result.push_str("xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ");
    result.push_str("xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ");
    result.push_str("xmlns:SOAP-ENV=\"http://schemas.xmlsoap.org/soap/envelope/\">");
    result.push_str("<SOAP-ENV:Body>");

    for (name, res) in responses {
        result.push_str("<ns1:");
        result.push_str(name.as_str());
        result.push_str("Response>");

        match res.vl {
            Value::String(string) => {
                result.push_str("<return xsi:type=\"xsd:string\">");
                result.push_str(string.as_str());
                result.push_str("</return>");
            },
            _ => {
                result.push_str("<result xsi:type=\"xsd:string\"/>");
            },
        }

        result.push_str("</ns1:");
        result.push_str(name.as_str());
        result.push_str("Response>");
    }

    result.push_str("</SOAP-ENV:Body>");
    result.push_str("</SOAP-ENV:Envelope>");

    result
}
