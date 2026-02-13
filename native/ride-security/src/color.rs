/*---------------------------------------------------------------------------------------------
 *  Copyright (c) RIDE Contributors. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

//! Color manipulation — Rust port of `src/vs/base/common/color.ts`.
//! HSL/HSV/RGB/Hex conversion, blending, contrast, and accessibility.

use napi_derive::napi;
use napi::bindgen_prelude::*;

#[napi(object)]
pub struct RgbaColor { pub r: u32, pub g: u32, pub b: u32, pub a: f64 }

#[napi(object)]
pub struct HslaColor { pub h: f64, pub s: f64, pub l: f64, pub a: f64 }

#[napi(object)]
pub struct HsvaColor { pub h: f64, pub s: f64, pub v: f64, pub a: f64 }

#[napi]
pub fn hex_to_rgba(hex: String) -> Result<RgbaColor> {
    let h = hex.trim_start_matches('#');
    let (r, g, b, a) = match h.len() {
        3 => {
            let r = u32::from_str_radix(&h[0..1].repeat(2), 16).unwrap_or(0);
            let g = u32::from_str_radix(&h[1..2].repeat(2), 16).unwrap_or(0);
            let b = u32::from_str_radix(&h[2..3].repeat(2), 16).unwrap_or(0);
            (r, g, b, 1.0)
        }
        6 => {
            let r = u32::from_str_radix(&h[0..2], 16).unwrap_or(0);
            let g = u32::from_str_radix(&h[2..4], 16).unwrap_or(0);
            let b = u32::from_str_radix(&h[4..6], 16).unwrap_or(0);
            (r, g, b, 1.0)
        }
        8 => {
            let r = u32::from_str_radix(&h[0..2], 16).unwrap_or(0);
            let g = u32::from_str_radix(&h[2..4], 16).unwrap_or(0);
            let b = u32::from_str_radix(&h[4..6], 16).unwrap_or(0);
            let a = u32::from_str_radix(&h[6..8], 16).unwrap_or(255) as f64 / 255.0;
            (r, g, b, a)
        }
        _ => return Err(Error::from_reason(format!("Invalid hex: {}", hex))),
    };
    Ok(RgbaColor { r, g, b, a })
}

#[napi]
pub fn rgba_to_hex(r: u32, g: u32, b: u32, a: Option<f64>) -> String {
    match a {
        Some(alpha) if alpha < 1.0 => format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, (alpha * 255.0) as u32),
        _ => format!("#{:02x}{:02x}{:02x}", r, g, b),
    }
}

#[napi]
pub fn rgba_to_hsla(r: u32, g: u32, b: u32, a: f64) -> HslaColor {
    let rf = r as f64 / 255.0;
    let gf = g as f64 / 255.0;
    let bf = b as f64 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return HslaColor { h: 0.0, s: 0.0, l, a };
    }
    let d = max - min;
    let s = if l > 0.5 { d / (2.0 - max - min) } else { d / (max + min) };
    let h = if (max - rf).abs() < f64::EPSILON {
        ((gf - bf) / d + if gf < bf { 6.0 } else { 0.0 }) / 6.0
    } else if (max - gf).abs() < f64::EPSILON {
        ((bf - rf) / d + 2.0) / 6.0
    } else {
        ((rf - gf) / d + 4.0) / 6.0
    };
    HslaColor { h: h * 360.0, s, l, a }
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

#[napi]
pub fn hsla_to_rgba(h: f64, s: f64, l: f64, a: f64) -> RgbaColor {
    if s.abs() < f64::EPSILON {
        let v = (l * 255.0).round() as u32;
        return RgbaColor { r: v, g: v, b: v, a };
    }
    let hh = h / 360.0;
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    RgbaColor {
        r: (hue_to_rgb(p, q, hh + 1.0 / 3.0) * 255.0).round() as u32,
        g: (hue_to_rgb(p, q, hh) * 255.0).round() as u32,
        b: (hue_to_rgb(p, q, hh - 1.0 / 3.0) * 255.0).round() as u32,
        a,
    }
}

#[napi]
pub fn luminance(r: u32, g: u32, b: u32) -> f64 {
    fn linearize(c: u32) -> f64 {
        let s = c as f64 / 255.0;
        if s <= 0.03928 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    }
    0.2126 * linearize(r) + 0.7152 * linearize(g) + 0.0722 * linearize(b)
}

#[napi]
pub fn contrast_ratio(r1: u32, g1: u32, b1: u32, r2: u32, g2: u32, b2: u32) -> f64 {
    let l1 = luminance(r1, g1, b1) + 0.05;
    let l2 = luminance(r2, g2, b2) + 0.05;
    if l1 > l2 { l1 / l2 } else { l2 / l1 }
}

#[napi]
pub fn blend(r1: u32, g1: u32, b1: u32, r2: u32, g2: u32, b2: u32, factor: f64) -> RgbaColor {
    let f = factor.clamp(0.0, 1.0);
    RgbaColor {
        r: (r1 as f64 * (1.0 - f) + r2 as f64 * f).round() as u32,
        g: (g1 as f64 * (1.0 - f) + g2 as f64 * f).round() as u32,
        b: (b1 as f64 * (1.0 - f) + b2 as f64 * f).round() as u32,
        a: 1.0,
    }
}

#[napi]
pub fn lighten(r: u32, g: u32, b: u32, amount: f64) -> RgbaColor {
    let hsl = rgba_to_hsla(r, g, b, 1.0);
    hsla_to_rgba(hsl.h, hsl.s, (hsl.l + amount).min(1.0), 1.0)
}

#[napi]
pub fn darken(r: u32, g: u32, b: u32, amount: f64) -> RgbaColor {
    let hsl = rgba_to_hsla(r, g, b, 1.0);
    hsla_to_rgba(hsl.h, hsl.s, (hsl.l - amount).max(0.0), 1.0)
}

#[napi]
pub fn to_css_rgba(r: u32, g: u32, b: u32, a: f64) -> String {
    if (a - 1.0).abs() < f64::EPSILON {
        format!("rgb({}, {}, {})", r, g, b)
    } else {
        format!("rgba({}, {}, {}, {:.2})", r, g, b, a)
    }
}

#[napi]
pub fn to_css_hsla(h: f64, s: f64, l: f64, a: f64) -> String {
    if (a - 1.0).abs() < f64::EPSILON {
        format!("hsl({:.0}, {:.0}%, {:.0}%)", h, s * 100.0, l * 100.0)
    } else {
        format!("hsla({:.0}, {:.0}%, {:.0}%, {:.2})", h, s * 100.0, l * 100.0, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hex_roundtrip() {
        let rgba = hex_to_rgba("#ff8040".into()).unwrap();
        assert_eq!(rgba.r, 255); assert_eq!(rgba.g, 128); assert_eq!(rgba.b, 64);
        assert_eq!(rgba_to_hex(255, 128, 64, None), "#ff8040");
    }
    #[test]
    fn test_contrast() {
        let ratio = contrast_ratio(0, 0, 0, 255, 255, 255);
        assert!(ratio > 20.0); // Black vs white is 21:1
    }
    #[test]
    fn test_blend() {
        let c = blend(0, 0, 0, 255, 255, 255, 0.5);
        assert!(c.r > 120 && c.r < 130);
    }
}
