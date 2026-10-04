// SPDX-License-Identifier: LGPL-3.0-or-later
//! A structure the tectonic history left in the crust: a fold train or a fault, as the simulation
//! carries it. It rides its plate like a pluton ([`advance`]), and at the end of the run becomes
//! - a kinematic event of the Noddy port ([`Feature::to_noddy`]), for evaluating a region on a
//!   lattice, and
//! - a baked event of the world data ([`Feature::to_baked`]), for the generator's column rule.
//!
//! The two describe the same displacement; `tests.rs` holds them together.
//!
//! Frames. The simulation's block space has x east, z south and y up; Noddy's has x east, y north
//! and z up. A feature's azimuth is clockwise from north in both.

use deeptime_format::GridDesc;
use deeptime_format::structure::{Shape, StructEvent, Window};
use deeptime_noddy as noddy;
use glam::DVec3;
use serde::{Deserialize, Serialize};

use crate::plates::kinematics::rotation;
use crate::plates::state::CONTINENTAL;
use crate::sphere::BlockFrame;
use crate::structure::plutons::cell_of;

/// What kind of structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    /// A train of upright folds (shortening).
    FoldTrain,
    /// A thrust: a reverse fault on a gentle ramp (shortening).
    Thrust,
    /// A normal fault (extension).
    Normal,
    /// A strike-slip fault (oblique convergence).
    StrikeSlip,
    /// A stock: a small granitic intrusion in an orogen, cutting its folds.
    Stock,
    /// A dyke: a sheet of basalt along a rift.
    Dyke,
    /// A plug: the solid conduit of an extinct volcano.
    Plug,
}

impl Kind {
    pub const ALL: [Kind; 7] = [
        Kind::FoldTrain,
        Kind::Thrust,
        Kind::Normal,
        Kind::StrikeSlip,
        Kind::Stock,
        Kind::Dyke,
        Kind::Plug,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Kind::FoldTrain => "fold_train",
            Kind::Thrust => "thrust",
            Kind::Normal => "normal_fault",
            Kind::StrikeSlip => "strike_slip_fault",
            Kind::Stock => "stock",
            Kind::Dyke => "dyke",
            Kind::Plug => "plug",
        }
    }

    pub fn is_fault(self) -> bool {
        matches!(self, Kind::Thrust | Kind::Normal | Kind::StrikeSlip)
    }

    /// An intrusion moves no rock: it replaces what is older than itself inside its ellipsoid.
    pub fn is_intrusion(self) -> bool {
        matches!(self, Kind::Stock | Kind::Dyke | Kind::Plug)
    }
}

/// One structure. Lengths are blocks (the same horizontally and vertically: `params.rs`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub id: u32,
    pub kind: Kind,
    /// Block position of its centre: the middle of a fold train, the middle of a fault's trace
    /// (x in [−C/2, C/2)).
    pub x: f64,
    pub z: f64,
    /// The plate it rides.
    pub plate: u32,
    /// When it formed, Myr relative to world creation. Rock laid down later is not deformed.
    pub t_myr: f32,
    /// Azimuth of its long axis, degrees clockwise from north: a fold's axis, a fault's strike
    /// with the dip to the right (the fault dips towards azimuth + 90°).
    pub azimuth_deg: f32,
    /// Half its length along the axis, and half its width across it (a fault: how far its
    /// displacement reaches from the trace).
    pub half_along: f32,
    pub half_across: f32,
    /// Folds: wavelength and the amplitude at the middle of the train.
    pub wavelength: f32,
    pub amplitude: f32,
    /// Folds: where in the wavelength the train's middle falls, 0–1.
    pub phase: f32,
    /// Faults: dip, degrees from horizontal.
    pub dip_deg: f32,
    /// Faults: the slip at the middle of the trace along the slip line, positive when the hanging
    /// wall went down the dip (normal), negative when it went up (thrust); for a strike-slip
    /// fault, along the strike.
    pub slip: f32,
    /// The tectonic event that made it (the event log's id), 0 when none was found.
    pub cause: u64,
    /// Plate steps in a row it has not had continental crust under it.
    pub adrift: u8,
    /// How much the deep-time surface has eroded or buried the ground at its centre since it
    /// formed, m: what wears its relief off the ground (`relief_left`).
    pub wear_m: f32,
    /// Intrusions are rooted: the ellipsoid's centre lies on the world's floor and its
    /// horizontal half-axes are `half_along` and `half_across`, so it has no floor of its own
    /// and narrows upwards to its top. `rise`: how far above the ground the top would stand, as
    /// a share of the ground's height above the floor (dykes and plugs: they reach the ground
    /// over the middle of their plan; 0 for a stock).
    pub rise: f32,
    /// Intrusions: how far the top lies under the ground, blocks (stocks; negative: it crops
    /// out).
    pub depth: f32,
    /// Intrusions: the rock class (index into the class table).
    pub rock: u16,
}

