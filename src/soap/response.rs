use std::collections::HashMap;

extern crate sxd_document;
use self::sxd_document::Package;
use self::sxd_document::writer::format_document;

use soap::Part;

pub struct Response {
    operation: String,
    responses: HashMap<String, Part>,
}

impl Response {
    pub fn from(operation: &str, responses: HashMap<String, Part>) -> Response {
        Response {
            operation: operation.to_string(),
            responses: responses,
        }
    }

    pub fn not_found() -> String {
        String::new()
    }

    pub fn to_xml_string(&self) -> String {
        let package  = Package::new();
        let document = package.as_document();

        let envelope = document.create_element("SOAP-ENV:Envelope");
        envelope.set_attribute_value("xmlns:xsd", "http://www.w3.org/2001/XMLSchema");
        envelope.set_attribute_value("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance");
        envelope.set_attribute_value("xmlns:SOAP-ENV", "http://schemas.xmlsoap.org/soap/envelope/");

        let body = document.create_element("SOAP-ENV:Body");

        let res_name = String::from("ns1:");
        res_name.push_str(self.operation.as_str());
        res_name.push_str("Response");

        let res = document.create_element(res_name.as_str());

        for (name, part) in self.responses {
            let ret = document.create_element("return");
            ret.set_attribute_value("xsi:type", part.xsd_type().as_str());

            let content = match part {
                Part::String(string) => string,
                _ => String::new(),
            };

            if content != "" {
                ret.append_child(document.create_text(content.as_str()));
            }
            
            res.append_child(ret);
        }

        body.append_child(res);
        envelope.append_child(body);
        document.root().append_child(envelope);

        let mut buffer = vec![];
        format_document(&document, &mut buffer).ok()
            .expect("Error while formatting SOAP XML");
                    
        String::from_utf8(buffer).unwrap()
    }
}

