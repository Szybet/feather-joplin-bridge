use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::*;
use std::fmt;

// https://stackoverflow.com/questions/51550167/how-to-manually-return-a-result-boxdyn-error
#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MyError {}

//

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FoldersArray {
    id: String,
    parent_id: String,
    title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FoldersGet {
    items: Vec<FoldersArray>,
    has_more: bool,
}

pub struct JoplinData {
    token_string: String,
    dir_list: Vec<FoldersArray>, // We can't request only specific dirs so we need to do this, so save it for later
}

impl JoplinData {
    pub fn ping(&self) -> Result<(), Box<dyn Error>> {
        // https://stackoverflow.com/questions/54159232/best-practice-to-return-a-result-impl-error-and-not-a-result-str-in-rus
        let resp = reqwest::blocking::get("http://127.0.0.1:41184/ping");
        match resp {
            Ok(x) => {
                debug!("Ping succesfull: {:#?}", x);
                if x.text().unwrap() == "JoplinClipperServer" {
                    info!("Ping worked");
                } else {
                    warn!("Ping failure?");
                }
            }
            Err(x) => {
                error!("Failed to ping: {}", x);
                return Err(Box::new(MyError("Ping failed".into())));
            }
        }
        Result::Ok(())
    }

    pub fn new(provided_token: String) -> Result<JoplinData, Box<dyn Error>> {
        let mut new = JoplinData {
            token_string: format!("?token={}", provided_token),
            dir_list: Vec::new(),
        };
        new.ping().unwrap();

        let mut request = format!("http://127.0.0.1:41184/{}{}", "/folders", &new.token_string);
        let mut responses = &new.request_pages_iterate(&mut request).unwrap();

        for response in responses {
            let mut page: FoldersGet = serde_json::from_str(response.as_str())?;
            new.dir_list.append(&mut page.items);
        }

        debug!("Got all folders: {:#?}", new.dir_list);
        debug!("There are {} folders in total", new.dir_list.len());

        Result::Ok(new)
    }

    pub fn add_page(&self, page: i32, request: &mut String) -> String {
        format!("{}&page={}", request, page)
    }

    pub fn request_pages_iterate(
        &self,
        request: &mut String,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let mut page = 1;
        let request_page = self.add_page(page, request);
        debug!("Request URL: {}", request);
        let mut responses: Vec<String> = Vec::new();
        let mut resp = reqwest::blocking::get(request_page)?;
        let mut text = resp.text().unwrap();
        debug!("Text from first run: {:#?}", text);
        responses.push(text.clone());
        loop {
            let v: Value = serde_json::from_str(text.as_str())?;
            if v["has_more"].as_bool().unwrap() {
                page += 1;
                debug!("Running for page: {}", page);
                let request_page = self.add_page(page, request);
                resp = reqwest::blocking::get(request_page)?;
                text = resp.text().unwrap();
                responses.push(text.clone());
            } else {
                debug!("There are no more pages. Last page was: {}", page);
                break;
            }
        }
        Result::Ok(responses)
    }

    pub fn look_for_children(&self, folder_storage: &mut Vec<FoldersArray>, root_id: String) {
        for folder in &self.dir_list {
            if folder.parent_id == root_id {
                folder_storage.push(folder.to_owned().clone());
            }
        }
    }

    pub fn lookup_folder(&self, folder_path: String) -> Result<Vec<FoldersArray>, Box<dyn Error>> {
        // We also push here the root that was asked for
        let mut folders_children: Vec<FoldersArray> = Vec::new();

        let root_index_option = self.dir_list.iter().position(|x| x.title == folder_path);
        
        if root_index_option.is_some() {
            let index = root_index_option.unwrap();
            let root = &self.dir_list[index];
            folders_children.push(root.to_owned().clone());


        } else {
            return Err(Box::new(MyError("Couldn't found requested folder name".into())));
        }



        Result::Ok(folders_children)
    }
}
