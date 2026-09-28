// SPDX-License-Identifier: LGPL-3.0-or-later
//! The colour ramps shared by the core's keyframes, the viewer and (copied into the jar) the
//! in-game timelapse (plan §3.3): one file, `crates/deeptime-viewer/assets/ramps.json`. Stops are
//! `[physical value, "#rrggbb"]`, linearly interpolated and clamped at the ends.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::error::{FormatError, Result};

/// The ramps file, compiled in.
pub const RAMPS_JSON: &str = include_str!("../../deeptime-viewer/assets/ramps.json");

/// `ramps.json`.
#[derive(Clone, Debug, Deserialize)]
pub struct Ramps {
    pub ramps: BTreeMap<String, Ramp>,
    /// Layer name → ramp name.
    pub layers: BTreeMap<String, String>,
    /// The ramp of layers not in `layers`.
    pub default: String,
}

/// One ramp, its `#rrggbb` stops parsed once (keyframes colour every pixel with it).
#[derive(Clone, Debug, Deserialize)]
#[serde(from = "RampJson")]
pub struct Ramp {
    pub stops: Vec<(f64, [u8; 3])>,
}

#[derive(Deserialize)]
struct RampJson {
    stops: Vec<(f64, String)>,
}

impl From<RampJson> for Ramp {
    fn from(r: RampJson) -> Ramp {
        Ramp {
            stops: r.stops.iter().map(|(v, c)| (*v, parse_hex(c))).collect(),
        }
    }
}

impl Ramps {
    /// The compiled-in ramps.
    pub fn builtin() -> Ramps {
        Ramps::parse(RAMPS_JSON).expect("the built-in ramps.json parses")
    }

    pub fn parse(json: &str) -> Result<Ramps> {
        let r: Ramps = serde_json::from_str(json).map_err(|source| FormatError::Json {
            what: "ramps.json".into(),
            source,
        })?;
        if !r.ramps.contains_key(&r.default) {
            return Err(FormatError::Invalid(format!(
                "ramps.json: default ramp {:?} is not defined",
                r.default
            )));
        }
        Ok(r)
    }

    /// The ramp name and ramp for layer `layer` (its own, else the default).
    pub fn for_layer(&self, layer: &str) -> (&str, &Ramp) {
        let name = self.layers.get(layer).unwrap_or(&self.default);
        match self.ramps.get(name) {
            Some(r) => (name, r),
            None => (&self.default, &self.ramps[&self.default]),
        }
    }
}

fn parse_hex(s: &str) -> [u8; 3] {
    let s = s.trim_start_matches('#');
    let p = |k: usize| u8::from_str_radix(s.get(k..k + 2).unwrap_or("00"), 16).unwrap_or(0);
    [p(0), p(2), p(4)]
}

impl Ramp {
    /// The colour of physical value `v`.
    pub fn colour(&self, v: f64) -> [u8; 3] {
        let stops = &self.stops;
        if stops.is_empty() {
            return [128, 128, 128];
        }
        if v <= stops[0].0 {
            return stops[0].1;
        }
        for w in stops.windows(2) {
            let (a, b) = (&w[0], &w[1]);
            if v <= b.0 {
                let t = if b.0 > a.0 {
                    (v - a.0) / (b.0 - a.0)
                } else {
                    1.0
                };
                let (ca, cb) = (a.1, b.1);
                return std::array::from_fn(|k| {
                    let (x, y) = (f64::from(ca[k]), f64::from(cb[k]));
                    (x + (y - x) * t).round() as u8
                });
            }
        }
        stops[stops.len() - 1].1
    }

    /// RGBA: the colour with alpha 255, or transparent black for NaN (nodata).
    pub fn rgba(&self, v: f64) -> [u8; 4] {
        if v.is_nan() {
            return [0, 0, 0, 0];
        }
        let [r, g, b] = self.colour(v);
        [r, g, b, 255]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_ramps_cover_every_layer() {
        let r = Ramps::builtin();
        for ramp in r.layers.values() {
            assert!(r.ramps.contains_key(ramp), "{ramp}");
        }
        let (name, e) = r.for_layer("elevation");
        assert_eq!(name, "elevation");
        assert_eq!(e.colour(-1000.0), [0x08, 0x14, 0x2e]);
        assert_eq!(e.colour(63.0), [0x3f, 0x7f, 0x3a]);
        assert_eq!(e.colour(1000.0), [255, 255, 255]);
        assert_eq!(e.rgba(f64::NAN), [0, 0, 0, 0]);
        assert_eq!(r.for_layer("no/such/layer").0, r.default);
    }
}
