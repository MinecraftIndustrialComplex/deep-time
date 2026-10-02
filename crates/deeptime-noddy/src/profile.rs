// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Ported from Noddy (Loop3D/noddyverse, commit 6a21b23, MIT License, Copyright (c) 2021 Loop3D):
// `src/proflib.c` (`calcProfile`) and the profile constants of `src/nodStruc.h`. See NOTICE.md.
//! Fold and curved-fault profiles: one period of a shape, read at a phase in radians.
//!
//! Noddy keeps a profile as a table of [`X_RESOLUTION`] samples over one period, in units of
//! 1/[`Y_RESOLUTION`], and interpolates linearly ([`Profile::Table`], the original's
//! `calcProfile`). [`Profile::Sine`] is the exact sine the default table approximates.

/// Samples over one period (`X_PROFILE_RESOLUTION`).
pub const X_RESOLUTION: usize = 628; // ledger: DEF-068
/// The table's unit: a stored 100 is a profile value of 1 (`Y_PROFILE_RESOLUTION`).
pub const Y_RESOLUTION: f64 = 100.0; // ledger: DEF-068

/// One period of a fold's (or a curved fault's) shape.
#[derive(Clone, Debug, PartialEq)]
pub enum Profile {
    /// sin(phase), evaluated exactly.
    Sine,
    /// Noddy's sampled profile: at least [`X_RESOLUTION`] values in units of 1/100, sample i at
    /// phase 2π·i/628, linear between samples. Shared, so events can be cloned cheaply.
    Table(std::sync::Arc<[f32]>),
}

impl Profile {
    /// The sine as Noddy's table: `100·sin(2π·i/628)` rounded to `f32`.
    pub fn sine_table() -> Profile {
        Profile::Table(
            (0..=X_RESOLUTION)
                .map(|i| {
                    (Y_RESOLUTION * (std::f64::consts::TAU * i as f64 / X_RESOLUTION as f64).sin())
                        as f32
                })
                .collect(),
        )
    }

    /// A table from any shape `f(phase)`, phase in [0, 2π).
    pub fn sampled(f: impl Fn(f64) -> f64) -> Profile {
        Profile::Table(
            (0..=X_RESOLUTION)
                .map(|i| {
                    (Y_RESOLUTION * f(std::f64::consts::TAU * i as f64 / X_RESOLUTION as f64))
                        as f32
                })
                .collect(),
        )
    }

    /// The profile at `position` radians (`calcProfile`).
    #[inline]
    pub fn at(&self, position: f64) -> f64 {
        match self {
            Profile::Sine => position.sin(),
            Profile::Table(t) => calc_profile(t, position),
        }
    }
}

/// `calcProfile`: linear interpolation in the table, the original's index arithmetic kept
/// (a negative position reads the table backwards from its end).
pub fn calc_profile(table: &[f32], position: f64) -> f64 {
    let res = X_RESOLUTION as i64;
    let (percent_diff, mut index1, mut index2);
    if position < 0.0 {
        let position = position.abs();
        let d_index = (position / 6.2831853) * (X_RESOLUTION as f64);
        percent_diff = d_index - d_index.floor();
        index1 = res - (d_index.floor() as i64) % res;
        index2 = res - (d_index.ceil() as i64) % res;
    } else {
        let d_index = (position / 6.2831853) * (X_RESOLUTION as f64);
        percent_diff = d_index - d_index.floor();
        index1 = (d_index.floor() as i64) % res;
        index2 = (d_index.ceil() as i64) % res;
    }
    index1 = index1.clamp(0, res - 1);
    index2 = index2.clamp(0, res - 1);
    let get = |i: i64| f64::from(table.get(i as usize).copied().unwrap_or(0.0)) / Y_RESOLUTION;
    let value1 = get(index1);
    let value2 = get(index2);
    let diff = value2 - value1;
    value1 + percent_diff * diff
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sine_table_follows_the_sine() {
        let t = Profile::sine_table();
        let mut worst = 0.0f64;
        for i in 0..5000 {
            let x = i as f64 * 0.0031;
            worst = worst.max((t.at(x) - Profile::Sine.at(x)).abs());
        }
        // 628 samples a period: the table's step is 0.01 rad, and 6.2831853 is not quite 2π.
        assert!(worst < 2e-4, "{worst}");
    }

    #[test]
    fn negative_positions_read_the_table_backwards() {
        let t = Profile::sine_table();
        for i in 1..600 {
            let x = i as f64 * 0.01;
            // Away from the wrap the table is odd like the sine it holds, to within a sample.
            assert!((t.at(-x) + t.at(x)).abs() < 0.03, "{x}");
        }
    }
}
