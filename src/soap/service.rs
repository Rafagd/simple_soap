use std::collections::HashMap;
use std::string::ToString;
use std::sync::{ Arc, Mutex };

extern crate sxd_document;
use self::sxd_document::parser as xml_parser;

use types::*;
use error::SoapError;
use service;
use soap::wsdl::Wsdl;

pub enum Part {
    // String-derived
    Id(String),
    IdRef(String),
    Language(String),
    Name(String),
    NmToken(String),
    NormalizedString(String),
    String(String),
    Token(String),

    // Date-derived
    Date(String),
    Time(String),
    DateTime(String),
    Duration(String),

    // Numeric types
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    UnsignedByte(u8),
    UnsignedShort(u16),
    UnsignedInt(u32),
    UnsignedLong(u64),
    Decimal(f64),
    Integer(i64),
    NegativeInteger(u64),    // -1, -2, -3...
    PositiveInteger(u64),    // 1, 2, 3...
    NonNegativeInteger(u64), // 0, 1, 2, 3...
    NonPositiveInteger(u64), // 0, -1, -2, -3...

    // Misc.
    Boolean(bool),
    Base64Binary(String),
    HexBinary(String),
    AnyUri(String),

    // Attrs, Content.
    ComplexType(HashMap<String, Part>, HashMap<String, Part>),
}

impl Part {
    pub fn xsd_type(&self) -> String {
        match self {
            &Part::String(_) => "xsd:string",
            _ => "xsd:string",
        }.to_string()
    }
}

pub struct Operation {
    pub doc:     String,
    pub name:    String,
    pub inputs:  HashMap<String, Part>,
    pub outputs: HashMap<String, Part>,
    pub closure: Box<FnMut(HashMap<String, Part>) -> HashMap<String, Part>>,
}

pub struct Options {
    pub bind_addr:    String,
    pub namespace:    String,
    pub service_name: String,
}

impl Options {
    pub fn default() -> Options {
        Options {
            bind_addr:    String::from(""),
            namespace:    String::from("server"),
            service_name: String::from("Service"),
        }
    }
}

pub struct Service {
        service:    service::Service,
    pub options:    Options,
    pub operations: Arc<Mutex<Vec<Operation>>>,
        wsdl:       String,
        changed:    bool,
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
            
        let mut soap_service = Service {
            service:    service,
            options:    opts,
            operations: Arc::new(Mutex::new(vec![])),
            wsdl:       String::new(),
            changed:    false,
        };

        soap_service.wsdl = Wsdl::from(&soap_service).to_string();
        soap_service
    }

    pub fn add_operation(&mut self, operation: Operation) {
        let mut operations = self.operations.lock().unwrap();
        operations.push(operation);

        self.changed = true;
    }

    pub fn get_uri(&self) -> String {
        self.service.get_uri()
    }

    pub fn start(&mut self) -> Result<(), SoapError> {
        let wsdl = self.wsdl.clone();

        if self.changed {
            self.wsdl = Wsdl::from(&self).to_string();
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
