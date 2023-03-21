use crate::pandoc::write_debug_file;
use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::Deserialize;
use serde::Serialize;
use std::error::*;
use std::fmt;

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

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Node {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "$text")]
    pub text: String,
    #[serde(default)]
    pub node: Vec<Node>, // https://github.com/tafia/quick-xml/issues/510
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
#[warn(non_camel_case_types)]
#[allow(non_camel_case_types)]
pub struct feathernotes {
    pub node: Vec<Node>,
    #[serde(rename = "@txtfont")]
    pub txtfont: String,
    #[serde(rename = "@nodefont")]
    pub nodefont: String,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct FeatherStruct {
    pub struct_xml: feathernotes,
}

impl FeatherStruct {
    pub fn log_feather(&self, title: &str) {
        if log_enabled!(log::Level::Debug) {
            let struct_copy = self.struct_xml.clone();
            let xml = to_string(&struct_copy).unwrap();
            write_debug_file(title, xml);
        }
    }

    pub fn new() -> Result<FeatherStruct, Box<dyn Error>> {
        let feather_inner = feathernotes { 
            node: Vec::new(),
            txtfont: String::from("Monospace,11,-1,5,400,0,0,0,0,0,0,0,0,0,0,1"),
            nodefont: String::from("Cantarell,11,-1,5,400,0,0,0,0,0,0,0,0,0,0,1"),
         };

        let feather = FeatherStruct {
            struct_xml: feather_inner,
        };

        Ok(feather)
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

        let feathernotes_from_str: feathernotes =
            from_str(std::fs::read_to_string(provided_path).unwrap().as_ref()).unwrap();

        debug!("Parsed struct: {:#?}", feathernotes_from_str);

        let new_feather = FeatherStruct {
            struct_xml: feathernotes_from_str,
        };

        Ok(new_feather)
    }


}

    // If path doesn't exist, throw an error
    pub fn create_node_at_path(
        currect_node: &mut Vec<Node>,
        title: &str,
        body: &str,
        path: Vec<&str>,
        path_progress: Option<usize>, // if found, this gives the next index to look for
        children_count: usize,        // 0 is root, other are children, this gets bigger and bigger
    ) -> Result<(), Box<dyn Error>> {
        let last_name_tmpvec = path.clone();
        let last_name = last_name_tmpvec.last().unwrap();

        let mut path_name_index = 0;

        match path_progress {
            Some(x) => {
                path_name_index = x;
            }
            None => {
                debug!("path_progress is root");
            },
        }

        if &path[path_name_index] != last_name {
            for node in currect_node.iter_mut() {
                if node.name == path[path_name_index] {
                    if !node.node.is_empty() {
                        let result_child = create_node_at_path(
                            &mut node.node,
                            title,
                            body,
                            path,
                            Some(path_name_index + 1),
                            children_count + 1,
                        );
                        match result_child {
                            Ok(_) => {
                                return Ok(());
                            }
                            Err(_) => {
                                if children_count == 0 {
                                    debug!("What?????");
                                }
                                return Err(Box::new(MyError(String::new())));
                            }
                        }
                    } else {
                        let err =
                            format!("Node of name {} doesn't have a any subfolders", node.name);
                        //return Err(Box::new(MyError(err)));
                    }
                }
            }
        } else {
            debug!("Found dir?");
            let new_node = Node {
                name: title.to_string(),
                text: body.to_string(),
                node: Vec::new(),
            };
            currect_node.push(new_node);
            debug!("Children number {} has written to folder", children_count);
            return Ok(());
        }

        if children_count == 0 {
            return Err(Box::new(MyError(
                "Couldn't find requested folder name".into(),
            )));
        }

        return Err(Box::new(MyError(String::new())));
    }
