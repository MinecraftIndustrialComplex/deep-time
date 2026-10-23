// SPDX-License-Identifier: LGPL-3.0-or-later
//! The rock class at a depth below the surface, over the whole planet map: "Rock at depth".
//!
//! Pixel (px, py) of a w × h map is what a keyframe pixel is: centred on longitude 360°·px/w (the
//! seam is 0°) and on the row the projection gives py (`Projection::picture_z`: Lambert, sin φ =
//! 1 − 2·py/h), i.e. block x = −C/2 + C·px/w, z = −R + 2R·py/h,
//! so the picture sits under the workbench's map exactly. On a Petroff–Guyou world (format 0.5) the
//! map is the world itself, 2 : 1: z = −b/2 + b·py/h with C = 2b. The depth is below the terrain's top
//! block there (⌈H⌉ − 1, the evaluator's `top_y`), H the generated terrain height; `Floor` is the
//! lowest block of the world.

use std::sync::Mutex;

use deeptime_format::strata::StackScratch;

use super::world::{NONE, RockWorld};

/// How far down a slice looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Depth {
    /// This many blocks below the terrain's top block (0 = the surface rock).
    Below(u32),
    /// The world's lowest block, whatever the terrain above it.
    Floor,
}

impl Depth {
    /// `"100"` or `"floor"`.
    pub fn parse(s: &str) -> Option<Depth> {
        if s == "floor" {
            return Some(Depth::Floor);
        }
        s.parse::<u32>().ok().map(Depth::Below)
    }

    pub fn label(self) -> String {
        match self {
            Depth::Below(d) => d.to_string(),
            Depth::Floor => "floor".into(),
        }
    }
}

/// A map of class indices (bake order; [`NONE`] = nothing).
pub struct Slice {
    pub w: u32,
    pub h: u32,
    pub class: Vec<u16>,
}

/// The block (x, z) of pixel (px, py) on a w × h map of a world laid out by `wp`.
pub fn pixel_block(
    wp: &deeptime_format::projection::WorldProjection,
    c: i64,
    w: u32,
    h: u32,
    px: u32,
    py: u32,
) -> (i64, i64) {
    use deeptime_format::projection::WorldProjection;
    if let WorldProjection::Cube(v) = wp {
        // The cube's window: x from −3s/2 over 4s, z from −3s/2 over 3s.
        let s = v.cube.side();
        let x = -3 * s / 2 + 4 * s * i64::from(px) / i64::from(w.max(1));
        let z = -3 * s / 2 + 3 * s * i64::from(py) / i64::from(h.max(1));
        return (x, z);
    }
    let x = -c / 2 + c * i64::from(px) / i64::from(w);
    let z = match wp {
        WorldProjection::Lambert(p) => p.picture_z(f64::from(py), h).floor() as i64,
        WorldProjection::PetroffGuyou(g) => {
            let b = g.hemisphere() as f64;
            (-b / 2.0 + b * f64::from(py) / f64::from(h.max(1))).floor() as i64
        }
        WorldProjection::Cube(_) => unreachable!(),
    };
    (x, z)
}

/// The height of a whole-world map `w` pixels wide: π : 1 on Lambert, 2 : 1 on Petroff–Guyou,
/// 4 : 3 on a cube's window.
pub fn picture_height(wp: &deeptime_format::projection::WorldProjection, w: u32) -> u32 {
    match wp {
        deeptime_format::projection::WorldProjection::Lambert(_) => {
            deeptime_format::Projection::picture_height(w)
        }
        deeptime_format::projection::WorldProjection::PetroffGuyou(_) => (w / 2).max(1),
        deeptime_format::projection::WorldProjection::Cube(_) => (3 * w / 4).max(1),
    }
}

