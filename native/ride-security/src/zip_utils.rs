/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! ZIP archive utilities — Rust port of `src/vs/base/node/zip.ts`.
//! Create and extract ZIP archives with filtering and progress.

use napi_derive::napi;
use napi::bindgen_prelude::*;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[napi(object)]
pub struct ZipEntry {
    pub name: String,
    pub size: f64,
    pub compressed_size: f64,
    pub is_directory: bool,
}

#[napi(object)]
pub struct ZipExtractOptions {
    pub overwrite: Option<bool>,
    pub source_path: Option<String>,
}

/// List entries in a ZIP archive.
#[napi]
pub fn zip_list(zip_path: String) -> Result<Vec<ZipEntry>> {
    let file = fs::File::open(&zip_path)
        .map_err(|e| Error::from_reason(format!("Cannot open zip: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| Error::from_reason(format!("Invalid zip: {}", e)))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i)
            .map_err(|e| Error::from_reason(format!("Zip entry error: {}", e)))?;
        entries.push(ZipEntry {
            name: entry.name().to_string(),
            size: entry.size() as f64,
            compressed_size: entry.compressed_size() as f64,
            is_directory: entry.is_dir(),
        });
    }
    Ok(entries)
}

/// Extract a ZIP archive to a target directory.
#[napi]
pub fn zip_extract(zip_path: String, target_path: String, options: Option<ZipExtractOptions>) -> Result<u32> {
    let opts = options.unwrap_or(ZipExtractOptions { overwrite: None, source_path: None });
    let overwrite = opts.overwrite.unwrap_or(false);
    let source_filter = opts.source_path.unwrap_or_default();

    let file = fs::File::open(&zip_path)
        .map_err(|e| Error::from_reason(format!("Cannot open zip: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| Error::from_reason(format!("Invalid zip: {}", e)))?;

    let target = Path::new(&target_path);
    if overwrite {
        let _ = fs::remove_dir_all(target);
    }
    fs::create_dir_all(target)
        .map_err(|e| Error::from_reason(format!("Cannot create target: {}", e)))?;

    let mut extracted = 0u32;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| Error::from_reason(format!("Zip entry error: {}", e)))?;

        let entry_name = entry.name().to_string();

        // Apply source path filter
        if !source_filter.is_empty() && !entry_name.starts_with(&source_filter) {
            continue;
        }

        // Strip source path prefix
        let relative_name = if !source_filter.is_empty() {
            entry_name.strip_prefix(&source_filter).unwrap_or(&entry_name)
        } else {
            &entry_name
        };

        if relative_name.is_empty() { continue; }

        let out_path = target.join(relative_name);

        // Security: prevent path traversal
        if !out_path.starts_with(target) {
            continue;
        }

        if entry.is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut outfile = fs::File::create(&out_path)
                .map_err(|e| Error::from_reason(format!("Cannot create file: {}", e)))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| Error::from_reason(format!("Extract error: {}", e)))?;

            // Set permissions on unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = entry.unix_mode() {
                    fs::set_permissions(&out_path, fs::Permissions::from_mode(mode)).ok();
                }
            }
            extracted += 1;
        }
    }
    Ok(extracted)
}

