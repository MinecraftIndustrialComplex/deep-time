// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime-inspect`: renders and checks Minecraft region files for review (plan §9.7). A dev
//! tool; never linked into the mod.
//!
//! It reads Java Edition 1.21.1 worlds: `region/r.<x>.<z>.mca` ([`region`]), chunk NBT
//! ([`nbt`], [`chunk`]), and Deep Time's world data ([`worlddata`]), then draws top-down, rock,
//! ore, cross-section and seam images and compares the generated surface with the world data
//! ([`render`]). The surface a height check measures is defined in [`world`]; the columns it
//! leaves out in [`surface`].

pub mod area;
pub mod blocks;
pub mod chunk;
pub mod colors;
pub mod error;
pub mod hosts;
pub mod image;
pub mod nbt;
pub mod palette;
pub mod region;
pub mod render;
pub mod rivermap;
pub mod surface;
pub mod world;
pub mod worlddata;
pub mod zip;

pub use error::{Error, Result};
