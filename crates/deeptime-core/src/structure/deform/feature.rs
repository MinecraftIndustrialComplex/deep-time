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
}

impl Kind {
    pub const ALL: [Kind; 4] = [
        Kind::FoldTrain,
        Kind::Thrust,
        Kind::Normal,
        Kind::StrikeSlip,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Kind::FoldTrain => "fold_train",
            Kind::Thrust => "thrust",
            Kind::Normal => "normal_fault",
            Kind::StrikeSlip => "strike_slip_fault",
        }
    }

    pub fn is_fault(self) -> bool {
        self != Kind::FoldTrain
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
}

/// Counters for the run summary.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Counts {
    pub fold_trains: u64,
    pub thrusts: u64,
    pub normal_faults: u64,
    pub strike_slip_faults: u64,
    /// Not started because enough structures of the kind were already there.
    pub saturated: u64,
    /// Lost with oceanic crust that went under another plate.
    pub subducted: u64,
}

impl Counts {
    pub fn add(&mut self, kind: Kind) {
        match kind {
            Kind::FoldTrain => self.fold_trains += 1,
            Kind::Thrust => self.thrusts += 1,
            Kind::Normal => self.normal_faults += 1,
            Kind::StrikeSlip => self.strike_slip_faults += 1,
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
    /// fault's trace lies there); `profile` the fold profile (the sine table).
    pub fn to_noddy(
        &self,
        origin_x: f64,
        origin_z: f64,
        c: f64,
        surface_y: f64,
        profile: &noddy::Profile,
    ) -> noddy::Event {
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
        };
        noddy::Event::Windowed(noddy::Windowed::new(inner, window))
    }

    /// Folds: the across coordinate of the profile's origin relative to the centre.
    fn fold_offset(&self) -> f64 {
        let wl = f64::from(self.wavelength);
        let waves = (f64::from(self.half_across) / wl).ceil();
        -(waves + f64::from(self.phase)) * wl
    }

    /// The feature as a baked event of the world data. The vectors are taken from the Noddy
    /// event's own frame, so the two agree.
    pub fn to_baked(&self, surface_y: f64) -> StructEvent {
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
        };
        StructEvent {
            id: self.id,
            t_years: f64::from(self.t_myr) * 1.0e6,
            window,
            shape,
        }
    }
}

/// Moves every feature with its plate over `dt_yr` (Ω per plate id, rad/yr), turning its azimuth
/// with the plate, then takes the plate now under it; features on oceanic crust that went under
/// another plate are lost.
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
            let step = 8.0;
            let z2 = (f.z + step * d[1]).clamp(-c / 4.0, c / 4.0);
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
        let now = plate_id[cell].max(0.0) as u32;
        if now != f.plate {
            if crust_type[cell] as u8 != CONTINENTAL {
                counts.subducted += 1;
                return false;
            }
            f.plate = now;
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
        // On oceanic crust under another plate: lost.
        let other = vec![3.0f32; n];
        let ocean = vec![0.0f32; n];
        advance(
            &mut fs,
            &grid,
            &|_| [0.0; 3],
            0.0,
            &other,
            &ocean,
            &mut counts,
        );
        assert!(fs.is_empty() && counts.subducted == 1);
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
