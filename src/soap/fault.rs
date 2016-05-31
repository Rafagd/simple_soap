extern crate sxd_document;
use self::sxd_document::dom::{ Document, Element };

enum FaultCode {
    VersionMismatch,
    MustUnderstand,
    Client,
    Server,
}

impl FaultCode {
    pub fn to_xml<'a>(&'a self, document: &Document<'a>) -> Element {
        let code = document.create_element("faultcode");
        code.set_attribute_value("xsi:type", "xsd:string");

        code.append_child(match self {
            &FaultCode::VersionMismatch =>
                document.create_text("SOAP-ENV:VersionMismatch"),

            &FaultCode::MustUnderstand =>
                document.create_text("SOAP-ENV:MustUnderstand"),

            &FaultCode::Server =>
                document.create_text("SOAP-ENV:Server"),

            &FaultCode::Client =>
                document.create_text("SOAP-ENV:Client"),
        });

        code
    }
}

pub struct Fault {
    code:   FaultCode,
    string: String,
    actor:  String,
    detail: String,
}

impl Fault {
    fn new(code: FaultCode, string: &str, actor: &str, detail: &str)
        -> Fault
    {
        Fault {
            code:   code,
            string: string.to_string(),
            actor:  actor.to_string(),
            detail: detail.to_string(),
        }
    }

    pub fn version_mismatch(string: &str, actor: &str, detail: &str)
        -> Fault
    {
        Fault::new(FaultCode::VersionMismatch, string, actor, detail)
    }

    pub fn must_understand(string: &str, actor: &str, detail: &str)
        -> Fault
    {
        Fault::new(FaultCode::MustUnderstand, string, actor, detail)
    }

    pub fn server(string: &str, actor: &str, detail: &str)
        -> Fault
    {
        Fault::new(FaultCode::Server, string, actor, detail)
    }

    pub fn client(string: &str, actor: &str, detail: &str)
        -> Fault
    {
        Fault::new(FaultCode::Client, string, actor, detail)
    }

    pub fn to_xml<'a>(&'a self, document: &Document<'a>) -> Element {
        let fault = document.create_element("SOAP-ENV:Fault");

        let code = self.code.to_xml(document);
        fault.append_child(code);

        // This one I want to be an explicit empty tag, if empty.
        let string = document.create_element("faultstring");
        string.set_attribute_value("xsi:type", "xsd:string");
        if self.string != "" {
            string.append_child(document.create_text(self.string.as_str()));
        }
        fault.append_child(string);

        if self.actor != "" {
            let actor = document.create_element("faultactor");
            actor.set_attribute_value("xsi:type", "xsd:string");
            actor.append_child(document.create_text(self.actor.as_str()));
            fault.append_child(actor);
        }

        if self.detail != "" {
            let detail = document.create_element("detail");
            detail.set_attribute_value("xsi:type", "xsd:string");
            detail.append_child(document.create_text(self.detail.as_str()));
            fault.append_child(detail);
        }

        fault
    }
}
