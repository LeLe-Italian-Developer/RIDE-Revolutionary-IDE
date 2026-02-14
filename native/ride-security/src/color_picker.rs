/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Color Picker — Rust port of `src/vs/editor/contrib/colorPicker/browser/colorPickerModel.ts` logic.
//! Detects and parses colors in text (Hex, RGB, HSL).

use napi_derive::napi;
use napi::bindgen_prelude::*;
use regex::Regex;
use std::sync::OnceLock;

static HEX_REGEX: OnceLock<Regex> = OnceLock::new();
static RGB_REGEX: OnceLock<Regex> = OnceLock::new();
static HSL_REGEX: OnceLock<Regex> = OnceLock::new();

#[napi(object)]
#[derive(Clone, Debug)]
pub struct ColorRange {
    pub start: u32,
    pub end: u32,
    pub color_string: String,
    pub format: String, // "hex", "rgb", "hsl"
}

fn get_hex_regex() -> &'static Regex {
    HEX_REGEX.get_or_init(|| Regex::new(r"#([0-9a-fA-F]{3}|[0-9a-fA-F]{6})\b").unwrap())
}

fn get_rgb_regex() -> &'static Regex {
    RGB_REGEX.get_or_init(|| Regex::new(r"rgb\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*\)").unwrap())
}

fn get_hsl_regex() -> &'static Regex {
    HSL_REGEX.get_or_init(|| Regex::new(r"hsl\(\s*(\d{1,3})\s*,\s*(\d{1,3})%\s*,\s*(\d{1,3})%\s*\)").unwrap())
}

#[napi]
pub fn find_colors(text: String) -> Vec<ColorRange> {
    let mut results = Vec::new();

    // Hex
    for caps in get_hex_regex().captures_iter(&text) {
        if let Some(m) = caps.get(0) {
            results.push(ColorRange {
                start: m.start() as u32,
                end: m.end() as u32,
                color_string: m.as_str().to_string(),
                format: "hex".to_string(),
            });
        }
    }

    // RGB
    for caps in get_rgb_regex().captures_iter(&text) {
        if let Some(m) = caps.get(0) {
            results.push(ColorRange {
                start: m.start() as u32,
                end: m.end() as u32,
                color_string: m.as_str().to_string(),
                format: "rgb".to_string(),
            });
        }
    }

    // HSL
    for caps in get_hsl_regex().captures_iter(&text) {
        if let Some(m) = caps.get(0) {
            results.push(ColorRange {
                start: m.start() as u32,
                end: m.end() as u32,
                color_string: m.as_str().to_string(),
                format: "hsl".to_string(),
            });
        }
    }

    results.sort_by_key(|a| a.start);
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_hex() {
        let text = "color: #ff0000; background: #00f;";
        let colors = find_colors(text.into());
        assert_eq!(colors.len(), 2);
        assert_eq!(colors[0].color_string, "#ff0000");
        assert_eq!(colors[1].color_string, "#00f");
    }

    #[test]
    fn test_find_rgb() {
        let text = "color: rgb(255, 0, 0);";
        let colors = find_colors(text.into());
        assert_eq!(colors.len(), 1);
        assert_eq!(colors[0].color_string, "rgb(255, 0, 0)");
    }
}
