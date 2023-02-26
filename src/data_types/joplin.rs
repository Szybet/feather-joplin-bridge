use reqwest::blocking;
use reqwest::Response;
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

pub struct JoplinData {
    token_string: String,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Debug)]
pub struct FoldersPure {
    Folders: Vec<FoldersArray>,
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

    pub fn new(provided_token: String) -> JoplinData {
        let new = JoplinData {
            token_string: format!("?token={}", provided_token),
        };
        new.ping().unwrap();
        new
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

    pub fn lookup_folder(&self, folder_path: String) -> Result<FoldersPure, Box<dyn Error>> {
        let mut search_for: bool = false;
        if folder_path == "/" {
            debug!("Root directory selected");
        } else {
            debug!(
                "Selected directory: {}. We will need to search for it's id",
                folder_path
            );
            search_for = true;
        }

        let mut request = format!("http://127.0.0.1:41184/{}{}", "/folders", self.token_string);
        let mut responses = self.request_pages_iterate(&mut request).unwrap();

        // uh, to create folders from first page
        let first_page: FoldersGet = serde_json::from_str(responses.remove(0).as_str())?;
        let mut folders = FoldersPure {
            Folders: first_page.items,
        };

        for response in responses {
            let mut page: FoldersGet = serde_json::from_str(response.as_str())?;
            folders.Folders.append(&mut page.items);
        }

        debug!("Root: Got folders: {:#?}", folders);
        debug!("Root: There are {} folders", folders.Folders.len());

        // Actual searching for the sub folder
        if search_for {
            let mut id = String::new();
            let mut count = 0;
            for one_folder in folders.Folders.into_iter() {
                if one_folder.title == folder_path {
                    count += 1;
                    id = one_folder.id;
                }
            }
            if count == 0 {
                error!("There is no such folder {}", folder_path);
                return Err(Box::new(MyError("Folder doesn't exist".into())));

            } else if count > 1 {
                warn!("There are more directories of the same name");
                return Err(Box::new(MyError("Duplicate name".into())));
            } else {
                let mut request_id = format!(
                    "http://127.0.0.1:41184/folders/:{}{}",
                    id, self.token_string
                );
                debug!("Request of specific folder: {}", request_id);
                let mut responses = self.request_pages_iterate(&mut request_id).unwrap();

                let first_page: FoldersGet = serde_json::from_str(responses.remove(0).as_str())?;
                let mut folders_id = FoldersPure {
                    Folders: first_page.items,
                };

                for response in responses {
                    let mut page: FoldersGet = serde_json::from_str(response.as_str())?;
                    folders_id.Folders.append(&mut page.items);
                }

                debug!("Inside: Got folders: {:#?}", folders_id);
                debug!("Inside: There are {} folders", folders_id.Folders.len());

                return Result::Ok(folders_id);
            }
        }
        Result::Ok(folders)
    }
}
