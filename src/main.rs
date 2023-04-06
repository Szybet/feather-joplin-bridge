#[macro_use]
extern crate log;

use std::env;
use std::process::ExitCode;

mod convert_logic;
mod data_types;
mod pandoc;

use crate::convert_logic::overwrite_joplin_to_feathernotes;
use crate::data_types::feather::create_node_at_path;

use data_types::feather::*;
use data_types::joplin::*;

//use crate::pandoc::{convert_md_to_html, write_debug_file};
// https://docs.rs/clap/latest/clap/ to to to
fn main() -> ExitCode {
    env_logger::init();
    info!("Starting");
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        error!("Joplin token, feather file and main folder are needed to be provided");
        return ExitCode::FAILURE;
    }
    let joplin_arg = 1;
    let feather_arg = 2;
    let id_arg = 3;

    let mut joplin = JoplinData::new(args[joplin_arg].clone()).unwrap();
    let _folders_root = joplin.lookup_folder(args[id_arg].clone()).unwrap();
    //let _folders_root = joplin.dir_list.clone(); // For root

    /*
    debug!(
        "Id of folder {} is {}",
        _folders_root.first().unwrap().title,
        _folders_root.first().unwrap().id
    );
    joplin.get_notes_of_folder(_folders_root.first().unwrap().id.clone()).unwrap();

    let _note_md = joplin.get_note_body("54aac932ffc54e219698de18fdee0f37").unwrap();
    */

    //joplin.get_path_folder("eb4a68b569df4d758ac9890069ee15bb");

    let mut feather = FeatherStruct::read(args[feather_arg].clone()).unwrap();
    //feather.log_feather("Parsed_xml.fnx");

    /*
    let vec = vec!["Node 2", "Node 2.1", "Node 2.1.1"];
    let node_vec = &mut feather.struct_xml.node;
    create_node_at_path(
        node_vec,
        "New node title",
        "New body for node",
        vec,
        Option::None,
        0,
    )
    .unwrap();
    //feather.log_feather("");
    */
    
    overwrite_joplin_to_feathernotes(
        feather,
        joplin,
        _folders_root,
        "4c6d7baa06af4166b7b7c05b381874f4",
    );

    ExitCode::SUCCESS
}
