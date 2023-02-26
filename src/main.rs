#[macro_use]
extern crate log;

use std::env;
use std::process::ExitCode;

mod data_types;
use data_types::joplin::*;

fn main() -> ExitCode {
    env_logger::init();
    info!("Starting");
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        error!("Joplin token, feather file and main folder are needed to be provided");
        return ExitCode::FAILURE;
    }
    let path_int = 3;


    let joplin = JoplinData::new(args[1].clone());

    let foldersRoot: FoldersPure = joplin.lookup_folder(args[path_int].clone()).unwrap();


    ExitCode::SUCCESS
}
