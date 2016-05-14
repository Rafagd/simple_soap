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
                    let soap = Reader::from(request);
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
