use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use chrono::prelude::*;

use crate::types::*;
use crate::util::create_file;

pub fn create_toc_xhtml(epub_info: &EpubInfo, pages: &[Page], dest_folder: &str) {
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
            let page_link = format!("content/{}.xhtml", page.file);
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

pub fn create_xhtml_files(epub_info: &EpubInfo, pages: &[Page], dest_folder: &str) {
    for page in pages {
        let file_name = format!("{}.xhtml", page.file);
        let file_path = Path::new(dest_folder).join("OPS/content").join(&file_name);
        
        // Use page.title if it is not empty, else use epub_info.title
        let title = if !page.title.trim().is_empty() { &page.title } else { &epub_info.title };

        let xhtml_content = format!(
            r#"<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <meta name="EPB-UUID" content="{}" />
    <meta charset="UTF-8" />
    <link rel="stylesheet" href="../css/book.css" type="text/css" />
</head>
<body>
    {}
</body>
</html>
"#,
            title,
            epub_info.id.as_ref().unwrap_or(&"".to_string()),
            page.body
        );

        fs::write(&file_path, xhtml_content)
            .unwrap_or_else(|err| eprintln!("Error writing XHTML file: {}", err));
    }
}

pub fn create_epub(dest_path: &Path, epub_info: &EpubInfo, pages: &Vec<Page>) {
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
                r#"<item id="item-{}" href="content/{}.xhtml" media-type="application/xhtml+xml" />"#,
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

    // create a manifest entry for each image
    let image_items = match &epub_info.images {
        Some(images) => images
            .iter()
            .map(|image| {
                let image_id = image.split('.').next().unwrap_or_default();
                format!(
                    r#"<item id="{}" href="images/{}" media-type="image/jpeg"/>"#,
                    image_id, image
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
    <item id="stylesheet" href="css/book.css" media-type="text/css"/>
    {}
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
        image_items,
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
                        <content src=\"content/{}.xhtml\"/>
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
