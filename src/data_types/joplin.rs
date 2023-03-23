use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::*;
use std::fmt;
use std::io::Write;
use std::path;

use crate::pandoc::repair_md_katex;
use crate::pandoc::write_debug_file;

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

// The same as Folders, but to avoid confusing
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NotesArray {
    pub id: String,
    pub parent_id: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NotesGet {
    pub items: Vec<NotesArray>,
    pub has_more: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FoldersArray {
    pub id: String,
    pub parent_id: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FoldersGet {
    pub items: Vec<FoldersArray>,
    pub has_more: bool,
}

pub struct JoplinData {
    pub token_string: String,
    pub dir_list: Vec<FoldersArray>, // We can't request only specific dirs so we need to do this, so save it for later
    pub notes_list: Vec<NotesArray>, // Searching doesn't work, some weird token error, but it's there
}

#[derive(Debug)]
pub struct MinimumFolder {
    pub title: String,
    pub id: String,
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
            notes_list: Vec::new(),
        };
        new.ping().unwrap();

        let mut request = format!("http://127.0.0.1:41184/{}{}", "/folders", &new.token_string);
        let responses = &new.request_pages_iterate(&mut request).unwrap();

        for response in responses {
            let mut page: FoldersGet = serde_json::from_str(response.as_str())?;
            new.dir_list.append(&mut page.items);
        }

        debug!("Got all folders: {:#?}", new.dir_list);
        debug!("There are {} folders in total", new.dir_list.len());

        let mut request = format!("http://127.0.0.1:41184/{}{}", "/notes", &new.token_string);
        let responses = &new.request_pages_iterate(&mut request).unwrap();

        for response in responses {
            let mut page: NotesGet = serde_json::from_str(response.as_str())?;
            new.notes_list.append(&mut page.items);
        }

        debug!("There are {} notes in total", new.notes_list.len());

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

    pub fn look_for_children_folders(
        &self,
        folder_storage: &mut Vec<FoldersArray>,
        root_id: String,
    ) {
        for folder in &self.dir_list {
            if folder.parent_id == root_id {
                folder_storage.push(folder.to_owned().clone());
                self.look_for_children_folders(folder_storage, folder.id.clone());
            }
        }
    }

    pub fn lookup_folder(&self, folder_id: String) -> Result<Vec<FoldersArray>, Box<dyn Error>> {
        // We also push here the root that was asked for
        let mut folders_children: Vec<FoldersArray> = Vec::new();

        let root_index_option = self.dir_list.iter().position(|x| x.id == folder_id);

        if root_index_option.is_some() {
            let index = root_index_option.unwrap();
            let root = &self.dir_list[index];
            folders_children.push(root.to_owned());
            self.look_for_children_folders(&mut folders_children, root.id.clone());

            debug!(
                "For root: {} found children: {:#?}. There are {} items",
                root.title,
                folders_children,
                folders_children.len()
            );
        } else {
            return Err(Box::new(MyError(
                "Couldn't found requested folder name".into(),
            )));
        }

        Result::Ok(folders_children)
    }

    pub fn get_notes_of_folder(
        &mut self,
        folder_id: &str,
    ) -> Result<Vec<NotesArray>, Box<dyn Error>> {
        let mut notes: Vec<NotesArray> = Vec::new();
        for note in &self.notes_list {
            if note.parent_id == folder_id {
                notes.push(note.clone());
            }
        }

        debug!("Found {} notes", notes.len());

        Ok(notes)
    }

    pub fn get_note_body(&self, note_id: &str) -> Result<String, Box<dyn Error>> {
        let request = format!(
            "http://127.0.0.1:41184/notes/{}{}&fields=body",
            note_id, self.token_string
        );

        debug!("get_note_body request: {}", request);

        let resp = reqwest::blocking::get(request)?;
        let v: Value = serde_json::from_str(&resp.text().unwrap())?;

        let mut str = v["body"].as_str().unwrap().to_string();

        str = repair_md_katex(str);

        write_debug_file("", str.to_string(), ".md");

        Ok(str)
    }

    // http://127.0.0.1:41184/folders/12b29e02391b48a29cf730ddee8b01ff?token=f7367f972d8d645a85c1ede0a9daabb5e1a43637570437b9289ff4cba45b6066c7a0072eabd70eab7e7d471338f5786d3b425e108f9b6149b60e0f105ab2525e
    pub fn get_path_folder(&self, folder_id: &str) -> Result<Vec<MinimumFolder>, Box<dyn Error>> {
        let mut path_not_inversed: Vec<MinimumFolder> = Vec::new();
        let mut id_to_look_for = folder_id.to_string();
        loop {
            let request = format!(
                "http://127.0.0.1:41184/folders/{}{}",
                id_to_look_for, self.token_string
            );

            debug!("get_path_folder request: {}", request);

            let resp = reqwest::blocking::get(request)?;
            let v: Value = serde_json::from_str(&resp.text().unwrap())?;

            let parent_id = v["parent_id"].as_str().unwrap();

            if parent_id.is_empty() {
                let folder = MinimumFolder {
                    title: v["title"].as_str().unwrap().to_string(),
                    id: v["id"].as_str().unwrap().to_string(),
                };
                path_not_inversed.push(folder);
                break;
            } else {
                let folder = MinimumFolder {
                    title: v["title"].as_str().unwrap().to_string(),
                    id: v["id"].as_str().unwrap().to_string(),
                };
                path_not_inversed.push(folder);
                id_to_look_for = parent_id.to_string();
            }
        }

        let path_inversed: Vec<MinimumFolder> = path_not_inversed.into_iter().rev().collect();
        debug!("path_inversed: {:#?}", path_inversed);

        Ok(path_inversed)
    }

    pub fn get_parent_of_note(&self, note_id: String) -> Result<String, Box<dyn Error>> {
        let request = format!(
            "http://127.0.0.1:41184/notes/{}/{}",
            note_id, self.token_string
        );
        debug!("get_parent_of_note request: {}", request);

        let resp = reqwest::blocking::get(request)?;
        let v: Value = serde_json::from_str(&resp.text().unwrap())?;

        Ok(v["parent_id"].as_str().unwrap().to_string())
    }
}
