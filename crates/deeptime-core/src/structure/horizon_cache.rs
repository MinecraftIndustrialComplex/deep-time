// SPDX-License-Identifier: LGPL-3.0-or-later
//! The horizons baked once for the three bakes that read them (P1-19). The structure stage's last
//! step bakes the horizon stacks for its summary; the deposit stage's cover (`Cover::new`) and the
//! bake's strata pass (`crate::bake::strata`) used to bake them again from the same inputs, about
//! 100 s of CPU each at full 16k. The first bake keeps them here, LZ4-packed, beside the world;
//! a later bake takes them back when what it would read is what the first one read, and bakes its
//! own when it is not.
//!
//! "What it would read" is a [`Fingerprint`] of the bake's inputs: the settings and scalars as
//! they are (the grid, the scale and structure sections, the surface's time and sea level, the
//! run's start) and a CRC32 of every run of [`RUN`] values of each input array (the plate model's
//! pools, crust, regime, orogeny and heat flow, the climate's mean temperature, the surface's
//! rates, elevation, regolith and the whole strata log). Nothing between the structure stage's
//! end and the bake writes those (the stage finishes, the deposit stage, the young landscape and
//! its amplification write fields of their own), so the later bakes take the kept horizons; the
//! fingerprint is what makes that safe should a later change write one of them.
//!
//! The cache is not in checkpoints (a resumed run bakes again) and does not take part in the
//! world's equality.

use std::sync::Arc;

use deeptime_format::GridDesc;
use rayon::prelude::*;

use super::bake_horizons::Horizons;
use super::{CellSource, StructureParams};
use crate::params::PlanetPreset;
use crate::surface::strata_log::Layer;

/// Values per CRC32 of a [`Fingerprint`] and per LZ4 block of the kept horizons.
const RUN: usize = 1 << 16;

/// What a horizon bake reads, condensed ([`CellSource::fingerprint`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Fingerprint {
    /// The settings and scalars, exactly (postcard bytes and f64 bits).
    head: Vec<u8>,
    /// A CRC32 per run of [`RUN`] values of every input array, in order.
    runs: Vec<u32>,
}

/// One input array of the bake.
enum Input<'a> {
    F32(&'a [f32]),
    U8(&'a [u8]),
    Log(&'a [Layer]),
}

impl Input<'_> {
    fn len(&self) -> usize {
        match self {
            Input::F32(v) => v.len(),
            Input::U8(v) => v.len(),
            Input::Log(v) => v.len(),
        }
    }

    /// CRC32 of values `from..to` (little-endian bytes; a layer's four fields in order).
    fn crc(&self, from: usize, to: usize) -> u32 {
        let mut h = crc32fast::Hasher::new();
        let mut buf = [0u8; 12 * 256];
        match self {
            Input::U8(v) => h.update(&v[from..to]),
            Input::F32(v) => {
                for part in v[from..to].chunks(256) {
                    for (k, x) in part.iter().enumerate() {
                        buf[4 * k..4 * k + 4].copy_from_slice(&x.to_bits().to_le_bytes());
                    }
                    h.update(&buf[..4 * part.len()]);
                }
            }
            Input::Log(v) => {
                for part in v[from..to].chunks(256) {
                    for (k, l) in part.iter().enumerate() {
                        let b = &mut buf[12 * k..12 * k + 12];
                        b[0..4].copy_from_slice(&l.age_myr.to_bits().to_le_bytes());
                        b[4..8].copy_from_slice(&l.thickness_m.to_bits().to_le_bytes());
                        b[8..10].copy_from_slice(&l.class.to_le_bytes());
                        b[10..12].copy_from_slice(&l.provenance.to_le_bytes());
                    }
                    h.update(&buf[..12 * part.len()]);
                }
            }
        }
        h.finalize()
    }
}

impl CellSource<'_> {
    /// What [`super::bake_horizons::bake_with`] reads of this source with `preset` and `p`.
    pub fn fingerprint(&self, preset: &PlanetPreset, p: &StructureParams) -> Fingerprint {
        let s = self.surface;
        let mut head = postcard::to_stdvec(&(&self.grid, &preset.scale, p, &self.p))
            .expect("the bake's settings serialise");
        for x in [self.now_myr, self.sea_level_m, self.t_start_myr] {
            head.extend_from_slice(&x.to_bits().to_le_bytes());
        }
        head.push(u8::from(self.t_mean.is_some()));
        head.push(u8::from(self.rates.is_some()));
        head.extend_from_slice(&s.log.k.to_le_bytes());
        let mut inputs: Vec<Input<'_>> = self.pools.iter().map(|&v| Input::F32(v)).collect();
        inputs.extend(
            [
                self.crust,
                self.regime,
                self.orogeny,
                self.orogeny_age,
                self.heat,
                s.elevation_m.as_slice(),
                s.regolith_m.as_slice(),
            ]
            .map(Input::F32),
        );
        if let Some(t) = self.t_mean {
            inputs.push(Input::F32(t));
        }
        if let Some((dep, ero)) = self.rates {
            inputs.push(Input::F32(dep));
            inputs.push(Input::F32(ero));
        }
        inputs.push(Input::U8(&s.regolith_kind));
        inputs.push(Input::U8(&s.log.count));
        inputs.push(Input::Log(&s.log.layers));
        let mut jobs = Vec::new();
        for (k, input) in inputs.iter().enumerate() {
            head.extend_from_slice(&(input.len() as u64).to_le_bytes());
            jobs.extend(
                (0..input.len())
                    .step_by(RUN)
                    .map(|from| (k, from, (from + RUN).min(input.len()))),
            );
        }
        let runs = jobs
            .par_iter()
            .map(|&(k, from, to)| inputs[k].crc(from, to))
            .collect();
        Fingerprint { head, runs }
    }
}

