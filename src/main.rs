use std::env;
use std::io;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use compress::compress_epub;
use pulldown_cmark::{html, Options, Parser};
use uuid::Uuid;

mod preprocess;
mod compress;
mod types;
mod util;
mod epub;

use types::{EpubInfo, Page};
use util::*;
use epub::*;

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Determine the folder path
    let folder_path = if args.len() >= 2 {
        &args[1]
    } else {
        eprintln!("Error: Folder path argument not specified");
        std::process::exit(1);
    };

    // Determine the YAML file path
    let yaml_path = PathBuf::from(folder_path).join("book.yaml");

    // Deserialize YAML file into EpubInfo struct
    let mut epub_info: EpubInfo = match read_yaml_file(yaml_path.to_str().unwrap()) {
        Ok(epub_info) => epub_info,
        Err(err) => {
            eprintln!("Error: Failed to read YAML file: {}", err);
            std::process::exit(1);
        },
    };


    epub_info.id = Some(Uuid::new_v4().hyphenated().to_string());

    match check_font_files(&folder_path) {
        Ok(fonts) => epub_info.fonts = fonts,
        Err(e) => println!("Error checking for font files: {}", e),
    }

    match check_image_files(&folder_path) {
        Ok(images) => epub_info.images = images,
        Err(e) => println!("Error checking for image files: {}", e),
    }
    
    // Determine the EPUB name
    let epub_name = epub_info.name.clone();

    // Determine the destination folder
    let dest_folder = if args.len() >= 3 {
        // Destination folder provided as second argument
        &args[2]
    } else {
        // Use the current folder as the destination
        "."
    };

    // Create the destination path
    let dest_path = PathBuf::from(dest_folder).join(epub_name);

    let raw_pages = process_markdown_files(&folder_path);

    let pages = rearrange_start_page(&epub_info, &raw_pages);

    create_epub(&dest_path, &epub_info, &pages);

    create_xhtml_files( &epub_info, &pages, dest_path.to_str().unwrap());

    copy_files(folder_path, dest_path.to_str().unwrap());

    create_toc_xhtml(&epub_info, &pages, dest_path.to_str().unwrap());

    create_mimetype_file(dest_path.to_str().unwrap());

    compress_epub(dest_path.to_str().unwrap());
}

fn check_font_files(folder_path: &str) -> io::Result<Option<Vec<String>>> {
    let entries: Vec<DirEntry> = fs::read_dir(Path::new(folder_path))?.collect::<Result<_, _>>()?;
    let mut font_files = Vec::new();

    for entry in entries {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "ttf" || ext == "otf" {
                    if let Some(file_name) = path.file_name() {
                        if let Some(file_name_str) = file_name.to_str() {
                            font_files.push(String::from(file_name_str));
                        }
                    }
                }
            }
        }
    }

    Ok(Some(font_files))
}

fn check_image_files(folder_path: &str) -> io::Result<Option<Vec<String>>> {
    let entries: Vec<DirEntry> = fs::read_dir(Path::new(folder_path))?.collect::<Result<_, _>>()?;
    let mut image_files = Vec::new();

    for entry in entries {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "jpg" || ext == "png" {
                    if let Some(file_name) = path.file_name() {
                        if let Some(file_name_str) = file_name.to_str() {
                            image_files.push(String::from(file_name_str));
                        }
                    }
                }
            }
        }
    }

    Ok(Some(image_files))
}

fn create_mimetype_file(dest_folder: &str) {
    let file_path = Path::new(dest_folder).join("mimetype");

    fs::write(&file_path, "application/epub+zip")
        .unwrap_or_else(|err| eprintln!("Error writing mimetype file: {}", err));
}

fn rearrange_start_page(epub_info: &EpubInfo, pages: &[Page]) -> Vec<Page> {
    let mut rearranged_pages: Vec<Page> = Vec::new();

    for page in pages {
        if let Some(start_page) = &epub_info.start {
            if page.name.trim() == start_page {
                let start_page_title = epub_info.start_title.as_deref().filter(|&title| !title.is_empty()).unwrap_or("Title page");

                rearranged_pages.push(Page {
                    name: page.name.clone(),
                    file: page.file.clone(),
                    title: start_page_title.to_string(),
                    body: page.body.clone(),
                });
            } else {
                rearranged_pages.push(page.clone());
            }
        } else {
            rearranged_pages.push(page.clone());
        }
    }
    rearranged_pages
}

fn copy_files(source_path: &str, dest_path: &str) -> io::Result<()> {
    // create necessary directories
    let ops_subdirs = ["css", "images", "fonts", "js"];
    for subdir in ops_subdirs.iter() {
        let dir = Path::new(dest_path).join(format!("OPS/{}", subdir));
        fs::create_dir_all(&dir)?;
    }

    // helper function to copy files with a certain extension to a directory
    fn copy_files_to_dir(dir: &Path, ext: &str, entries: &Vec<DirEntry>) -> io::Result<()> {
        for entry in entries {
            let path = entry.path();
            if path.is_file() && path.extension() == Some(ext.as_ref()) {
                let dest = dir.join(path.file_name().unwrap());
                fs::copy(&path, &dest)?;
            }
        }
        Ok(())
    }

    // get all entries in the source directory
    let entries: Vec<DirEntry> = fs::read_dir(Path::new(source_path))?.collect::<Result<_, _>>()?;

    // copy files to the appropriate directories
    copy_files_to_dir(&Path::new(dest_path).join("OPS/css"), "css", &entries)?;
    copy_files_to_dir(&Path::new(dest_path).join("OPS/images"), "png", &entries)?;
    copy_files_to_dir(&Path::new(dest_path).join("OPS/images"), "jpg", &entries)?;
    copy_files_to_dir(&Path::new(dest_path).join("OPS/fonts"), "ttf", &entries)?;
    copy_files_to_dir(&Path::new(dest_path).join("OPS/fonts"), "otf", &entries)?;
    copy_files_to_dir(&Path::new(dest_path).join("OPS/js"), "js", &entries)?;

    Ok(())
}

fn render_markdown_to_page(source: &str) -> Page {
    // Read the Markdown file content
    let raw_content = fs::read_to_string(source).expect("Failed to read the Markdown file");

    let markdown_content = preprocess::preprocess_markdown(&raw_content);

    // Parse the Markdown content
    let parser = Parser::new_ext(&markdown_content, Options::all());

    // Render the Markdown as XHTML
    let mut xhtml_content = String::new();
    html::push_html(&mut xhtml_content, parser);

    // Extract the title from the Markdown content
    let title = extract_title(&markdown_content);

    // Get the file name without full path and extension
    let name = get_file_name(source);

    let file = sanitize_name(&name);

    // Create a new Page instance with the extracted title, XHTML content, and file name
    Page {
        name,
        file,
        title,
        body: xhtml_content,
    }
}

fn process_markdown_files(path: &str) -> Vec<Page> {
    // Read the directory contents
    let dir_entries = fs::read_dir(path).expect("Failed to read directory");

    // Collect and sort Markdown files by name
    let mut markdown_files: Vec<PathBuf> = dir_entries
        .filter_map(Result::ok)
        .filter(|entry| {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            entry.path().is_file()
                && entry.path().extension() == Some("md".as_ref())
                && !file_name_str.starts_with('_')
        })
        .map(|entry| entry.path())
        .collect();
    markdown_files.sort();

    // Process each Markdown file
    let mut results: Vec<Page> = Vec::new();
    for file_path in markdown_files {
        let file_path_str = file_path.to_string_lossy().to_string();
        let result = render_markdown_to_page(&file_path_str);
        results.push(result);
    }

    results
}


