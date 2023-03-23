use crate::pandoc::*;
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
    pub fn write_file(&self, title: &str) {
        let struct_copy = self.struct_xml.clone();
        let mut xml = to_string(&struct_copy).unwrap();
        xml = final_touches_xml(xml);

        if log_enabled!(log::Level::Debug) {
            write_debug_file(title, xml, ".fnx");
        }
        // TODO: normal write
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
    path: Vec<String>,
    path_progress: Option<usize>, // if found, this gives the next index to look for
    children_count: usize,        // 0 is root, other are children, this gets bigger and bigger
) -> Result<(), Box<dyn Error>> {
    if children_count == 0 {
        debug!(
            "Trying to write {} at {:?} with current node: {:?}",
            title, path, currect_node
        );
    }

    let last_name_tmpvec = path.clone();

    let mut last_name: String = String::new();

    // Check if writing at root is needed, then skip the rest
    match last_name_tmpvec.last() {
        Some(x) => {
            last_name = x.clone();
        }
        None => {
            debug!("Writing at root of feather file");
            let new_node = Node {
                name: title.to_string(),
                text: body.to_string(),
                node: Vec::new(),
            };

            let mut duplicate = false;
            for item in currect_node.iter_mut() {
                if item.name == new_node.name {
                    duplicate = true;
                }
            }

            if !duplicate {
                currect_node.push(new_node);
                debug!(
                    "Children number {} has written note of title {}",
                    children_count,
                    title.to_string()
                );
            } else {
                debug!("Avoiding writing duplicate of title: {}", title.to_string());
            }

            return Ok(());
        }
    }

    let mut path_name_index = 0;

    match path_progress {
        Some(x) => {
            path_name_index = x;
        }
        None => {
            debug!("path_progress is root");
        }
    }

    // If its in root, first go INTO this one node. thats why path_name_index != 0
    if path[path_name_index] != last_name {
        for node in currect_node.iter_mut() {
            if node.name == path[path_name_index] {
                debug!(
                    "String \"{}\" IS equal \"{}\"? for children {}",
                    node.name, path[path_name_index], children_count
                );
                if !node.node.is_empty() {
                    debug!("Running a child");
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
                    let err = format!("Node of name {} doesn't have a any subfolders", node.name);
                    debug!("{}", &err);
                    if &node.name == path.last().unwrap() {
                        debug!("Testing!");
                        let new_node = Node {
                            name: title.to_string(),
                            text: body.to_string(),
                            node: Vec::new(),
                        };
                        node.node.push(new_node);
                        return Ok(());
                    }
                    // Nope, acceptable. another child will find it... propably
                    //return Err(Box::new(MyError(err)));
                }
            } else {
                debug!(
                    "String \"{}\" is NOT equal \"{}\"? for children {}",
                    node.name, path[path_name_index], children_count
                );
            }
        }
    } else {
        debug!(
            "Found dir?, path[path_name_index]: {}, last_name: {}",
            path[path_name_index], last_name
        );

        if path[path_name_index] == last_name {
            debug!("Edge case?");
            for node in currect_node.iter_mut() {
                if node.name == path[path_name_index] {
                    let new_node = Node {
                        name: title.to_string(),
                        text: body.to_string(),
                        node: Vec::new(),
                    };

                    let mut duplicate = false;
                    for item in node.node.iter_mut() {
                        if item.name == new_node.name {
                            duplicate = true;
                        } else {
                            debug!(
                                "TEST: item.name: {} and new_node.name: {} are not equal",
                                item.name, new_node.name
                            );
                        }
                    }
                    if !duplicate {
                        node.node.push(new_node);
                    } else {
                        debug!("Avoiding writing duplicate of title: {}", title.to_string());
                    }
                    return Ok(());
                }
            }
        }

        let new_node = Node {
            name: title.to_string(),
            text: body.to_string(),
            node: Vec::new(),
        };

        let mut duplicate = false;
        for item in currect_node.iter_mut() {
            if item.name == new_node.name {
                duplicate = true;
            }
        }

        if !duplicate {
            currect_node.push(new_node);
            debug!(
                "Children number {} has written note of title {}",
                children_count,
                title.to_string()
            );
        } else {
            debug!("Avoiding writing duplicate of title: {}", title.to_string());
        }
        return Ok(());
    }

    if children_count == 0 {
        return Err(Box::new(MyError(format!(
            "Couldn't find requested folder name, {:?}",
            path
        ))));
    }
    Err(Box::new(MyError(String::new())))
}