/// The kept horizons: [`Horizons`] with its three arrays LZ4-packed in runs of [`RUN`] values.
struct Kept {
    fingerprint: Fingerprint,
    grid: GridDesc,
    k: u32,
    min_y: i32,
    tongues: (u32, f64),
    min_blocks: Vec<u8>,
    slots: usize,
    tops: Vec<Vec<u8>>,
    classes: Vec<Vec<u8>>,
    keys: Vec<Vec<u8>>,
}

impl Kept {
    fn bytes(&self) -> u64 {
        [&self.tops, &self.classes, &self.keys]
            .iter()
            .flat_map(|blocks| blocks.iter())
            .map(|b| b.len() as u64)
            .sum()
    }
}

/// Packs `values` (`B` little-endian bytes each, written by `put`) in runs of [`RUN`].
fn pack<T: Sync, const B: usize>(values: &[T], put: fn(&T) -> [u8; B]) -> Vec<Vec<u8>> {
    values
        .par_chunks(RUN)
        .map(|run| {
            let mut raw = Vec::with_capacity(run.len() * B);
            for v in run {
                raw.extend_from_slice(&put(v));
            }
            lz4_flex::block::compress(&raw)
        })
        .collect()
}

/// [`pack`] undone into `slots` values (`None`: a block does not unpack to its run).
fn unpack<T: Send + Copy + Default, const B: usize>(
    blocks: &[Vec<u8>],
    slots: usize,
    get: fn([u8; B]) -> T,
) -> Option<Vec<T>> {
    let mut out = vec![T::default(); slots];
    let ok = out
        .par_chunks_mut(RUN)
        .zip(blocks.par_iter())
        .all(|(run, block)| {
            let Ok(raw) = lz4_flex::block::decompress(block, run.len() * B) else {
                return false;
            };
            if raw.len() != run.len() * B {
                return false;
            }
            for (v, b) in run.iter_mut().zip(raw.as_chunks::<B>().0) {
                *v = get(*b);
            }
            true
        });
    (ok && blocks.len() == slots.div_ceil(RUN)).then_some(out)
}

/// The horizons of the last bake that kept them, with what it read (the module docs).
#[derive(Clone, Default)]
pub struct HorizonCache(Option<Arc<Kept>>);

