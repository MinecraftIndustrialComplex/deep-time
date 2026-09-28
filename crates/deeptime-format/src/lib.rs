// SPDX-License-Identifier: LGPL-3.0-or-later
//! The Deep Time world-data format: the on-disk contract between the Rust core and the Java
//! adapter (plan §5.7). No simulation, no Minecraft, no JNI.

/// The world-data format version this crate reads and writes (plan §5.7.2).
pub const FORMAT_VERSION: &str = "0.0";

#[cfg(test)]
mod tests {
    #[test]
    fn format_version_is_pre_1_0() {
        assert!(super::FORMAT_VERSION.starts_with("0."));
    }
}
