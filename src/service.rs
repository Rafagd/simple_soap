use std::collections::HashMap;
use std::io::Read;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{ Arc, Mutex };

extern crate hyper;
use self::hyper::Server as HttpServer;
use self::hyper::server::Request as HttpRequest;
use self::hyper::server::Response as HttpResponse;
use self::hyper::status::StatusCode;
use self::hyper::uri::RequestUri;

use error::SoapError;

extern crate core;
use self::core::ops::FnMut;

#[derive(Default)]
pub struct Request {
    pub header:  HashMap<String, String>,
    pub content: String,
}

impl Request {
    pub fn new(header: HashMap<String, String>, content: String) -> Request {
        Request {
            header:  header,
            content: content,
        }
    }
}

impl<'a, 'b> From<HttpRequest<'a, 'b>> for Request {
    fn from(mut other: HttpRequest<'a, 'b>) -> Request {
        let mut request = Request::default();

        for h in other.headers.iter() {
            request.header.insert(h.name().to_string(), h.value_string());
        }

        request.content = String::new();
        let _ = other.read_to_string(&mut request.content);

        request
    }
}

#[derive(Default)]
pub struct Response {
    pub header:  HashMap<String, String>,
    pub content: String,
}

impl Response {
    pub fn new(header: HashMap<String, String>, content: String) -> Response {
        Response {
            header:  header,
            content: content,
        }
    }

    pub fn write_response(&self, mut response: HttpResponse) -> Result<(), SoapError> {
        for (key, value) in (&self.header).iter() {
            match key.as_str() {
                "status-code" => {
                    *response.status_mut() = StatusCode::Unregistered(value.parse().unwrap());
                },
                _ => {
                    response.headers_mut().set_raw(
                        key.clone(),
                        vec![value.clone().into_bytes()]
                    );
                }
            }
        }

        match response.send(self.content.as_bytes()) {
            Ok(_)  => Ok(()),
            Err(e) => Err(SoapError::Io(e)),
        }
    }
}

fn error404() -> Box<FnMut(Request) -> Response + Sized + Send + 'static> {
    Box::new(|_request: Request| -> Response {
        let mut response = Response::default();
        response.header.insert("status-code".to_string(), "404".to_string());
        response
    })
}

pub struct Service {
    pub listen_addr: SocketAddr,
    pub server_addr: SocketAddr,
        routes: Arc<Mutex<HashMap<
            String,
            Box<FnMut(Request) -> Response + Sized + Send + 'static>
        >>>,
}

impl Service {
    pub fn new() -> Service {
        Service {
            listen_addr: SocketAddr::from_str("0.0.0.0:80").unwrap(),
            server_addr: SocketAddr::from_str("127.0.0.1:80").unwrap(),
            routes:      Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_route<
        F: FnMut(Request) -> Response + Sized + Send + 'static
    >(&mut self, route: &str, handler: F)
    {
        let mut routes = self.routes.lock().unwrap();
        routes.insert(route.to_string(), Box::new(handler));
    }

    pub fn start(&mut self) -> Result<(), SoapError> {
        let http  = try!(HttpServer::http(self.listen_addr));
        let mutex = self.routes.clone();

        let _ = http.handle(move |request: HttpRequest, response: HttpResponse| {
            let route = {
                match request.uri {
                    RequestUri::AbsolutePath(ref path) => path.clone(),
                    _ => String::new(),
                }
            };

            let mut e404    = error404();
            let mut routes  = mutex.lock().unwrap();
            let mut handler = match routes.get_mut(route.as_str()) {
                Some(handler) => handler,
                None          => &mut e404,
            };

            let res = handler(From::from(request));
            let _   = res.write_response(response);
        });

        Ok(())
    }
}

