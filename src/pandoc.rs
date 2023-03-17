use pandoc::InputKind;
use pandoc::OutputKind;
use pandoc::PandocOutput::*;
use std::io::Write;
use rand::{distributions::Alphanumeric, Rng};

pub fn convert_md_to_html(str: String) -> String {
    let mut pandoc = pandoc::new();

    pandoc.set_input(InputKind::Pipe(str));
    pandoc.set_output(OutputKind::Pipe);
    pandoc.set_output_format(pandoc::OutputFormat::Html, Vec::new());
    pandoc.set_input_format(pandoc::InputFormat::Commonmark, Vec::new());
    let y = pandoc.execute().unwrap();
    match y {
        ToBuffer(x) => {
            // Feather notes replaces this because of xml and html merging
            let x_fixed = x.replace("<", "&lt;");
            return x_fixed
        },
        _ => error!("Pandoc wring output kind"),
    }
    String::new()
}

pub fn convert_html_to_md(str: String) -> String {
    let mut pandoc = pandoc::new();

    pandoc.set_input(InputKind::Pipe(str.replace("&lt;", "<")));
    pandoc.set_output(OutputKind::Pipe);
    pandoc.set_output_format(pandoc::OutputFormat::Commonmark , Vec::new());
    pandoc.set_input_format(pandoc::InputFormat::Html, Vec::new());
    let y = pandoc.execute().unwrap();
    match y {
        ToBuffer(x) => return x,
        _ => error!("Pandoc wring output kind"),
    }
    String::new()
}

pub fn write_debug_file(title: &str, content: String) {
    if log_enabled!(log::Level::Debug) {
        let mut file_name = String::new();

        if title.is_empty() {
            let r: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(20)
            .map(char::from)
            .collect();

            file_name = format!("convert_tests/{}", r);
        } else {
            file_name = format!("convert_tests/{}", title);
        }

        debug!("Writing file {} with body in it", file_name);
        std::fs::remove_file(file_name.clone()); // no unwrap
        let mut file = std::fs::File::create(file_name).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
}
