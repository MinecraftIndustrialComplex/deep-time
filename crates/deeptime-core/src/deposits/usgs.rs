// SPDX-License-Identifier: LGPL-3.0-or-later
//! The public-domain USGS grade–tonnage tables the deposit types' sizing comes from
//! (`crates/deeptime-core/data/usgs/*.json`, digitised in P1-14 with a source, a location —
//! table or figure and page — and a method for every number). A value whose method is
//! `placeholder` or `not found` is not a USGS number; the deposit types mark their own placeholders
//! in `notes`, and a test checks that every non-placeholder value a type uses equals its table's.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;

/// The tables, by id.
pub const TABLES: &[(&str, &str)] = &[
    (
        "porphyry_cu",
        include_str!("../../data/usgs/porphyry_cu.json"),
    ),
    (
        "supergene_cu",
        include_str!("../../data/usgs/supergene_cu.json"),
    ),
    ("placer_au", include_str!("../../data/usgs/placer_au.json")),
    ("bif_fe", include_str!("../../data/usgs/bif_fe.json")),
    ("coal", include_str!("../../data/usgs/coal.json")),
    ("mvt_zn_pb", include_str!("../../data/usgs/mvt_zn_pb.json")),
    (
        "sedex_zn_pb",
        include_str!("../../data/usgs/sedex_zn_pb.json"),
    ),
    ("petroleum", include_str!("../../data/usgs/petroleum.json")),
];

/// One sourced number (or percentile set).
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Sourced {
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub p90: Option<f64>,
    #[serde(default)]
    pub p50: Option<f64>,
    #[serde(default)]
    pub p10: Option<f64>,
    #[serde(default)]
    pub n_reported: Option<f64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default, rename = "where")]
    pub location: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
}

impl Sourced {
    /// True when the numbers are the USGS's (not a placeholder, not missing).
    pub fn is_usgs(&self) -> bool {
        !matches!(
            self.method.as_deref(),
            None | Some("placeholder") | Some("not found") | Some("not applicable")
        )
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Correlation {
    pub a: String,
    pub b: String,
    pub r: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceRef {
    pub key: String,
    pub citation: String,
    #[serde(default)]
    pub url: Option<String>,
}

/// One table.
#[derive(Clone, Debug, Deserialize)]
pub struct Table {
    pub id: String,
    pub title: String,
    pub sources: Vec<SourceRef>,
    #[serde(default)]
    pub n_deposits: Option<f64>,
    pub tonnage_mt: Sourced,
    #[serde(default)]
    pub grades: BTreeMap<String, Sourced>,
    #[serde(default)]
    pub correlations: Vec<Correlation>,
    pub ore_density_t_m3: Sourced,
    pub deposits_per_1e5_km2: Sourced,
    pub min_separation_m: Sourced,
    #[serde(default)]
    pub extra: BTreeMap<String, Value>,
}

impl Table {
    /// An `extra` entry's `value`.
    pub fn extra_value(&self, key: &str) -> Option<f64> {
        self.extra.get(key)?.get("value")?.as_f64()
    }
}

/// Parses every table.
pub fn tables() -> Result<Vec<Table>, String> {
    TABLES
        .iter()
        .map(|(id, text)| {
            let t: Table = serde_json::from_str(text).map_err(|e| format!("{id}.json: {e}"))?;
            if t.id != *id {
                return Err(format!("{id}.json says id {}", t.id));
            }
            Ok(t)
        })
        .collect()
}

/// The table `id`.
pub fn table(id: &str) -> Option<Table> {
    tables().ok()?.into_iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deposits::types::builtin;

    #[test]
    fn tables_parse_and_cite_their_sources() {
        for t in tables().unwrap() {
            let keys: Vec<&str> = t.sources.iter().map(|s| s.key.as_str()).collect();
            let check = |what: &str, s: &Sourced| {
                if let Some(src) = &s.source {
                    for part in src
                        .split([',', ';', '+'])
                        .map(str::trim)
                        .filter(|p| !p.is_empty())
                    {
                        assert!(
                            keys.contains(&part),
                            "{}: {what} cites unknown source {part}",
                            t.id
                        );
                    }
                }
                if s.is_usgs() {
                    assert!(s.location.is_some(), "{}: {what} has no location", t.id);
                }
            };
            check("tonnage_mt", &t.tonnage_mt);
            for (k, g) in &t.grades {
                check(k, g);
            }
            check("ore_density_t_m3", &t.ore_density_t_m3);
            check("deposits_per_1e5_km2", &t.deposits_per_1e5_km2);
            check("min_separation_m", &t.min_separation_m);
        }
    }

    /// Every USGS number a built-in type uses equals its table's (placeholders excepted).
    #[test]
    fn types_use_their_tables() {
        let close = |a: f64, b: f64| (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0);
        for ty in builtin().unwrap() {
            let s = &ty.sizing;
            let t = table(&s.grade_tonnage).unwrap_or_else(|| panic!("{}: no table", ty.id));
            if t.tonnage_mt.is_usgs() {
                assert!(
                    close(s.tonnage_mt.p90, t.tonnage_mt.p90.unwrap()),
                    "{} p90",
                    ty.id
                );
                assert!(
                    close(s.tonnage_mt.p50, t.tonnage_mt.p50.unwrap()),
                    "{} p50",
                    ty.id
                );
                assert!(
                    close(s.tonnage_mt.p10, t.tonnage_mt.p10.unwrap()),
                    "{} p10",
                    ty.id
                );
            }
            for (k, g) in &s.grades {
                let Some(tg) = t.grades.get(k).filter(|x| x.is_usgs()) else {
                    continue;
                };
                assert!(close(g.p50, tg.p50.unwrap_or(0.0)), "{} {k} p50", ty.id);
                assert!(close(g.p10, tg.p10.unwrap_or(0.0)), "{} {k} p10", ty.id);
                assert!(
                    close(g.p90.unwrap_or(0.0), tg.p90.unwrap_or(0.0)),
                    "{} {k} p90",
                    ty.id
                );
                if let (Some(n), Some(total)) = (tg.n_reported, t.n_deposits) {
                    assert!(
                        (g.presence - n / total).abs() < 0.005,
                        "{} {k} presence",
                        ty.id
                    );
                }
            }
            for (a, b, r) in &s.correlations {
                let found = t
                    .correlations
                    .iter()
                    .find(|c| (c.a == *a && c.b == *b) || (c.a == *b && c.b == *a));
                if let Some(c) = found {
                    assert!(close(*r, c.r), "{} {a}–{b}", ty.id);
                }
            }
            if t.ore_density_t_m3.is_usgs() {
                assert!(
                    close(s.ore_density_t_m3, t.ore_density_t_m3.value.unwrap()),
                    "{} density",
                    ty.id
                );
            }
            if t.deposits_per_1e5_km2.is_usgs() {
                assert!(
                    close(
                        s.density_per_1e5km2.p50,
                        t.deposits_per_1e5_km2.p50.unwrap()
                    ),
                    "{} deposit density",
                    ty.id
                );
            }
            if t.min_separation_m.is_usgs() {
                assert!(
                    close(s.min_separation_m, t.min_separation_m.value.unwrap()),
                    "{} separation",
                    ty.id
                );
            }
        }
    }
}
