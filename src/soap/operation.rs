use std::collections::HashMap;

use soap::Part;

pub struct Operation {
    pub doc:     String,
    pub name:    String,
    pub inputs:  HashMap<String, Part>,
    pub outputs: HashMap<String, Part>,
    pub closure: Box<FnMut(HashMap<String, Part>) -> HashMap<String, Part>>,
}

