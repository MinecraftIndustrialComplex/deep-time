// SPDX-License-Identifier: LGPL-3.0-or-later
//! The sub-cell detail term of the terrain (FORMAT.md, "The detail layer" and "The terrain
//! height"): the game adds **A(x, z) · N(x, z)** to the L3 `elevation`, where A is the baked layer
//! `detail/amplitude` and N the value noise here. Both sides of the contract evaluate N:
//! `deeptime_core::surface::detail` bakes A with it (P1-06) and re-exports it, the Java reader has
//! its twin (`worldgen/DetailNoise`, P1-08), and the column evaluator (`column.rs`) sums the two.
//!
//! N is exactly specified so the twins agree bit for bit: IEEE-754 double arithmetic with
//! `+ − × /`, `floor` and `round` only (no `sin`, no library `pow`), integers hashed with
//! splitmix64's finaliser. Value noise of wavelength λ (blocks): n = max(1, round(C/λ)) lattice
//! cells around the planet, spacing s = C/n; u = (x + C/2)/s, v = z/s; the four lattice corners
//! (⌊u⌋ mod n, ⌊v⌋) … (+1, +1) (the column index mod n, so the lattice is periodic in x) get
//! [`Noise::lattice`] values in [−1, 1]; the result blends them bilinearly with the fade
//! t = f²(3 − 2f) on each axis. [`Noise::fbm`] sums `octaves` of them at λ, λ/2, λ/4, … with
//! weights 1, ½, ¼, … normalised to [−1, 1].
//!
//! A cube world (format 0.7) reads the noise on the cube's surface instead ([`Noise::surface_value`],
//! [`DetailSpec::cube_noise`]): its net's turning seams glue faces whose net coordinates do not run on,
//! so noise drawn in net coordinates would break at every one of them.
//!
//! Which noise a world uses (seed, salt, octaves, wavelength) is `manifest.json`
//! `notes.bake.detail` ([`DetailSpec`]), written by the bake.

use serde::{Deserialize, Serialize};

use crate::cube::{Cube, CubeFold};
use crate::error::{FormatError, Result};
use crate::manifest::Manifest;