/// A structure that has had no continental crust under or beside it for this many plate steps is
/// lost (its crust went under another plate).
pub const LOST_AFTER_STEPS: u8 = 10; // ledger: DEF-035

/// How far (L1 cells) a structure over oceanic crust looks for its continent's edge.
const EDGE_CELLS: i64 = 3; // ledger: DEF-035

/// Counters for the run summary.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Counts {
    pub fold_trains: u64,
    pub thrusts: u64,
    pub normal_faults: u64,
    pub strike_slip_faults: u64,
    /// Not started because enough structures of the kind were already there.
    pub saturated: u64,
    /// Old structures that gave way to a new one at a place that was full.
    pub overprinted: u64,
    /// Lost: no continental crust under them for [`LOST_AFTER_STEPS`] plate steps.
    pub subducted: u64,
    pub stocks: u64,
    pub dykes: u64,
    /// Plugs are made at the end of the run from the extinct volcanoes standing then.
    pub plugs: u64,
    /// Fold trains (first) and thrusts (second) started, by where the shortening came from: the
    /// collision regime, a lifted Andean margin, the foreland of an Andean margin and the foreland
    /// basin of a collision (`emplace::Source` without `None`, in that order).
    pub shortened_by: [[u64; 2]; 4],
}

impl Counts {
    /// Counts a fold train or thrust under the rule that strained its cell.
    pub fn add_source(&mut self, kind: Kind, source: super::emplace::Source) {
        use super::emplace::Source;
        let row = match source {
            Source::None => return,
            Source::Collision => 0,
            Source::Arc => 1,
            Source::Foreland => 2,
            Source::Basin => 3,
        };
        match kind {
            Kind::FoldTrain => self.shortened_by[row][0] += 1,
            Kind::Thrust => self.shortened_by[row][1] += 1,
            _ => {}
        }
    }

    pub fn add(&mut self, kind: Kind) {
        match kind {
            Kind::FoldTrain => self.fold_trains += 1,
            Kind::Thrust => self.thrusts += 1,
            Kind::Normal => self.normal_faults += 1,
            Kind::StrikeSlip => self.strike_slip_faults += 1,
            Kind::Stock => self.stocks += 1,
            Kind::Dyke => self.dykes += 1,
            Kind::Plug => self.plugs += 1,
        }
    }
}

/// The unit vector in block (x, z) of an azimuth (degrees clockwise from north; north is −z).
pub fn direction(azimuth_deg: f64) -> [f64; 2] {
    let a = azimuth_deg.to_radians();
    [a.sin(), -a.cos()]
}

/// The azimuth of a block-space direction.
pub fn azimuth_of(dx: f64, dz: f64) -> f64 {
    dx.atan2(-dz).to_degrees().rem_euclid(360.0)
}

impl Feature {
    /// Whether the block position (x, z) lies inside the feature's window (its weight there is
    /// not zero), on a planet `c` blocks around.
    pub fn covers(&self, x: f64, z: f64, c: f64) -> bool {
        let along = direction(f64::from(self.azimuth_deg));
        let across = direction(f64::from(self.azimuth_deg) + 90.0);
        let mut dx = x - self.x;
        dx -= (dx / c + 0.5).floor() * c;
        let dz = z - self.z;
        (dx * along[0] + dz * along[1]).abs() < f64::from(self.half_along)
            && (dx * across[0] + dz * across[1]).abs() < f64::from(self.half_across)
    }

