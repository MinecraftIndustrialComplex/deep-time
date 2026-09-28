// SPDX-License-Identifier: LGPL-3.0-or-later
//! Writes static HTML viewer bundles for world data and stage checkpoints (plan §9.3).
//!
//! v0 (P0-09): `index.html`, `viewer.js`, `ramps.json`, `bundle.js` and per layer
//! `layers/<slug>.png` (coloured, at most 2,048 px wide) plus `layers/<slug>.js` (the raw values on
//! a grid at most 1,024 wide, for click-to-inspect). Values travel as `<script>` files rather than
//! fetched JSON so the page also works opened straight from disk (`file://`). No npm, no build step.

use std::fs;
use std::io;
use std::path::Path;

use deeptime_format::{DType, LoadedLayer, RasterRead};
use serde::Deserialize;
use serde_json::json;

pub use deeptime_format::FORMAT_VERSION;

/// The colour ramps (also copied into every bundle).
pub const RAMPS_JSON: &str = include_str!("../assets/ramps.json");
const INDEX_HTML: &str = include_str!("../assets/index.html");
const VIEWER_JS: &str = include_str!("../assets/viewer.js");

/// Widest PNG written per layer.
pub const MAX_IMAGE_W: u32 = 2048;
/// Widest value grid written per layer.
pub const MAX_VALUES_W: u32 = 1024;

/// Page header facts.
pub struct BundleInfo {
    pub title: String,
    pub source: String,
    pub circumference: u32,
    pub sea_level_y: i32,
    pub format_version: String,
    pub created_utc: String,
}

/// A horizon-stack layer shown in the inspector.
pub struct Strata<'a> {
    pub layer: &'a LoadedLayer,
    pub classes: Vec<String>,
}

#[derive(Deserialize)]
struct Ramps {
    ramps: std::collections::BTreeMap<String, Ramp>,
    layers: std::collections::BTreeMap<String, String>,
    default: String,
}

#[derive(Deserialize)]
struct Ramp {
    stops: Vec<(f64, String)>,
}

struct Rgb([u8; 3]);

fn parse_hex(s: &str) -> Rgb {
    let s = s.trim_start_matches('#');
    let p = |k: usize| u8::from_str_radix(s.get(k..k + 2).unwrap_or("00"), 16).unwrap_or(0);
    Rgb([p(0), p(2), p(4)])
}

impl Ramp {
    fn colour(&self, v: f64) -> [u8; 3] {
        let stops = &self.stops;
        if stops.is_empty() {
            return [128, 128, 128];
        }
        if v <= stops[0].0 {
            return parse_hex(&stops[0].1).0;
        }
        for w in stops.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            if v <= b.0 {
                let t = if b.0 > a.0 { (v - a.0) / (b.0 - a.0) } else { 1.0 };
                let (ca, cb) = (parse_hex(&a.1).0, parse_hex(&b.1).0);
                return std::array::from_fn(|k| {
                    let (x, y) = (f64::from(ca[k]), f64::from(cb[k]));
                    (x + (y - x) * t).round() as u8
                });
            }
        }
        parse_hex(&stops[stops.len() - 1].1).0
    }
}