impl HorizonCache {
    /// Keeps `hz`, baked from inputs with `fingerprint`, in place of what was kept.
    pub fn keep(&mut self, fingerprint: Fingerprint, hz: &Horizons) {
        self.0 = None;
        self.0 = Some(Arc::new(Kept {
            fingerprint,
            grid: hz.grid.clone(),
            k: hz.k,
            min_y: hz.min_y,
            tongues: hz.tongues,
            min_blocks: hz.min_blocks.clone(),
            slots: hz.tops.len(),
            tops: pack(&hz.tops, |v: &i16| v.to_le_bytes()),
            classes: pack(&hz.classes, |v: &u8| [*v]),
            keys: pack(&hz.keys, |v: &[u16; 2]| {
                let (a, b) = (v[0].to_le_bytes(), v[1].to_le_bytes());
                [a[0], a[1], b[0], b[1]]
            }),
        }));
    }

    /// The kept horizons when they were baked from inputs with `fingerprint`.
    pub fn get(&self, fingerprint: &Fingerprint) -> Option<Horizons> {
        let kept = self
            .0
            .as_deref()
            .filter(|k| k.fingerprint == *fingerprint)?;
        let (tops, (classes, keys)) = rayon::join(
            || unpack(&kept.tops, kept.slots, i16::from_le_bytes),
            || {
                rayon::join(
                    || unpack(&kept.classes, kept.slots, |[b]: [u8; 1]| b),
                    || {
                        unpack(&kept.keys, kept.slots, |b: [u8; 4]| {
                            [
                                u16::from_le_bytes([b[0], b[1]]),
                                u16::from_le_bytes([b[2], b[3]]),
                            ]
                        })
                    },
                )
            },
        );
        Some(Horizons {
            grid: kept.grid.clone(),
            k: kept.k,
            min_y: kept.min_y,
            tops: tops?,
            classes: classes?,
            keys: keys?,
            tongues: kept.tongues,
            min_blocks: kept.min_blocks.clone(),
        })
    }

    /// Bytes the packed horizons hold.
    pub fn bytes(&self) -> u64 {
        self.0.as_deref().map_or(0, Kept::bytes)
    }
}

impl PartialEq for HorizonCache {
    /// Always: a cache is not part of what a world is.
    fn eq(&self, _: &HorizonCache) -> bool {
        true
    }
}

impl std::fmt::Debug for HorizonCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HorizonCache({} bytes)", self.bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kept_horizons_come_back_whole_and_only_for_their_inputs() {
        let grid = GridDesc::for_planet("L1", 512, 16).unwrap();
        let k = 4usize;
        let slots = grid.cells() * k + 3;
        let hz = Horizons {
            grid: grid.clone(),
            k: k as u32,
            min_y: -64,
            tops: (0..slots).map(|i| (i as i16).wrapping_mul(37)).collect(),
            classes: (0..slots).map(|i| (i % 251) as u8).collect(),
            keys: (0..slots)
                .map(|i| [i as u16, (i as u16).wrapping_add(9)])
                .collect(),
            tongues: (3, 0.25),
            min_blocks: vec![1, 2, 3],
        };
        let fp = Fingerprint {
            head: vec![1, 2, 3],
            runs: vec![7, 8],
        };
        let mut cache = HorizonCache::default();
        assert!(cache.get(&fp).is_none());
        cache.keep(fp.clone(), &hz);
        assert!(cache.bytes() > 0);
        assert_eq!(cache.get(&fp), Some(hz.clone()));
        let other = Fingerprint {
            head: vec![1, 2, 3],
            runs: vec![7, 9],
        };
        assert!(cache.get(&other).is_none());
    }

    #[test]
    fn a_fingerprint_sees_one_value_change() {
        let a = vec![1.0f32; RUN + 5];
        let mut b = a.clone();
        b[RUN + 2] = 1.0 + f32::EPSILON;
        let (x, y) = (Input::F32(&a), Input::F32(&b));
        assert_eq!(x.crc(0, RUN), y.crc(0, RUN));
        assert_ne!(x.crc(RUN, RUN + 5), y.crc(RUN, RUN + 5));
        let log = vec![Layer::default(); 10];
        let mut log2 = log.clone();
        log2[4].provenance = 1;
        assert_ne!(Input::Log(&log).crc(0, 10), Input::Log(&log2).crc(0, 10));
    }
}
