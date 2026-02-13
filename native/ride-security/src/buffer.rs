/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Buffer utilities — Rust port of `src/vs/base/common/buffer.ts`.
//! Provides buffer creation, manipulation, encoding/decoding.

use napi_derive::napi;
use napi::bindgen_prelude::*;

/// Create a buffer filled with a byte value.
#[napi]
pub fn buffer_alloc(size: u32, fill: Option<u32>) -> Buffer {
    let f = fill.unwrap_or(0) as u8;
    Buffer::from(vec![f; size as usize])
}

/// Copy bytes from source buffer into target buffer, returning the new target.
#[napi]
pub fn buffer_copy(source: Buffer, target: Buffer, target_start: Option<u32>, source_start: Option<u32>, source_end: Option<u32>) -> Buffer {
    let ss = source_start.unwrap_or(0) as usize;
    let se = source_end.unwrap_or(source.len() as u32) as usize;
    let ts = target_start.unwrap_or(0) as usize;
    let src = &source[ss..se.min(source.len())];
    let mut tgt = target.to_vec();
    let copy_len = src.len().min(tgt.len().saturating_sub(ts));
    tgt[ts..ts + copy_len].copy_from_slice(&src[..copy_len]);
    Buffer::from(tgt)
}

/// Concatenate multiple buffers.
#[napi]
pub fn buffer_concat(buffers: Vec<Buffer>) -> Buffer {
    let total: usize = buffers.iter().map(|b| b.len()).sum();
    let mut result = Vec::with_capacity(total);
    for buf in buffers { result.extend_from_slice(&buf); }
    Buffer::from(result)
}

/// Compare two buffers.
#[napi]
pub fn buffer_compare(a: Buffer, b: Buffer) -> i32 {
    match a.as_ref().cmp(b.as_ref()) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// Check if two buffers are equal.
#[napi]
pub fn buffer_equals(a: Buffer, b: Buffer) -> bool { a.as_ref() == b.as_ref() }

/// Encode buffer to base64.
#[napi]
pub fn buffer_to_base64(buf: Buffer) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(buf.as_ref())
}

/// Decode base64 to buffer.
#[napi]
pub fn base64_to_buffer(encoded: String) -> Result<Buffer> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(encoded.as_bytes())
        .map(Buffer::from)
        .map_err(|e| Error::from_reason(format!("Invalid base64: {}", e)))
}

/// Encode buffer to hex string.
#[napi]
pub fn buffer_to_hex(buf: Buffer) -> String {
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Decode hex string to buffer.
#[napi]
pub fn hex_to_buffer(hex: String) -> Result<Buffer> {
    let bytes: std::result::Result<Vec<u8>, _> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect();
    bytes.map(Buffer::from).map_err(|e| Error::from_reason(format!("Invalid hex: {}", e)))
}

/// Encode buffer to UTF-8 string.
#[napi]
pub fn buffer_to_string(buf: Buffer) -> Result<String> {
    String::from_utf8(buf.to_vec()).map_err(|e| Error::from_reason(format!("Invalid UTF-8: {}", e)))
}

/// Encode UTF-8 string to buffer.
#[napi]
pub fn string_to_buffer(s: String) -> Buffer {
    Buffer::from(s.into_bytes())
}

/// Read a uint32 from a buffer at the given offset (big-endian).
#[napi]
pub fn read_uint32_be(buf: Buffer, offset: u32) -> Result<u32> {
    let o = offset as usize;
    if o + 4 > buf.len() { return Err(Error::from_reason("Out of bounds")); }
    Ok(u32::from_be_bytes([buf[o], buf[o+1], buf[o+2], buf[o+3]]))
}

/// Write a uint32 to a buffer at the given offset (big-endian), returning the new buffer.
#[napi]
pub fn write_uint32_be(buf: Buffer, offset: u32, value: u32) -> Result<Buffer> {
    let o = offset as usize;
    let mut b = buf.to_vec();
    if o + 4 > b.len() { return Err(Error::from_reason("Out of bounds")); }
    let bytes = value.to_be_bytes();
    b[o..o+4].copy_from_slice(&bytes);
    Ok(Buffer::from(b))
}

/// XOR two buffers (for simple obfuscation).
#[napi]
pub fn buffer_xor(a: Buffer, b: Buffer) -> Buffer {
    let len = a.len().min(b.len());
    let result: Vec<u8> = a.iter().zip(b.iter()).take(len).map(|(x, y)| x ^ y).collect();
    Buffer::from(result)
}

/// Fill a buffer with random bytes.
#[napi]
pub fn random_buffer(size: u32) -> Buffer {
    let mut bytes = vec![0u8; size as usize];
    // Use uuid for pseudo-random data
    for chunk in bytes.chunks_mut(16) {
        let uuid = uuid::Uuid::new_v4();
        let uuid_bytes = uuid.as_bytes();
        let len = chunk.len().min(16);
        chunk[..len].copy_from_slice(&uuid_bytes[..len]);
    }
    Buffer::from(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_alloc_fill() {
        let buf = buffer_alloc(4, Some(0xFF));
        assert_eq!(buf.as_ref(), &[0xFF, 0xFF, 0xFF, 0xFF]);
    }
    #[test]
    fn test_concat() {
        let a = Buffer::from(vec![1, 2]);
        let b = Buffer::from(vec![3, 4]);
        let c = buffer_concat(vec![a, b]);
        assert_eq!(c.as_ref(), &[1, 2, 3, 4]);
    }
    #[test]
    fn test_base64_roundtrip() {
        let buf = Buffer::from(vec![72, 101, 108, 108, 111]);
        let encoded = buffer_to_base64(buf);
        let decoded = base64_to_buffer(encoded).unwrap();
        assert_eq!(buffer_to_string(decoded).unwrap(), "Hello");
    }
    #[test]
    fn test_hex_roundtrip() {
        let buf = Buffer::from(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let hex = buffer_to_hex(buf);
        assert_eq!(hex, "deadbeef");
        let back = hex_to_buffer(hex).unwrap();
        assert_eq!(back.as_ref(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    }
}
