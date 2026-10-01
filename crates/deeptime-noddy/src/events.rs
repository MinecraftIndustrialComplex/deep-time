// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Ported from Noddy (Loop3D/noddyverse, commit 6a21b23, MIT License, Copyright (c) 2021 Loop3D):
// `src/unEvents.c` (`unfold`, `unplan`, `unrot`, `unplug`, `undyke`, `unfault`), `src/ellFault.c`
// (`NewtonRaphson`, `ProfilePosition`, `OneYdisplace`, `CalcYDisplace`) and the orientation set-up
// of each event in `src/getEvent.c` (`loadFold`, `loadFault`, `loadDyke`, `loadPlug`, `load_tilts`,
// `load_strains`, `load_unconformities`). See NOTICE.md.
//! The events and their inverses.
//!
//! Each event's `undo` takes a point of the model as it is after the event and returns where that
//! point was before it ([`Step::Moved`]), or says the point is the event's own rock
//! ([`Step::Stop`]: above an unconformity, in a dyke, in a plug) and ends the unravelling.
//! `place` is the event's index in its history (0 is the base stratigraphy), where it writes its
//! [`Code`].

use crate::orient::{self, Rotation, irotate, radius, rotset, vm};
use crate::profile::Profile;
use crate::taste::{Code, Sequence};
use crate::{M3, TOLERANCE, V3};

/// What undoing one event did to a point.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step {
    /// The point before the event (possibly unmoved).
    Moved(V3),
    /// The point is this event's rock; older events never saw it.
    Stop,
}

#[inline]
fn sub(p: V3, q: V3) -> V3 {
    [p[0] - q[0], p[1] - q[1], p[2] - q[2]]
}

#[inline]
fn add(p: V3, q: V3) -> V3 {
    [p[0] + q[0], p[1] + q[1], p[2] + q[2]]
}

/// `0.693147 / c²`, the Gaussian's rate for a half-width `c` (0: no decay), with the original's
/// rounded ln 2.
#[inline]
fn half_width_rate(c: f64, strict: bool) -> f64 {
    let on = if strict {
        c > TOLERANCE
    } else {
        c.abs() > TOLERANCE
    };
    if on { 0.693147 / (c * c) } else { 0.0 }
}

// ------------------------------------------------------------------------------------------ fold

/// A fold: simple shear along the axial plane, a periodic profile across it (`unfold`).
///
/// In the fold's frame x runs across the fold (the profile's phase is 2π·x/wavelength), y along
/// the fold axis, and the displacement is along z, the third axis, which lies in the axial
/// plane: a point moves by `profile(phase) · amplitude · exp(−ln 2 · y² / cycle²)`. With `cycle`
/// 0 the fold is cylindrical; otherwise its amplitude halves `cycle` along the axis from the
/// position, a dome-and-basin or doubly plunging shape.
#[derive(Clone, Debug, PartialEq)]
pub struct Fold {
    pub position: V3,
    pub wavelength: f64,
    pub amplitude: f64,
    /// Noddy's `cycle` ("cylindricity"): the half-width of the amplitude's Gaussian along the
    /// axis; ≤ 0.001 is cylindrical.
    pub cycle: f64,
    /// One wavelength only (0 ≤ x < wavelength) instead of a train.
    pub single: bool,
    pub profile: Profile,
    pub rotation: Rotation,
}

impl Fold {
    /// A fold whose axial plane dips `dip`° towards `dip_direction`° and whose axis pitches
    /// `axis_pitch`° in it (`loadFold`).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position: V3,
        dip_direction: f64,
        dip: f64,
        axis_pitch: f64,
        wavelength: f64,
        amplitude: f64,
        cycle: f64,
        single: bool,
        profile: Profile,
    ) -> Fold {
        Fold {
            position,
            wavelength,
            amplitude,
            cycle,
            single,
            profile,
            rotation: rotset(dip, dip_direction, axis_pitch),
        }
    }

    /// The displacement along local z at local (x, y).
    #[inline]
    pub fn displacement(&self, x: f64, y: f64) -> f64 {
        if self.wavelength.abs() <= TOLERANCE {
            return 0.0;
        }
        if self.single && !(x >= 0.0 && x < self.wavelength) {
            return 0.0;
        }
        let tcyl = half_width_rate(self.cycle, true);
        let temp2 = x * 6.2831853 / self.wavelength;
        self.profile.at(temp2) * self.amplitude * (-(y * y) * tcyl).exp()
    }

    pub fn undo(&self, p: V3) -> Step {
        let mut q = vm(sub(p, self.position), &self.rotation.forward);
        q[2] -= self.displacement(q[0], q[1]);
        Step::Moved(add(vm(q, &self.rotation.reverse), self.position))
    }
}

