// SPDX-License-Identifier: LGPL-3.0-or-later
//! RGBA images, PNG in and out, and a 3 × 5 pixel font for chart labels.

use std::io::BufWriter;
use std::path::Path;

use crate::error::{Error, Result};

pub type Rgba = [u8; 4];

pub const TRANSPARENT: Rgba = [0, 0, 0, 0];

pub struct Image {
    pub w: usize,
    pub h: usize,
    pub px: Vec<Rgba>,
}

impl Image {
    pub fn new(w: usize, h: usize, fill: Rgba) -> Image {
        Image {
            w,
            h,
            px: vec![fill; w * h],
        }
    }

    pub fn set(&mut self, x: usize, y: usize, c: Rgba) {
        if x < self.w && y < self.h {
            self.px[y * self.w + x] = c;
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Rgba {
        self.px[y * self.w + x]
    }

    pub fn fill_rect(&mut self, x: usize, y: usize, w: usize, h: usize, c: Rgba) {
        for yy in y..(y + h).min(self.h) {
            for xx in x..(x + w).min(self.w) {
                self.px[yy * self.w + xx] = c;
            }
        }
    }

    /// Nearest-neighbour enlargement by an integer factor.
    pub fn scaled(&self, s: usize) -> Image {
        if s <= 1 {
            return Image {
                w: self.w,
                h: self.h,
                px: self.px.clone(),
            };
        }
        let mut out = Image::new(self.w * s, self.h * s, TRANSPARENT);
        for y in 0..out.h {
            for x in 0..out.w {
                out.px[y * out.w + x] = self.px[(y / s) * self.w + x / s];
            }
        }
        out
    }

    pub fn save_png(&self, path: &Path) -> Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
        }
        let file = std::fs::File::create(path).map_err(|e| Error::io(path, e))?;
        let mut enc = png::Encoder::new(BufWriter::new(file), self.w as u32, self.h as u32);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut wr = enc.write_header()?;
        let flat: Vec<u8> = self.px.iter().flat_map(|p| p.iter().copied()).collect();
        wr.write_image_data(&flat)?;
        wr.finish()?;
        Ok(())
    }

    /// Draws `text` with the 3 × 5 font, each font pixel `scale` pixels square, top-left at
    /// (x, y). Characters the font lacks are left blank.
    pub fn text(&mut self, x: usize, y: usize, text: &str, scale: usize, c: Rgba) {
        let mut cx = x;
        for ch in text.chars() {
            if let Some(rows) = glyph(ch) {
                for (ry, row) in rows.iter().enumerate() {
                    for rx in 0..3 {
                        if row & (0b100 >> rx) != 0 {
                            self.fill_rect(cx + rx * scale, y + ry * scale, scale, scale, c);
                        }
                    }
                }
            }
            cx += 4 * scale;
        }
    }
}

/// Width in pixels of `text` drawn at `scale`.
pub fn text_width(text: &str, scale: usize) -> usize {
    (text.chars().count() * 4).saturating_sub(1) * scale
}

/// Rows of a 3 × 5 glyph, bit 2 = left column. Letters are upper case only; lower case is
/// drawn as upper case.
fn glyph(c: char) -> Option<[u8; 5]> {
    Some(match c.to_ascii_uppercase() {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        'A' => [0b010, 0b101, 0b111, 0b101, 0b101],
        'B' => [0b110, 0b101, 0b110, 0b101, 0b110],
        'C' => [0b011, 0b100, 0b100, 0b100, 0b011],
        'D' => [0b110, 0b101, 0b101, 0b101, 0b110],
        'E' => [0b111, 0b100, 0b110, 0b100, 0b111],
        'F' => [0b111, 0b100, 0b110, 0b100, 0b100],
        'G' => [0b011, 0b100, 0b101, 0b101, 0b011],
        'H' => [0b101, 0b101, 0b111, 0b101, 0b101],
        'I' => [0b111, 0b010, 0b010, 0b010, 0b111],
        'J' => [0b001, 0b001, 0b001, 0b101, 0b010],
        'K' => [0b101, 0b101, 0b110, 0b101, 0b101],
        'L' => [0b100, 0b100, 0b100, 0b100, 0b111],
        'M' => [0b101, 0b111, 0b111, 0b101, 0b101],
        'N' => [0b110, 0b101, 0b101, 0b101, 0b101],
        'O' => [0b010, 0b101, 0b101, 0b101, 0b010],
        'P' => [0b110, 0b101, 0b110, 0b100, 0b100],
        'Q' => [0b010, 0b101, 0b101, 0b110, 0b011],
        'R' => [0b110, 0b101, 0b110, 0b101, 0b101],
        'S' => [0b011, 0b100, 0b010, 0b001, 0b110],
        'T' => [0b111, 0b010, 0b010, 0b010, 0b010],
        'U' => [0b101, 0b101, 0b101, 0b101, 0b111],
        'V' => [0b101, 0b101, 0b101, 0b101, 0b010],
        'W' => [0b101, 0b101, 0b111, 0b111, 0b101],
        'X' => [0b101, 0b101, 0b010, 0b101, 0b101],
        'Y' => [0b101, 0b101, 0b010, 0b010, 0b010],
        'Z' => [0b111, 0b001, 0b010, 0b100, 0b111],
        '-' => [0b000, 0b000, 0b111, 0b000, 0b000],
        '+' => [0b000, 0b010, 0b111, 0b010, 0b000],
        '<' => [0b001, 0b010, 0b100, 0b010, 0b001],
        '>' => [0b100, 0b010, 0b001, 0b010, 0b100],
        '=' => [0b000, 0b111, 0b000, 0b111, 0b000],
        '.' => [0b000, 0b000, 0b000, 0b000, 0b010],
        ',' => [0b000, 0b000, 0b000, 0b010, 0b100],
        ':' => [0b000, 0b010, 0b000, 0b010, 0b000],
        '%' => [0b101, 0b001, 0b010, 0b100, 0b101],
        '(' => [0b001, 0b010, 0b010, 0b010, 0b001],
        ')' => [0b100, 0b010, 0b010, 0b010, 0b100],
        '/' => [0b001, 0b001, 0b010, 0b100, 0b100],
        ' ' => [0; 5],
        _ => return None,
    })
}

