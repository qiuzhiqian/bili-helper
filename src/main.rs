use std::process::{Command, Stdio};
use std::io::Read;
use walkdir::WalkDir;

use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PageData {
    part: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct EntryData {
    media_type: i32,
    title: String,
    page_data: PageData,
}

fn main() -> std::io::Result<()> {
    worker(".")?;

    return Ok(());
}

fn worker(root: &str) -> std::io::Result<()> {
    for entry in WalkDir::new(root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let f_path = entry.path().to_str().expect("path is invalid.");
            if f_path.ends_with("entry.json") {
                println!("{}", f_path);
                let f = std::fs::File::open(f_path).unwrap();
                let values: EntryData = serde_json::from_reader(f)?;
                println!("{:#?}", values);

                if values.media_type == 2 {
                    let name = format!("{}-{}",values.title, values.page_data.part);
                    project_handler(entry.path().parent().expect("can not find parent").to_str().expect("path is invalid"), &name)?;
                }
            }
        }
    }

    Ok(())
}

fn project_handler(path: &str,name: &str) -> std::io::Result<()> {
    let mut inputs = Vec::new();
    for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let f_path = entry.path().to_str().expect("path is invalid.");
            if f_path.ends_with(".m4s") {
                println!("{}", f_path);
                inputs.push(f_path.to_string());
            }
        }
    }

    let input = inputs.iter().map(|val| {val.as_str()}).collect();
    let output = format!("{}/{}.mp4", path, name);
    println!("ffmpeg {:?} to {}", input, output);
    do_ffmpeg(input, &output)?;
    Ok(())
}

// audio: audio.m4s video video.m4s output: output.mp4
fn do_ffmpeg(inputs: Vec<&str>, output: &str) -> std::io::Result<()> {
    let mut input_args = Vec::new();
    for i in inputs {
        input_args.push("-i");
        input_args.push(i);
    }

    let child = Command::new("ffmpeg")
        .args(input_args)
        .args(["-codec", "copy"])
        .arg(output)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start ffmpeg process");

    // Note that `echo_child` is moved here, but we won't be needing
    // `echo_child` anymore
    let mut child_out = child.stdout.expect("Failed to open echo stdout");

    loop {
        let mut buffer = String::new();

        let data_size = child_out.read_to_string(&mut buffer)?;
        if data_size == 0 {
            break;
        }

        println!("{}", buffer);
    }
    Ok(())
}
