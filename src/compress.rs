use std::fs::{File, read_dir};
use std::io::{Write, Read};
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

fn add_dir_to_zip<P: AsRef<Path>>(dir_path: &P, prefix: &str, zip: &mut ZipWriter<File>) -> zip::result::ZipResult<()> {
    for entry in read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        let name = path.file_name().unwrap().to_str().unwrap();

        if path.is_dir() {
            add_dir_to_zip(&path, &format!("{}/{}", prefix, name), zip)?;
        } else {
            let mut file = File::open(&path)?;
            let options = FileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .unix_permissions(0o755);

            zip.start_file(format!("{}/{}", prefix, name), options)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }

    Ok(())
}

pub fn compress_epub(folder_path: &str) {
    let file_name = format!("{}.epub", folder_path);
    let path = Path::new(&file_name);
    let file = match File::create(&path) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to create the file {}: {}", file_name, e);
            return;
        },
    };

    let mut zip = ZipWriter::new(file);

    // Start with the "mimetype" file.
    let mut mimetype_file = match File::open(format!("{}/mimetype", folder_path)) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open the mimetype file: {}", e);
            return;
        },
    };

    let options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);

    if let Err(e) = zip.start_file("mimetype", options) {
        println!("Failed to start the mimetype file: {}", e);
        return;
    }

    let mut buffer = Vec::new();
    if let Err(e) = mimetype_file.read_to_end(&mut buffer) {
        println!("Failed to read the mimetype file: {}", e);
        return;
    };

    if let Err(e) = zip.write_all(&buffer) {
        println!("Failed to write the mimetype file: {}", e);
        return;
    }

    // Add the contents of the META-INF and OPS directories.
    for dir in ["META-INF", "OPS"].iter() {
        let dir_path = format!("{}/{}", folder_path, dir);
        if let Err(e) = add_dir_to_zip(&dir_path, dir, &mut zip) {
            println!("Failed to add directory {}: {}", dir, e);
            return;
        }
    }

    if let Err(e) = zip.finish() {
        println!("Failed to finish the zip: {}", e);
    }
}