// ----------------------------------------------------------------------------------------- fault

/// What kind of discontinuity (`FAULT_OPTIONS.type`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultKind {
    /// A sharp fault: the slip jumps at the surface.
    Fault,
    /// A ductile shear zone: the slip grows across `width` as a half sine.
    ShearZone,
    /// A kink band: the slip grows linearly across `width`.
    Kink,
}

/// The fault surface and how slip is distributed (`FAULT_OPTIONS.geometry`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultGeometry {
    /// An infinite plane, uniform slip along the pitch line.
    Translation,
    /// An infinite plane, rotation about the plane's pole through the position.
    Rotation,
    /// A plane with slip that dies out towards the edge of an ellipsoid.
    Elliptical,
    /// A profiled (listric or wavy) surface inside an ellipsoid.
    Curved,
    /// A vertical-axis ring: a cylinder of `radius` about the local y axis.
    Ring,
}

/// Which side moves (`FAULT_OPTIONS.movement`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Movement {
    HangingWall,
    FootWall,
    Both,
}

/// A fault, shear zone or kink (`unfault`).
///
/// In the fault's frame x is across the surface (the pole's side is "above": the hanging wall of
/// a dipping fault), y along the slip line (the pitch in the plane) and z across the slip in the
/// plane. Undoing a translation fault moves the hanging wall by −slip along y, the foot wall by
/// +slip, each only if `movement` says that side moved (`Both` gives each half).
#[derive(Clone, Debug, PartialEq)]
pub struct Fault {
    pub kind: FaultKind,
    pub geometry: FaultGeometry,
    pub movement: Movement,
    pub position: V3,
    /// Rotation faults: degrees about the pole.
    pub rotation_deg: f64,
    pub slip: f64,
    /// Curved faults: the profile's amplitude.
    pub amplitude: f64,
    /// Ring faults: the cylinder's radius.
    pub radius: f64,
    /// Shear zones and kinks: the band's width.
    pub width: f64,
    /// Elliptical and curved faults: the ellipsoid's semi-axes along the slip (x), across the
    /// surface (y) and across the slip in the surface (z), Noddy's names.
    pub x_axis: f64,
    pub y_axis: f64,
    pub z_axis: f64,
    /// Curved faults: the half-width of the profile's Gaussian (0: none).
    pub cylindrical_index: f64,
    /// Curved faults: degrees the profile is turned in the surface.
    pub profile_pitch: f64,
    pub profile: Profile,
    pub rotation: Rotation,
    /// The plane a·x + b·y + c·z + d = 0 (kept as the original does; the kinematics use the
    /// frame).
    pub plane: [f64; 4],
}

impl Fault {
    /// A planar fault with uniform slip: the plane dips `dip`° towards `dip_direction`°, the slip
    /// line pitches `pitch`° in it (90: dip-slip, 0: strike-slip).
    pub fn translation(
        position: V3,
        dip_direction: f64,
        dip: f64,
        pitch: f64,
        slip: f64,
        movement: Movement,
    ) -> Fault {
        Fault::new(
            FaultKind::Fault,
            FaultGeometry::Translation,
            movement,
            position,
            dip_direction,
            dip,
            pitch,
        )
        .with_slip(slip)
    }