impl RockWorld {
    /// The rock at `depth` over a w × h map, rows split among `threads` workers.
    pub fn slice(&self, depth: Depth, w: u32, h: u32, threads: usize) -> Slice {
        let c = self.cw.c;
        let wp = self.cw.manifest.world_projection();
        let (min_y, max_y) = (self.cw.min_y, self.cw.max_y);
        let mut class = vec![NONE; w as usize * h as usize];
        // Rows are claimed one at a time, so a slow row (mountains, many bodies) does not hold a
        // worker's whole share up; every worker writes only the rows it claimed.
        let queue = Mutex::new(class.chunks_mut(w as usize).enumerate());
        std::thread::scope(|s| {
            for _ in 0..threads.max(1) {
                s.spawn(|| {
                    let (mut stack, mut scratch) = (Vec::new(), StackScratch::default());
                    loop {
                        let next = queue.lock().unwrap_or_else(|e| e.into_inner()).next();
                        let Some((py, row)) = next else { break };
                        for px in 0..w {
                            let (x, z) = pixel_block(&wp, c, w, h, px, py as u32);
                            let surface = self.cw.height(x, z);
                            if !surface.is_finite() {
                                continue;
                            }
                            let top = ((surface.ceil() as i64) - 1)
                                .clamp(i64::from(min_y) - 1, i64::from(max_y))
                                as i32;
                            let y = match depth {
                                Depth::Below(d) => (i64::from(top) - i64::from(d))
                                    .clamp(i64::from(min_y), i64::from(max_y))
                                    as i32,
                                Depth::Floor => min_y,
                            };
                            // A column whose ground lies below the world's floor (a deep trench)
                            // has no rock at the floor: the evaluator's column ends below it.
                            if y > top {
                                continue;
                            }
                            self.stack(&mut scratch, x, z, &mut stack);
                            row[px as usize] = self.class_at(x, y, z, &stack);
                        }
                    }
                });
            }
        });
        Slice { w, h, class }
    }
}

impl Slice {
    /// RGBA (transparent where there is no rock), coloured by `colours` (bake class index →
    /// colour).
    pub fn rgba(&self, colours: &[[u8; 3]]) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.class.len() * 4);
        for &k in &self.class {
            match colours.get(usize::from(k)) {
                Some(c) if k != NONE => out.extend_from_slice(&[c[0], c[1], c[2], 255]),
                _ => out.extend_from_slice(&[0, 0, 0, 0]),
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_sit_where_keyframe_pixels_do() {
        // Pixel 0 is the seam (x = −C/2) and the north pole row (z = −R, R = C/2π = 2,607.6); the
        // middle pixel is x = 0, z = 0 (the map is π : 1, 1024 × 326).
        let l = deeptime_format::projection::WorldProjection::Lambert(
            deeptime_format::Projection::new(16384),
        );
        assert_eq!(pixel_block(&l, 16384, 1024, 326, 0, 0), (-8192, -2608));
        assert_eq!(pixel_block(&l, 16384, 1024, 326, 512, 163), (0, 0));
        assert_eq!(pixel_block(&l, 16384, 1024, 326, 1023, 325), (8176, 2591));
        let g = deeptime_format::projection::WorldProjection::PetroffGuyou(
            deeptime_format::projection::PetroffGuyou::new(6656, 0.0),
        );
        assert_eq!(picture_height(&g, 1024), 512);
        assert_eq!(pixel_block(&g, 13312, 1024, 512, 0, 0), (-6656, -3328));
        assert_eq!(pixel_block(&g, 13312, 1024, 512, 512, 256), (0, 0));
    }

    #[test]
    fn depths_parse() {
        assert_eq!(Depth::parse("0"), Some(Depth::Below(0)));
        assert_eq!(Depth::parse("100"), Some(Depth::Below(100)));
        assert_eq!(Depth::parse("floor"), Some(Depth::Floor));
        assert_eq!(Depth::parse("-5"), None);
        assert_eq!(Depth::parse("deep"), None);
        assert_eq!(Depth::Floor.label(), "floor");
    }
}
