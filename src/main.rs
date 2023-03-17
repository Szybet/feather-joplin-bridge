#[macro_use]
extern crate log;

use std::env;
use std::process::ExitCode;

mod convert_logic;
mod data_types;
mod pandoc;

use crate::convert_logic::overwrite_joplin_to_feathernotes;
use crate::data_types::feather::FeatherStruct;
use data_types::joplin::*;

//use crate::pandoc::{convert_md_to_html, write_debug_file};

fn main() -> ExitCode {
    env_logger::init();
    info!("Starting");
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        error!("Joplin token, feather file and main folder are needed to be provided");
        return ExitCode::FAILURE;
    }
    let path_int = 3;

    let mut joplin = JoplinData::new(args[1].clone()).unwrap();

    let _folders_root = joplin.lookup_folder(args[path_int].clone()).unwrap();

    debug!(
        "Id of folder {} is {}",
        _folders_root.first().unwrap().title,
        _folders_root.first().unwrap().id
    );
    /*
    joplin.get_notes_of_folder(_folders_root.first().unwrap().id.clone()).unwrap();

    let _note_md = joplin.get_note_body("54aac932ffc54e219698de18fdee0f37").unwrap();
    */
    let mut feather = FeatherStruct::new().unwrap();

    overwrite_joplin_to_feathernotes(
        feather,
        joplin,
        _folders_root,
        "4c6d7baa06af4166b7b7c05b381874f4",
    );

    ExitCode::SUCCESS
}