    /// Any fault; the scale fields start at zero (`loadFault`: a vertical plane is taken as
    /// 89.999°, a rotation fault's pitch as 0).
    pub fn new(
        kind: FaultKind,
        geometry: FaultGeometry,
        movement: Movement,
        position: V3,
        dip_direction: f64,
        dip: f64,
        pitch: f64,
    ) -> Fault {
        let pitch = if geometry == FaultGeometry::Rotation {
            0.0
        } else {
            pitch
        };
        let dip = if dip == 90.0 { 89.999 } else { dip };
        let att = orient::convrt(dip, dip_direction, pitch);
        Fault {
            kind,
            geometry,
            movement,
            position,
            rotation_deg: 0.0,
            slip: 0.0,
            amplitude: 0.0,
            radius: 0.0,
            width: 0.0,
            x_axis: 0.0,
            y_axis: 0.0,
            z_axis: 0.0,
            cylindrical_index: 0.0,
            profile_pitch: 0.0,
            profile: Profile::Sine,
            rotation: rotset(dip, dip_direction, pitch),
            plane: orient::plane(&att, position),
        }
    }

    pub fn with_slip(mut self, slip: f64) -> Fault {
        self.slip = slip;
        self
    }

    pub fn with_rotation(mut self, degrees: f64) -> Fault {
        self.rotation_deg = degrees;
        self
    }

    pub fn with_width(mut self, width: f64) -> Fault {
        self.width = width;
        self
    }

    pub fn with_radius(mut self, radius: f64) -> Fault {
        self.radius = radius;
        self
    }

    pub fn with_ellipsoid(mut self, x_axis: f64, y_axis: f64, z_axis: f64) -> Fault {
        self.x_axis = x_axis;
        self.y_axis = y_axis;
        self.z_axis = z_axis;
        self
    }

    pub fn with_curve(
        mut self,
        amplitude: f64,
        cylindrical_index: f64,
        profile_pitch: f64,
        profile: Profile,
    ) -> Fault {
        self.amplitude = amplitude;
        self.cylindrical_index = cylindrical_index;
        self.profile_pitch = profile_pitch;
        self.profile = profile;
        self
    }

