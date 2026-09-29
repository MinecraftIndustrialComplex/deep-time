// SPDX-License-Identifier: LGPL-3.0-or-later
//! A field history recorded by the real pipeline (P1-04): the frames the history stages publish
//! for the deposit engine (`fields.rs`), kept in the world state so a checkpoint carries them and
//! pass 2 can replay them.
//!
//! Frames are packed: each field is quantised over its own range in that frame — 16 bits for the
//! elevation and the erosion and deposition rates, 8 bits for the rest (code 0 is NaN) — and
//! LZ4-compressed (most facies and arc fields are zero over most of the planet and shrink to almost
//! nothing). The error is at
//! most half a step of the frame's range: 1/131,070 of it at 16 bits, 1/508 at 8 bits (1 °C of
//! burial temperature, 12 m of elevation-sized ranges). 16k quick: about 4 MiB per frame.

use std::collections::BTreeMap;

use deeptime_format::GridDesc;
use serde::{Deserialize, Serialize};

use super::fields::{
    FieldFrame, FieldHistory, HOST_PREFIX, ORIGIN_FIELDS, origins_from_offsets, spec,
};
use crate::error::CoreError;

/// One packed field of a frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackedField {
    pub name: String,
    pub lo: f32,
    pub hi: f32,
    /// 8 or 16.
    pub bits: u8,
    /// LZ4 (size-prepended) of the codes, little-endian.
    pub data: Vec<u8>,
}

/// One packed frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PackedFrame {
    pub t_years: f64,
    pub dt_years: f64,
    pub fields: Vec<PackedField>,
    pub clocks: BTreeMap<String, f32>,
}

/// The recorded history of one run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RecordedHistory {
    pub grid: GridDesc,
    pub frames: Vec<PackedFrame>,
    /// Present-day drainage (L1 cell → downstream cell), set when the history ends.
    pub receivers: Option<Vec<u32>>,
}

/// Fields kept at 16 bits: the ones whose small values matter over a wide range (elevation for
/// the ledger's burial, the heavy-tailed rates). Everything else is 8-bit over the frame's range.
const WIDE: [&str; 3] = [
    "deeptime:elevation_m",
    "deeptime:erosion_rate_m_myr",
    "deeptime:deposition_rate_m_myr",
];

fn bits_for(name: &str) -> u8 {
    // Host-rock fractions (`deeptime:host/<class>`, 0–1) are an open family outside the specs.
    let known = spec(name).is_some() || name.starts_with(HOST_PREFIX);
    if WIDE.contains(&name) || !known {
        16
    } else {
        8
    }
}

fn pack(name: &str, v: &[f32]) -> PackedField {
    let bits = bits_for(name);
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for &x in v {
        if x.is_finite() {
            lo = lo.min(x);
            hi = hi.max(x);
        }
    }
    if !lo.is_finite() {
        (lo, hi) = (0.0, 0.0);
    }
    let max = if bits == 8 { 255.0 } else { 65535.0 };
    let span = f64::from(hi) - f64::from(lo);
    let code = |x: f32| -> u32 {
        if !x.is_finite() {
            0
        } else if span <= 0.0 {
            1
        } else {
            1 + ((f64::from(x) - f64::from(lo)) / span * (max - 1.0)).round() as u32
        }
    };
    let raw: Vec<u8> = if bits == 8 {
        v.iter().map(|&x| code(x) as u8).collect()
    } else {
        v.iter()
            .flat_map(|&x| (code(x) as u16).to_le_bytes())
            .collect()
    };
    PackedField {
        name: name.to_owned(),
        lo,
        hi,
        bits,
        data: lz4_flex::compress_prepend_size(&raw),
    }
}

fn unpack(f: &PackedField, cells: usize) -> Result<Vec<f32>, CoreError> {
    let raw = lz4_flex::decompress_size_prepended(&f.data)
        .map_err(|e| CoreError::InvalidState(format!("history field {}: {e}", f.name)))?;
    let max = if f.bits == 8 { 255.0 } else { 65535.0 };
    let span = f64::from(f.hi) - f64::from(f.lo);
    let value = |code: u32| -> f32 {
        if code == 0 {
            f32::NAN
        } else if span <= 0.0 {
            f.lo
        } else {
            (f64::from(f.lo) + f64::from(code - 1) / (max - 1.0) * span) as f32
        }
    };
    let out: Vec<f32> = if f.bits == 8 {
        raw.iter().map(|&b| value(u32::from(b))).collect()
    } else {
        raw.as_chunks::<2>()
            .0
            .iter()
            .map(|b| value(u32::from(u16::from_le_bytes(*b))))
            .collect()
    };
    if out.len() != cells {
        return Err(CoreError::InvalidState(format!(
            "history field {}: {} cells, the grid has {cells}",
            f.name,
            out.len()
        )));
    }
    Ok(out)
}

impl RecordedHistory {
    pub fn new(grid: GridDesc) -> RecordedHistory {
        RecordedHistory {
            grid,
            frames: Vec::new(),
            receivers: None,
        }
    }

    /// Packs and appends `frame` (fields in name order).
    pub fn push(&mut self, frame: &FieldFrame) {
        let fields = frame.fields.iter().map(|(name, v)| pack(name, v)).collect();
        self.frames.push(PackedFrame {
            t_years: frame.t_years,
            dt_years: frame.dt_years,
            fields,
            clocks: frame.clocks.clone(),
        });
    }

