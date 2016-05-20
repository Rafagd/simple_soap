use std::collections::HashMap;
use std::io::Read;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::sync::{ Arc, Mutex };

use types::*;
use reader::Reader;
use error::SoapError;
use service;
use wsdl::Wsdl;

pub struct Service {
    service: service::Service,
    wsdl:    Arc<Mutex<Wsdl>>,
}

impl Service {
    pub fn new<A: ToSocketAddrs>(address: A, wsdl: Wsdl) -> Service {
        let     server = address.to_socket_addrs().unwrap().next().unwrap();
        let mut listen = server.clone();
        listen.set_ip(FromStr::from_str("0.0.0.0").unwrap());

        let mut service = service::Service::new();
        service.server_addr = server;
        service.listen_addr = listen;

        Service {
            service: service::Service::new(),
            wsdl:    Arc::new(Mutex::new(wsdl)),
        }
    }

    pub fn start(&self) -> Result<(), SoapError> {
        let mutex = self.wsdl.clone();

        self.service.add_route("/", |request: service::Request|
            -> service::Response
        {
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
        });

        self.service.add_route("/?wsdl", |request: service::Request|
            -> service::Response
        {
            let wsdl = mutex.lock().unwrap();

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