/// `climate/temperature` → `climate_temperature`.
pub fn slug(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

fn stride_for(w: u32, max: u32) -> u32 {
    w.div_ceil(max).max(1)
}

/// Writes the bundle into `dir`.
pub fn write_bundle(
    dir: &Path,
    info: &BundleInfo,
    layers: &[&LoadedLayer],
    strata: Option<Strata<'_>>,
) -> io::Result<()> {
    let ramps: Ramps = serde_json::from_str(RAMPS_JSON).map_err(io::Error::other)?;
    fs::create_dir_all(dir.join("layers"))?;
    fs::write(dir.join("index.html"), INDEX_HTML.replace("{{TITLE}}", &html_escape(&info.title)))?;
    fs::write(dir.join("viewer.js"), VIEWER_JS)?;
    fs::write(dir.join("ramps.json"), RAMPS_JSON)?;

    let mut entries = Vec::new();
    for l in layers {
        let d = &l.desc;
        if d.dtype == DType::HorizonStack {
            continue;
        }
        let s = slug(&d.name);
        let ramp_name = ramps.layers.get(&d.name).unwrap_or(&ramps.default);
        let ramp = ramps
            .ramps
            .get(ramp_name)
            .or_else(|| ramps.ramps.get(&ramps.default))
            .ok_or_else(|| io::Error::other("ramps.json has no default ramp"))?;
        let (iw, ih, istride) = write_png(&dir.join(format!("layers/{s}.png")), l, ramp)?;
        let (vw, vh, vstride) = write_values(&dir.join(format!("layers/{s}.js")), l)?;
        entries.push(json!({
            "name": d.name, "slug": s, "units": d.units, "scale": d.scale, "offset": d.offset,
            "nodata": d.nodata, "grid": d.grid, "ramp": ramp_name,
            "image": { "file": format!("layers/{s}.png"), "w": iw, "h": ih, "stride": istride },
            "values": { "file": format!("layers/{s}.js"), "w": vw, "h": vh, "stride": vstride,
                        "type": if d.dtype == DType::I16 { "i16" } else { "f32" } }
        }));
    }

    let mut strata_json = serde_json::Value::Null;
    if let Some(st) = strata {
        let d = &st.layer.desc;
        let k = d.stack_depth.unwrap_or(0);
        let stride = stride_for(d.grid.w, MAX_VALUES_W);
        let (vw, vh) = (d.grid.w.div_ceil(stride), d.grid.h.div_ceil(stride));
        let mut tops = Vec::with_capacity((vw * vh * k * 2) as usize);
        let mut classes = Vec::with_capacity((vw * vh * k) as usize);
        for h in 0..k {
            for vj in 0..vh {
                for vi in 0..vw {
                    let (t, c) = st.layer.horizon(vi * stride, vj * stride, h);
                    tops.extend_from_slice(&t.to_le_bytes());
                    classes.push(c);
                }
            }
        }
        fs::write(
            dir.join("layers/strata.js"),
            format!(
                "DT.addStrata({});\n",
                json!({ "w": vw, "h": vh, "stride": stride, "k": k, "scale": d.scale,
                        "grid": d.grid, "classes": st.classes,
                        "tops": base64(&tops), "class_index": base64(&classes) })
            ),
        )?;
        strata_json = json!({ "file": "layers/strata.js" });
    }

    let ramps_value: serde_json::Value = serde_json::from_str(RAMPS_JSON).map_err(io::Error::other)?;
    let bundle = json!({
        "title": info.title, "source": info.source, "circumference": info.circumference,
        "sea_level_y": info.sea_level_y, "format_version": info.format_version,
        "created_utc": info.created_utc, "layers": entries, "strata": strata_json,
        "ramps": ramps_value
    });
    fs::write(dir.join("bundle.js"), format!("window.DT_BUNDLE = {bundle};\n"))?;
    Ok(())
}

fn write_png(path: &Path, l: &LoadedLayer, ramp: &Ramp) -> io::Result<(u32, u32, u32)> {
    let g = &l.desc.grid;
    let stride = stride_for(g.w, MAX_IMAGE_W);
    let (w, h) = (g.w.div_ceil(stride), g.h.div_ceil(stride));
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for j in 0..h {
        for i in 0..w {
            let v = l.get(i64::from(i * stride), i64::from(j * stride));
            if v.is_nan() {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let c = ramp.colour(v);
                rgba.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
    }
    let file = io::BufWriter::new(fs::File::create(path)?);
    let mut enc = png::Encoder::new(file, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(io::Error::other)?;
    writer.write_image_data(&rgba).map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;
    Ok((w, h, stride))
}

fn write_values(path: &Path, l: &LoadedLayer) -> io::Result<(u32, u32, u32)> {
    let d = &l.desc;
    let stride = stride_for(d.grid.w, MAX_VALUES_W);
    let (w, h) = (d.grid.w.div_ceil(stride), d.grid.h.div_ceil(stride));
    let mut bytes = Vec::new();
    for j in 0..h {
        for i in 0..w {
            let raw = l.raw_cell(i * stride, j * stride);
            if d.dtype == DType::I16 {
                bytes.extend_from_slice(&(raw as i16).to_le_bytes());
            } else {
                bytes.extend_from_slice(&(raw as f32).to_le_bytes());
            }
        }
    }
    fs::write(
        path,
        format!(
            "DT.addValues({}, {});\n",
            json!(d.name),
            json!({ "w": w, "h": h, "stride": stride, "data": base64(&bytes) })
        ),
    )?;
    Ok((w, h, stride))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Standard base64 with padding.
pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sees_format() {
        assert_eq!(super::FORMAT_VERSION, "0.0");
    }

    #[test]
    fn base64_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn ramps_parse_and_every_layer_ramp_exists() {
        let r: Ramps = serde_json::from_str(RAMPS_JSON).unwrap();
        assert!(r.ramps.contains_key(&r.default));
        for ramp in r.layers.values() {
            assert!(r.ramps.contains_key(ramp), "{ramp}");
        }
        let e = &r.ramps["elevation"];
        assert_eq!(e.colour(-1000.0), [0x08, 0x14, 0x2e]);
        assert_eq!(e.colour(63.0), [0x3f, 0x7f, 0x3a]);
        assert_eq!(e.colour(1000.0), [255, 255, 255]);
    }

    #[test]
    fn slugs() {
        assert_eq!(slug("climate/temperature"), "climate_temperature");
    }
}
