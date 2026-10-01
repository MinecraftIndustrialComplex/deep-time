// SPDX-License-Identifier: LGPL-3.0-or-later
//! A run's bake opened for rock views: the column evaluator ([`ColumnWorld`]) plus the whole
//! `strata/horizons` layer in memory (so many threads read stacks without a lock) and an index of
//! the bodies that name a rock class. Everything here only reads the world data.
//!
//! The rock at a block is what the game's strata pass paints: the horizon stack's class at the
//! block ([`deeptime_format::column::horizon_at`] over [`LoadedLayer::stack_at`], the column
//! rule), then a body's class where one names a class there (the first body in id order wins, as
//! `StrataPass.paintBodies` leaves it: a block a body painted is no longer crust or the horizon's
//! block, so a later body cannot take it).

use std::collections::HashMap;
use std::path::Path;

use deeptime_core::synth::EVERY_CLASS;
use deeptime_format::column::horizon_at;
use deeptime_format::sites::read_sites;
use deeptime_format::strata::{ColumnHorizon, StackScratch};
use deeptime_format::{
    Body, ColumnWorld, FormatError, LoadedLayer, Ramps, Site, SiteId, TilePackReader,
};

/// "No rock here": air, or a block no horizon and no body names.
pub const NONE: u16 = u16::MAX;

/// The widest bucket edge of the body index, blocks (the edge used divides the circumference).
const BUCKET: i64 = 256;

/// The bucket edge for a planet of circumference `c`: 256 or the largest power of two under it that
/// divides `c`, so the buckets tile the wrap exactly.
fn bucket_for(c: i64) -> i64 {
    let mut b = BUCKET;
    while b > 1 && c % b != 0 {
        b /= 2;
    }
    b
}

/// An index of bodies by the buckets of the world's block grid their footprint touches.
struct BodyGrid {
    c: i64,
    bucket: i64,
    nx: i64,
    nz: i64,
    /// Body indices (into `ColumnWorld::bodies`, ascending) per bucket, row by row.
    buckets: Vec<Vec<u32>>,
}

impl BodyGrid {
    fn new(c: i64, bodies: &[Body], pick: impl Fn(&Body) -> bool) -> BodyGrid {
        let bucket = bucket_for(c);
        let nx = (c / bucket).max(1);
        let nz = nx;
        let mut buckets = vec![Vec::new(); (nx * nz) as usize];
        for (n, b) in bodies.iter().enumerate() {
            if !pick(b) {
                continue;
            }
            let [lo, hi] = b.aabb;
            let (i0, i1) = (
                (i64::from(lo[0]) + c / 2).div_euclid(bucket),
                (i64::from(hi[0]) + c / 2).div_euclid(bucket),
            );
            let (j0, j1) = (
                ((i64::from(lo[2]) + c / 2).div_euclid(bucket)).clamp(0, nz - 1),
                ((i64::from(hi[2]) + c / 2).div_euclid(bucket)).clamp(0, nz - 1),
            );
            let wraps_all = i1 - i0 + 1 >= nx;
            for j in j0..=j1 {
                if wraps_all {
                    for i in 0..nx {
                        buckets[(j * nx + i) as usize].push(n as u32);
                    }
                } else {
                    for i in i0..=i1 {
                        buckets[(j * nx + i.rem_euclid(nx)) as usize].push(n as u32);
                    }
                }
            }
        }
        BodyGrid {
            c,
            bucket,
            nx,
            nz,
            buckets,
        }
    }

    /// The bodies whose bucket holds column (wx, z), ascending.
    fn near(&self, wx: i64, z: i64) -> &[u32] {
        let i = ((wx + self.c / 2).rem_euclid(self.c) / self.bucket).min(self.nx - 1);
        let j = ((z + self.c / 2).div_euclid(self.bucket)).clamp(0, self.nz - 1);
        &self.buckets[(j * self.nx + i) as usize]
    }
}

