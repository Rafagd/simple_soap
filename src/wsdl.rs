use std::collections::HashMap;

use error::*;
use types::*;
use xml::*;

pub struct Wsdl {
    namespace:        String,
    target_namespace: String,
    calls:            Vec<RemoteCall>,
    bytes:            Vec<u8>,
    changed:          bool,
}

impl Wsdl {
    pub fn new(namespace: &str, target_namespace: &str) -> Wsdl {
        Wsdl {
            namespace:        namespace.to_string(),
            target_namespace: target_namespace.to_string(),
            calls:            vec![],
            bytes:            vec![],
            changed:          true,
        }
    }

    pub fn add(&mut self, call: RemoteCall) {
        self.calls.push(call);
        self.changed = true;
    }

    pub fn get_callback_mut<'a>(&'a mut self, method: &str) -> Result<&'a mut RemoteCall, SoapError> {
        for call in self.calls.iter_mut() {
            if method == call.name {
                return Ok(call);
            }
        }
        
        Err(SoapError::NotFound)
    }

    pub fn as_bytes<'a>(&'a mut self) -> &'a [u8] {
        if self.changed {
            let mut document = Document::new("UTF-8");

            let mut tns = String::from("urn:");
            tns.push_str(self.namespace.as_str());
            tns.push_str(".");
            tns.push_str(self.target_namespace.as_str());

            let mut definitions = Element::new_node("definitions",
                vec![
                    ("xmlns",           "http://schemas.xmlsoap.org/wsdl/"),
                    ("xmlns:soap",      "http://schemas.xmlsoap.org/wsdl/soap/"),
                    ("xmlns:xsd",       "http://www.w3.org/2001/XMLSchema"),
                    ("xmlns:xsi",       "http://www.w3.org/2001/XMLSchema-instance"),
                    ("xmlns:SOAP-ENV",  "http://schemas.xmlsoap.org/soap/envelope/"),
                    ("xmlns:SOAP-ENC",  "http://schemas.xmlsoap.org/soap/encoding/"),
                    ("xmlns:tns",       tns.as_str()),
                    ("targetNamespace", tns.as_str()),
                ],
                {
                    let mut children = vec![
                        Element::new_node("types", vec![], vec![
                            Element::new_node("xsd:schema",
                                vec![("targetNamespace", tns.as_str())],
                                vec![
                                    Element::new_node("xsd:import",
                                        vec![("namespace", "http://schemas.xmlsoap.org/soap/encoding/")],
                                        vec![]
                                    ),
                                    Element::new_node("xsd:import",
                                        vec![("namespace", "http://schemas.xmlsoap.org/wsdl/")],
                                        vec![]
                                    ),
                                ]
                            ),
                        ]),
                    ];

                    for call in self.calls.iter() {
                        // Request
                        let mut type_name = call.name.clone();
                        type_name.push_str("Request");

                        children.push(Element::new_node("message",
                            vec![("name", type_name.as_str())],
                            {
                                let mut children = vec![];

                                for (name, arg) in call.arguments.iter() {
                                    children.push(Element::new_node("part",
                                        vec![
                                            ("name", name.as_str()),
                                            ("type", type_to_xsd(arg.ty.clone()).as_str()),
                                        ],
                                        vec![]
                                    ));
                                }

                                children
                            }
                        ));

                        // Response

                        let mut type_name = call.name.clone();
                        type_name.push_str("Response");

                        children.push(Element::new_node("message",
                            vec![("name", type_name.as_str())],
                            vec![
                                Element::new_node("part", vec![
                                    ("name", call.result.name.as_str()),
                                    ("type", type_to_xsd(call.result.ty.clone()).as_str()),
                                ], vec![])
                            ]
                        ));
                    }

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
    }
}

fn type_to_xsd(ty: Type) -> String {
    match ty {
        Type::String => String::from("xsd:string"),
        _ => String::new(),
    }
}