    pub fn undo(&self, p: V3, place: usize, seq: &mut Sequence) -> Step {
        use FaultGeometry::{Curved, Elliptical, Ring, Rotation as Rot, Translation};
        let g = self.geometry;
        let both = self.movement == Movement::Both;
        let mut rot = self.rotation_deg;
        let mut tcyl = 0.0;
        let (mut mat_pos, mut mat_neg) = ([[0.0; 3]; 3], [[0.0; 3]; 3]);
        if g == Rot {
            if both {
                rot /= 2.0;
            }
            let rrot = rot * 3.1415927 / 180.0;
            mat_pos = irotate(1.0, 0.0, 0.0, -rrot);
            mat_neg = irotate(1.0, 0.0, 0.0, rrot);
        } else if g == Curved {
            tcyl = half_width_rate(self.cylindrical_index, false);
        }
        let sfactor = if g == Curved || g == Elliptical {
            if both {
                self.slip / (2.0 * self.x_axis)
            } else {
                self.slip / self.x_axis
            }
        } else if both {
            self.slip / 2.0
        } else {
            self.slip
        };

        let mut q = vm(sub(p, self.position), &self.rotation.forward);

        // Where the surface is under this point.
        let mut old_y = 0.0;
        let mut concdist = 0.0;
        let ycut = match g {
            Curved => {
                old_y = q[1];
                let pos = profile_position(q[1], q[2], self.profile_pitch, self.x_axis);
                one_y_displace(pos, &self.profile, self.x_axis, self.amplitude, tcyl)
            }
            Ring => {
                concdist = ((q[0] * q[0]) + (q[2] * q[2])).sqrt();
                self.radius
            }
            _ => 0.0,
        };
        // Where the point is in the ellipsoid.
        let efactor = if g == Curved || g == Elliptical {
            (q[1] / self.x_axis).powf(2.0)
                + (q[0] / self.y_axis).powf(2.0)
                + (q[2] / self.z_axis).powf(2.0)
        } else {
            0.0
        };

        if self.kind == FaultKind::Fault {
            let limited = g == Curved || g == Elliptical;
            // The original's four cases in its order: above an infinite planar fault, inside a
            // ring fault, above a curved or elliptical fault inside its ellipsoid, below one.
            let code = if (efactor < 1.0 && q[0] > ycut && g != Ring && !limited)
                || (g == Ring && concdist < ycut)
            {
                Some(Code::Fault1)
            } else if efactor < 1.0 && q[0] > ycut && g != Ring && limited {
                Some(Code::Fault3)
            } else if efactor < 1.0 && q[0] <= ycut && limited {
                Some(Code::Fault2)
            } else {
                None
            };
            if let Some(code) = code {
                seq.set(place, code);
            }
        }

        if efactor < 1.0 {
            let width = self.width;
            let sz_factor = match (self.kind, g == Ring) {
                (FaultKind::ShearZone, false) => {
                    if q[0] > width / 2.0 || q[0] < -width / 2.0 {
                        1.0
                    } else {
                        // `fourier` of the sine coefficients: sin of the scaled position.
                        (q[0].abs() * 3.1415927 / width).sin()
                    }
                }
                (FaultKind::Kink, false) => {
                    if q[0] > width / 2.0 || q[0] < -width / 2.0 {
                        1.0
                    } else {
                        2.0 * q[0].abs() / width
                    }
                }
                (FaultKind::ShearZone, true) => {
                    if concdist < (self.radius - (width / 2.0))
                        || concdist > (self.radius + (width / 2.0))
                    {
                        1.0
                    } else {
                        ((self.radius - concdist).abs() * 3.1415927 / width).sin()
                    }
                }
                (FaultKind::Kink, true) => {
                    if concdist < (self.radius - (width / 2.0))
                        || concdist > (self.radius + (width / 2.0))
                    {
                        1.0
                    } else {
                        2.0 * (self.radius - concdist).abs() / width
                    }
                }
                (FaultKind::Fault, _) => 1.0,
            };

            let ytest = if g == Ring { concdist } else { q[0] };
            // Opposite movement for the hanging wall and the foot wall.
            let sign: i32 = if ytest > ycut {
                if self.movement == Movement::HangingWall || both {
                    -1
                } else {
                    0
                }
            } else if self.movement == Movement::FootWall || both {
                1
            } else {
                0
            };

            match g {
                Curved | Elliptical => {
                    let nfactor = ((self.y_axis - (q[0] - ycut).abs()) / self.y_axis).powf(2.0)
                        * ((self.z_axis - q[2].abs()) / self.z_axis).powf(2.0);
                    let factor = sfactor * f64::from(-sign);
                    let xprime = newton_raphson(q[1], self.x_axis, factor);
                    let delta_x = xprime - q[1];
                    q[1] += delta_x * nfactor * sz_factor;
                }
                Translation | Ring => {
                    q[1] += sz_factor * f64::from(sign) * sfactor;
                }
                Rot => {
                    if sign != 0 {
                        let (pos, neg) = if sz_factor < 1.0 {
                            let rrot = sz_factor * rot * 3.1415927 / 180.0;
                            (irotate(1.0, 0.0, 0.0, -rrot), irotate(1.0, 0.0, 0.0, rrot))
                        } else {
                            (mat_pos, mat_neg)
                        };
                        q = vm(q, if sign == -1 { &pos } else { &neg });
                    }
                }
            }
            // The extra displacement across a curved fault, so the walls follow its surface.
            if g == Curved {
                q[0] = calc_y_displace(
                    q[0],
                    old_y,
                    q[1],
                    q[2],
                    self.profile_pitch,
                    self.x_axis,
                    &self.profile,
                    self.amplitude,
                    tcyl,
                );
            }
        }
        Step::Moved(add(vm(q, &self.rotation.reverse), self.position))
    }
}

/// `NewtonRaphson`: the position along an elliptical fault's slip axis before a slip of
/// `f1` (as a fraction of the axis), solving x/X = g + F·(1 − g²)² for g. The original evaluates
/// the derivative at the starting point on every iteration; kept.
pub fn newton_raphson(x: f64, x_axis: f64, f1: f64) -> f64 {
    let mut g = x / x_axis;
    let xratio = x / x_axis;
    let mut i = 0;
    loop {
        let ng = g
            - ((xratio - g - f1 + (2.0 * f1 * g * g) - (f1 * g * g * g * g))
                / (-1.0 + (4.0 * f1 * x / (x_axis * x_axis))
                    - (4.0 * f1 * x * x * x / (x_axis * x_axis * x_axis * x_axis))));
        let error = ng - g;
        g = ng;
        i += 1;
        if !(error.abs() > 1e-5 && i < 100) {
            break;
        }
    }
    g * x_axis
}