/// splitmix64's finaliser.
#[inline]
fn mix(mut x: u64) -> u64 {
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Periodic value noise (see the module documentation for the exact definition).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Noise {
    pub seed: u64,
    /// C, blocks: the period in x.
    pub circumference: u32,
    /// A cube world's net (format 0.7): [`DetailSpec::noise`] reads the noise on the cube's
    /// surface ([`DetailSpec::cube_noise`]), so it runs on across every seam and corner.
    pub cube: Option<&'static Cube>,
}

impl Noise {
    pub fn new(seed: u64, circumference: u32) -> Noise {
        Noise {
            seed,
            circumference,
            cube: None,
        }
    }

    /// The noise of a world laid out by `wp`: a cube world's has its net.
    pub fn for_world(
        seed: u64,
        circumference: u32,
        wp: &crate::projection::WorldProjection,
    ) -> Noise {
        let mut n = Noise::new(seed, circumference);
        if let crate::projection::WorldProjection::Cube(v) = wp {
            n.cube = Some(v.cube);
        }
        n
    }

    /// The lattice value at (ix, jz) for `salt`, in [−1, 1].
    #[inline]
    pub fn lattice(&self, salt: u32, ix: i64, jz: i64) -> f64 {
        let a = mix(self.seed ^ u64::from(salt).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let b = mix(a ^ (ix as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93));
        let h = mix(b ^ (jz as u64).wrapping_mul(0xA076_1D64_78BD_642F));
        ((h >> 11) as f64) * (2.0 / 9_007_199_254_740_992.0) - 1.0
    }

    /// Value noise of wavelength `wavelength` blocks at block (x, z), in [−1, 1], periodic in x
    /// with period C.
    pub fn value(&self, x: f64, z: f64, wavelength: f64, salt: u32) -> f64 {
        let c = f64::from(self.circumference);
        let n = (c / wavelength).round().max(1.0);
        let s = c / n;
        let u = (x + c / 2.0) / s;
        let v = z / s;
        let (fu, fv) = (u.floor(), v.floor());
        let (fx, fz) = (u - fu, v - fv);
        let n_i = n as i64;
        let ix0 = (fu as i64).rem_euclid(n_i);
        let ix1 = (ix0 + 1) % n_i;
        let jz0 = fv as i64;
        let jz1 = jz0 + 1;
        let tx = fx * fx * (3.0 - 2.0 * fx);
        let tz = fz * fz * (3.0 - 2.0 * fz);
        let h00 = self.lattice(salt, ix0, jz0);
        let h10 = self.lattice(salt, ix1, jz0);
        let h01 = self.lattice(salt, ix0, jz1);
        let h11 = self.lattice(salt, ix1, jz1);
        let a = h00 + (h10 - h00) * tx;
        let b = h01 + (h11 - h01) * tx;
        a + (b - a) * tz
    }

    /// `octaves` of [`Noise::value`] at λ, λ/2, λ/4, … with weights 1, ½, ¼, … (salt + octave),
    /// normalised to [−1, 1].
    pub fn fbm(&self, x: f64, z: f64, wavelength: f64, octaves: u32, salt: u32) -> f64 {
        let (mut sum, mut norm, mut weight, mut wl) = (0.0, 0.0, 1.0, wavelength);
        for o in 0..octaves {
            sum += weight * self.value(x, z, wl, salt.wrapping_add(o));
            norm += weight;
            weight *= 0.5;
            wl *= 0.5;
        }
        sum / norm
    }

    /// The lattice value at the point (ix, iy, iz) of a cube's surface lattice for `salt`, in
    /// [−1, 1] ([`Noise::lattice`] with a third index).
    #[inline]
    pub fn lattice3(&self, salt: u32, ix: i64, iy: i64, iz: i64) -> f64 {
        let a = mix(self.seed ^ u64::from(salt).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let b = mix(a ^ (ix as u64).wrapping_mul(0xD6E8_FEB8_6659_FD93));
        let c = mix(b ^ (iy as u64).wrapping_mul(0xA076_1D64_78BD_642F));
        let h = mix(c ^ (iz as u64).wrapping_mul(0xE703_7ED1_A0B4_28DB));
        ((h >> 11) as f64) * (2.0 / 9_007_199_254_740_992.0) - 1.0
    }

    /// Value noise of wavelength `wavelength` blocks on the surface of a cube of side `s` blocks
    /// (ledger: SCL-215, game-choice), at the surface point `p`, blocks from the cube's centre in the
    /// cube frame ([`crate::cube::Face::frame`]): its face's axis is `fixed` (0 x, 1 y, 2 z), where
    /// p is +s/2 when `positive` and −s/2 otherwise, and the other two lie in [−s/2, s/2]. The
    /// lattice has m = max(1, round(s/λ)) cells along every cube edge: index l = (p + s/2)·m/s on
    /// each axis, so the cube's edges are lattice lines and two faces share the lattice points of
    /// their common edge (three faces those of a corner). A face blends its own square of the
    /// lattice bilinearly with the fade f²(3 − 2f) along its two free axes, the lower axis first, so
    /// the noise is one function on the surface, continuous through every seam and corner, and on
    /// each face a value noise of spacing s/m like [`Noise::value`]'s.
    pub fn surface_value(
        &self,
        s: f64,
        p: [f64; 3],
        fixed: usize,
        positive: bool,
        wavelength: f64,
        salt: u32,
    ) -> f64 {
        let half = 0.5 * s;
        let m = (s / wavelength).round().max(1.0);
        let (a1, a2) = match fixed {
            0 => (1, 2),
            1 => (0, 2),
            _ => (0, 1),
        };
        let l1 = (p[a1] + half) * m / s;
        let l2 = (p[a2] + half) * m / s;
        let (fl1, fl2) = (l1.floor(), l2.floor());
        let (f1, f2) = (l1 - fl1, l2 - fl2);
        let t1 = f1 * f1 * (3.0 - 2.0 * f1);
        let t2 = f2 * f2 * (3.0 - 2.0 * f2);
        let (i1, i2) = (fl1 as i64, fl2 as i64);
        let mut idx = [0i64; 3];
        idx[fixed] = if positive { m as i64 } else { 0 };
        let mut at = |d1: i64, d2: i64| {
            idx[a1] = i1 + d1;
            idx[a2] = i2 + d2;
            self.lattice3(salt, idx[0], idx[1], idx[2])
        };
        let h00 = at(0, 0);
        let h10 = at(1, 0);
        let h01 = at(0, 1);
        let h11 = at(1, 1);
        let a = h00 + (h10 - h00) * t1;
        let b = h01 + (h11 - h01) * t1;
        a + (b - a) * t2
    }

    /// `octaves` of [`Noise::surface_value`] at λ, λ/2, λ/4, … with weights 1, ½, ¼, … (salt +
    /// octave), normalised to [−1, 1], as [`Noise::fbm`].
    #[allow(clippy::too_many_arguments)]
    pub fn surface_fbm(
        &self,
        s: f64,
        p: [f64; 3],
        fixed: usize,
        positive: bool,
        wavelength: f64,
        octaves: u32,
        salt: u32,
    ) -> f64 {
        let (mut sum, mut norm, mut weight, mut wl) = (0.0, 0.0, 1.0, wavelength);
        for o in 0..octaves {
            sum += weight * self.surface_value(s, p, fixed, positive, wl, salt.wrapping_add(o));
            norm += weight;
            weight *= 0.5;
            wl *= 0.5;
        }
        sum / norm
    }
}

/// The `notes` key the bake writes its record under (`manifest.json` `notes.bake`).
pub const BAKE_NOTES: &str = "bake";

/// The game's detail term of a baked world (`notes.bake.detail`): N = [`Noise::fbm`] of
/// `Noise::new(seed, C)` at `wavelength_blocks`, `octaves`, `salt`, times the `layer` sampled
/// bilinearly. The seed is written as a decimal string: JSON readers that hold numbers as doubles
/// (and Java's `long`) cannot carry every u64.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DetailSpec {
    /// The amplitude layer (`detail/amplitude`).
    pub layer: String,
    /// The noise seed, a decimal u64.
    pub seed: String,
    pub salt: u32,
    pub octaves: u32,
    pub wavelength_blocks: f64,
}

impl DetailSpec {
    pub fn new(seed: u64, salt: u32, octaves: u32, wavelength_blocks: f64) -> DetailSpec {
        DetailSpec {
            layer: "detail/amplitude".into(),
            seed: seed.to_string(),
            salt,
            octaves,
            wavelength_blocks,
        }
    }

    /// The seed as a number.
    pub fn seed(&self) -> Result<u64> {
        self.seed
            .parse()
            .map_err(|_| FormatError::Invalid(format!("notes.bake.detail.seed {:?}", self.seed)))
    }

    /// Checks the values a reader relies on.
    pub fn validate(&self) -> Result<()> {
        self.seed()?;
        if !(1..=16).contains(&self.octaves)
            || !(self.wavelength_blocks.is_finite() && self.wavelength_blocks > 0.0)
        {
            return Err(FormatError::Invalid(format!(
                "notes.bake.detail: octaves {} (1–16), wavelength {} (> 0)",
                self.octaves, self.wavelength_blocks
            )));
        }
        Ok(())
    }

    /// The spec a manifest carries, if any (a synthetic world has none: its terrain is the
    /// `elevation` layer alone).
    pub fn of(m: &Manifest) -> Result<Option<DetailSpec>> {
        let Some(v) = m.notes.get(BAKE_NOTES).and_then(|b| b.get("detail")) else {
            return Ok(None);
        };
        let spec: DetailSpec =
            serde_json::from_value(v.clone()).map_err(|source| FormatError::Json {
                what: "manifest.json notes.bake.detail".into(),
                source,
            })?;
        spec.validate()?;
        Ok(Some(spec))
    }

    /// N at block (x, z) for a planet of circumference `c`: the fBm, or on a cube world
    /// (`noise.cube`) [`DetailSpec::cube_noise`].
    pub fn noise(&self, noise: &Noise, x: f64, z: f64) -> f64 {
        if let Some(cube) = noise.cube {
            return self.cube_noise(noise, cube, x, z);
        }
        noise.fbm(x, z, self.wavelength_blocks, self.octaves, self.salt)
    }

    /// N at block (x, z) of a cube world (cube plan G3; ledger: SCL-215, game-choice): the noise on
    /// the cube's surface ([`Noise::surface_fbm`]) at the block's centre (x + ½, z + ½), brought onto
    /// the planet by [`Cube::fold`] of its block: the slide, and past a net edge the edge's motion,
    /// which takes block centres to block centres. A block off the faces therefore reads exactly
    /// its fold partner's noise, and the noise runs on through every seam and corner with no blend.
    /// Past the net's corners ([`CubeFold::Ocean`]) it is 0.
    pub fn cube_noise(&self, noise: &Noise, cube: &Cube, x: f64, z: f64) -> f64 {
        let (px, pz) = (x + 0.5, z + 0.5);
        let (bx, bz) = (px.floor(), pz.floor());
        // The face and the point's local coordinates on it, blocks from its north-west corner.
        let (face, u, v) = match cube.fold(bx as i64, bz as i64) {
            CubeFold::Face { face, i, j } => (face, i as f64 + (px - bx), j as f64 + (pz - bz)),
            CubeFold::Outside {
                x: qx, z: qz, iso, ..
            } => {
                let Some((face, i, j)) = cube.face_of_block(qx, qz) else {
                    return 0.0;
                };
                let (rx, rz) = iso.point(px, pz);
                (
                    face,
                    i as f64 + (rx - qx as f64),
                    j as f64 + (rz - qz as f64),
                )
            }
            CubeFold::Ocean => return 0.0,
        };
        let s = cube.side() as f64;
        let half = 0.5 * s;
        let (u, v) = (u - half, v - half);
        let [c, eu, ev] = face.frame();
        let p = [
            half * c[0] + u * eu[0] + v * ev[0],
            half * c[1] + u * eu[1] + v * ev[1],
            half * c[2] + u * eu[2] + v * ev[2],
        ];
        let fixed = if c[0] != 0.0 {
            0
        } else if c[1] != 0.0 {
            1
        } else {
            2
        };
        noise.surface_fbm(
            s,
            p,
            fixed,
            c[fixed] > 0.0,
            self.wavelength_blocks,
            self.octaves,
            self.salt,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_noise_is_periodic_in_x_and_bounded() {
        for c in [16_384u32, 32_768, 4096, 1000] {
            let n = Noise::new(7, c);
            let cf = f64::from(c);
            for k in 0..500 {
                let x = -cf / 2.0 + (k as f64) * 13.37 % cf;
                let z = -cf / 4.0 + (k as f64) * 7.91 % (cf / 2.0);
                for wl in [4.0, 8.0, 32.0, 100.0] {
                    let a = n.value(x, z, wl, 3);
                    assert!((-1.0..=1.0).contains(&a));
                    // A lap changes the input's low bits, so equal to rounding, not to the bit.
                    let east = n.value(x + cf, z, wl, 3);
                    let west = n.value(x - 2.0 * cf, z, wl, 3);
                    assert!((a - east).abs() < 1e-9, "one lap east, wl {wl}: {a} {east}");
                    assert!((a - west).abs() < 1e-9, "two laps west: {a} {west}");
                }
                let f = n.fbm(x, z, 32.0, 3, 1);
                assert!((-1.0..=1.0).contains(&f));
                assert!((f - n.fbm(x + cf, z, 32.0, 3, 1)).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn the_seam_is_continuous() {
        let c = 16_384.0;
        let n = Noise::new(1, 16_384);
        // The seam is at x = ±C/2: a step of a tenth of a block across it changes the value like
        // a step anywhere else does.
        let mut worst_seam = 0.0f64;
        let mut worst_else = 0.0f64;
        for k in 0..200 {
            let z = -3000.0 + f64::from(k) * 29.3;
            worst_seam = worst_seam.max(
                (n.fbm(c / 2.0 - 0.05, z, 32.0, 3, 1) - n.fbm(-c / 2.0 + 0.05, z, 32.0, 3, 1))
                    .abs(),
            );
            worst_else = worst_else.max(
                (n.fbm(1000.0 - 0.05, z, 32.0, 3, 1) - n.fbm(1000.0 + 0.05, z, 32.0, 3, 1)).abs(),
            );
        }
        assert!(
            worst_seam < 0.05 && worst_else < 0.05,
            "{worst_seam} {worst_else}"
        );
    }

    #[test]
    fn noise_is_deterministic_and_seed_dependent() {
        let a = Noise::new(1, 1024);
        let b = Noise::new(2, 1024);
        let v = a.value(10.5, 20.25, 16.0, 0);
        assert_eq!(v, Noise::new(1, 1024).value(10.5, 20.25, 16.0, 0));
        assert_ne!(v, b.value(10.5, 20.25, 16.0, 0));
        assert_ne!(v, a.value(10.5, 20.25, 16.0, 1), "salt");
        // A lattice point returns the lattice value exactly.
        let s = 1024.0 / (1024.0f64 / 16.0).round();
        assert_eq!(
            a.value(-512.0 + 3.0 * s, 5.0 * s, 16.0, 0),
            a.lattice(0, 3, 5)
        );
    }

    /// Fixed values the Java twin (`DetailNoiseTest`) checks too: a change here is a change of the
    /// world-data contract.
    #[test]
    fn known_values() {
        let n = Noise::new(0xDEAD_BEEF_0123_4567, 16_384);
        assert_eq!(n.lattice(7, -3, 11), -0.9207152765116788);
        assert_eq!(n.fbm(-8191.0, 4095.0, 32.0, 3, 1729), -0.06020044448424485);
        assert_eq!(n.fbm(1234.0, -567.0, 32.0, 3, 1729), 0.48318066841072793);
        let seed = u64::MAX - 5;
        let spec = DetailSpec::new(seed, 1729, 3, 32.0);
        assert_eq!(spec.seed().unwrap(), seed);
        let json = serde_json::to_value(&spec).unwrap();
        assert_eq!(json["seed"], "18446744073709551610");
    }

    /// The noise of the 4k test cube's net (s = 1024, period 4096).
    fn cube_fixture() -> (Noise, DetailSpec, &'static Cube) {
        let cube = Cube::shared(1024);
        let n = Noise {
            cube: Some(cube),
            ..Noise::new(0xDEAD_BEEF_0123_4567, 4096)
        };
        (
            n,
            DetailSpec::new(0xDEAD_BEEF_0123_4567, 1729, 3, 32.0),
            cube,
        )
    }

    /// Fixed values of a cube world's noise that the Java twin (`DetailNoiseTest`) checks too.
    #[test]
    fn cube_known_values() {
        let (n, spec, _) = cube_fixture();
        assert_eq!(n.lattice3(7, -3, 11, 5), -0.016565307453047184);
        // F0, F3, N, S, F2, past N's west edge (F3's top), past the net's corners.
        let at = [
            (100.0, -50.0),
            (-900.0, 300.0),
            (10.0, -900.0),
            (-200.0, 1000.0),
            (2000.0, 100.0),
            (-600.0, -700.0),
        ];
        let want = [
            -0.16032818137761118,
            0.2689732692952809,
            0.15544098362422432,
            0.23472733008415383,
            0.03172617598775018,
            0.017139648300822326,
        ];
        for ((x, z), w) in at.into_iter().zip(want) {
            assert_eq!(spec.noise(&n, x, z), w, "({x}, {z})");
        }
        assert_eq!(
            spec.noise(&n, 1000.0, -1600.0),
            0.0,
            "deep ocean past the corners"
        );
    }

    /// A block off the faces reads its fold partner's noise exactly, a lap east the same block's,
    /// and past the net's corners 0.
    #[test]
    fn the_cube_noise_is_its_fold_partners() {
        let (n, spec, cube) = cube_fixture();
        let (mut outside, mut ocean) = (0, 0);
        for x in (-1536..2560).step_by(7) {
            for z in (-1700..1700).step_by(7) {
                let here = spec.noise(&n, x as f64, z as f64);
                assert!((-1.0..=1.0).contains(&here));
                match cube.fold(x, z) {
                    CubeFold::Outside { x: qx, z: qz, .. } => {
                        outside += 1;
                        assert_eq!(
                            here,
                            spec.noise(&n, qx as f64, qz as f64),
                            "({x}, {z}) shows ({qx}, {qz})"
                        );
                    }
                    CubeFold::Ocean => {
                        ocean += 1;
                        assert_eq!(here, 0.0);
                    }
                    CubeFold::Face { .. } => {}
                }
                assert_eq!(spec.noise(&n, (x + 4096) as f64, z as f64), here, "a lap");
            }
        }
        assert!(outside > 50_000 && ocean > 1_000, "{outside} {ocean}");
    }

    /// Across every net edge (the slide and the twelve turning edges) a point on the edge line
    /// reads one value from both sides, and the step between the two blocks beside the line is
    /// the size of a step across a lattice line inside the face (the edge is a lattice line of
    /// every octave, where the fade's slope is 0; with s = 1024 and λ = 32, 16, 8 so are the lines
    /// 32k blocks in). Noise drawn in net coordinates jumps by about its own range there.
    #[test]
    fn the_cube_noise_runs_on_through_every_seam() {
        use crate::projection::{Axis, Side};
        let (n, spec, cube) = cube_fixture();
        let at = |a: Axis, nn: i64, t: f64| match a {
            Axis::X => spec.noise(&n, nn as f64, t),
            Axis::Z => spec.noise(&n, t, nn as f64),
        };
        let at_f = |a: Axis, nn: f64, t: f64| match a {
            Axis::X => spec.noise(&n, nn, t),
            Axis::Z => spec.noise(&n, t, nn),
        };
        for e in cube.edges() {
            let inward = match e.beyond {
                Side::Min => 1,
                Side::Max => -1,
            };
            let (mut across, mut inside, mut worst_across, mut worst_inside) =
                (0.0, 0.0, 0.0f64, 0.0f64);
            let mut count = 0.0;
            // Away from the span's ends, where a corner square's nearer edge takes over.
            for t in (e.span[0] + 2..e.span[1] - 2).step_by(3) {
                let tf = t as f64;
                // The blocks coord − 1 and coord straddle the line x = coord (or z = coord).
                let a = (at(e.axis, e.coord - 1, tf) - at(e.axis, e.coord, tf)).abs();
                across += a;
                worst_across = worst_across.max(a);
                for k in 1..=4 {
                    let l = e.coord + inward * 32 * k;
                    let w = (at(e.axis, l - 1, tf) - at(e.axis, l, tf)).abs();
                    inside += w / 4.0;
                    worst_inside = worst_inside.max(w);
                }
                count += 1.0;
                // On the line itself, approached from both sides (the noise reads block centres,
                // so the point x + ½ is on the line at x = coord − ½).
                let line = e.coord as f64 - 0.5;
                let (l, r) = (at_f(e.axis, line - 1e-7, tf), at_f(e.axis, line + 1e-7, tf));
                assert!((l - r).abs() < 1e-5, "{} at {t}: {l} vs {r}", e.name);
            }
            let (across, inside) = (across / count, inside / count);
            assert!(
                across < 2.0 * inside && worst_across < 3.0 * worst_inside,
                "{}: step across the edge mean {across} worst {worst_across}, across lattice lines inside mean {inside} worst {worst_inside}",
                e.name
            );
        }
    }

    /// The noise on a face is a value noise of about the wavelength: its lattice cells are s/m
    /// blocks with m = round(s/λ), and its values spread as the plain noise's do.
    #[test]
    fn the_cube_noise_spreads_like_the_plain_noise() {
        let (n, spec, _) = cube_fixture();
        let plain = Noise::new(n.seed, 4096);
        let (mut sc, mut sp) = (0.0, 0.0);
        let mut k = 0.0;
        for x in (-1536..2560).step_by(5) {
            for z in (-512..512).step_by(5) {
                let c = spec.noise(&n, x as f64, z as f64);
                let p = spec.noise(&plain, x as f64, z as f64);
                sc += c * c;
                sp += p * p;
                k += 1.0;
            }
        }
        let (rc, rp) = ((sc / k).sqrt(), (sp / k).sqrt());
        assert!((rc / rp - 1.0).abs() < 0.1, "rms cube {rc}, plain {rp}");
    }
}
