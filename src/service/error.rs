use service::Request;
use service::Response;

pub fn error_handler(code: u16)
    -> Box<FnMut(Request) -> Response + Send + 'static>
{
    Box::new(move |_request: Request| -> Response {
        let mut response = Response::default();
        response.header.insert("status-code".to_string(), code.to_string());
        response
    })
}
