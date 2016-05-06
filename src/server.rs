use std::collections::HashMap;
use std::io::Read;
use std::sync::{ Arc, Mutex };

extern crate hyper;
use self::hyper::Server as HttpServer;
use self::hyper::server::{ Request, Response };
use self::hyper::status::StatusCode;
use self::hyper::uri::RequestUri;

use types::*;

#[derive(Debug)]
pub enum SoapError {
    Http(hyper::error::Error),
}
              
impl From<hyper::error::Error> for SoapError {
    fn from(other: hyper::error::Error) -> SoapError {
        SoapError::Http(other)
    }
}

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

    pub fn register(&mut self, name: &str, call: RemoteCall) {
        let mut calls = self.calls.lock().unwrap();
        calls.insert(name.to_string(), call);
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
                    let mut buffer = String::new();
                    let     _      = request.read_to_string(&mut buffer);

                    println!("URI: {:?}", request.uri);
                    println!("Body: {:?}", buffer);

                    for (name, call) in calls.iter() {
                        println!("CALL: {}", name);
                    }
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

#[cfg(test)]
mod test {
    use super::Server;

    #[test]
    fn main() {
        let mut server = Server::new("0.0.0.0:1337");
        server.register("call", remote! {
            hello() -> Void {
                println!("Hello, macro2!");
            }
        });
        server.start();
    }
}
