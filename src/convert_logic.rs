use crate::data_types::feather::*;
use crate::data_types::joplin::*;

pub fn overwrite_joplin_to_feathernotes(
    mut feather_file: FeatherStruct,
    mut joplin_struct: JoplinData,
    joplin_folders: Vec<FoldersArray>,
    joplin_id: &str,
) -> String {
    info!("STARTING overwrite_joplin_to_feathernotes");

    let mut output_string = String::new();

    // Create the folder structure
    for folder in joplin_folders {
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
    feather_file.log_feather("");

    output_string
}