/// Create a ZIP archive from files.
#[napi]
pub fn zip_create(zip_path: String, files: Vec<String>, base_dir: Option<String>) -> Result<u32> {
    let base = base_dir.map(|b| std::path::PathBuf::from(b));
    let file = fs::File::create(&zip_path)
        .map_err(|e| Error::from_reason(format!("Cannot create zip: {}", e)))?;
    let mut zip_writer = zip::ZipWriter::new(file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut count = 0u32;
    for file_path in &files {
        let p = Path::new(file_path);
        if !p.exists() { continue; }

        let archive_name = if let Some(ref base) = base {
            p.strip_prefix(base).unwrap_or(p).to_string_lossy().to_string()
        } else {
            p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
        };

        if p.is_dir() {
            zip_writer.add_directory(&archive_name, options)
                .map_err(|e| Error::from_reason(format!("Zip error: {}", e)))?;
            // Recurse into directory
            add_dir_to_zip(&mut zip_writer, p, &archive_name, options)?;
        } else {
            zip_writer.start_file(&archive_name, options)
                .map_err(|e| Error::from_reason(format!("Zip error: {}", e)))?;
            let mut f = fs::File::open(p)
                .map_err(|e| Error::from_reason(format!("Cannot read file: {}", e)))?;
            std::io::copy(&mut f, &mut zip_writer)
                .map_err(|e| Error::from_reason(format!("Zip write error: {}", e)))?;
            count += 1;
        }
    }

    zip_writer.finish()
        .map_err(|e| Error::from_reason(format!("Zip finalize error: {}", e)))?;
    Ok(count)
}

fn add_dir_to_zip<W: Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    dir: &Path,
    prefix: &str,
    options: zip::write::SimpleFileOptions,
) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|e| Error::from_reason(e.to_string()))? {
        let entry = entry.map_err(|e| Error::from_reason(e.to_string()))?;
        let path = entry.path();
        let name = format!("{}/{}", prefix, entry.file_name().to_string_lossy());

        if path.is_dir() {
            zip.add_directory(&name, options)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            add_dir_to_zip(zip, &path, &name, options)?;
        } else {
            zip.start_file(&name, options)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            let mut f = fs::File::open(&path)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            std::io::copy(&mut f, zip)
                .map_err(|e| Error::from_reason(e.to_string()))?;
        }
    }
    Ok(())
}

/// Read a single file from a ZIP archive as a buffer.
#[napi]
pub fn zip_read_file(zip_path: String, file_name: String) -> Result<Buffer> {
    let file = fs::File::open(&zip_path)
        .map_err(|e| Error::from_reason(format!("Cannot open zip: {}", e)))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| Error::from_reason(format!("Invalid zip: {}", e)))?;
    let mut entry = archive.by_name(&file_name)
        .map_err(|e| Error::from_reason(format!("File not found in zip: {}", e)))?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf)
        .map_err(|e| Error::from_reason(format!("Read error: {}", e)))?;
    Ok(Buffer::from(buf))
}

/// Read a single file from a ZIP archive as a string.
#[napi]
pub fn zip_read_file_string(zip_path: String, file_name: String) -> Result<String> {
    let buf = zip_read_file(zip_path, file_name)?;
    String::from_utf8(buf.to_vec())
        .map_err(|e| Error::from_reason(format!("Invalid UTF-8: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_create_and_extract() {
        let tmp = std::env::temp_dir();
        let src_dir = tmp.join("ride_zip_test_src");
        let _ = fs::remove_dir_all(&src_dir);
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("a.txt"), "hello").unwrap();
        fs::write(src_dir.join("b.txt"), "world").unwrap();

        let zip_path = tmp.join("ride_zip_test.zip").to_string_lossy().to_string();
        let files = vec![
            src_dir.join("a.txt").to_string_lossy().to_string(),
            src_dir.join("b.txt").to_string_lossy().to_string(),
        ];
        let count = zip_create(zip_path.clone(), files, None).unwrap();
        assert_eq!(count, 2);

        let entries = zip_list(zip_path.clone()).unwrap();
        assert_eq!(entries.len(), 2);

        let extract_dir = tmp.join("ride_zip_test_extract");
        let _ = fs::remove_dir_all(&extract_dir);
        let extracted = zip_extract(zip_path.clone(), extract_dir.to_string_lossy().to_string(), None).unwrap();
        assert_eq!(extracted, 2);

        // Cleanup
        let _ = fs::remove_dir_all(&src_dir);
        let _ = fs::remove_dir_all(&extract_dir);
        let _ = fs::remove_file(&zip_path);
    }

    #[test]
    fn test_zip_read_file() {
        let tmp = std::env::temp_dir();
        let src = tmp.join("ride_zip_read_test.txt");
        fs::write(&src, "test content").unwrap();

        let zip_path = tmp.join("ride_zip_read_test.zip").to_string_lossy().to_string();
        zip_create(zip_path.clone(), vec![src.to_string_lossy().to_string()], None).unwrap();

        let content = zip_read_file_string(zip_path.clone(), "ride_zip_read_test.txt".into()).unwrap();
        assert_eq!(content, "test content");

        let _ = fs::remove_file(&src);
        let _ = fs::remove_file(&zip_path);
    }
}
