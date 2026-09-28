// SPDX-License-Identifier: LGPL-3.0-or-later
//! Reads the world-data format version from its one source, the workspace `Cargo.toml`
//! (`[workspace.metadata.deeptime] format_version`), so `FORMAT_VERSION` can never drift from
//! what Gradle writes into `natives.json` (plan §6.5).

use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let workspace_toml = manifest_dir.join("../../Cargo.toml");
    println!("cargo:rerun-if-changed={}", workspace_toml.display());
    let text = std::fs::read_to_string(&workspace_toml)
        .unwrap_or_else(|e| panic!("reading {}: {e}", workspace_toml.display()));
    let version = format_version(&text).unwrap_or_else(|| {
        panic!(
            "{} has no format_version under [workspace.metadata.deeptime]",
            workspace_toml.display()
        )
    });
    println!("cargo:rustc-env=DEEPTIME_FORMAT_VERSION={version}");
}

/// The `format_version = "…"` line inside `[workspace.metadata.deeptime]`. A line scan, not a TOML
/// parser: the build script stays dependency-free, and Gradle reads the file the same way.
fn format_version(text: &str) -> Option<String> {
    let mut in_section = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_section = line == "[workspace.metadata.deeptime]";
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some(rest) = line.strip_prefix("format_version") {
            let value = rest.trim_start().strip_prefix('=')?.trim();
            let value = value.strip_prefix('"')?;
            return Some(value[..value.find('"')?].to_string());
        }
    }
    None
}
