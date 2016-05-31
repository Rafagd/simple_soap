use std::string::ToString;

extern crate sxd_document;
use self::sxd_document::Package;
use self::sxd_document::dom::{ Element, Text };
use self::sxd_document::writer::format_document;

use soap::{ Operation, Service };

pub struct Wsdl<'a> {
    service: &'a Service,
    package: Package,
}

impl<'a> Wsdl<'a> {
    pub fn from(service: &'a Service) -> Wsdl {
        let wsdl = Wsdl {
            service: service,
            package: Package::new(),
        };

        wsdl.generate();
        wsdl
    }

    fn create_element(&self, tag_name: &str) -> Element {
        self.package.as_document().create_element(tag_name)
    }

    fn create_text(&self, text: &str) -> Text {
        self.package.as_document().create_text(text)
    }

    fn generate(&self) {
        let definitions = self.create_element("definitions");

        definitions.set_default_namespace_uri(Some("http://schemas.xmlsoap.org/wsdl/"));

        definitions.set_attribute_value("xmlns:soap",      "http://schemas.xmlsoap.org/wsdl/soap/");
        definitions.set_attribute_value("xmlns:xsd",       "http://www.w3.org/2001/XMLSchema");
        definitions.set_attribute_value("xmlns:xsi",       "http://www.w3.org/2001/XMLSchema-instance");
        definitions.set_attribute_value("xmlns:SOAP-ENV",  "http://schemas.xmlsoap.org/soap/envelope/");
        definitions.set_attribute_value("xmlns:SOAP-ENC",  "http://schemas.xmlsoap.org/soap/encoding/");

        {
            let mut urn = String::from("urn:");
            urn.push_str(self.service.options.namespace.as_str());

            definitions.set_attribute_value("xmlns:tns",       urn.as_str());
            definitions.set_attribute_value("targetNamespace", urn.as_str());
        }

        definitions.append_child(
            self.import_schemas(vec![
                "http://schemas.xmlsoap.org/soap/encoding/",
                "http://schemas.xmlsoap.org/wsdl/",
            ])
        );

        {
            let operations = self.service.operations.lock().unwrap();

            for (_, operation) in operations.iter() {
                definitions.append_child(
                    self.register_inputs(&operation)
                );

                definitions.append_child(
                    self.register_outputs(&operation)
                );
            }
        }

        definitions.append_child(
            self.register_ports()
        );

        definitions.append_child(
            self.register_bindings()
        );

        definitions.append_child(
            self.register_service()
        );

        self.package.as_document().root().append_child(definitions);
    }

    fn import_schemas(&self, schemas: Vec<&str>) -> Element {
        let types  = self.create_element("types");

        let schema = self.create_element("xsd:schema");
        schema.set_attribute_value("targetNamespace",
            self.service.options.namespace.as_str());

        for sch in schemas.iter() {
            let import = self.create_element("xsd:import");
            import.set_attribute_value("namespace", sch);

            schema.append_child(import);
        }

        types.append_child(schema);

        types
    }

    fn register_inputs(&self, operation: &Operation) -> Element {
        let mut name = operation.name.clone();
        name.push_str("Request");

        let message = self.create_element("message");
        message.set_attribute_value("name", name.as_str());
        
        for (name, arg) in operation.inputs.iter() {
            let part = self.create_element("part");
            part.set_attribute_value("name", name);
            part.set_attribute_value("type", arg.xsd_type().as_str());

            message.append_child(part);
        }

        message
    }

    fn register_outputs(&self, operation: &Operation) -> Element {
        let mut name = operation.name.clone();
        name.push_str("Response");

        let message = self.create_element("message");
        message.set_attribute_value("name", name.as_str());
        
        for (name, arg) in operation.outputs.iter() {
            let part = self.create_element("part");
            part.set_attribute_value("name", name);
            part.set_attribute_value("type", arg.xsd_type().as_str());

            message.append_child(part);
        }

        message
    }

