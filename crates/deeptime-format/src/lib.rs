// SPDX-License-Identifier: LGPL-3.0-or-later
//! The Deep Time world-data format: the on-disk contract between the Rust core and the Java
//! adapter (plan §5.7). No simulation, no Minecraft, no JNI.

/// The world-data format version this crate reads and writes (plan §5.7.2). Its one source is the
/// workspace `Cargo.toml` (`[workspace.metadata.deeptime] format_version`), read by `build.rs`;
/// Gradle reads the same line for `natives.json`.
pub const FORMAT_VERSION: &str = env!("DEEPTIME_FORMAT_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn format_version_is_pre_1_0() {
        assert!(super::FORMAT_VERSION.starts_with("0."));
    }

    /// The constant is the Cargo.toml value, verbatim: there is no second copy to drift.
    #[test]
    fn format_version_matches_workspace_metadata() {
        let toml = include_str!("../../../Cargo.toml");
        let line = toml
            .lines()
            .skip_while(|l| l.trim() != "[workspace.metadata.deeptime]")
            .find(|l| l.trim_start().starts_with("format_version"))
            .expect("format_version line");
        assert!(
            line.contains(&format!("\"{}\"", super::FORMAT_VERSION)),
            "{line}"
        );
    }
}
