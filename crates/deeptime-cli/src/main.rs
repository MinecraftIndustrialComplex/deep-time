// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime`: the standalone driver for every simulation stage (plan §9.3).

fn main() {
    println!(
        "deeptime {} (format {})",
        deeptime_core::VERSION,
        deeptime_core::FORMAT_VERSION
    );
    let _ = deeptime_viewer::FORMAT_VERSION;
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_nonempty() {
        assert!(!deeptime_core::VERSION.is_empty());
    }
}
