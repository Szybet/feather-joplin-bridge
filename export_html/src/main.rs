use std::env;
use std::fs::{File, create_dir_all};
use std::io::{BufReader, BufRead, Write};
use regex::Regex;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Please provide the path to an HTML file as the first argument.");
    }
    let file_path = &args[1];

    let file = File::open(file_path).unwrap();
    let reader = BufReader::new(file);

    let mut new_lines = Vec::new();

    let img_regex = Regex::new(r#"(?s)&lt;img.*?&gt;"#).unwrap();
    let base64_regex = Regex::new(r#"base64,([^&"]+)"#).unwrap();

    let file_path_buf = PathBuf::from(file_path);
    let file_name = file_path_buf.file_stem().unwrap().to_str().unwrap();
    let output_dir = format!("{}_exports", file_name);
    create_dir_all(&output_dir).unwrap();
    println!("Exporting images to directory: {}", output_dir);

    let mut image_name_count = 0;
    for (i, line) in reader.lines().enumerate() {
        let line = line.unwrap();
        let mut new_line = line.clone();
        for capture in img_regex.captures_iter(&line) {
            let img_str = capture.get(0).unwrap().as_str();
            println!("Captured line {}: {}", i + 1, line);
            if let Some(base64_capture) = base64_regex.captures(img_str) {
                image_name_count += 1;
                let base64_str = base64_capture.get(1).unwrap().as_str();
                let file_name = format!("image{}.png", image_name_count);
                let file_path = format!("{}/{}", &output_dir, &file_name);
                let decoded_image = base64::decode(base64_str).unwrap();
                let mut file = File::create(&file_path).unwrap();
                file.write_all(&decoded_image).unwrap();
                println!("Exported image to file: {}", file_path);
                let new_img_tag = format!("&lt;img src=\"{}\"&gt;", file_path);
                new_line = new_line.replace(img_str, &new_img_tag);
            }
        }
        new_lines.push(new_line);
    }

    let mut output_file = File::create(format!("{}_exportsSupport.fnx", file_name)).unwrap();
    for line in new_lines {
        output_file.write_all(line.as_bytes()).unwrap();
        output_file.write_all(b"\n").unwrap();
    }
}
