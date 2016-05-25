use std::collections::HashMap;
use std::fmt::{ Debug, Error, Formatter };
use std::string::ToString;
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
    host_name: String,
    bind_addr: String,
    port:      u16,
    ssl:       bool,
    routes: Arc<Mutex<HashMap<
        String,
        Box<FnMut(Request) -> Response + Send + 'static>
    >>>,
}

impl Service {
    pub fn new(host: &str, port: u16) -> Service {
        Service::new_with_bind(host, "0.0.0.0", port)
    }

    pub fn new_with_bind(host: &str, bind: &str, port: u16) -> Service {
        Service {
            host_name: host.to_string(),
            bind_addr: bind.to_string(),
            port:      port,
            ssl:       false,
            routes:    Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn default() -> Service {
        Service::new("localhost", 80)
    }

    pub fn get_uri(&self) -> String {
        let mut uri = if self.ssl {
            String::from("https://")
        } else {
            String::from("http://")
        };

        uri.push_str(self.host_name.as_str());

        if self.port != 80 {
            uri.push_str(":");
            uri.push_str(self.port.to_string().as_str());
        }

        uri.push_str("/");
        uri
    }

    pub fn get_bind(&self) -> String {
        let mut bind = self.bind_addr.clone();
        bind.push_str(":");
        bind.push_str(self.port.to_string().as_str());
        bind
    }

    pub fn add_route<
        F: FnMut(Request) -> Response + Send + 'static
    >(&mut self, route: &str, handler: F)
    {
        let mut routes = self.routes.lock().unwrap();
        routes.insert(route.to_string(), Box::new(handler));
    }

    pub fn start(&mut self) -> Result<(), SoapError> {
        let http  = try!(HttpServer::http(self.get_bind().as_str()));
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
        write!(f, "Serving \"{}\", listenning on \"{:?}\".",
            self.get_uri(), self.get_bind())
    }
}
