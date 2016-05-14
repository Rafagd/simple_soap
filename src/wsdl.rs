use std::collections::HashMap;

use types::*;
use xml::*;

pub struct Wsdl {
    namespace: String,
    calls:     HashMap<String, RemoteCall>,
    bytes:     Vec<u8>,
    changed:   bool,
}

impl Wsdl {
    pub fn new(namespace: &str) -> Wsdl {
        Wsdl {
            namespace: namespace.to_string(),
            calls:     HashMap::new(),
            bytes:     vec![],
            changed:   true,
        }
    }

    fn init_xml(namespace: &str) -> Document {
        let mut document = Document::new("UTF-8");


        document
    }

    pub fn add(&mut self, call: RemoteCall) {
        self.calls.insert(call.name.clone(), call);
        self.changed = true;
    }

    pub fn as_bytes<'a>(&'a mut self) -> &'a [u8] {
        if self.changed {
            let mut document = Document::new("UTF-8");

            let mut tns = String::from("urn:");
            tns.push_str(self.namespace.as_str());

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
                                vec![
                                    ("targetNamespace", tns.as_str()),
                                ],
                                vec![
                                    Element::new_node("xsd:import", vec![
                                        ("namespace", "http://schemas.xmlsoap.org/soap/encoding/"),
                                    ], vec![]),
                                    Element::new_node("xsd:import", vec![
                                        ("namespace", "http://schemas.xmlsoap.org/wsdl/"),
                                    ], vec![]),
                                ]
                            ),
                        ]),
                    ];

                    for (key, call) in self.calls.iter() {
                        // Request
                        let mut type_name = call.name.clone();
                        type_name.push_str("Request");

                        children.push(Element::new_node("message",
                            vec![
                                ("name", type_name.as_str()),
                            ],
                            {
                                let mut children = vec![];

                                for arg in call.arguments.iter() {
                                    children.push(Element::new_node("part",
                                        vec![
                                            ("name", arg.name.as_str()),
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
                            vec![
                                ("name", type_name.as_str()),
                            ],
                            vec![
                                Element::new_node("part", vec![
                                    ("name", call.result.name.as_str()),
                                    ("type", type_to_xsd(call.result.ty.clone()).as_str()),
                                ], vec![])
                            ]
                        ));
                    }

                    children.push(Element::new_node("portType",
                        vec![
                            ("name", "WSDLPortType"),
                        ],
                        {
                            let mut children = vec![];
                            
                            for (_, call) in self.calls.iter() { 
                                children.push(Element::new_node("operation",
                                    vec![
                                        ("name", call.name.as_str()),
                                    ],
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
                            ("name", "WSDLBinding"),
                            ("type", "tns:WSDLBinding"),
                        ],
                        {
                            let mut children = vec![

                            ];
                        }
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

