use std::collections::HashMap;
use std::io::Read;
use std::sync::{ Arc, Mutex };

extern crate hyper;
use self::hyper::Server as HttpServer;
use self::hyper::server::{ Request, Response };
use self::hyper::status::StatusCode;
use self::hyper::uri::RequestUri;

use types::*;
use reader::Reader;
use error::SoapError;
use wsdl::Wsdl;

pub struct Server {
    address: String,
    wsdl:    Arc<Mutex<Wsdl>>,
}

impl Server {
    pub fn new(address: &str, wsdl: Wsdl) -> Server {
        Server {
            address: address.to_string(),
            wsdl:    Arc::new(Mutex::new(wsdl)),
        }
    }

    pub fn start(&self) -> Result<(), SoapError> {
        let http  = try!(HttpServer::http(self.address.as_str()));
        let mutex = self.wsdl.clone();

        http.handle(move |ref mut request: Request, mut response: Response| {
            let path = {
                match request.uri {
                    RequestUri::AbsolutePath(ref path) => path.clone(),
                    _ => String::new(),
                }
            };

            let mut wsdl = mutex.lock().unwrap();

            match path.as_str() {
                "/" => {
                    let requests = Reader::from(request);
                    let mut res  = vec![];

                    for request in requests.iter() {
                        let callback = {
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

                    let _ = response.send(soap_response(res).as_bytes());
                },
                "/?wsdl" => {
                    let _ = response.send(wsdl.as_bytes());
                },
                _ => {
                    *response.status_mut() = StatusCode::NotFound;
                }
            }
        });

        Ok(())
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