/// Whether the footprint of `b` holds column (wx, z) (x wraps), as `ColumnWorld::bodies_at`.
fn footprint_holds(b: &Body, c: i64, wx: i64, z: i64) -> bool {
    let [lo, hi] = b.aabb;
    if z < i64::from(lo[2]) || z > i64::from(hi[2]) {
        return false;
    }
    let span = i64::from(hi[0]) - i64::from(lo[0]);
    span >= c - 1 || (wx - i64::from(lo[0])).rem_euclid(c) <= span
}

/// One run's bake, ready for slices and sections.
pub struct RockWorld {
    pub cw: ColumnWorld,
    strata: LoadedLayer,
    /// Bake class index → colour (`rock_class` ramp, by the class's name).
    pub colours: Vec<[u8; 3]>,
    by_name: HashMap<String, u16>,
    /// Bodies that name a rock class (their own or a zone's).
    painters: BodyGrid,
    /// Every body, for the deposits a section crosses.
    all: BodyGrid,
    sites: HashMap<SiteId, Site>,
}

impl RockWorld {
    pub fn open(world: &Path) -> Result<RockWorld, FormatError> {
        let cw = ColumnWorld::open(world)?;
        let entry = cw
            .manifest
            .layer(deeptime_format::column::HORIZONS)
            .ok_or_else(|| FormatError::Invalid("the world data has no strata layer".into()))?;
        let strata = TilePackReader::open(&world.join(&entry.path))?.load()?;
        let c = cw.c;
        let by_name: HashMap<String, u16> = cw
            .classes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i as u16))
            .collect();
        let painters = BodyGrid::new(c, &cw.bodies, |b| {
            b.rock_class.is_some() || b.zones.iter().any(|z| z.rock_class.is_some())
        });
        let all = BodyGrid::new(c, &cw.bodies, |_| true);
        let colours = class_colours(&cw.classes);
        let sites = read_sites(world)?
            .map(|(sites, _)| sites.into_iter().map(|s| (s.id, s)).collect())
            .unwrap_or_default();
        Ok(RockWorld {
            cw,
            strata,
            colours,
            by_name,
            painters,
            all,
            sites,
        })
    }

    /// The site `id`, if the world has sites.
    pub fn site(&self, id: SiteId) -> Option<&Site> {
        self.sites.get(&id)
    }

    /// The horizon stack at block (x, z).
    pub fn stack(&self, scratch: &mut StackScratch, x: i64, z: i64, out: &mut Vec<ColumnHorizon>) {
        self.strata.stack_at_with(scratch, x, z, self.cw.min_y, out);
    }

    /// The class a painting body gives block (wx, y, z), if one does (the first by id).
    fn body_class_at(&self, wx: i64, y: i32, z: i64) -> Option<u16> {
        for &n in self.painters.near(wx, z) {
            let b = &self.cw.bodies[n as usize];
            if y < b.aabb[0][1] || y > b.aabb[1][1] || !footprint_holds(b, self.cw.c, wx, z) {
                continue;
            }
            let Some(at) = b.locate(self.cw.c as u32, wx as i32, y, z as i32) else {
                continue;
            };
            let mut class = b.rock_class.as_deref();
            for zone in b.zones_at(at) {
                if let Some(rc) = zone.rock_class.as_deref() {
                    class = Some(rc);
                }
            }
            if let Some(i) = class.and_then(|n| self.by_name.get(n)) {
                return Some(*i);
            }
        }
        None
    }

    /// The rock class at block (x, y, z) given the column's horizon stack: a body's, else the
    /// horizon's; [`NONE`] above the first horizon.
    pub fn class_at(&self, x: i64, y: i32, z: i64, stack: &[ColumnHorizon]) -> u16 {
        let wx = self.cw.wrap_x(x);
        if let Some(k) = self.body_class_at(wx, y, z) {
            return k;
        }
        horizon_at(stack, y).map_or(NONE, |h| u16::from(stack[h].1))
    }

    /// Every body whose footprint holds column (x, z), ascending id.
    pub fn bodies_over(&self, x: i64, z: i64) -> impl Iterator<Item = &Body> {
        let wx = self.cw.wrap_x(x);
        let c = self.cw.c;
        self.all
            .near(wx, z)
            .iter()
            .map(|&n| &self.cw.bodies[n as usize])
            .filter(move |b| footprint_holds(b, c, wx, z))
    }

    /// Whether block (wx, y, z) is inside body `b` (for the section's deposit spans).
    pub fn inside(&self, b: &Body, wx: i64, y: i32, z: i64) -> bool {
        b.locate(self.cw.c as u32, wx as i32, y, z as i32).is_some()
    }

    /// Paints the classes the painting bodies give column (x, z) into `out`, where `out[i]` is
    /// block y = `hi` − i (down to `lo`); only blocks still [`NONE`] are written, so the first
    /// body by id keeps what it painted.
    pub fn paint_column(&self, x: i64, z: i64, lo: i32, hi: i32, out: &mut [u16]) {
        let wx = self.cw.wrap_x(x);
        let c = self.cw.c;
        for &n in self.painters.near(wx, z) {
            let b = &self.cw.bodies[n as usize];
            if !footprint_holds(b, c, wx, z) {
                continue;
            }
            let (y0, y1) = (b.aabb[0][1].max(lo), b.aabb[1][1].min(hi));
            for y in (y0..=y1).rev() {
                let slot = &mut out[(hi - y) as usize];
                if *slot != NONE {
                    continue;
                }
                let Some(at) = b.locate(c as u32, wx as i32, y, z as i32) else {
                    continue;
                };
                let mut class = b.rock_class.as_deref();
                for zone in b.zones_at(at) {
                    if let Some(rc) = zone.rock_class.as_deref() {
                        class = Some(rc);
                    }
                }
                if let Some(i) = class.and_then(|n| self.by_name.get(n)) {
                    *slot = *i;
                }
            }
        }
    }
}

