// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime-inspect`: renders Minecraft region files for review (plan §9.7). A dev tool; never
//! linked into the mod.

fn main() {
    println!("deeptime-inspect {}", env!("CARGO_PKG_VERSION"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_nonempty() {
        assert!(!env!("CARGO_PKG_VERSION").is_empty());
    }
}
