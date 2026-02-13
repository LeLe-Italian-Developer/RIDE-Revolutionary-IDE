/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Fast compression engine using ZSTD and ZIP formats.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

/// Compression statistics.
#[napi(object)]
pub struct CompressionStats {
    pub original_size: f64,
    pub compressed_size: f64,
    pub ratio: f64,
    pub duration_ms: f64,
}

/// Archive entry information.
#[napi(object)]
#[derive(Clone)]
pub struct ArchiveEntry {
    pub name: String,
    pub size: f64,
    pub compressed_size: f64,
    pub is_directory: bool,
}

/// Compress data using ZSTD algorithm.
///
/// # Arguments
/// * `data` - The data to compress (as UTF-8 string)
/// * `level` - Compression level 1-22 (default: 3, higher = smaller but slower)
#[napi]
pub fn compress(data: String, level: Option<i32>) -> Result<Buffer> {
    let _start = std::time::Instant::now();
    let lvl = level.unwrap_or(3);
    let compressed = zstd::encode_all(data.as_bytes(), lvl)
        .map_err(|e| Error::from_reason(format!("Compression failed: {}", e)))?;
    Ok(Buffer::from(compressed))
}

/// Decompress ZSTD-compressed data.
#[napi]
pub fn decompress(data: Buffer) -> Result<String> {
    let decompressed = zstd::decode_all(data.as_ref())
        .map_err(|e| Error::from_reason(format!("Decompression failed: {}", e)))?;
    String::from_utf8(decompressed)
        .map_err(|e| Error::from_reason(format!("Invalid UTF-8 data: {}", e)))
}

