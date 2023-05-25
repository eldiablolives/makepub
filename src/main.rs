use std::env;
use std::io;
use std::fs::{self, DirEntry};
use std::io::Write;
use std::path::{Path, PathBuf};
use compress::compress_epub;
use serde_yaml;
use serde_derive::Deserialize;
use pulldown_cmark::{html, Options, Parser};
use uuid::Uuid;
use chrono::prelude::*;

mod preprocess;
mod compress;

#[derive(Debug, Deserialize)]
struct EpubInfo {
    id: Option<String>,
    name: String,
    author: String,
    title: String,
    start: Option<String>,
    start_title: Option<String>,
    fonts: Option<Vec<String>>
}

#[derive(Clone)]
struct Page {
    name: String,
    file: String,
    title: String,
    body: String,
}
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


fn read_yaml_file<T: serde::de::DeserializeOwned>(file_path: &str) -> Result<T, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file_path)?;
    let result = serde_yaml::from_str(&contents)?;
    Ok(result)
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

fn create_toc_xhtml(epub_info: &EpubInfo, pages: &[Page], dest_folder: &str) {
    // Create the destination path for toc.xhtml
    let toc_path = PathBuf::from(dest_folder).join("OPS").join("toc.xhtml");

    // Generate the content of toc.xhtml
    let mut toc_content = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<html xml:lang="en" xmlns:epub="http://www.idpf.org/2007/ops" xmlns="http://www.w3.org/1999/xhtml">
<head>
    <meta charset="UTF-8" />
    <title>Table of Contents</title>
    <link rel="stylesheet" href="css/book.css" type="text/css" />
    <meta name="EPB-UUID" content="" />
</head>
<body>
    <nav id="toc" role="doc-toc" epub:type="toc">
    <ol class="s2">
"#);

    // Iterate over the pages and generate <li> tags for pages with non-empty titles
    for page in pages {
        if !page.title.trim().is_empty() {
            let page_link = format!("{}.xhtml", page.file);
            let li_tag = format!(
                r#"        <li><a href="{}">{}</a></li>"#,
                page_link, page.title
            );
            toc_content.push_str(&li_tag);
            toc_content.push('\n');
        }
    }

    // Replace the content of the EPB-UUID meta tag
    let epub_uuid = epub_info.id.as_deref().unwrap_or("");
    toc_content = toc_content.replace(r#"meta name="EPB-UUID" content=""#, &format!(r#"meta name="EPB-UUID" content="{}"#, epub_uuid));

    // Add the closing tags to toc_content
    toc_content.push_str(r#"    </ol>
    </nav>
</body>
</html>
"#);

    // Write the toc.xhtml content to the destination file
    fs::write(&toc_path, toc_content)
        .unwrap_or_else(|err| eprintln!("Error writing toc.xhtml file: {}", err));
}


fn create_xhtml_files(epub_info: &EpubInfo, pages: &[Page], dest_folder: &str) {
    for page in pages {
        let file_name = format!("{}.xhtml", page.file);
        let file_path = Path::new(dest_folder).join("OPS/content").join(&file_name);
        let xhtml_content = format!(
            r#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <meta name="EPB-UUID" content="{}" />
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="css/book.css" type="text/css" />
</head>
<body>
    {}
</body>
</html>
"#,
            page.title,
            epub_info.id.as_ref().unwrap_or(&"".to_string()),
            page.body
        );

        fs::write(&file_path, xhtml_content)
            .unwrap_or_else(|err| eprintln!("Error writing XHTML file: {}", err));
    }
}

fn create_epub(dest_path: &Path, epub_info: &EpubInfo, pages: &Vec<Page>) {
    // Create the destination folder if it doesn't exist
    // if let Some(parent_dir) = dest_path.parent() {
    //     fs::create_dir_all(parent_dir).expect("Failed to create destination folder");
    // }

    fs::create_dir_all(dest_path).expect("Failed to create destination folder");

    // Create the necessary subdirectories within the EPUB structure
    let epub_folders = vec!["META-INF", "OPS", "OPS/content"]; //
    for folder in &epub_folders {
        let folder_path = dest_path.join(folder);
        fs::create_dir(&folder_path).expect("Failed to create folder");

        // Create the core skeleton files in the appropriate folders
        match *folder {
            "META-INF" => {
                create_file(&folder_path.join("container.xml"), create_container_xml_content());
                
                if let Some(_) = epub_info.fonts {
                    create_file(&folder_path.join("com.apple.ibooks.display-options.xml"), create_apple_xml_meta());
                }
            }
            "OPS" => {
                create_file(&folder_path.join("epb.opf"), create_content_opf_content(epub_info, &pages));
                create_file(&folder_path.join("epb.ncx"), create_toc_ncx_content(epub_info, &pages));
            }
            _ => {}
        }
    }

    println!("Uncompressed skeleton EPUB structure created at: {:?}", dest_path);
}

fn create_file(file_path: &Path, file_content: String) {
    let mut file = fs::File::create(file_path).expect("Failed to create file");
    file.write_all(file_content.as_bytes())
        .expect("Failed to write to file");
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

fn sanitize_name(input: &str) -> String {
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

fn extract_title(markdown_content: &str) -> String {
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

fn get_file_name(source: &str) -> String {
    let file_path = Path::new(source);
    file_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(|name| name.to_string())
        .unwrap_or_else(|| String::new())
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


fn create_content_opf_content(epub_info: &EpubInfo, pages: &[Page]) -> String {
    let book_id = match &epub_info.id {
        Some(id) => id,
        None => "",
    };

    let manifest_items = pages
        .iter()
        .enumerate()
        .map(|(index, page)| {
            format!(
                r#"<item id="item-{}" href="{}.xhtml" media-type="application/xhtml+xml" />"#,
                index + 1,
                page.file
            )
        })
        .collect::<Vec<String>>()
        .join("\n");

    let spine_items = pages
        .iter()
        .enumerate()
        .map(|(index, page)| format!("<itemref idref=\"item-{}\" />", index + 1))
        .collect::<Vec<String>>()
        .join("\n");

    let modified = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    // create a manifest entry for each font file
    let font_items = match &epub_info.fonts {
        Some(fonts) => fonts
            .iter()
            .enumerate()
            .map(|(index, font)| {
                format!(
                    r#"<item href="fonts/{}" id="font{}" media-type="application/x-font-otf"/>"#,
                    font, index + 1
                )
            })
            .collect::<Vec<String>>()
            .join("\n"),
        None => String::new(),
    };

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="BookID">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="BookID">{}</dc:identifier>
    <dc:title>{}</dc:title>
    <dc:creator>{}</dc:creator>
    <dc:language>en</dc:language>
    <meta property="dcterms:modified">{}</meta>
  </metadata>
  <manifest>
    <item id="toc" href="toc.xhtml" media-type="application/xhtml+xml" properties="nav"/>

    {}

    <item id="ncx" href="epb.ncx" media-type="application/x-dtbncx+xml"/>
    <item id="cover-image" href="images/cover.jpg" media-type="image/jpeg"/>
    <item id="stylesheet" href="css/book.css" media-type="text/css"/>
    {}

</manifest>
  <spine toc="ncx">
    {}
  </spine>
</package>
"#,
        book_id,
        epub_info.title,
        epub_info.author,
        modified,
        manifest_items,
        font_items,
        spine_items
    )
}

fn create_container_xml_content() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OPS/epb.opf" media-type="application/oebps-package+xml" />
  </rootfiles>
</container>
"#
    .to_string()
}

fn create_apple_xml_meta() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
    <display_options>
        <platform name="*">
            <option name="specified-fonts">true</option>
        </platform>
    </display_options>
    "#
    .to_string()
}

fn create_toc_ncx_content(epub_info: &EpubInfo, pages: &[Page]) -> String {
    let nav_map = pages
        .iter()
        .enumerate()
        .filter(|(_, page)| !page.title.trim().is_empty())
        .fold((String::new(), 1), |(acc, play_order), (index, page)| {
            (
                format!(
                    "{}<navPoint id=\"navpoint-{}\" playOrder=\"{}\">
                        <navLabel>
                            <text>{}</text>
                        </navLabel>
                        <content src=\"{}.xhtml\"/>
                    </navPoint>\n",
                    acc,
                    index + 1,
                    play_order,
                    page.title,
                    page.file
                ),
                play_order + 1,
            )
        })
        .0;

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/" version="2005-1">
  <head>
    <meta name="dtb:uid" content="{}" />
    <meta name="dtb:depth" content="1" />
    <meta name="dtb:totalPageCount" content="0" />
    <meta name="dtb:maxPageNumber" content="0" />
  </head>
  <docTitle>
    <text>{}</text>
  </docTitle>
  <docAuthor>
    <text>{}</text>
  </docAuthor>
  <navMap>
    {}
  </navMap>
</ncx>
"#,
        epub_info.id.as_ref().unwrap_or(&"".to_string()),
        epub_info.title,
        epub_info.author,
        nav_map
    )
}