/// The `rock_class` ramp's colour for each class of the bake, found by name in the ramp's own
/// order (`synth::EVERY_CLASS`); a class the ramp does not know is grey.
pub fn class_colours(classes: &[String]) -> Vec<[u8; 3]> {
    let ramps = Ramps::builtin();
    let ramp = &ramps.ramps["rock_class"];
    classes
        .iter()
        .map(|n| {
            EVERY_CLASS
                .iter()
                .position(|e| e == n)
                .map_or([128, 128, 128], |k| ramp.colour(k as f64))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::Shape;
    use deeptime_format::bodies::NO_CLIP;

    #[test]
    fn buckets_tile_the_wrap() {
        for c in [1200i64, 1000, 4096, 16384, 32768] {
            assert_eq!(c % bucket_for(c), 0, "{c}");
        }
        assert_eq!(bucket_for(16384), 256);
        assert_eq!(bucket_for(1200), 16);
    }

    /// A body over the seam is found from the column on either side of it, and a column far
    /// from every body finds none.
    #[test]
    fn the_index_finds_a_body_from_both_sides_of_the_seam() {
        for c in [4096i64, 1200] {
            let body = Body::new(
                7,
                None,
                Shape::Ellipsoid,
                "pluton",
                [(c / 2 - 3) as f64, 0.0, 10.0],
                [40.0, 10.0, 40.0],
                0.0,
                0.0,
                NO_CLIP,
            );
            let grid = BodyGrid::new(c, std::slice::from_ref(&body), |_| true);
            for wx in [c / 2 - 30, c / 2 - 1, -c / 2, -c / 2 + 20] {
                assert_eq!(grid.near(wx, 10), [0], "C {c}, x {wx}");
                assert!(footprint_holds(&body, c, wx, 10), "C {c}, x {wx}");
            }
            assert!(grid.near(0, 10).is_empty(), "C {c}");
            assert!(grid.near(c / 2 - 30, 1000).is_empty(), "C {c}");
        }
    }
}
