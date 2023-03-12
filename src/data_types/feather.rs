use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use serde::Deserialize;
use std::error::*;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
// https://stackoverflow.com/questions/51550167/how-to-manually-return-a-result-boxdyn-error
#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MyError {}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct Node {
    #[serde(rename = "@name")]
    name: String,
    #[serde(rename = "$text")]
    text: String,
    node: Vec<Node>,
}

#[derive(Debug, PartialEq, Default, Deserialize)]
pub struct Feathernotes {
    node: Vec<Node>,
}

pub struct FeatherStruct {
    reader: Reader<BufReader<File>>,
    struct_xml: Feathernotes,
}

impl FeatherStruct {
    pub fn new(provided_path: String) -> Result<FeatherStruct, Box<dyn Error>> {
        debug!("Provided path for feather XML file: {}", provided_path);
        let mut reader_new = Reader::from_file(provided_path.clone()).unwrap();
        reader_new.trim_text(true);

        if log_enabled!(log::Level::Debug) {
            let mut buf = Vec::new();
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
            reader: reader_new,
            struct_xml: feathernotes,
        };
        Ok(new_feather)
    }
}
