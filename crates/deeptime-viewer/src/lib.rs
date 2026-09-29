// SPDX-License-Identifier: LGPL-3.0-or-later
//! Writes static HTML viewer bundles for world data and stage checkpoints (plan §9.3).
//!
//! v0 (P0-09): `index.html`, `viewer.js`, `ramps.json`, `bundle.js` and per layer
//! `layers/<slug>.png` (coloured, at most 2,048 px wide) plus `layers/<slug>.js` (the raw values on
//! a grid at most 1,024 wide, for click-to-inspect). Values travel as `<script>` files rather than
//! fetched JSON so the page also works opened straight from disk (`file://`). No npm, no build step.
//!
//! P1-01 adds a run's timeline: `keyframes/kf-<n>-<layer>.png` (the core's already-coloured
//! frames), `events.json` and `sites.json` (also embedded in `bundle.js`), shown as a keyframe
//! strip with a time scrubber, an event ticker and a site list with the "Why here?" cause chain.
//! [`write_bench_page`] writes `deeptime bench`'s timing page.

use std::fs;
use std::io;
use std::path::Path;

use deeptime_format::keyframes::{KeyframeIndex, read_keyframe};
use deeptime_format::{
    DType, Event, EventLog, KeyframeImage, LoadedLayer, Manifest, Ramp, Ramps, RasterRead, Site,
    TilePackReader,
};
use serde_json::{Value, json};

pub use deeptime_format::FORMAT_VERSION;
pub use deeptime_format::ramps::RAMPS_JSON;

const INDEX_HTML: &str = include_str!("../assets/index.html");
const VIEWER_JS: &str = include_str!("../assets/viewer.js");
const BENCH_HTML: &str = include_str!("../assets/bench.html");

/// Widest PNG written per layer.
pub const MAX_IMAGE_W: u32 = 2048;
/// Widest value grid written per layer.
pub const MAX_VALUES_W: u32 = 1024;
/// Widest horizon-stack grid written for the inspector (3K bytes per cell).
pub const MAX_STRATA_W: u32 = 512;

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

/// A run's history for the timelapse panel: keyframes (by frame number), events (any order) and
/// sites.
#[derive(Default)]
pub struct Timeline {
    pub keyframes: Vec<KeyframeImage>,
    pub events: Vec<Event>,
    pub sites: Vec<Site>,
}

