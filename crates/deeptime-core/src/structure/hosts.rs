// SPDX-License-Identifier: LGPL-3.0-or-later
//! Host-rock fractions for the deposit engine (`deeptime:host/<class>`, 0–1; P1-14's field
//! interface): per L1 cell and frame, the share of the ore window — the top `host_window_m` of the
//! column (`columns.rs`), plutons included — each class takes. Published for the classes some
//! deposit type names as a host (its `hosts.any_of` with groups expanded, and `host_bias`), since
//! only those change a favourability (`deposits::rules`: a type's host score is
//! min(1, Σ w_c · frac_c); a frame without host fields means "no lithology opinion").

use std::collections::BTreeSet;

use crate::deposits::types::DepositType;
use crate::surface::lithology::index;
use crate::synth::EVERY_CLASS;

use super::columns::{Source, Unit};

/// The classes `types` name as hosts, sorted by class index.
pub fn host_classes(types: &[DepositType]) -> Vec<u16> {
    let mut out = BTreeSet::new();
    for t in types {
        if let Ok(w) = t.hosts.weights() {
            for name in w.keys() {
                if EVERY_CLASS.contains(&name.as_str()) {
                    out.insert(index(name));
                }
            }
        }
    }
    out.into_iter().collect()
}

/// The field name of class `k`'s fraction.
pub fn field_name(k: u16) -> String {
    format!(
        "{}{}",
        crate::deposits::fields::HOST_PREFIX,
        EVERY_CLASS[usize::from(k)]
    )
}

/// Adds each unit's share of the window [0, `window_m`] to `out[slot[class]]` (slot `usize::MAX`
/// for classes not published).
pub fn fractions(units: &[Unit], window_m: f64, slot: &[usize], out: &mut [f32]) {
    fractions_inherited(units, window_m, slot, None, out);
}

/// [`fractions`], with `inherited = Some((s, share))` giving `share` of every basement unit's part
/// to slot `s` instead of its own class: banded iron formation inherited in a craton's basement
/// (`deeptime:inherited_bif`, `deposits::clocks`) is the BIF type's host where the basement is
/// within the window.
pub fn fractions_inherited(
    units: &[Unit],
    window_m: f64,
    slot: &[usize],
    inherited: Option<(usize, f32)>,
    out: &mut [f32],
) {
    let mut top = 0.0f64;
    for u in units {
        if top >= window_m {
            break;
        }
        let bottom = top + f64::from(u.thickness_m);
        let part = ((bottom.min(window_m) - top) / window_m) as f32;
        let s = slot[usize::from(u.class)];
        let moved = match (inherited, u.source) {
            (Some((to, share)), Source::Basement(_)) if part > 0.0 => {
                out[to] += part * share;
                part * share
            }
            _ => 0.0,
        };
        if s != usize::MAX && part > 0.0 {
            out[s] += part - moved;
        }
        top = bottom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_fractions_cover_the_window() {
        let types = crate::deposits::types::builtin().unwrap();
        let classes = host_classes(&types);
        for name in [
            "granodiorite",
            "dolostone",
            "limestone",
            "black_shale",
            "sandstone",
            "coal",
            "banded_iron_formation",
            "gravel",
            "serpentinite",
            "andesite",
        ] {
            assert!(classes.contains(&index(name)), "{name}");
        }
        let mut slot = vec![usize::MAX; EVERY_CLASS.len()];
        for (s, &k) in classes.iter().enumerate() {
            slot[usize::from(k)] = s;
        }
        let unit = |name: &str, th: f32| Unit {
            class: index(name),
            thickness_m: th,
            source: Source::Log,
            key: [0, 1],
        };
        let units = [
            unit("sandstone", 500.0),
            unit("halite", 500.0),
            unit("limestone", 800.0),
            unit("granite", 5000.0),
        ];
        let mut out = vec![0.0f32; classes.len()];
        fractions(&units, 2000.0, &slot, &mut out);
        let at = |name: &str| out[slot[usize::from(index(name))]];
        assert!((at("sandstone") - 0.25).abs() < 1e-6);
        assert!((at("limestone") - 0.4).abs() < 1e-6);
        assert!((at("granite") - 0.1).abs() < 1e-6);
        assert!(out.iter().sum::<f32>() <= 1.0 + 1e-6);
        assert_eq!(field_name(index("dolostone")), "deeptime:host/dolostone");
    }

    /// Inherited BIF takes its share of the basement's part of the window, and nothing else.
    #[test]
    fn inherited_bif_is_hosted_by_the_basement() {
        let types = crate::deposits::types::builtin().unwrap();
        let classes = host_classes(&types);
        let mut slot = vec![usize::MAX; EVERY_CLASS.len()];
        for (s, &k) in classes.iter().enumerate() {
            slot[usize::from(k)] = s;
        }
        let bif = slot[usize::from(index("banded_iron_formation"))];
        let units = [
            Unit {
                class: index("sandstone"),
                thickness_m: 500.0,
                source: Source::Log,
                key: [0, 1],
            },
            Unit {
                class: index("granite"),
                thickness_m: 5000.0,
                source: Source::Basement(0),
                key: [2, 3],
            },
        ];
        let mut out = vec![0.0f32; classes.len()];
        fractions_inherited(&units, 2000.0, &slot, Some((bif, 0.5)), &mut out);
        let at = |name: &str| out[slot[usize::from(index(name))]];
        assert!((at("sandstone") - 0.25).abs() < 1e-6);
        assert!((at("banded_iron_formation") - 0.375).abs() < 1e-6);
        assert!((at("granite") - 0.375).abs() < 1e-6);
        let mut plain = vec![0.0f32; classes.len()];
        fractions(&units, 2000.0, &slot, &mut plain);
        assert_eq!(plain[bif], 0.0);
    }
}
