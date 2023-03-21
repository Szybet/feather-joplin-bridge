use crate::data_types::feather::*;
use crate::data_types::joplin::*;

pub fn overwrite_joplin_to_feathernotes(
    feather_file: FeatherStruct,
    mut joplin_struct: JoplinData,
    joplin_folders: Vec<FoldersArray>,
    joplin_id: &str,
) -> String {
    let mut output_string = String::new();
    for folder in joplin_folders {
        let notes = joplin_struct.get_notes_of_folder(&folder.id).unwrap();
        debug!("Got {} notes of folder {}", notes.len(), folder.title);
    }

    output_string
}