/// `ProfilePosition`: a point of the fault surface in the turned profile's coordinates.
fn profile_position(x: f64, z: f64, prof_rot: f64, prof_dim: f64) -> [f64; 2] {
    let s = (prof_rot * 3.1415927 / 180.0).sin();
    let c = (prof_rot * 3.1415927 / 180.0).cos();
    // rot_mat = [[s, −c], [c, s]]
    [(s * x) + (-c * z) + prof_dim, (c * x) + (s * z)]
}

/// `OneYdisplace`: the curved surface's offset across the fault at a profile position.
fn one_y_displace(
    pos: [f64; 2],
    profile: &Profile,
    prof_dim: f64,
    prof_amp: f64,
    cyl_index: f64,
) -> f64 {
    let scale_xpos = pos[0] * (3.1415927 / prof_dim);
    if pos[0] >= 0.0 && pos[0] < prof_dim * 2.0 {
        profile.at(scale_xpos) * prof_amp * (-(pos[1] * pos[1]) * cyl_index).exp()
    } else {
        0.0
    }
}

/// `CalcYDisplace` as `unfault` calls it. The original's parameters are named (YPos, newXpos,
/// oldXpos, …) and the call passes the position before the slip as `newXpos` and the one after
/// as `oldXpos`; the names here say what the values are.
#[allow(clippy::too_many_arguments)]
fn calc_y_displace(
    y_pos: f64,
    before_slip: f64,
    after_slip: f64,
    z_pos: f64,
    prof_rot: f64,
    prof_dim: f64,
    profile: &Profile,
    prof_amp: f64,
    cyl_index: f64,
) -> f64 {
    let oldposvec = profile_position(after_slip, z_pos, prof_rot, prof_dim);
    let newposvec = profile_position(before_slip, z_pos, prof_rot, prof_dim);
    let old_y = one_y_displace(oldposvec, profile, prof_dim, prof_amp, cyl_index);
    let new_y = one_y_displace(newposvec, profile, prof_dim, prof_amp, cyl_index);
    y_pos - (new_y - old_y)
}

// ---------------------------------------------------------------------------------- unconformity

/// An unconformity: an erosion surface and the cover laid on it (`unplan`). A point on the pole's
/// side of the plane (a·x + b·y + c·z + d ≥ 0) is cover.
#[derive(Clone, Debug, PartialEq)]
pub struct Unconformity {
    pub position: V3,
    pub plane: [f64; 4],
}

impl Unconformity {
    /// A plane through `position` dipping `dip`° towards `dip_direction`° (0: horizontal, cover
    /// above).
    pub fn new(position: V3, dip_direction: f64, dip: f64) -> Unconformity {
        let att = orient::convrt(dip, dip_direction, 0.0);
        Unconformity {
            position,
            plane: orient::plane(&att, position),
        }
    }

    pub fn undo(&self, p: V3, place: usize, seq: &mut Sequence) -> Step {
        let [a, b, c, d] = self.plane;
        let sub = (a * p[0]) + (b * p[1]) + (c * p[2]) + d;
        if sub >= 0.0 {
            seq.set(place, Code::Unconformity);
            Step::Stop
        } else {
            Step::Moved(p)
        }
    }
}

// ------------------------------------------------------------------------------------------ dyke

/// How a dyke makes room (`DYKE_OPTIONS.type`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DykeKind {
    /// The walls move apart by the width (and slip along the dyke).
    Dilation,
    /// The dyke replaces the rock it cuts; nothing moves.
    Stope,
}

/// A dyke: a sheet of `width` on the pole's side of its plane (`undyke`).
#[derive(Clone, Debug, PartialEq)]
pub struct Dyke {
    pub kind: DykeKind,
    /// Take the previous event's rock when that event is igneous too (one intrusion in several
    /// events).
    pub merge: bool,
    pub position: V3,
    pub slip_pitch: f64,
    pub slip_length: f64,
    pub width: f64,
    pub rotation: Rotation,
}

