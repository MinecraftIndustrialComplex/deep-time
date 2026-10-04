// SPDX-License-Identifier: LGPL-3.0-or-later
//! PROTOTYPE, not shipped: a quincuncial world (Peirce quincuncial or its Guyou aspect) whose edges
//! are glued seamlessly, for the Immersive Portals prototype of `docs/design/seamless-wrapping.md`
//! (Ben, decisions log 2026-10-03).
//!
//! - [`elliptic`]: Jacobi sn, cn, dn and Carlson's R_F, real and complex.
//! - [`peirce`]: the conformal square (forward, inverse, scale, singular points).
//! - [`world`]: the two aspects as Minecraft worlds (block frame, canonical domain, seams).
//! - [`glue`]: the gluing table and the portal placements.
//! - [`resample`]: a baked world-data folder resampled onto a quincuncial world.
//! - [`stats`]: the measurements behind the design doc.
//!
//! Nothing in the mod, the core or the CLI depends on this crate. If Ben picks a quincuncial world,
//! `peirce` and `world` fold into `deeptime_format::projection`.

pub mod complex;
pub mod elliptic;
pub mod glue;
pub mod peirce;
pub mod resample;
pub mod stats;
pub mod world;
