use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::fs;
use std::io::{self, Write};

#[napi]
pub struct TextFileService {}

#[napi]
impl TextFileService {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {}
    }

    #[napi]
    pub fn read(&self, path: String, encoding: Option<String>) -> Result<String> {
        // Basic implementation validating strictly UTF-8 for now
        // Real implementation should handle encoding logic (e.g. windows-1252)
        if let Some(enc) = encoding {
            if enc != "utf-8" && enc != "utf8" {
                return Err(Error::from_reason("Only UTF-8 encoding is currently supported in Rust layer"));
            }
        }
        
        fs::read_to_string(&path)
            .map_err(|e| Error::from_reason(format!("Failed to read file {}: {}", path, e)))
    }

    #[napi]
    pub fn write(&self, path: String, content: String, encoding: Option<String>) -> Result<()> {
        if let Some(enc) = encoding {
             if enc != "utf-8" && enc != "utf8" {
                return Err(Error::from_reason("Only UTF-8 encoding is currently supported in Rust layer"));
            }
        }

        let mut file = fs::File::create(&path)
            .map_err(|e| Error::from_reason(format!("Failed to create file {}: {}", path, e)))?;
            
        file.write_all(content.as_bytes())
            .map_err(|e| Error::from_reason(format!("Failed to write to file {}: {}", path, e)))?;
            
        Ok(())
    }
    
    #[napi]
    pub fn create(&self, path: String, content: Option<String>) -> Result<()> {
       self.write(path, content.unwrap_or_default(), None)
    }
    
    #[napi]
    pub fn exists(&self, path: String) -> bool {
        std::path::Path::new(&path).exists()
    }
}