impl Timeline {
    pub fn is_empty(&self) -> bool {
        self.keyframes.is_empty() && self.events.is_empty() && self.sites.is_empty()
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

/// Writes the bundle into `dir` (world-data layers only).
pub fn write_bundle(
    dir: &Path,
    info: &BundleInfo,
    layers: &[&LoadedLayer],
    strata: Option<Strata<'_>>,
) -> io::Result<()> {
    write_bundle_with(dir, info, layers, strata, &Timeline::default())
}

/// Writes the bundle into `dir`: world-data layers plus a run's timeline.
pub fn write_bundle_with(
    dir: &Path,
    info: &BundleInfo,
    layers: &[&LoadedLayer],
    strata: Option<Strata<'_>>,
    timeline: &Timeline,
) -> io::Result<()> {
    let ramps = Ramps::parse(RAMPS_JSON).map_err(io::Error::other)?;
    fs::create_dir_all(dir.join("layers"))?;
    fs::write(
        dir.join("index.html"),
        INDEX_HTML.replace("{{TITLE}}", &html_escape(&info.title)),
    )?;
    fs::write(dir.join("viewer.js"), VIEWER_JS)?;
    fs::write(dir.join("ramps.json"), RAMPS_JSON)?;

    let mut entries = Vec::new();
    for l in layers {
        let d = &l.desc;
        if d.dtype == DType::HorizonStack {
            continue;
        }
        let s = slug(&d.name);
        let (ramp_name, ramp) = ramps.for_layer(&d.name);
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

    let mut strata_json = Value::Null;
    if let Some(st) = strata {
        let d = &st.layer.desc;
        let k = d.stack_depth.unwrap_or(0);
        let stride = stride_for(d.grid.w, MAX_STRATA_W);
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

    let (keyframes, events, sites) = write_timeline(dir, timeline)?;

    let ramps_value: Value = serde_json::from_str(RAMPS_JSON).map_err(io::Error::other)?;
    let bundle = json!({
        "title": info.title, "source": info.source, "circumference": info.circumference,
        "sea_level_y": info.sea_level_y, "format_version": info.format_version,
        "created_utc": info.created_utc, "layers": entries, "strata": strata_json,
        "ramps": ramps_value, "keyframes": keyframes, "events": events, "sites": sites,
    });
    fs::write(
        dir.join("bundle.js"),
        format!("window.DT_BUNDLE = {bundle};\n"),
    )?;
    Ok(())
}

/// Keyframe PNGs, `events.json` and `sites.json`; returns what `bundle.js` embeds.
fn write_timeline(dir: &Path, t: &Timeline) -> io::Result<(Value, Value, Value)> {
    let mut frames: Vec<&KeyframeImage> = t.keyframes.iter().collect();
    frames.sort_by_key(|k| k.n);
    if !frames.is_empty() {
        fs::create_dir_all(dir.join("keyframes"))?;
    }
    let mut kf = Vec::with_capacity(frames.len());
    for k in frames {
        let mut layers = Vec::with_capacity(k.layers.len());
        for (name, rgba) in &k.layers {
            let file = format!("keyframes/kf-{}-{}.png", k.n, slug(name));
            write_rgba_png(&dir.join(&file), k.w, k.h, rgba)?;
            layers.push(json!({ "name": name, "file": file }));
        }
        kf.push(json!({
            "n": k.n, "t": k.t, "stage": k.stage, "w": k.w, "h": k.h, "layers": layers,
        }));
    }
    let mut events: Vec<&Event> = t.events.iter().collect();
    events.sort_by(|a, b| a.t_start.total_cmp(&b.t_start).then(a.id.cmp(&b.id)));
    let events = Value::Array(events.iter().map(|e| e.to_json()).collect());
    let sites = Value::Array(t.sites.iter().map(Site::to_json).collect());
    fs::write(dir.join("events.json"), format!("{events:#}\n"))?;
    fs::write(dir.join("sites.json"), format!("{sites:#}\n"))?;
    Ok((Value::Array(kf), events, sites))
}

/// Writes the bundle of a finished world-data folder: its raster layers (and strata), keyframes,
/// events and sites.
pub fn write_world_bundle(dir: &Path, world: &Path, title: &str) -> io::Result<()> {
    let m = Manifest::read(world).map_err(io::Error::other)?;
    let mut loaded = Vec::new();
    let mut strata = None;
    for e in &m.layers {
        let layer = TilePackReader::open(&world.join(&e.path))
            .and_then(|r| r.load())
            .map_err(io::Error::other)?;
        if layer.desc.dtype == DType::HorizonStack {
            strata = Some(layer);
        } else {
            loaded.push(layer);
        }
    }
    loaded.sort_by_key(|l| l.desc.name != "elevation"); // elevation first
    let classes = match fs::read(world.join("strata/classes.json")) {
        Ok(b) => serde_json::from_slice::<Value>(&b)
            .map_err(io::Error::other)?
            .get("classes")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .map(|c| c.as_str().unwrap_or("?").to_owned())
                    .collect()
            })
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };
    let index = KeyframeIndex::read(world).map_err(io::Error::other)?;
    let keyframes = index
        .frames
        .iter()
        .map(|e| read_keyframe(world, e))
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?;
    let events = EventLog::load(world).map_err(io::Error::other)?;
    let sites = deeptime_format::sites::read_sites(world)
        .map_err(io::Error::other)?
        .map(|(s, _)| s)
        .unwrap_or_default();
    let info = BundleInfo {
        title: title.to_owned(),
        source: m.source.clone(),
        circumference: m.scale.circumference_blocks,
        sea_level_y: m.scale.sea_level_y,
        format_version: m.format_version.clone(),
        created_utc: m.created_utc.clone(),
    };
    let refs: Vec<&LoadedLayer> = loaded.iter().collect();
    write_bundle_with(
        dir,
        &info,
        &refs,
        strata.as_ref().map(|layer| Strata { layer, classes }),
        &Timeline {
            keyframes,
            events: events.events().to_vec(),
            sites,
        },
    )
}

/// Writes `deeptime bench`'s page: `index.html` plus `bench.js` (`window.DT_BENCH = …`), the
/// same object as `bench.json`.
pub fn write_bench_page(dir: &Path, bench: &Value) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("index.html"), BENCH_HTML)?;
    fs::write(
        dir.join("bench.js"),
        format!("window.DT_BENCH = {bench};\n"),
    )?;
    Ok(())
}

