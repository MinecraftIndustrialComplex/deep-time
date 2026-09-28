// SPDX-License-Identifier: LGPL-3.0-or-later
//! Writes static HTML viewer bundles for world data and stage checkpoints (plan §9.3).

pub use deeptime_format::FORMAT_VERSION;

#[cfg(test)]
mod tests {
    #[test]
    fn sees_format() {
        assert_eq!(super::FORMAT_VERSION, "0.0");
    }
}
