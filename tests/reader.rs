extern crate simple_soap as soap;

#[test]
pub fn constructor() {
    let reader = {
        soap::reader::Reader::from_file("tests/request.xml")
    };

    assert!(reader.is_ok());
}

#[test]
pub fn read() {
    let reader = {
        soap::reader::Reader::from_file("tests/request.xml")
            .unwrap()
    };
}
