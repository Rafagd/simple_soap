

extern crate sxd_document;
use self::sxd_document::Package;
use self::sxd_document::dom::{ Document, Element };
use self::sxd_document::writer::format_document;

use types::*;
use soap::Service;

pub fn from(namespace: &str, service: &Service) -> String {
    let     package  = Package::new();
    let mut document = package.as_document();
    let mut root     = document.root();

    root.append_child(
        definitions(&document, &service, namespace)
    );

    let mut buffer = Vec::new();
    format_document(&document, &mut buffer);

    println!("{:?}", String::from_utf8(buffer.clone()));

    String::from_utf8(buffer).unwrap()
}

fn definitions<'a, 'b>(document: &'a Document, service: &'b Service, namespace: &str) -> Element<'a> {
    let (server_ns, service_ns) = {
        let string_ns  = namespace.to_string();
        let mut split  = string_ns.split(".");
        let server_ns  = split.next().unwrap().to_string();
        let service_ns = split.next().unwrap().to_string();
        (server_ns, service_ns)
    };

    let definitions = document.create_element("definitions");

    definitions.set_default_namespace_uri(Some("http://schemas.xmlsoap.org/wsdl/"));

    definitions.set_attribute_value("xmlns:soap",      "http://schemas.xmlsoap.org/wsdl/soap/");
    definitions.set_attribute_value("xmlns:xsd",       "http://www.w3.org/2001/XMLSchema");
    definitions.set_attribute_value("xmlns:xsi",       "http://www.w3.org/2001/XMLSchema-instance");
    definitions.set_attribute_value("xmlns:SOAP-ENV",  "http://schemas.xmlsoap.org/soap/envelope/");
    definitions.set_attribute_value("xmlns:SOAP-ENC",  "http://schemas.xmlsoap.org/soap/encoding/");
    definitions.set_attribute_value("xmlns:tns",       namespace);
    definitions.set_attribute_value("targetNamespace", namespace);

    definitions.append_child(
        schema_import(document, namespace, vec![
            "http://schemas.xmlsoap.org/soap/encoding/",
            "http://schemas.xmlsoap.org/wsdl/",
        ])
    );

    let mutex = service.get_calls();
    let calls = mutex.lock().unwrap();

    for call in calls.iter() {
        definitions.append_child(
            register_arguments(document, &call)
        );

        definitions.append_child(
            register_return(document, &call)
        );
    }

    definitions
}

fn schema_import<'a>(document: &'a Document, namespace: &str, schemas: Vec<&str>) -> Element<'a> {
    let types  = document.create_element("types");

    let schema = document.create_element("xsd:schema");
    schema.set_attribute_value("targetNamespace", namespace);

    for sch in schemas.iter() {
        let import = document.create_element("xsd:import");
        import.set_attribute_value("namespace", sch);

        schema.append_child(import);
    }

    types.append_child(schema);

    types
}

fn register_arguments<'a, 'b>(document: &'a Document, call: &'b RemoteCall) -> Element<'a> {
    let mut call_name = call.name.clone();
    call_name.push_str("Request");

    let mut message = document.create_element("message");
    message.set_attribute_value("name", call_name.as_str());
    
    for (name, arg) in call.arguments.iter() {
        let mut part = document.create_element("part");
        part.set_attribute_value("name", name);
        part.set_attribute_value("type", type_to_xsd(&arg.vl).as_str());

        message.append_child(part);
    }

    message
}

fn register_return<'a, 'b>(document: &'a Document, call: &'b RemoteCall) -> Element<'a> {
    let mut call_name = call.name.clone();
    call_name.push_str("Response");

    let mut message = document.create_element("message");
    message.set_attribute_value("name", call_name.as_str());
    
    let mut part = document.create_element("part");
    part.set_attribute_value("name", "result");
    part.set_attribute_value("type", type_to_xsd(&call.result.vl).as_str());

    message.append_child(part);
    message
}