fn write_rgba_png(path: &Path, w: u32, h: u32, rgba: &[u8]) -> io::Result<()> {
    let file = io::BufWriter::new(fs::File::create(path)?);
    let mut enc = png::Encoder::new(file, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(io::Error::other)?;
    writer.write_image_data(rgba).map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;
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
            rgba.extend_from_slice(&ramp.rgba(v));
        }
    }
    write_rgba_png(path, w, h, &rgba)?;
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
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Standard base64 with padding.
pub fn base64(data: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::{EventId, Place};

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
    fn slugs() {
        assert_eq!(slug("climate/temperature"), "climate_temperature");
    }

    #[test]
    fn timeline_bundle_has_frames_events_and_sites() {
        let dir = std::env::temp_dir().join(format!("dt-viewer-tl-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let (w, h) = (8u32, 4u32);
        let frame = |n: u32| KeyframeImage {
            n,
            t: -1.0e8 + f64::from(n) * 1.0e7,
            stage: "dummy".into(),
            w,
            h,
            layers: vec![("elevation".into(), vec![200; (w * h * 4) as usize])],
        };
        let events = vec![
            Event::new(
                EventId::new(1, 1),
                -5.0e7,
                "deeptime:collision",
                "deeptime:dummy",
            )
            .at(Place::Point { lat: 1.0, lon: 2.0 })
            .caused_by(EventId::new(1, 0)),
            Event::new(
                EventId::new(1, 0),
                -9.0e7,
                "deeptime:rift_onset",
                "deeptime:dummy",
            ),
        ];
        let info = BundleInfo {
            title: "t <run>".into(),
            source: "run:dummy".into(),
            circumference: 1024,
            sea_level_y: 63,
            format_version: "0.0".into(),
            created_utc: "2026-09-28T00:00:00Z".into(),
        };
        let timeline = Timeline {
            keyframes: vec![frame(1), frame(0)],
            events,
            sites: Vec::new(),
        };
        write_bundle_with(&dir, &info, &[], None, &timeline).unwrap();
        assert!(dir.join("keyframes/kf-1-elevation.png").is_file());
        let bundle = fs::read_to_string(dir.join("bundle.js")).unwrap();
        let v: Value = serde_json::from_str(
            bundle
                .trim()
                .strip_prefix("window.DT_BUNDLE = ")
                .unwrap()
                .strip_suffix(';')
                .unwrap(),
        )
        .unwrap();
        assert_eq!(v["keyframes"][0]["n"], 0, "frames sorted by number");
        assert_eq!(
            v["events"][0]["kind"], "deeptime:rift_onset",
            "events by time"
        );
        assert_eq!(v["layers"], json!([]));
        let events: Value =
            serde_json::from_str(&fs::read_to_string(dir.join("events.json")).unwrap()).unwrap();
        assert_eq!(events.as_array().unwrap().len(), 2);
        assert!(
            fs::read_to_string(dir.join("index.html"))
                .unwrap()
                .contains("t &lt;run&gt;")
        );
        write_bench_page(&dir.join("bench"), &json!({ "runs": [] })).unwrap();
        assert!(dir.join("bench/bench.js").is_file());
        fs::remove_dir_all(&dir).unwrap();
    }
}