impl Dyke {
    /// A dyke whose plane dips `dip`° towards `dip_direction`° (`loadDyke`: a vertical dyke's dip
    /// direction is turned by 180°, and the frame is set with a pitch of 90°).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        kind: DykeKind,
        merge: bool,
        position: V3,
        dip_direction: f64,
        dip: f64,
        slip_pitch: f64,
        slip_length: f64,
        width: f64,
    ) -> Dyke {
        let dip_direction = if dip == 90.0 {
            dip_direction + 180.0
        } else {
            dip_direction
        };
        Dyke {
            kind,
            merge,
            position,
            slip_pitch,
            slip_length,
            width,
            rotation: rotset(dip, dip_direction, 90.0),
        }
    }

    pub fn undo(&self, p: V3, place: usize, previous_igneous: bool, seq: &mut Sequence) -> Step {
        let pitch = if self.kind == DykeKind::Dilation {
            self.slip_pitch
        } else {
            0.0
        };
        let rad = orient::rad_per_deg();
        let temp_p = (-pitch + 90.0) * rad;
        let deltay = temp_p.cos() * self.slip_length;
        let deltaz = temp_p.sin() * self.slip_length;

        let mut q = vm(sub(p, self.position), &self.rotation.forward);
        if q[0] <= 0.0 && self.kind == DykeKind::Dilation {
            seq.set(place, Code::Fault1);
            q[0] += self.width;
            q[1] -= deltay;
            q[2] -= deltaz;
            Step::Moved(add(vm(q, &self.rotation.reverse), self.position))
        } else if q[0] > 0.0 && q[0] <= self.width {
            igneous(self.merge, place, previous_igneous, seq);
            Step::Stop
        } else {
            Step::Moved(p)
        }
    }
}

/// An igneous rock's code: at the event, or at the event before it when the two merge.
#[inline]
fn igneous(merge: bool, place: usize, previous_igneous: bool, seq: &mut Sequence) {
    if merge && previous_igneous && place > 0 {
        seq.set(place - 1, Code::Igneous);
    } else {
        seq.set(place, Code::Igneous);
    }
}

// ------------------------------------------------------------------------------------------ plug

/// A plug's shape (`PLUG_OPTIONS.type`), about the local x axis (down the plug for a vertical
/// one: the body is on the negative-x side of the position for cones and paraboloids).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlugShape {
    Cylinder {
        radius: f64,
    },
    /// Opening away from the position at `apical_angle`° from the axis.
    Cone {
        apical_angle: f64,
    },
    /// Radius √(|x|·b) at distance x along the axis.
    Paraboloid {
        b: f64,
    },
    /// Semi-axes: `a` along the local x, `c` along y, `b` along z (Noddy's pairing).
    Ellipsoid {
        a: f64,
        b: f64,
        c: f64,
    },
}

/// A plug: an intrusion that replaces what it cuts (`unplug`).
#[derive(Clone, Debug, PartialEq)]
pub struct Plug {
    pub shape: PlugShape,
    pub merge: bool,
    pub position: V3,
    pub rotation: Rotation,
}

impl Plug {
    /// A plug whose axis is set by `dip`° and `dip_direction`° (`loadPlug`: the frame is built
    /// from dip − 90°, so a dip of 90° is a vertical axis; the pitch counts for ellipsoids only).
    pub fn new(
        shape: PlugShape,
        merge: bool,
        position: V3,
        dip_direction: f64,
        dip: f64,
        axis_pitch: f64,
    ) -> Plug {
        let pitch = if matches!(shape, PlugShape::Ellipsoid { .. }) {
            axis_pitch
        } else {
            0.0
        };
        Plug {
            shape,
            merge,
            position,
            rotation: rotset(dip - 90.0, dip_direction, pitch),
        }
    }

    /// Whether the point is inside the plug.
    pub fn holds(&self, p: V3) -> bool {
        let q = vm(sub(p, self.position), &self.rotation.forward);
        match self.shape {
            PlugShape::Cone { apical_angle } => {
                let apic = (apical_angle * 0.01745329278).tan();
                let wfactr = -q[0] * apic;
                wfactr > radius(q[1], q[2]) && q[0] < 0.0
            }
            PlugShape::Paraboloid { b } => {
                let wfactr = (q[0] * b).abs().sqrt();
                wfactr > radius(q[1], q[2]) && q[0] < 0.0
            }
            PlugShape::Cylinder { radius: rad } => rad > radius(q[1], q[2]),
            PlugShape::Ellipsoid { a, b, c } => {
                let sq = |v: f64| if v == 0.0 { 0.00000000001 } else { v * v };
                let (ta, tb, tc) = (sq(a), sq(b), sq(c));
                let r = ((q[0] * q[0]) / ta) + ((q[1] * q[1]) / tc) + ((q[2] * q[2]) / tb);
                1.0 > r
            }
        }
    }

