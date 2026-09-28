// SPDX-License-Identifier: LGPL-3.0-or-later
//! Bakes the Rust target triple and the git commit into `version()` (plan §6.1):
//! `<semver>+<git sha>;format=<f>;target=<rust triple>`. The sha is informational; the loader
//! compares semver and format only (plan §6.5).

use std::process::Command;

fn main() {
    let target = std::env::var("TARGET").expect("TARGET");
    println!("cargo:rustc-env=DEEPTIME_TARGET={target}");

    println!("cargo:rerun-if-env-changed=DEEPTIME_GIT_SHA");
    let sha = std::env::var("DEEPTIME_GIT_SHA")
        .ok()
        .filter(|s| !s.is_empty())
        .or_else(|| git(&["rev-parse", "--short=7", "HEAD"]))
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=DEEPTIME_GIT_SHA={sha}");

    // Re-run when HEAD moves: HEAD itself (a branch switch) and the branch's ref file (a commit).
    for path in [
        git(&["rev-parse", "--git-path", "HEAD"]),
        git(&["symbolic-ref", "-q", "HEAD"])
            .and_then(|r| git(&["rev-parse", "--git-path", r.as_str()])),
    ]
    .into_iter()
    .flatten()
    {
        let path = std::path::Path::new(&path);
        if path.exists() {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    println!("cargo:rerun-if-changed=build.rs");
}

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    (!s.is_empty()).then_some(s)
}