fn type_to_xsd(value: &Value) -> String {
    match value {
        &Value::String(ref s) => s.clone(),
        // Error?
        _ => String::new(),
    }
}
        /*
        if self.changed {
            let mut document = Document::new("UTF-8");

            let mut tns = String::from("urn:");
            tns.push_str(self.namespace.as_str());
            tns.push_str(".");
            tns.push_str(self.target_namespace.as_str());

            let mut definitions = Element::new_node("definitions",
                vec![
                    ("xmlns",           "http://schemas.xmlsoap.org/wsdl/"),
                ],
                {

                    children.push(Element::new_node("portType",
                        vec![("name", "Registros do WSDLPortType")],
                        {
                            let mut children = vec![];
                            
                            for call in self.calls.iter() { 
                                children.push(Element::new_node("operation",
                                    vec![("name", call.name.as_str())],
                                    vec![
                                        Element::new_node("documentation", vec![], vec![
                                            Element::new_text(call.doc.as_str())
                                        ]),
                                        Element::new_node("input", vec![
                                            ("message", {
                                                let mut s = String::from("tns:");
                                                s.push_str(call.name.as_str());
                                                s.push_str("Request");
                                                s
                                            }.as_str())
                                        ], vec![]),
                                        Element::new_node("output", vec![
                                            ("message", {
                                                let mut s = String::from("tns:");
                                                s.push_str(call.name.as_str());
                                                s.push_str("Response");
                                                s
                                            }.as_str())
                                        ], vec![]),
                                    ]
                                ));
                            }

                            children
                        }
                    ));

                    children.push(Element::new_node("binding",
                        vec![
                            ("name", "Registros do WSDLBinding"),
                            ("type", "tns:Registros do WSDLPortType"),
                        ],
                        {
                            let mut children = vec![
                                Element::new_node("soap:binding", vec![
                                    ("style", "rpc"),
                                    ("transport", "http://schemas.xmlsoap.org/soap/http"),
                                ], vec![]),
                            ];

                            for call in self.calls.iter() {
                                let mut tns = String::from("urn:");
                                tns.push_str(self.namespace.as_str());
                                tns.push_str(".");
                                tns.push_str(call.name.as_str());

                                let mut tns_anchor = tns.clone();
                                tns_anchor.push_str("#");
                                tns_anchor.push_str(call.name.as_str());

                                children.push(Element::new_node("operation",
                                    vec![("name", call.name.as_str())],
                                    vec![
                                        Element::new_node("soap:operation", vec![
                                            ("soapAction", tns_anchor.as_str()),
                                            ("style", "rpc"),
                                        ], vec![]),
                                        Element::new_node("input", vec![], vec![
                                            Element::new_node("soap:body", vec![
                                                ("use", "encoded"),
                                                ("namespace", tns.as_str()),
                                                ("encodingStyle", "http://schemas.xmlsoap.org/soap/encoding/"),
                                            ], vec![]),
                                        ]),
                                        Element::new_node("output", vec![], vec![
                                            Element::new_node("soap:body", vec![
                                                ("use", "encoded"),
                                                ("namespace", tns.as_str()),
                                                ("encodingStyle", "http://schemas.xmlsoap.org/soap/encoding/"),
                                            ], vec![]),
                                        ]),
                                    ],
                                ));
                            }

                            children
                        }
                    ));
                    
                    children.push(Element::new_node("service",
                        vec![("name", "Registros do WSDL")],
                        vec![
                            Element::new_node("port",
                                vec![
                                    ("name", "Registros do WSDLPort"),
                                    ("binding", "tns:Registros do WSDLBinding"),
                                ],
                                vec![
                                    Element::new_node("soap:address", vec![
                                        ("location", "http://localhost:1337/"),
                                    ], vec![]),
                                ]
                            )
                        ]
                    ));

                    children
                }
            );

            document.set_root(definitions);

            self.bytes   = document.to_string().into_bytes();
            self.changed = false;
        }

        self.bytes.as_slice()
            */