    pub fn undo(&self, p: V3, place: usize, previous_igneous: bool, seq: &mut Sequence) -> Step {
        if self.holds(p) {
            igneous(self.merge, place, previous_igneous, seq);
            Step::Stop
        } else {
            Step::Moved(p)
        }
    }
}

// --------------------------------------------------------------------------------- tilt, strain

/// A tilt: a rigid rotation about an axis through a point (`unrot` for `TILT`).
#[derive(Clone, Debug, PartialEq)]
pub struct Tilt {
    pub position: V3,
    /// The matrix `unrot` multiplies by.
    pub matrix: M3,
}

impl Tilt {
    /// A rotation of `rotation`° about the line through `position` plunging `plunge`° towards
    /// `plunge_direction`° (`load_tilts`, with its 0.0174532 rad per degree).
    pub fn new(position: V3, rotation: f64, plunge_direction: f64, plunge: f64) -> Tilt {
        let angle = rotation * 0.0174532;
        let axis = orient::paxis(plunge, plunge_direction);
        Tilt {
            position,
            matrix: irotate(axis[0], axis[1], axis[2], angle),
        }
    }

    pub fn undo(&self, p: V3) -> Step {
        Step::Moved(add(vm(sub(p, self.position), &self.matrix), self.position))
    }
}

/// Homogeneous strain about the origin (`unrot` for `STRAIN`).
#[derive(Clone, Debug, PartialEq)]
pub struct Strain {
    /// The inverse tensor `unrot` multiplies by.
    pub inverse: M3,
}

impl Strain {
    /// From the strain tensor as a history file lists it (`load_strains`: the matrix inverted is
    /// the transpose of the listed one). `None` when the tensor is singular.
    pub fn new(tensor: &M3) -> Option<Strain> {
        let mut t = [[0.0; 3]; 3];
        for (i, row) in tensor.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                t[j][i] = *v;
            }
        }
        orient::invert(&t).map(|inverse| Strain { inverse })
    }

    pub fn undo(&self, p: V3) -> Step {
        Step::Moved(vm(p, &self.inverse))
    }
}

// ----------------------------------------------------------------------------------------- event

/// One event of a history.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Fold(Fold),
    Fault(Fault),
    Unconformity(Unconformity),
    Dyke(Dyke),
    Plug(Plug),
    Tilt(Tilt),
    Strain(Strain),
    /// Noddy's `STOP`: the unravelling ends here, as if the history began after this event.
    Stop,
    /// One of the above under a window of finite extent (Deep Time's addition, `windowed.rs`).
    Windowed(crate::windowed::Windowed),
}

impl Event {
    /// A dyke or a plug.
    pub fn is_igneous(&self) -> bool {
        match self {
            Event::Dyke(_) | Event::Plug(_) => true,
            Event::Windowed(w) => w.inner.is_igneous(),
            _ => false,
        }
    }

    /// Undoes the event for one point. `previous_igneous`: whether the event before this one in
    /// the history is a dyke or a plug.
    pub fn undo(&self, p: V3, place: usize, previous_igneous: bool, seq: &mut Sequence) -> Step {
        match self {
            Event::Fold(e) => e.undo(p),
            Event::Fault(e) => e.undo(p, place, seq),
            Event::Unconformity(e) => e.undo(p, place, seq),
            Event::Dyke(e) => e.undo(p, place, previous_igneous, seq),
            Event::Plug(e) => e.undo(p, place, previous_igneous, seq),
            Event::Tilt(e) => e.undo(p),
            Event::Strain(e) => e.undo(p),
            Event::Stop => Step::Moved(p),
            Event::Windowed(e) => e.undo(p, place, previous_igneous, seq),
        }
    }
}
