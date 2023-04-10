#[macro_use]
extern crate log;

use std::process::ExitCode;

mod convert_logic;
mod data_types;
mod pandoc;

use crate::convert_logic::overwrite_joplin_to_feathernotes;

use data_types::feather::*;
use data_types::joplin::*;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long, help = "Joplin web clipper access token")]
    token: String,
    #[arg(
        short,
        long,
        help = "Feather notes file to operate on, if not specified, new file will be created"
    )]
    feather_file: Option<String>,
    #[arg(
        short,
        long,
        help = "Joplin folder ID, for which only subfolders will be \"Bridged\", if not specified, everything will be bridged"
    )]
    joplin_folder_id: Option<String>,
    #[arg(short, long, help = "Feather notes output file, specify the same file as feather-file to overwrite it", default_value_t = String::from("FeatherNotes.fnx"))]
    output_file: String,
    #[arg(
        long,
        help = "Overwrites / Writes a new feather file, converting notes from Joplin"
    )]
    overwrite_feather: bool,
}

//use crate::pandoc::{convert_md_to_html, write_debug_file};
// https://docs.rs/clap/latest/clap/ to to to
fn main() -> ExitCode {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );
    info!("Starting");

    let args = Args::parse();

    let joplin = JoplinData::new(args.token.clone()).unwrap();

    let folders_to_get: Vec<FoldersArray>;
    if let Some(folder_id) = args.joplin_folder_id {
        folders_to_get = joplin.lookup_folder(folder_id).unwrap();
    } else {
        folders_to_get = joplin.dir_list.clone(); // For root, everything
    }

    let mut feather: FeatherStruct = FeatherStruct::new();
    if let Some(feather_file) = args.feather_file {
        feather = FeatherStruct::read(feather_file).unwrap();
    }

    if args.overwrite_feather {
        overwrite_joplin_to_feathernotes(feather, joplin, folders_to_get, &args.output_file);
    } else {
        println!("Sorry, only overwrite_feather option is available now");
    }

    ExitCode::SUCCESS
}
