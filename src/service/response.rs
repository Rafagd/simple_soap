use std::collections::HashMap;

extern crate hyper;
use self::hyper::server::Response as HttpResponse;
use self::hyper::status::StatusCode;

use error::*;

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

