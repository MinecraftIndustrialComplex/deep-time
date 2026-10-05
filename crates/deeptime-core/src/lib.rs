// SPDX-License-Identifier: LGPL-3.0-or-later
//! The Deep Time simulation core (plan §8). Depends on the format crate only; no Minecraft and
//! no JNI types, so the civ module and future planets can reuse it as a plain library.

// The counting allocator behind the memory cap (plan §5.7.5).
pub mod alloc;

// The stage runner and its persistence (P1-01).
pub mod checkpoint;
pub mod dummy;

// Deposits (P1-14).
pub mod deposits;

pub mod keyframes;
pub mod stage;
pub mod world;

pub mod error;
pub mod log;
pub mod params;
pub mod plates;
pub mod provinces;
pub mod satellite;
pub mod session;
pub mod sphere;
pub mod water_view;

// Deep-time surface processes on L1 (P1-04).
pub mod surface;

// The parametric climate on L1 (P1-05).
pub mod climate;

// Column strata, metamorphism, plutons and the horizon bake (P1-07).
pub mod structure;

// The bake: a finished planet run as the chunk-time layers the game reads (P1-08).
pub mod bake;

// Rivers and lakes on the finished terrain (P1-22).
pub mod hydro;

// What the game puts in the open space below ground: the lava floor and lava lakes.
pub mod underground;

// Presets, share codes and the synthetic terrains of the placeholder presets (P1-13).
pub mod preset;
pub mod share_code;
pub mod synth;

pub use deeptime_format::FORMAT_VERSION;
pub use error::CoreError;

/// The crate version, which is the Cargo workspace version (the mod's version too).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_workspace_version() {
        assert_eq!(super::VERSION, "0.0.1");
    }
}