    /// Adds fields to frame `k` (packed like [`RecordedHistory::push`]), replacing fields of the
    /// same name: stages that step after the one that closed the frame publish into it this way
    /// (P1-07's `trap_structure` and host fractions). Keeps the frame's fields in name order.
    pub fn add_fields(
        &mut self,
        k: usize,
        fields: &BTreeMap<String, Vec<f32>>,
    ) -> Result<(), CoreError> {
        let cells = self.grid.cells();
        let frame = self
            .frames
            .get_mut(k)
            .ok_or_else(|| CoreError::InvalidArgument(format!("no frame {k}")))?;
        for (name, v) in fields {
            if v.len() != cells {
                return Err(CoreError::InvalidArgument(format!(
                    "history field {name}: {} cells, the grid has {cells}",
                    v.len()
                )));
            }
            let packed = pack(name, v);
            match frame.fields.binary_search_by(|f| f.name.as_str().cmp(name)) {
                Ok(i) => frame.fields[i] = packed,
                Err(i) => frame.fields.insert(i, packed),
            }
        }
        Ok(())
    }

    /// Compressed bytes held.
    pub fn bytes(&self) -> u64 {
        self.frames
            .iter()
            .flat_map(|f| f.fields.iter())
            .map(|f| f.data.len() as u64)
            .sum()
    }
}

impl FieldHistory for RecordedHistory {
    fn grid(&self) -> &GridDesc {
        &self.grid
    }

    fn frame_count(&self) -> usize {
        self.frames.len()
    }

    fn frame(&self, k: usize) -> Result<FieldFrame, CoreError> {
        let p = self
            .frames
            .get(k)
            .ok_or_else(|| CoreError::InvalidArgument(format!("no frame {k}")))?;
        let cells = self.grid.cells();
        let mut fields = BTreeMap::new();
        for f in &p.fields {
            fields.insert(f.name.clone(), unpack(f, cells)?);
        }
        Ok(FieldFrame {
            t_years: p.t_years,
            dt_years: p.dt_years,
            fields,
            clocks: p.clocks.clone(),
        })
    }

    fn names(&self) -> Vec<String> {
        self.frames.first().map_or_else(Vec::new, |f| {
            f.fields
                .iter()
                .map(|x| x.name.clone())
                .chain(f.clocks.keys().cloned())
                .collect()
        })
    }

    fn receivers(&self) -> Option<&[u32]> {
        self.receivers.as_deref()
    }

    fn origins(&self, k: usize) -> Result<Option<Vec<u32>>, CoreError> {
        let Some(p) = self.frames.get(k) else {
            return Err(CoreError::InvalidArgument(format!("no frame {k}")));
        };
        if k == 0 {
            return Ok(None);
        }
        let find = |name: &str| p.fields.iter().find(|f| f.name == name);
        let (Some(di), Some(dj)) = (find(ORIGIN_FIELDS[0]), find(ORIGIN_FIELDS[1])) else {
            return Ok(None);
        };
        let cells = self.grid.cells();
        Ok(Some(origins_from_offsets(
            self.grid.w as usize,
            &unpack(di, cells)?,
            &unpack(dj, cells)?,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_round_trip_within_a_quantum() {
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let n = grid.cells();
        let mut h = RecordedHistory::new(grid);
        let mut frame = FieldFrame {
            t_years: -1.0e7,
            dt_years: 1.0e7,
            ..FieldFrame::default()
        };
        let elev: Vec<f32> = (0..n)
            .map(|c| (c as f32 * 37.0) % 9000.0 - 4500.0)
            .collect();
        let mut shelf = vec![0.0f32; n];
        shelf[5] = 1.0;
        shelf[6] = 0.5;
        let mut nan = elev.clone();
        nan[3] = f32::NAN;
        frame
            .fields
            .insert("deeptime:elevation_m".into(), elev.clone());
        frame.fields.insert("deeptime:shelf".into(), shelf.clone());
        frame.fields.insert("deeptime:heat_flow_mw_m2".into(), nan);
        frame
            .clocks
            .insert("deeptime:clock/land_plants".into(), 1.0);
        h.push(&frame);
        let back = h.frame(0).unwrap();
        assert_eq!(back.t_years, -1.0e7);
        let e = &back.fields["deeptime:elevation_m"];
        let step = 9000.0 / 65534.0;
        assert!(e.iter().zip(&elev).all(|(a, b)| (a - b).abs() <= step));
        let s = &back.fields["deeptime:shelf"];
        assert!((s[5] - 1.0).abs() < 1e-6 && (s[6] - 0.5).abs() < 0.5 / 254.0 + 1e-6);
        assert!(back.fields["deeptime:heat_flow_mw_m2"][3].is_nan());
        assert_eq!(h.names().len(), 4);
        // Two dense 16-bit fields of incompressible test data and a nearly empty fraction.
        assert!(h.bytes() < (n * 5) as u64, "{} bytes", h.bytes());
        // A later stage adds a field and replaces one; the names stay sorted.
        let mut more = BTreeMap::new();
        more.insert("deeptime:trap_structure".to_owned(), vec![0.25f32; n]);
        more.insert("deeptime:shelf".to_owned(), vec![1.0f32; n]);
        h.add_fields(0, &more).unwrap();
        let back = h.frame(0).unwrap();
        assert!((back.fields["deeptime:trap_structure"][7] - 0.25).abs() < 1e-6);
        assert!(back.fields["deeptime:shelf"].iter().all(|&x| x == 1.0));
        let names: Vec<&str> = h.frames[0].fields.iter().map(|f| f.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        assert_eq!(names, sorted);
        assert!(h.add_fields(3, &more).is_err());
    }
}
