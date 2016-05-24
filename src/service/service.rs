use std::collections::HashMap;
use std::fmt::{ Debug, Error, Formatter };
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{ Arc, Mutex };

extern crate hyper;
use self::hyper::server::{ 
    Request as HttpRequest,
    Response as HttpResponse,
    Server as HttpServer,
};
use self::hyper::uri::RequestUri;

use error::*;
use service::error::*;
use service::{ Request, Response };

pub struct Service {
    pub listen_addr: SocketAddr,
    pub server_addr: SocketAddr,
        routes: Arc<Mutex<HashMap<
            String,
            Box<FnMut(Request) -> Response + Send + 'static>
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
        F: FnMut(Request) -> Response + Send + 'static
    >(&mut self, route: &str, handler: F)
    {
        let mut routes = self.routes.lock().unwrap();
        routes.insert(route.to_string(), Box::new(handler));
    }

    pub fn start(&mut self) -> Result<(), SoapError> {
        let http  = try!(HttpServer::http(self.listen_addr));
        let mutex = self.routes.clone();

        let handle = http.handle(
            move |request: HttpRequest, response: HttpResponse| {
                let route = {
                    match request.uri {
                        RequestUri::AbsolutePath(ref path) => path.clone(),
                        _ => String::new(),
                    }
                };

                let mut e404    = error_handler(404);
                let mut routes  = mutex.lock().unwrap();
                let mut handler = match routes.get_mut(route.as_str()) {
                    Some(handler) => handler,
                    None          => &mut e404,
                };

                let res = handler(From::from(request));
                let _   = res.write_response(response);
            }
        );

        match handle {
            Ok(_)  => Ok(()),
            Err(e) => Err(From::from(e)),
        }
    }
}

impl Debug for Service {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Serving \"{}\", listenning on \"{}\".",
            self.server_addr, self.listen_addr)
    }
}