    /// The feature as an event of the Noddy port, in a local frame whose origin is the block
    /// position (`origin_x`, `origin_z`) at height 0: Noddy x = block x − origin, y = −(block z −
    /// origin), z = block y. `surface_y` is the ground's height at the feature's centre (a
    /// fault's trace lies there); `profile` the fold profile (the sine table). `None` for an
    /// intrusion, which moves nothing.
    pub fn to_noddy(
        &self,
        origin_x: f64,
        origin_z: f64,
        c: f64,
        surface_y: f64,
        profile: &noddy::Profile,
    ) -> Option<noddy::Event> {
        if self.kind.is_intrusion() {
            return None;
        }
        let mut dx = self.x - origin_x;
        dx -= (dx / c + 0.5).floor() * c;
        let centre = [dx, -(self.z - origin_z)];
        let az = f64::from(self.azimuth_deg);
        let window = noddy::Window::new(
            centre,
            az,
            f64::from(self.half_along),
            f64::from(self.half_across),
        );
        let inner = match self.kind {
            Kind::FoldTrain => {
                // The profile's origin sits at the window's far edge, a whole number of
                // wavelengths plus the phase from the centre, so the phase angle is never
                // negative inside the window (Noddy reads its profile table backwards there).
                let offset = self.fold_offset();
                let position = [
                    centre[0] + window.across[0] * offset,
                    centre[1] + window.across[1] * offset,
                    0.0,
                ];
                noddy::Event::Fold(noddy::Fold::new(
                    position,
                    az + 90.0,
                    90.0,
                    0.0,
                    f64::from(self.wavelength),
                    f64::from(self.amplitude),
                    0.0,
                    false,
                    profile.clone(),
                ))
            }
            Kind::Thrust | Kind::Normal | Kind::StrikeSlip => {
                let pitch = if self.kind == Kind::StrikeSlip {
                    0.0
                } else {
                    90.0
                };
                noddy::Event::Fault(noddy::Fault::translation(
                    [centre[0], centre[1], surface_y],
                    az + 90.0,
                    f64::from(self.dip_deg),
                    pitch,
                    f64::from(self.slip),
                    noddy::Movement::HangingWall,
                ))
            }
            Kind::Stock | Kind::Dyke | Kind::Plug => return None,
        };
        Some(noddy::Event::Windowed(noddy::Windowed::new(inner, window)))
    }

    /// Folds: the across coordinate of the profile's origin relative to the centre.
    fn fold_offset(&self) -> f64 {
        let wl = f64::from(self.wavelength);
        let waves = (f64::from(self.half_across) / wl).ceil();
        -(waves + f64::from(self.phase)) * wl
    }

    /// The feature as a baked event of the world data (`key`: its age as a stratigraphic key;
    /// `floor_y`: the world's floor, where intrusions are rooted). The vectors are taken from
    /// the Noddy event's own frame, so the two agree.
    pub fn to_baked(&self, surface_y: f64, key: u16, floor_y: f64) -> StructEvent {
        let az = f64::from(self.azimuth_deg);
        let window = Window {
            centre: [self.x, self.z],
            along: direction(az),
            across: direction(az + 90.0),
            half_along: f64::from(self.half_along),
            half_across: f64::from(self.half_across),
        };
        let shape = match self.kind {
            Kind::FoldTrain => Shape::Fold {
                wavelength: f64::from(self.wavelength),
                amplitude: f64::from(self.amplitude),
                offset: self.fold_offset(),
                profile: 0,
            },
            Kind::Thrust | Kind::Normal | Kind::StrikeSlip => {
                let pitch = if self.kind == Kind::StrikeSlip {
                    0.0
                } else {
                    90.0
                };
                let f = noddy::Fault::translation(
                    [0.0, 0.0, 0.0],
                    az + 90.0,
                    f64::from(self.dip_deg),
                    pitch,
                    f64::from(self.slip),
                    noddy::Movement::HangingWall,
                );
                // Noddy (east, north, up) → block (x, y, z) = (east, up, −north). The pole is
                // the first column of the forward matrix; the slip line the second row of the
                // reverse one.
                let fwd = &f.rotation.forward;
                let rev = &f.rotation.reverse;
                let s = f64::from(self.slip);
                Shape::Fault {
                    y: surface_y,
                    pole: [fwd[0][0], fwd[2][0], -fwd[1][0]],
                    slip: [s * rev[1][0], s * rev[1][2], -s * rev[1][1]],
                }
            }
            Kind::Stock | Kind::Dyke | Kind::Plug => Shape::Intrusion {
                y: floor_y,
                half_height: ((surface_y - floor_y).max(0.0) * (1.0 + f64::from(self.rise))
                    - f64::from(self.depth))
                .max(8.0), // ledger: DEF-052
                class: self.rock as u8,
            },
        };
        StructEvent {
            id: self.id,
            key,
            t_years: f64::from(self.t_myr) * 1.0e6,
            window,
            shape,
        }
    }
}

/// Adds a step's change of the ground (`surface/material_m`: deposition positive, erosion
/// negative, m) at every feature's centre to its wear.
pub fn wear(features: &mut [Feature], grid: &GridDesc, material_m: &[f32]) {
    for f in features {
        let m = material_m[cell_of(grid, f.x, f.z)];
        if m.is_finite() {
            f.wear_m += m.abs();
        }
    }
}

