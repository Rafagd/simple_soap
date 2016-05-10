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

pub struct Server {
    address: String,
    calls:   Arc<Mutex<HashMap<String, RemoteCall>>>,
}

impl Server {
    pub fn new(address: &str) -> Server {
        Server {
            address: address.to_string(),
            calls:   Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&mut self, call: RemoteCall) {
        let mut calls = self.calls.lock().unwrap();
        calls.insert(call.name.clone(), call);
    }

    pub fn start(&self) -> Result<(), SoapError> {
        let http  = try!(HttpServer::http(self.address.as_str()));
        let mutex = self.calls.clone();

        http.handle(move |ref mut request: Request, mut response: Response| {
            let path = {
                match request.uri {
                    RequestUri::AbsolutePath(ref path) => path.clone(),
                    _ => String::new(),
                }
            };

            let calls = mutex.lock().unwrap();

            match path.as_str() {
                "/" => {
                    let soap = Reader::from(request);
                },
                "/?wsdl" => {
                    let _ = response.send(show_wsdl(&calls).as_bytes());
                },
                _ => {
                    *response.status_mut() = StatusCode::NotFound;
                }
            }
        });

        Ok(())
    }
}

fn show_wsdl(calls: &HashMap<String, RemoteCall>) -> String {
    let mut result = String::new();

    for (name, call) in calls {
        result = result + format!("{:?}\n", name).as_str();
    }

    result
}

