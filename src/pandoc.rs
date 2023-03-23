use pandoc::InputKind;
use pandoc::OutputKind;
use pandoc::PandocOutput::*;
use rand::{distributions::Alphanumeric, Rng};
use std::io::Write;
use std::ops::Add;
use std::process::exit;
use regex::Regex;

// repair_md_katex is done when calling for the md file
pub fn convert_md_to_html(str: String) -> String {
    let mut pandoc = pandoc::new();

    pandoc.set_input(InputKind::Pipe(str));
    pandoc.set_output(OutputKind::Pipe);
    pandoc.set_output_format(pandoc::OutputFormat::Html, Vec::new());

    //pandoc.add_option(pandoc::PandocOption::Katex(None));


    pandoc.set_input_format(pandoc::InputFormat::Commonmark, vec!(pandoc::MarkdownExtension::TexMathDollars));
    let y = pandoc.execute().unwrap();
    match y {
        ToBuffer(x) => {
            // Feather notes replaces this because of xml and html merging
            let mut x_fixed = x.replace("<", "&lt;");
            // This doesnt work here: x_fixed = x_fixed.replace("&amp;", "&"); // IDK?
            write_debug_file("", x_fixed.clone(), ".html");
            return x_fixed;
        }
        _ => error!("Pandoc error!"),
    }
    exit(-1);
}

pub fn convert_html_to_md(str: String) -> String {
    let mut pandoc = pandoc::new();

    let mut str_converted = str.replace("&lt;", "<");

    pandoc.set_input(InputKind::Pipe(str_converted));
    pandoc.set_output(OutputKind::Pipe);
    pandoc.set_output_format(pandoc::OutputFormat::Commonmark, Vec::new());
    pandoc.set_input_format(pandoc::InputFormat::Html, Vec::new());
    let y = pandoc.execute().unwrap();
    match y {
        ToBuffer(x) => {
            write_debug_file("", x.clone(), ".md");
            return x;
        }
        _ => error!("Pandoc error!"),
    }
    exit(-1);
}

// For a good reason quick_xml::se::to_string converts & to &amp;. Here we need to avoid that because html thing
pub fn final_touches_xml(mut xml: String) -> String{
    xml = (*xml.replace("&amp;", "&")).to_string();

    xml
}

pub fn write_debug_file(title: &str, content: String, extension: &str) {
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

        file_name += extension;

        debug!("Writing file {} with body in it", file_name);
        std::fs::remove_file(file_name.clone()); // no unwrap
        let mut file = std::fs::File::create(file_name).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }
}

// https://pandoc.org/MANUAL.html
// while the closing $ must have a non-space character immediately to its left, and must not be followed immediately by a digit.
// A bit of hacky and mess with this regex, but hey it works
pub fn repair_md_katex(md: String) -> String {
    let mut converted_md = md.clone();

    // ChatGPT:
    // (?<!\$)\${1,2}([\s\S]*?)(?<!\\)\${1,2}(?!\$) also (?s)\$\$.*?\$\$|\$.*?\$
    let re_math = Regex::new(r"(?s)\$\$.*?\$\$|\$.*?\$").unwrap();

    for cap in re_math.captures_iter(&md) {
        info!("Captured katex: {:#?}", cap);

        let mut str_katex: &str = &cap[0];

        let tmp = str_katex.replace('\n', "");

        str_katex = &tmp;

        let mut double_dollars = false;
        if str_katex.chars().nth(0).unwrap() == '$' && str_katex.chars().nth(1).unwrap() == '$' {
            double_dollars = true;
            str_katex = str_katex.split_at(str_katex.len() - 2).0;
        } else {
            str_katex = str_katex.split_at(str_katex.len() - 1).0;
        }

        // \s+$
        let re_spaces = Regex::new(r"\s+$").unwrap();

        let mut str_katex_cleaner = re_spaces.replace_all(str_katex, "").to_string();

        if double_dollars {
            str_katex_cleaner.push_str("$$");
        } else {
            str_katex_cleaner.push('$');
        }

        info!("double_dollars: {}", double_dollars);

        info!("Repaired katex: {:#?}", str_katex_cleaner);

        converted_md = converted_md.replace(&cap[0], &str_katex_cleaner);
    }

    converted_md

    // https://stackoverflow.com/questions/7525977/how-to-write-fraction-value-using-html
    // Make frac show as html sup
}
