pub struct Options {
    pub bind_addr:    String,
    pub namespace:    String,
    pub service_name: String,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            bind_addr:    String::from(""),
            namespace:    String::from("server"),
            service_name: String::from("Service"),
        }
    }
}