/// Decodes a PNG to RGBA8 (palette, grey, 16-bit and alpha-less images expanded).
pub fn decode_png(bytes: &[u8]) -> Result<Image> {
    let mut dec = png::Decoder::new(std::io::Cursor::new(bytes));
    dec.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = dec.read_info()?;
    let size = reader
        .output_buffer_size()
        .ok_or_else(|| Error::new("PNG: image too large"))?;
    let mut buf = vec![0u8; size];
    let info = reader.next_frame(&mut buf)?;
    let (w, h) = (info.width as usize, info.height as usize);
    let buf = &buf[..info.buffer_size()];
    let px: Vec<Rgba> = match info.color_type {
        png::ColorType::Rgba => buf
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| [p[0], p[1], p[2], p[3]])
            .collect(),
        png::ColorType::Rgb => buf
            .as_chunks::<3>()
            .0
            .iter()
            .map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => buf
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => buf.iter().map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err(Error::new("PNG: palette not expanded")),
    };
    if px.len() != w * h {
        return Err(Error::new("PNG: decoded size mismatch"));
    }
    Ok(Image { w, h, px })
}

/// `#rrggbb` or `#rrggbbaa`.
pub fn hex(c: Rgba) -> String {
    if c[3] == 255 {
        format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
    } else {
        format!("#{:02x}{:02x}{:02x}{:02x}", c[0], c[1], c[2], c[3])
    }
}

/// Parses `#rgb`-style hex written by [`hex`].
pub fn parse_hex(s: &str) -> Option<Rgba> {
    let s = s.strip_prefix('#')?;
    let b = |i: usize| u8::from_str_radix(s.get(i..i + 2)?, 16).ok();
    match s.len() {
        6 => Some([b(0)?, b(2)?, b(4)?, 255]),
        8 => Some([b(0)?, b(2)?, b(4)?, b(6)?]),
        _ => None,
    }
}

/// Multiplies the colour's RGB by `f` (clamped).
pub fn shade(c: Rgba, f: f64) -> Rgba {
    let m = |v: u8| (f64::from(v) * f).round().clamp(0.0, 255.0) as u8;
    [m(c[0]), m(c[1]), m(c[2]), c[3]]
}

/// `a` over `b` with `a`'s weight `t` in 0..1.
pub fn mix(a: Rgba, b: Rgba, t: f64) -> Rgba {
    let l = |x: u8, y: u8| (f64::from(x) * t + f64::from(y) * (1.0 - t)).round() as u8;
    [l(a[0], b[0]), l(a[1], b[1]), l(a[2], b[2]), 255]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_round_trip_and_hex() {
        let mut img = Image::new(3, 2, [10, 20, 30, 255]);
        img.set(2, 1, [255, 0, 128, 64]);
        let path = std::env::temp_dir().join(format!("dt-inspect-img-{}.png", std::process::id()));
        img.save_png(&path).unwrap();
        let back = decode_png(&std::fs::read(&path).unwrap()).unwrap();
        std::fs::remove_file(&path).ok();
        assert_eq!((back.w, back.h), (3, 2));
        assert_eq!(back.px, img.px);
        assert_eq!(hex([255, 0, 128, 64]), "#ff008040");
        assert_eq!(parse_hex("#ff0080"), Some([255, 0, 128, 255]));
        assert_eq!(parse_hex(&hex([1, 2, 3, 4])), Some([1, 2, 3, 4]));
        assert_eq!(text_width("12", 1), 7);
    }
}