    fn register_ports(&self) -> Element {
        let mut port_name = self.service.options.service_name.clone();
        port_name.push_str("PortType");

        let port = self.create_element("portType");
        port.set_attribute_value("name", port_name.as_str());

        let operations = self.service.operations.lock().unwrap();

        for (_, op) in operations.iter() {
            let operation = self.create_element("operation");
            operation.set_attribute_value("name", op.name.as_str());

            let doc = self.create_element("documentation");
            doc.append_child(self.create_text(op.doc.as_str()));

            let mut input_str = String::from("tns:");
            input_str.push_str(op.name.as_str());
            input_str.push_str("Request");

            let input = self.create_element("input");
            input.set_attribute_value("message", input_str.as_str());

            let mut output_str = String::from("tns:");
            output_str.push_str(op.name.as_str());
            output_str.push_str("Response");

            let output = self.create_element("output");
            output.set_attribute_value("message", output_str.as_str());

            operation.append_child(doc);
            operation.append_child(input);
            operation.append_child(output);
            port.append_child(operation);
        }

        port
    }

    fn register_bindings(&self) -> Element {
        let mut port_name = String::from("tns:");
        port_name.push_str(self.service.options.service_name.as_str());
        port_name.push_str("PortType");

        let mut bind_name = self.service.options.service_name.clone();
        bind_name.push_str("Binding");

        let server_ns = {
            let string_ns = self.service.options.namespace.to_string();
            let mut split = string_ns.split(".");
            split.next().unwrap().to_string()
        };

        let bind = self.create_element("binding");
        bind.set_attribute_value("name", bind_name.as_str());
        bind.set_attribute_value("type", port_name.as_str());

        let soap_bind = self.create_element("soap:binding");
        soap_bind.set_attribute_value("style", "rpc");
        soap_bind.set_attribute_value("transport", "http://schemas.xmlsoap.org/soap/http");
        bind.append_child(soap_bind);
        
        let operations = self.service.operations.lock().unwrap();

        for (_, op) in operations.iter() {
            let mut tns = String::from("urn:");
            tns.push_str(server_ns.as_str());
            tns.push_str(".");
            tns.push_str(op.name.as_str());

            let mut tns_anchor = tns.clone();
            tns_anchor.push_str("#");
            tns_anchor.push_str(op.name.as_str());

            let operation = self.create_element("operation");
            operation.set_attribute_value("name", op.name.as_str());

            let soap_op = self.create_element("soap:operation");
            soap_op.set_attribute_value("soapAction", tns_anchor.as_str());
            soap_op.set_attribute_value("style", "rpc");

            let soap_body = self.create_element("soap:body");
            soap_body.set_attribute_value("use", "encoded");
            soap_body.set_attribute_value("namespace", tns.as_str());
            soap_body.set_attribute_value("encodingStyle", "http://schemas.xmlsoap.org/soap/encoding/");

            let soap_input = self.create_element("input");
            soap_input.append_child(soap_body.clone());

            let soap_output = self.create_element("output");
            soap_output.append_child(soap_body);

            operation.append_child(soap_op);
            operation.append_child(soap_input);
            operation.append_child(soap_output);

            bind.append_child(operation);
        }

        bind
    }

    fn register_service(&self) -> Element {
        let service_name = &self.service.options.service_name;

        let mut port_name = service_name.clone();
        port_name.push_str("Port");

        let mut bind_name = String::from("tns:");
        bind_name.push_str(service_name.as_str());
        bind_name.push_str("Binding");
        

        let srv = self.create_element("service");
        srv.set_attribute_value("name", service_name.as_str());

        let port = self.create_element("port");
        port.set_attribute_value("name", port_name.as_str());
        port.set_attribute_value("binding", bind_name.as_str());

        let address = self.create_element("soap:address");
        address.set_attribute_value("location", self.service.get_uri().as_str());

        port.append_child(address);
        srv.append_child(port);

        srv
    }
}

impl<'a> ToString for Wsdl<'a> {
    fn to_string(&self) -> String {
        let mut buffer = Vec::new();
        let _ = format_document(&self.package.as_document(), &mut buffer);
        String::from_utf8(buffer).unwrap()
    }
}