/// Moves every feature with its plate over `dt_yr` (Ω per plate id, rad/yr; zero for a plate that
/// is gone), turning its azimuth with the plate, then looks at the crust now under it. On
/// continental crust it takes that crust's plate (its own, the fragment a rift left it on, or the
/// plate a collision welded it to). A structure is born on continental crust, so oceanic crust
/// under it means it sits at its continent's edge or left a boundary cell with the wrong plate:
/// it takes the plate of the nearest continental cell within three cells. Only one with no
/// continent that near for [`LOST_AFTER_STEPS`] steps is dropped.
pub fn advance(
    features: &mut Vec<Feature>,
    grid: &GridDesc,
    omega: &dyn Fn(u32) -> [f64; 3],
    dt_yr: f64,
    plate_id: &[f32],
    crust_type: &[f32],
    counts: &mut Counts,
) {
    let frame = BlockFrame::new(grid.circumference() as u32);
    let c = frame.circumference();
    features.retain_mut(|f| {
        let om = omega(f.plate);
        if om != [0.0; 3] {
            let rot = rotation(DVec3::from_array(om), dt_yr);
            // A second point a little way along the axis carries the azimuth.
            let d = direction(f64::from(f.azimuth_deg));
            let step = 8.0; // ledger: DEF-037
            let pole = frame.projection().pole_z();
            let z2 = (f.z + step * d[1]).clamp(-pole, pole);
            let (x1, z1) = frame.block(rot * frame.unit(f.x, f.z));
            let (x2, z2) = frame.block(rot * frame.unit(f.x + step * d[0], z2));
            let mut dx = x2 - x1;
            dx -= (dx / c + 0.5).floor() * c;
            if dx != 0.0 || z2 != z1 {
                f.azimuth_deg = azimuth_of(dx, z2 - z1) as f32;
            }
            f.x = x1;
            f.z = z1;
        }
        let cell = cell_of(grid, f.x, f.z);
        if crust_type[cell] as u8 == CONTINENTAL {
            f.plate = plate_id[cell].max(0.0) as u32;
            f.adrift = 0;
            return true;
        }
        // Oceanic crust under it: it sits at its continent's edge (a rifted margin, a coast the
        // last resampling moved by a cell) or rode the wrong plate out of a boundary cell. The
        // nearest continental cell within EDGE_CELLS is its continent.
        let w = grid.w as i64;
        let (i, j) = ((cell as i64) % w, (cell as i64) / w);
        let mut best: Option<(i64, u32)> = None;
        for dj in -EDGE_CELLS..=EDGE_CELLS {
            let jj = j + dj;
            if jj < 0 || jj >= i64::from(grid.h) {
                continue;
            }
            for di in -EDGE_CELLS..=EDGE_CELLS {
                let k = (jj * w + (i + di).rem_euclid(w)) as usize;
                let d2 = di * di + dj * dj;
                if crust_type[k] as u8 == CONTINENTAL && best.is_none_or(|b| d2 < b.0) {
                    best = Some((d2, plate_id[k].max(0.0) as u32));
                }
            }
        }
        if let Some((_, plate)) = best {
            f.plate = plate;
            f.adrift = 0;
            return true;
        }
        f.adrift = f.adrift.saturating_add(1);
        if f.adrift > LOST_AFTER_STEPS {
            counts.subducted += 1;
            return false;
        }
        if om == [0.0; 3] {
            // Its plate is gone: whatever is under it carries it.
            f.plate = plate_id[cell].max(0.0) as u32;
        }
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn fold(x: f64, z: f64, azimuth: f32) -> Feature {
        Feature {
            id: 1,
            kind: Kind::FoldTrain,
            x,
            z,
            plate: 0,
            t_myr: -50.0,
            azimuth_deg: azimuth,
            half_along: 300.0,
            half_across: 120.0,
            wavelength: 60.0,
            amplitude: 8.0,
            phase: 0.3,
            dip_deg: 0.0,
            slip: 0.0,
            cause: 0,
            adrift: 0,
            wear_m: 0.0,
            rise: 0.0,
            depth: 0.0,
            rock: 0,
        }
    }

    #[test]
    fn directions_and_azimuths_round_trip() {
        assert_eq!(direction(0.0), [0.0, -1.0], "north is −z");
        let e = direction(90.0);
        assert!(
            (e[0] - 1.0).abs() < 1e-12 && e[1].abs() < 1e-12,
            "east is +x"
        );
        for a in [0.0, 33.0, 90.0, 181.0, 275.5, 359.0] {
            let d = direction(a);
            assert!((azimuth_of(d[0], d[1]) - a).abs() < 1e-9);
        }
    }

    #[test]
    fn features_ride_their_plate_and_turn_with_it() {
        let grid = GridDesc::for_planet("L1", 4096, 16).unwrap();
        let n = grid.cells();
        let w = grid.w as usize;
        // Ω about the pole: one L1 cell of longitude per Myr. On the equator a rotation about
        // the pole keeps every azimuth.
        let dlon = std::f64::consts::TAU / w as f64;
        let omega = move |_: u32| [0.0, 0.0, dlon / 1.0e6];
        let plate = vec![0.0f32; n];
        let crust = vec![f32::from(CONTINENTAL); n];
        let mut fs = vec![fold(0.0, 0.0, 30.0)];
        let mut counts = Counts::default();
        advance(&mut fs, &grid, &omega, 1.0e6, &plate, &crust, &mut counts);
        assert!((fs[0].x.abs() - 16.0).abs() < 0.05, "{}", fs[0].x);
        assert!(
            (fs[0].azimuth_deg - 30.0).abs() < 0.05,
            "{}",
            fs[0].azimuth_deg
        );
        // Ω about an axis through the equator at the feature's own meridian turns the map under
        // it: the feature stays put and its azimuth turns by the angle.
        let q = BlockFrame::new(4096).unit(fs[0].x, fs[0].z);
        let turn = 20.0f64.to_radians();
        let omega = move |_: u32| [q.x * turn / 1.0e6, q.y * turn / 1.0e6, q.z * turn / 1.0e6];
        let before = fs[0].clone();
        advance(&mut fs, &grid, &omega, 1.0e6, &plate, &crust, &mut counts);
        assert!((fs[0].x - before.x).abs() < 1e-6 && (fs[0].z - before.z).abs() < 1e-6);
        let turned =
            (f64::from(fs[0].azimuth_deg) - f64::from(before.azimuth_deg)).rem_euclid(360.0);
        assert!(
            (turned - 20.0).abs() < 0.5 || (turned - 340.0).abs() < 0.5,
            "turned {turned}"
        );
        // Oceanic crust under it for a step or two (a boundary cell read from the other side):
        // kept, still on its plate while that plate lives.
        let other = vec![3.0f32; n];
        let ocean = vec![0.0f32; n];
        let alive = |_: u32| [0.0, 0.0, 1.0e-12];
        advance(&mut fs, &grid, &alive, 0.0, &other, &ocean, &mut counts);
        assert_eq!((fs.len(), fs[0].plate, fs[0].adrift), (1, 0, 1));
        // Back on continental crust: the count starts again.
        advance(&mut fs, &grid, &alive, 0.0, &plate, &crust, &mut counts);
        assert_eq!(fs[0].adrift, 0);
        // Off continental crust for good: lost after LOST_AFTER_STEPS steps.
        for _ in 0..LOST_AFTER_STEPS {
            advance(&mut fs, &grid, &alive, 0.0, &other, &ocean, &mut counts);
            assert_eq!(fs.len(), 1);
        }
        advance(&mut fs, &grid, &alive, 0.0, &other, &ocean, &mut counts);
        assert!(fs.is_empty() && counts.subducted == 1);
        // Its plate gone and oceanic crust under it, a continent one cell away: that plate.
        let mut fs = vec![fold(100.0, 50.0, 0.0)];
        let c0 = cell_of(&grid, 100.0, 50.0);
        let mut mixed_crust = ocean.clone();
        let mut mixed_plate = other.clone();
        mixed_crust[c0 + 1] = f32::from(CONTINENTAL);
        mixed_plate[c0 + 1] = 7.0;
        advance(
            &mut fs,
            &grid,
            &|_| [0.0; 3],
            0.0,
            &mixed_plate,
            &mixed_crust,
            &mut counts,
        );
        assert_eq!((fs[0].plate, fs[0].adrift), (7, 0));
        // On continental crust that changed plate (a terrane docked): kept, on the new plate.
        let mut fs = vec![fold(100.0, 50.0, 0.0)];
        advance(
            &mut fs,
            &grid,
            &|_| [0.0; 3],
            0.0,
            &other,
            &crust,
            &mut counts,
        );
        assert_eq!(fs[0].plate, 3);
    }

    #[test]
    fn a_feature_covers_its_window() {
        let f = fold(0.0, 0.0, 90.0);
        // Axis east-west: 300 along x, 120 along z.
        assert!(f.covers(250.0, 0.0, 4096.0));
        assert!(!f.covers(310.0, 0.0, 4096.0));
        assert!(f.covers(0.0, 100.0, 4096.0));
        assert!(!f.covers(0.0, 130.0, 4096.0));
        // Across the seam.
        let g = fold(2040.0, 0.0, 90.0);
        assert!(g.covers(-2040.0, 0.0, 4096.0));
    }
}
