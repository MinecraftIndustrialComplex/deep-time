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
//!
//! The workbench page (`deeptime watch`, decisions log 2026-09-28, P1-20): [`workbench_page`] and
//! [`WORKBENCH_JS`] are served by the CLI's watch server with `ramps.json`; frames travel as PNGs
//! from [`encode_png`]. Its live view draws the same keyframes with the same ramps and look as the
//! bundle's timelapse, fed by a stream instead of `bundle.js`; its editor is built from
//! [`EDITOR_FIELDS_JSON`], the catalogue the game's editor reads, labelled from the game's
//! [`LANG_EN_US_JSON`].

pub mod section;

use std::fs;
use std::io;
use std::path::Path;

use deeptime_format::keyframes::{KeyframeIndex, read_keyframe};
use deeptime_format::{
    Body, DType, Event, EventLog, KeyframeImage, LoadedLayer, Manifest, Ramp, Ramps, RasterRead,
    Site, TilePackReader,
};
use serde_json::{Value, json};

pub use deeptime_format::FORMAT_VERSION;
pub use deeptime_format::ramps::RAMPS_JSON;

const INDEX_HTML: &str = include_str!("../assets/index.html");
const VIEWER_JS: &str = include_str!("../assets/viewer.js");
/// The pages' map projection (`window.DTProj`, the twin of `deeptime_format::projection`), loaded
/// before every other script of the viewer bundle and the workbench.
pub const PROJECTION_JS: &str = include_str!("../assets/projection.js");
const BENCH_HTML: &str = include_str!("../assets/bench.html");
const WORKBENCH_HTML: &str = include_str!("../assets/workbench.html");
/// The workbench page's script (`deeptime watch` serves it as `workbench.js`).
pub const WORKBENCH_JS: &str = include_str!("../assets/workbench.js");
/// The workbench's deposits overlay (symbols, footprints, legend, hover text; `deposits.js`, served
/// beside `workbench.js`): a module of pure functions the page calls.
pub const DEPOSITS_JS: &str = include_str!("../assets/deposits.js");
/// The workbench's "Rock at depth" and cross-section tool (served as `sections.js`, after
/// `workbench.js`; it reads `/api/runs/<id>/rock…`).
pub const SECTIONS_JS: &str = include_str!("../assets/sections.js");
/// The workbench's "Final world" view (a zoomable map of a finished run's bake; `deeptime watch`
/// serves it as `worldmap.js`, loaded after `workbench.js`, which it hooks into).
pub const WORLDMAP_JS: &str = include_str!("../assets/worldmap.js");
/// The planet editor's field catalogue, shared with the game (Gradle ships it in the jar as
/// `deeptime/editor_fields.json` for `screen/model/EditorFields`); the workbench's form is built
/// from it.
pub const EDITOR_FIELDS_JSON: &str = include_str!("../assets/editor_fields.json");
/// The game's English strings: the workbench labels its form with the editor's own keys
/// (`deeptime.editor.*`).
pub const LANG_EN_US_JSON: &str =
    include_str!("../../../neoforge/src/main/resources/assets/deeptime/lang/en_us.json");

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
    fs::write(dir.join("projection.js"), PROJECTION_JS)?;
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

    // The satellite view (true colour, core satellite.rs) of the run's last frame that has one
    // leads the world-data layers, so the page opens on it; it is an image only (no values).
    if !entries.is_empty()
        && let Some(e) = satellite_entry(dir, info.circumference, timeline)?
    {
        entries.insert(0, e);
    }

    let mut strata_json = Value::Null;
    if let Some(st) = strata {
        let d = &st.layer.desc;
        let stride = stride_for(d.grid.w, MAX_STRATA_W);
        let (vw, vh) = (d.grid.w.div_ceil(stride), d.grid.h.div_ceil(stride));
        // Only as many horizons as some sampled cell uses (P1-07's stacks use a few of K = 32).
        let mut k = 0;
        for vj in 0..vh {
            for vi in 0..vw {
                let used = (0..d.stack_depth.unwrap_or(0))
                    .take_while(|&h| st.layer.horizon(vi * stride, vj * stride, h).is_used())
                    .count() as u32;
                k = k.max(used);
            }
        }
        let mut tops = Vec::with_capacity((vw * vh * k * 2) as usize);
        let mut classes = Vec::with_capacity((vw * vh * k) as usize);
        for h in 0..k {
            for vj in 0..vh {
                for vi in 0..vw {
                    let s = st.layer.horizon(vi * stride, vj * stride, h);
                    tops.extend_from_slice(&s.top.to_le_bytes());
                    classes.push(s.class);
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

    // An empty report; `write_report` replaces it (P1-14: the deposit census and bodies).
    fs::write(dir.join("report.js"), "window.DT_REPORT = null;\n")?;

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

/// The keyframe layer drawn as the world view's satellite layer.
pub const SATELLITE_LAYER: &str = "satellite";

/// Writes `layers/satellite.png` from the timeline's last frame with a satellite layer and returns
/// its `bundle.js` entry: a keyframe's pixel (px, py) sits at block x = −C/2 + px·C/w, z = −C/4 +
/// py·C/w, so its grid is the keyframe's own with r = C / w.
fn satellite_entry(dir: &Path, c: u32, t: &Timeline) -> io::Result<Option<Value>> {
    let Some((k, rgba)) = t
        .keyframes
        .iter()
        .filter_map(|k| {
            k.layers
                .iter()
                .find(|(n, _)| n == SATELLITE_LAYER)
                .map(|(_, rgba)| (k, rgba))
        })
        .max_by_key(|(k, _)| k.n)
    else {
        return Ok(None);
    };
    let file = format!("layers/{}.png", slug(SATELLITE_LAYER));
    write_rgba_png(&dir.join(&file), k.w, k.h, rgba)?;
    let r = f64::from(c) / f64::from(k.w);
    // The keyframe is a map picture: x linear in longitude, z linear from the north pole, so it is a
    // block-space picture whose rows start at the pole (rounded: its rows are 2R/h blocks apart,
    // within a block of r over the whole height at the π : 1 aspect).
    let pole = deeptime_format::Projection::new(c).pole_z().round() as i64;
    Ok(Some(json!({
        "name": SATELLITE_LAYER, "slug": slug(SATELLITE_LAYER), "units": "", "scale": 1.0,
        "offset": 0.0, "nodata": null, "ramp": SATELLITE_LAYER,
        "grid": { "name": "keyframe", "w": k.w, "h": k.h, "r": r, "wraps_x": true,
                  "origin_block_x": -i64::from(c / 2),
                  "origin_block_z": -pole },
        "image": { "file": file, "w": k.w, "h": k.h, "stride": 1 },
        "values": null,
        "view": { "n": k.n, "t": k.t, "stage": k.stage },
    })))
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
    )?;
    let bodies = deeptime_format::bodies::read_bodies(world)
        .map_err(io::Error::other)?
        .unwrap_or_default();
    if let Some(report) = deposit_report(m.notes.get("deposits"), &bodies) {
        write_report(dir, &report)?;
    }
    Ok(())
}

/// Writes `report.js` (`window.DT_REPORT = …`) next to a bundle's `index.html`: stage reports
/// the page shows beside the timeline. P1-14 puts `{"deposits": <census>, "bodies": [...]}` there
/// (`FORMAT.md`, "Bodies"); the page draws the census table and each site's body section.
pub fn write_report(dir: &Path, report: &Value) -> io::Result<()> {
    fs::write(
        dir.join("report.js"),
        format!("window.DT_REPORT = {report};\n"),
    )
}

/// The report of a world's deposit stage: its census (from `manifest.json` notes or the world's
/// notes) and every site's bodies (bodies without a site, such as P1-07's plutons, are left out:
/// the page draws bodies per site).
pub fn deposit_report(census: Option<&Value>, bodies: &[Body]) -> Option<Value> {
    let sited: Vec<Value> = bodies
        .iter()
        .filter(|b| b.site.is_some())
        .map(Body::to_json)
        .collect();
    if census.is_none() && sited.is_empty() {
        return None;
    }
    Some(json!({
        "deposits": census.cloned().unwrap_or(Value::Null),
        "bodies": sited,
    }))
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

/// The workbench page's `index.html` with `title` (`deeptime watch`).
pub fn workbench_page(title: &str) -> String {
    WORKBENCH_HTML.replace("{{TITLE}}", &html_escape(title))
}

/// RGBA8 (`w·h·4` bytes) as PNG bytes, with png's fast deflate: the live page asks for a frame's
/// layer once, while the run goes on.
pub fn encode_png(w: u32, h: u32, rgba: &[u8]) -> io::Result<Vec<u8>> {
    if rgba.len() as u64 != u64::from(w) * u64::from(h) * 4 {
        return Err(io::Error::other(format!(
            "{} bytes are not a {w}×{h} RGBA image",
            rgba.len()
        )));
    }
    let mut out = Vec::with_capacity(rgba.len() / 4);
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_compression(png::Compression::Fast);
    let mut writer = enc.write_header().map_err(io::Error::other)?;
    writer.write_image_data(rgba).map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;
    Ok(out)
}

/// RGB8 (`w·h·3` bytes) as PNG bytes, with png's fast deflate: the workbench's map tiles, which
/// have no transparent pixels.
pub fn encode_png_rgb(w: u32, h: u32, rgb: &[u8]) -> io::Result<Vec<u8>> {
    if rgb.len() as u64 != u64::from(w) * u64::from(h) * 3 {
        return Err(io::Error::other(format!(
            "{} bytes are not a {w}×{h} RGB image",
            rgb.len()
        )));
    }
    let mut out = Vec::with_capacity(rgb.len() / 3);
    let mut enc = png::Encoder::new(&mut out, w, h);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_compression(png::Compression::Fast);
    let mut writer = enc.write_header().map_err(io::Error::other)?;
    writer.write_image_data(rgb).map_err(io::Error::other)?;
    writer.finish().map_err(io::Error::other)?;
    Ok(out)
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
        assert_eq!(super::FORMAT_VERSION, "0.6");
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
    fn workbench_page_and_png() {
        let page = workbench_page("live <run>");
        assert!(page.contains("<title>live &lt;run&gt;</title>"));
        assert!(page.contains("workbench.js") && !page.contains("{{TITLE}}"));
        assert!(WORKBENCH_JS.contains("EventSource"));
        assert!(page.contains("deposits.js") && DEPOSITS_JS.contains("DTDeposits"));
        let fields: Value = serde_json::from_str(EDITOR_FIELDS_JSON).unwrap();
        assert!(fields["fields"].as_array().unwrap().len() > 10);
        let lang: Value = serde_json::from_str(LANG_EN_US_JSON).unwrap();
        assert!(lang["deeptime.editor.field.seed"].is_string());
        let rgba: Vec<u8> = (0..4 * 3 * 4).map(|i| (i * 7) as u8).collect();
        let png = encode_png(4, 3, &rgba).unwrap();
        assert_eq!(&png[1..4], b"PNG");
        let dec = png::Decoder::new(std::io::Cursor::new(&png));
        let mut reader = dec.read_info().unwrap();
        let mut buf = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut buf).unwrap();
        assert_eq!(buf, rgba, "lossless");
        assert!(encode_png(4, 4, &rgba).is_err());
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

    #[test]
    fn the_last_satellite_frame_leads_the_world_layers() {
        let dir = std::env::temp_dir().join(format!("dt-viewer-sat-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("layers")).unwrap();
        let frame = |n: u32, name: &str| KeyframeImage {
            n,
            t: -1.0e7 + f64::from(n),
            stage: "plates+climate".into(),
            w: 8,
            h: 4,
            layers: vec![(name.into(), vec![n as u8; 8 * 4 * 4])],
        };
        let t = Timeline {
            keyframes: vec![
                frame(0, "satellite"),
                frame(2, "deposits/sites"),
                frame(1, "satellite"),
            ],
            ..Timeline::default()
        };
        let e = satellite_entry(&dir, 1024, &t).unwrap().unwrap();
        assert_eq!(e["view"]["n"], 1, "the last frame that has one");
        assert_eq!(e["grid"]["r"], 128.0);
        assert_eq!(e["grid"]["origin_block_x"], -512);
        assert_eq!(e["ramp"], "satellite");
        assert!(e["values"].is_null());
        assert!(dir.join("layers/satellite.png").is_file());
        assert!(
            satellite_entry(&dir, 1024, &Timeline::default())
                .unwrap()
                .is_none()
        );
        fs::remove_dir_all(&dir).unwrap();
    }
}
