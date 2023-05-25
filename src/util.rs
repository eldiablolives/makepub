use std::fs::{self, DirEntry};
use std::io::Write;
use std::path::{Path, PathBuf};
use serde_yaml;

pub fn create_file(file_path: &Path, file_content: String) {
    let mut file = fs::File::create(file_path).expect("Failed to create file");
    file.write_all(file_content.as_bytes())
        .expect("Failed to write to file");
}


pub fn read_yaml_file<T: serde::de::DeserializeOwned>(file_path: &str) -> Result<T, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file_path)?;
    let result = serde_yaml::from_str(&contents)?;
    Ok(result)
}

pub fn sanitize_name(input: &str) -> String {
    let mut output = input.to_lowercase();

    // replace all non-alphanumeric characters including spaces with '-'
    output = output
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // remove instances of more than one dash in a row
    while output.contains("--") {
        output = output.replace("--", "-");
    }

    // remove trailing dashes
    while output.ends_with("-") {
        output.pop();
    }

    output
}

pub fn extract_title(markdown_content: &str) -> String {
    for line in markdown_content.lines() {
        if line.starts_with("##") {
            // Extract the title from the line
            let title = line.trim_start_matches("#").trim_start().to_string();
            return title;
        }
    }
    // If no title found, return an empty string or handle as desired
    String::new()
}

pub fn get_file_name(source: &str) -> String {
    let file_path = Path::new(source);
    file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|| String::new())
}

