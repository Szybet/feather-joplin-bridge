use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::Deserialize;
use serde::Serialize;
use std::error::*;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;
use std::process::ExitCode;


use quick_xml::se::to_string;
// https://stackoverflow.com/questions/51550167/how-to-manually-return-a-result-boxdyn-error
#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MyError {}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Node {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "$text")]
    text: String,
    #[serde(flatten)] // Why
    node: Option<Vec<Node>>, // WHYYYY
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Feathernotes {
    node: Vec<Node>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct FeatherStruct {
    struct_xml: Feathernotes,
}

impl FeatherStruct {
    pub fn new() -> Result<FeatherStruct, Box<dyn Error>> {
        let feather_inner = Feathernotes {
            node: Vec::new(),
        };

        let feather = FeatherStruct {
            struct_xml: feather_inner,
        };

        return Ok(feather);
    }
    pub fn read(provided_path: String) -> Result<FeatherStruct, Box<dyn Error>> {
        debug!("Provided path for feather XML file: {}", provided_path);


        if log_enabled!(log::Level::Debug) {
            let mut buf = Vec::new();
            let mut reader_new = Reader::from_file(provided_path.clone()).unwrap();
            reader_new.trim_text(true);
            // Debug thing
            loop {
                match reader_new.read_event_into(&mut buf) {
                    Err(e) => error!(
                        "Error at position {}: {:?}",
                        reader_new.buffer_position(),
                        e
                    ),
                    Ok(Event::Eof) => {
                        debug!("End of file");
                        break;
                    }
                    Ok(Event::Start(e)) => {
                        debug!("Event start {:#?}", e)
                    }
                    Ok(Event::Text(e)) => {
                        debug!("Text start {:#?}", e)
                    }
                    _ => (),
                }
                // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory usage low
                buf.clear();
            }
        }

        let feathernotes: Feathernotes =
            from_str(std::fs::read_to_string(provided_path).unwrap().as_ref()).unwrap();

        debug!("Parsed struct: {:#?}", feathernotes);


        let new_feather = FeatherStruct {
            struct_xml: feathernotes,
        };
        Ok(new_feather)
    }
    // If path doesn't exist, throw an error
    pub fn create_node_at_path(&mut self, title: &str, body: &str, path: Vec<String>) {
        let mut path_changing = &self.struct_xml.node;

        let last_name_tmpvec = path.clone();
        let last_name = last_name_tmpvec.last().unwrap();

        for path_name in path {
            if(&path_name != last_name) {
                for node in path_changing {
                    if node.name == path_name {
                        match &node.node {
                            Some(x) => {
                                path_changing = x;
                                break;
                            }
                            None => {
                                error!("Node of name {} doesn't have a any subfolders", node.name);
                                // TODO
                                exit(-1);
                                
                            }
                        }
                    }
                }
            } else {
                
            }
            

        }
    }
}
