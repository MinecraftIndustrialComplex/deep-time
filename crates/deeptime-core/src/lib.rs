// SPDX-License-Identifier: LGPL-3.0-or-later
//! The Deep Time simulation core (plan §8). Depends on the format crate only; no Minecraft and
//! no JNI types, so the civ module and future planets can reuse it as a plain library.

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
pub mod session;
pub mod sphere;

// Deep-time surface processes on L1 (P1-04).
pub mod surface;

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
