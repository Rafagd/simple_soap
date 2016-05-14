pub enum Element {
    Node(String, Vec<(String, String)>, Vec<Element>),
    Text(String),
}

impl Element {
    pub fn new_node(tag: &str, attrs: Vec<(&str, &str)>, children: Vec<Element>) -> Element {
        let mut s_attrs = vec![];

        for attr in attrs {
            s_attrs.push((attr.0.to_string(), attr.1.to_string()));
        }

        Element::Node(tag.to_string(), s_attrs, children)
    }

    pub fn new_text(content: &str) -> Element {
        Element::Text(content.to_string())
    }
}

impl ToString for Element {
    fn to_string(&self) -> String {
        match self {
            &Element::Node(ref tag, ref attrs, ref children) => {
                let mut content = String::from("<");
                content.push_str(tag.as_str());

                for attr in attrs.iter() {
                    content.push_str(" ");
                    content.push_str(attr.0.as_str());
                    content.push_str("=\"");
                    content.push_str(attr.1.as_str());
                    content.push_str("\"");
                }

                if children.len() == 0 {
                    content.push_str("/>");

                } else {
                    content.push_str(">");

                    for child in children.iter() {
                        content.push_str(child.to_string().as_str());
                    }

                    content.push_str("</");
                    content.push_str(tag.as_str());
                    content.push_str(">");
                }

                content
            },
            &Element::Text(ref content) => {
                content.clone()
            }
        }
    }
}
