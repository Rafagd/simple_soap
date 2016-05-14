use xml::Element;

pub struct Document {
    encoding: String,
    root:     Element,
}

impl Document {
    pub fn new(encoding: &str) -> Document {
        Document {
            encoding: encoding.to_string(),
            root:     Element::Node("xml".to_string(), vec![], vec![]),
        }
    }

    pub fn set_root(&mut self, element: Element) {
        self.root = element;
    }

    pub fn get_root<'a>(&'a self) -> &'a Element {
        &self.root
    }

    pub fn get_mut_root<'a>(&'a mut self) -> &'a mut Element {
        &mut self.root
    }
}

impl ToString for Document {
    fn to_string(&self) -> String {
        let mut string = String::new();
        
        string.push_str("<?xml version=\"1.0\" encoding=\"");
        string.push_str(self.encoding.as_str());
        string.push_str("\"?>");

        string.push_str(self.root.to_string().as_str());

        string
    }
}

