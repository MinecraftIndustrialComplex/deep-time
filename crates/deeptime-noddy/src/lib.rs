// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Ported from Noddy as published in Loop3D/noddyverse (https://github.com/Loop3D/noddyverse,
// commit 6a21b2374516f8506c19cabcbe3ead00be9cc45d), MIT License, Copyright (c) 2021 Loop3D;
// the original code is Mark Jessell's (Imperial College 1981-82, Monash University 1994, UWA),
// `matrix.c` (c) Ewen Christopher and Monash University 1994. See NOTICE.md.
//! Kinematic structural geology by inverse kinematics: a Rust port of Noddy's event engine
//! (plan §8.5, work item P2-01).
//!
//! A [`History`] is a base stratigraphy followed by an ordered list of [`Event`]s: folds, faults
//! (and shear zones and kinks), unconformities, dykes, plugs, tilts and homogeneous strain. The
//! rock at a point of the present-day model is found by "unravelling" the history: the events are
//! undone youngest first ([`History::restore`]), each moving the point to where it was before the
//! event (or deciding that the point belongs to the event's own rock: above an unconformity,
//! inside a dyke or a plug), and the layer of the base stratigraphy (or of the unconformity's
//! cover) that holds the restored point is the rock ([`History::rock_at`]). Nothing is stored
//! between points, so any point of an unbounded Cartesian space can be asked, in any order and at
//! any resolution (Jessell & Valenta 1996; Jessell et al. 2022, Earth Syst. Sci. Data 14:381-392;
//! Wellmann et al. 2016, Geosci. Model Dev. 9:1019-1035).
//!
//! # What is ported, and from where
//!
//! | Here | Noddy (noddyverse `src/`) |
//! |---|---|
//! | [`events`]: `Fold`, `Fault`, `Unconformity`, `Dyke`, `Plug`, `Tilt`, `Strain` and their `undo` | `unEvents.c`: `unfold`, `unfault` (faults, shear zones, kinks), `unplan`, `undyke`, `unplug`, `unrot`; the elliptical and curved faults' helpers of `ellFault.c` |
//! | [`History::restore`] | `unEvents.c` `reverseEvents` |
//! | [`taste`]: [`Sequence`], [`taste::taste`], `coincide`, `line_coincide`, `cypher_diff`, `last_diff` | `taste.c` |
//! | [`History::rock_at`], [`Stratigraphy::layer_at`] | `which.c` `whichRock`, `whichLayer` |
//! | [`orient`]: `convrt`, `rotset`, `irotate`, `paxis`, `plane` | `utils.c` (the helpers the three files need) |
//! | [`profile`] | `proflib.c` `calcProfile` |
//! | [`matrix`] | `matrix.c` |
//!
//! Not ported: surfaces read from DXF files (faults and unconformities are planes, or the curved
//! and ring geometries Noddy computes), imported voxel blocks (`unImport`), Biot folds (`unbiot`,
//! unreachable in the original: its call is commented out and its solver is a no-op), `unloft` and
//! `unbury` (dead code in the original), alteration zones, and everything geophysical
//! (`taste.c`'s `Remfind`, `Anifind` and `tasteRemanent`, which orient remanence and anisotropy).
//!
//! # Fidelity
//!
//! The arithmetic follows the original expression by expression, including its rounded constants
//! (π as 3.1415927 in the fault code, 2π as 6.2831853 and ln 2 as 0.693147 in the fold code), so
//! that `tests/reference.rs` can compare displacement fields with the reference C on a lattice:
//! the fixture is written by a harness that calls the original `unEvents.c`, `taste.c`, `utils.c`,
//! `ellFault.c` and `proflib.c` (`tools/validation/noddy/`). Two deliberate differences, both
//! where the original reads out of range: `rotset`'s second clamp tests the wrong variable (kept,
//! but an `acos` argument outside [−1, 1] is clamped instead of giving NaN), and `matrix.c`'s
//! `rotateX` writes `[1][1]` twice and never `[1][2]` (fixed).
//!
//! # Conventions
//!
//! Noddy's: x east, y north, z up, in any length unit. A plane's attitude is its dip (degrees
//! from horizontal) and dip direction (degrees clockwise from north); a line's its plunge and
//! plunge direction; a pitch is measured in the plane. A point is `[x, y, z]`.
//!
//! Everything is deterministic: plain `f64` arithmetic and `libm` functions, no randomness, no
//! global state (the original keeps `convrt`'s pole and axis in globals for `rotset` and `plane`;
//! here [`orient::Attitude`] carries them).

// The original's rounded constants are kept on purpose (see "Fidelity" above), and the matrix
// code keeps its index loops, whose order of summation is part of the fidelity.
#![allow(
    clippy::approx_constant,
    clippy::excessive_precision,
    clippy::needless_range_loop
)]

pub mod events;
pub mod history;
pub mod matrix;
pub mod orient;
pub mod profile;
pub mod taste;

pub use events::{
    Dyke, DykeKind, Event, Fault, FaultGeometry, FaultKind, Fold, Movement, Plug, PlugShape,
    Strain, Tilt, Unconformity,
};
pub use history::{History, Restored, Rock, Stratigraphy};
pub use profile::Profile;
pub use taste::{Code, Sequence};

/// A point or vector: `[x, y, z]`, x east, y north, z up.
pub type V3 = [f64; 3];
/// A 3 × 3 matrix, `m[row][column]`. Noddy multiplies row vectors: `p' = p · M` ([`orient::vm`]).
pub type M3 = [[f64; 3]; 3];

/// The commit of Loop3D/noddyverse the port was made from.
pub const NODDYVERSE_COMMIT: &str = "6a21b2374516f8506c19cabcbe3ead00be9cc45d";

/// `unEvents.c`'s tolerance for "zero": a wavelength, cycle or cylindrical index below it is off.
pub const TOLERANCE: f64 = 0.001;
