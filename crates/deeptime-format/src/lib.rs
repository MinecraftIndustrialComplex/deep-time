// SPDX-License-Identifier: LGPL-3.0-or-later
//! The Deep Time world-data format: the on-disk contract between the Rust core and the Java
//! adapter (plan §5.7). No simulation, no Minecraft, no JNI.
//!
//! The byte layout is specified in `FORMAT.md` next to this crate's `Cargo.toml`; the Java
//! reader (`com.minecraftindustrialcomplex.deeptime.worlddata`) implements the same document.

pub mod bodies;
pub mod container;
pub mod error;
pub mod events;
pub mod fsutil;
pub mod keyframes;
pub mod manifest;
pub mod ramps;
pub mod raster;
pub mod sites;
pub mod tilepack;

pub use bodies::{Body, BodyId, Shape, Zone};
pub use error::{FormatError, Result};
pub use events::{
    Event, EventId, EventIndex, EventLog, EventWriter, GEOLOGY_WRITER, ParamValue, Params, Place,
};
pub use keyframes::{KeyframeEntry, KeyframeImage, KeyframeIndex};
pub use manifest::{FileEntry, LayerEntry, Manifest, Scale, Writer};
pub use ramps::{Ramp, Ramps};
pub use raster::{Cell, Codec, DType, GridDesc, LayerDesc, LoadedLayer, Raster, RasterRead, TILE};
pub use sites::{Extraction, Product, Site, SiteId, SiteIndex, SiteKind};
pub use tilepack::{TilePackReader, TilePackWriter, Written, encode_tile_pack, write_tile_pack};

/// The world-data format version this crate reads and writes (plan §5.7.2). Its one source is the
/// workspace `Cargo.toml` (`[workspace.metadata.deeptime] format_version`), read by `build.rs`;
/// Gradle reads the same line for `natives.json`.
pub const FORMAT_VERSION: &str = env!("DEEPTIME_FORMAT_VERSION");

/// The minor part of [`FORMAT_VERSION`], written into every file header (plan §5.7.2).
pub const FORMAT_MINOR: u16 = 0;

#[cfg(test)]
mod tests {
    #[test]
    fn format_version_is_pre_1_0() {
        assert!(super::FORMAT_VERSION.starts_with("0."));
        assert_eq!(
            super::FORMAT_VERSION,
            format!("0.{}", super::FORMAT_MINOR),
            "FORMAT_MINOR must be FORMAT_VERSION's minor"
        );
    }

    /// The workspace `Cargo.toml` is the single source of the format version; this constant only
    /// mirrors it (Gradle and the native scripts read the TOML directly).
    #[test]
    fn format_version_matches_workspace() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../Cargo.toml");
        let toml = std::fs::read_to_string(path).expect("read the workspace Cargo.toml");
        let section = toml
            .split("[workspace.metadata.deeptime]")
            .nth(1)
            .expect("[workspace.metadata.deeptime] section");
        let section = section.split("\n[").next().unwrap_or(section);
        let value = section
            .lines()
            .map(str::trim)
            .find_map(|l| {
                let rest = l.strip_prefix("format_version")?.trim_start();
                let rest = rest.strip_prefix('=')?.trim();
                Some(rest.trim_matches('"').to_owned())
            })
            .expect("format_version in [workspace.metadata.deeptime]");
        assert_eq!(value, super::FORMAT_VERSION);
    }
}