/// Compress a file using ZSTD.
///
/// # Arguments
/// * `input_path` - Path to the file to compress
/// * `output_path` - Path for the compressed output
/// * `level` - Compression level (default: 3)
#[napi]
pub fn compress_file(input_path: String, output_path: String, level: Option<i32>) -> Result<CompressionStats> {
    let start = std::time::Instant::now();
    let lvl = level.unwrap_or(3);

    let input = fs::read(&input_path)
        .map_err(|e| Error::from_reason(format!("Failed to read {}: {}", input_path, e)))?;
    let original_size = input.len() as f64;

    let compressed = zstd::encode_all(input.as_slice(), lvl)
        .map_err(|e| Error::from_reason(format!("Compression failed: {}", e)))?;
    let compressed_size = compressed.len() as f64;

    fs::write(&output_path, &compressed)
        .map_err(|e| Error::from_reason(format!("Failed to write {}: {}", output_path, e)))?;

    Ok(CompressionStats {
        original_size,
        compressed_size,
        ratio: if original_size > 0.0 { compressed_size / original_size } else { 0.0 },
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

/// Decompress a ZSTD file.
#[napi]
pub fn decompress_file(input_path: String, output_path: String) -> Result<CompressionStats> {
    let start = std::time::Instant::now();

    let input = fs::read(&input_path)
        .map_err(|e| Error::from_reason(format!("Failed to read {}: {}", input_path, e)))?;
    let compressed_size = input.len() as f64;

    let decompressed = zstd::decode_all(input.as_slice())
        .map_err(|e| Error::from_reason(format!("Decompression failed: {}", e)))?;
    let original_size = decompressed.len() as f64;

    fs::write(&output_path, &decompressed)
        .map_err(|e| Error::from_reason(format!("Failed to write {}: {}", output_path, e)))?;

    Ok(CompressionStats {
        original_size,
        compressed_size,
        ratio: if original_size > 0.0 { compressed_size / original_size } else { 0.0 },
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

/// List contents of a ZIP archive.
#[napi]
pub fn list_archive(archive_path: String) -> Result<Vec<ArchiveEntry>> {
    let file = fs::File::open(&archive_path)
        .map_err(|e| Error::from_reason(format!("Failed to open {}: {}", archive_path, e)))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| Error::from_reason(format!("Invalid archive: {}", e)))?;

    let mut entries = Vec::new();
    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index_raw(i) {
            entries.push(ArchiveEntry {
                name: entry.name().to_string(),
                size: entry.size() as f64,
                compressed_size: entry.compressed_size() as f64,
                is_directory: entry.is_dir(),
            });
        }
    }
    Ok(entries)
}

/// Extract a ZIP archive to a directory.
#[napi]
pub fn extract_archive(archive_path: String, output_dir: String) -> Result<u32> {
    let file = fs::File::open(&archive_path)
        .map_err(|e| Error::from_reason(format!("Failed to open {}: {}", archive_path, e)))?;

    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| Error::from_reason(format!("Invalid archive: {}", e)))?;

    let out_path = Path::new(&output_dir);
    fs::create_dir_all(out_path)
        .map_err(|e| Error::from_reason(format!("Failed to create dir: {}", e)))?;

    let mut extracted = 0u32;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| Error::from_reason(format!("Failed to read entry: {}", e)))?;

        let entry_path = out_path.join(entry.name());

        // Security: prevent path traversal
        if !entry_path.starts_with(out_path) {
            continue;
        }

        if entry.is_dir() {
            fs::create_dir_all(&entry_path).ok();
        } else {
            if let Some(parent) = entry_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut outfile = fs::File::create(&entry_path)
                .map_err(|e| Error::from_reason(format!("Failed to create {}: {}", entry_path.display(), e)))?;
            std::io::copy(&mut entry, &mut outfile)
                .map_err(|e| Error::from_reason(format!("Failed to write: {}", e)))?;
            extracted += 1;
        }
    }

    Ok(extracted)
}

/// Create a ZIP archive from a directory.
#[napi]
pub fn create_archive(source_dir: String, output_path: String) -> Result<CompressionStats> {
    let start = std::time::Instant::now();
    let src = Path::new(&source_dir);

    if !src.exists() || !src.is_dir() {
        return Err(Error::from_reason(format!("Invalid directory: {}", source_dir)));
    }

    let file = fs::File::create(&output_path)
        .map_err(|e| Error::from_reason(format!("Failed to create {}: {}", output_path, e)))?;

    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut original_size = 0f64;
    add_dir_to_zip(&mut zip, src, src, &options, &mut original_size)?;

    zip.finish()
        .map_err(|e| Error::from_reason(format!("Failed to finalize archive: {}", e)))?;

    let compressed_size = fs::metadata(&output_path).map(|m| m.len() as f64).unwrap_or(0.0);

    Ok(CompressionStats {
        original_size,
        compressed_size,
        ratio: if original_size > 0.0 { compressed_size / original_size } else { 0.0 },
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

fn add_dir_to_zip(
    zip: &mut zip::ZipWriter<fs::File>,
    dir: &Path,
    base: &Path,
    options: &zip::write::SimpleFileOptions,
    total_size: &mut f64,
) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|e| Error::from_reason(e.to_string()))? {
        let entry = entry.map_err(|e| Error::from_reason(e.to_string()))?;
        let path = entry.path();
        let relative = path.strip_prefix(base).unwrap_or(&path);
        let name = relative.to_string_lossy().to_string();

        if path.is_dir() {
            zip.add_directory(&name, *options)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            add_dir_to_zip(zip, &path, base, options, total_size)?;
        } else {
            let mut content = Vec::new();
            fs::File::open(&path)
                .map_err(|e| Error::from_reason(e.to_string()))?
                .read_to_end(&mut content)
                .map_err(|e| Error::from_reason(e.to_string()))?;

            *total_size += content.len() as f64;
            zip.start_file(&name, *options)
                .map_err(|e| Error::from_reason(e.to_string()))?;
            zip.write_all(&content)
                .map_err(|e| Error::from_reason(e.to_string()))?;
        }
    }
    Ok(())
}

/// Get the compression ratio of a ZSTD-compressed buffer vs original.
#[napi]
pub fn estimate_compression(data: String, level: Option<i32>) -> Result<CompressionStats> {
    let start = std::time::Instant::now();
    let lvl = level.unwrap_or(3);
    let original = data.len() as f64;
    let compressed = zstd::encode_all(data.as_bytes(), lvl)
        .map_err(|e| Error::from_reason(e.to_string()))?;
    let comp_size = compressed.len() as f64;

    Ok(CompressionStats {
        original_size: original,
        compressed_size: comp_size,
        ratio: if original > 0.0 { comp_size / original } else { 0.0 },
        duration_ms: start.elapsed().as_secs_f64() * 1000.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress() {
        let original = "Hello, RIDE! This is a test of ZSTD compression. ".repeat(100);
        let compressed = compress(original.clone(), Some(3)).unwrap();
        assert!(compressed.len() < original.len());

        let decompressed = decompress(compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_file_compression() {
        let tmp_in = std::env::temp_dir().join("ride_test_compress_in.txt");
        let tmp_out = std::env::temp_dir().join("ride_test_compress_out.zst");
        let tmp_dec = std::env::temp_dir().join("ride_test_decompress_out.txt");

        let data = "Test data for file compression ".repeat(1000);
        fs::write(&tmp_in, &data).unwrap();

        let stats = compress_file(tmp_in.to_str().unwrap().to_string(), tmp_out.to_str().unwrap().to_string(), None).unwrap();
        assert!(stats.ratio < 1.0);

        decompress_file(tmp_out.to_str().unwrap().to_string(), tmp_dec.to_str().unwrap().to_string()).unwrap();
        assert_eq!(fs::read_to_string(&tmp_dec).unwrap(), data);

        let _ = fs::remove_file(&tmp_in);
        let _ = fs::remove_file(&tmp_out);
        let _ = fs::remove_file(&tmp_dec);
    }

    #[test]
    fn test_zip_create_and_extract() {
        let src = std::env::temp_dir().join("ride_test_zip_src");
        let out = std::env::temp_dir().join("ride_test.zip");
        let ext = std::env::temp_dir().join("ride_test_zip_ext");

        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&ext);
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("a.txt"), "Hello").unwrap();
        fs::write(src.join("b.txt"), "World").unwrap();

        let stats = create_archive(src.to_str().unwrap().to_string(), out.to_str().unwrap().to_string()).unwrap();
        assert!(stats.compressed_size > 0.0);

        let entries = list_archive(out.to_str().unwrap().to_string()).unwrap();
        assert!(entries.len() >= 2);

        let count = extract_archive(out.to_str().unwrap().to_string(), ext.to_str().unwrap().to_string()).unwrap();
        assert!(count >= 2);

        let _ = fs::remove_dir_all(&src);
        let _ = fs::remove_dir_all(&ext);
        let _ = fs::remove_file(&out);
    }

    #[test]
    fn test_estimate_compression() {
        let data = "Repeating data for estimation. ".repeat(500);
        let stats = estimate_compression(data, None).unwrap();
        assert!(stats.ratio < 0.5);
    }
}
