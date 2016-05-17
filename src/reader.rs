use std::fs::File;
use std::collections::HashMap;
use std::io::{ BufRead, BufReader, Read };
use std::path::Path;

extern crate quick_xml;
use self::quick_xml::{ XmlReader, Event, Element };

use error::SoapError;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub arguments: HashMap<String, String>,
}

pub struct Reader;

impl Reader {
    pub fn from<R: Read>(buffer: R) -> Vec<Request> {
        let     buffer = BufReader::new(buffer);
        let mut xml    = XmlReader::from_reader(buffer).trim_text(true);

        Reader::read_xml(&mut xml)
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Request>, SoapError> {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(err) => { return Err(SoapError::Io(err)) },
        };

        Ok(Reader::from(file))
    }

    fn tag_name(element: &Element) -> (String, String) {
        let full_name = String::from_utf8_lossy(element.name()).into_owned();
        let mut split = full_name.split(":");

        let mut namespace = "";
        let mut tag       = split.next().unwrap();

        match split.next() {
            Some(value) => {
                namespace = tag;
                tag       = value;
            },
            _ => (),
        }

        (namespace.to_string(), tag.to_string())
    }

    fn read_xml<R: BufRead>(xml: &mut XmlReader<R>) -> Vec<Request> {
        let mut requests = vec![];
        let mut steps    = 0;

        while true {
            let evt = match xml.next() {
                Some(evt) => evt,
                None      => { break; },
            };

            match evt {
                Ok(Event::Decl(_)) => {
                    if steps != 0 {
                        break;
                    }

                    steps += 1;
                }
                Ok(Event::Start(ref element)) => {
                    let (_, tag) = Reader::tag_name(element);

                    match tag.as_str() {
                        "Envelope" => {
                            if steps != 1 {
                                break;
                            }

                            steps += 1;
                        },
                        "Body" => {
                            if steps != 2 {
                                break;
                            }

                            steps += 1;
                        }
                        _ => {
                            if steps == 3 {
                                requests.push(
                                    Reader::read_method(tag.as_str(), xml)
                                );
                            }
                        },
                    }
                },
                _ => {
                }
            }
        }

        requests
    }

    fn read_method<R: BufRead>(tag: &str, xml: &mut XmlReader<R>) -> Request {
        let mut request = Request {
            method:    tag.to_string(),
            arguments: HashMap::new(),
        };

        let mut curr_arg = String::new();

        while true {
            let evt = match xml.next() {
                Some(evt) => evt,
                None      => { break; },
            };

            match evt {
                Ok(Event::Start(ref element)) => {
                    let (namespace, evttag) = Reader::tag_name(element);

                    curr_arg = evttag
                },
                Ok(Event::Text(element)) => {
                    if curr_arg == "" {
                        break;
                    }

                    let content = String::from_utf8_lossy(element.content())
                        .into_owned();

                    request.arguments.insert(curr_arg.clone(), content);
                }
                Ok(Event::End(ref element)) => {
                    let (namespace, evttag) = Reader::tag_name(element);

                    if tag == evttag {
                        break;
                    }

                    if tag == curr_arg {
                        curr_arg = String::new();
                    }
                }
                _ => {
                    println!("evt {:?}", tag);
                }
            }
        }

        request
    }
}

