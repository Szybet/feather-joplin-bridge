use crate::data_types::feather::*;
use crate::data_types::joplin::*;
use crate::pandoc::convert_md_to_html;

pub fn overwrite_joplin_to_feathernotes(
    mut feather_file: FeatherStruct,
    mut joplin_struct: JoplinData,
    joplin_folders: Vec<FoldersArray>,
    joplin_id: &str,
) {
    info!("STARTING overwrite_joplin_to_feathernotes");

    // Create the folder structure
    for folder in &joplin_folders {
        let path_vec = joplin_struct.get_path_folder(&folder.id).unwrap();
        for path_index in 1..path_vec.len() + 1 {
            debug!("path_index: {}", path_index);
            let path_small = path_vec.chunks(path_index).next().unwrap();

            let mut path_item: Vec<String> = Vec::new();
            for item in path_small.iter() {
                if item.id != path_small.last().unwrap().id
                    && path_small.last().unwrap().id != path_small.first().unwrap().id
                {
                    path_item.push(item.title.clone());
                }
            }

            debug!(
                "path_item: {:?} last item name: {}",
                path_item,
                &path_small.last().unwrap().title
            );

            let node_vec = &mut feather_file.struct_xml.node;
            create_node_at_path(
                node_vec,
                &path_small.last().unwrap().title,
                &path_small.last().unwrap().id,
                path_item,
                Option::None,
                0,
            )
            .unwrap();
        }
        //let notes = joplin_struct.get_notes_of_folder(&folder.id).unwrap();
        //debug!("Got {} notes of folder {}", notes.len(), folder.title);
    }
    //feather_file.log_feather("");

    // Manage notes
    for folder in &joplin_folders {
        let notes = joplin_struct.get_notes_of_folder(&folder.id).unwrap();

        for note in notes {
            let body_md = joplin_struct.get_note_body(&note.id).unwrap();

            let body = convert_md_to_html(body_md);

            let parent = joplin_struct.get_parent_of_note(note.id).unwrap();

            let path_small = joplin_struct.get_path_folder(&parent).unwrap();

            let mut path_item: Vec<String> = Vec::new();
            for item in path_small.iter() {
                path_item.push(item.title.clone());
            }

            debug!("Note with title: {} has path: {:?}", note.title, path_item);

            let node_vec = &mut feather_file.struct_xml.node;
            create_node_at_path(
                node_vec,
                &note.title,
                &body,
                path_item,
                Option::None,
                0,
            )
            .unwrap();
        }
    }
    feather_file.write_file("feather_file");
}
