// SPDX-License-Identifier: LGPL-3.0-or-later
//! Favourability (plan §8.6 step 1): each deposit type's mineral-system components evaluated on
//! every cell of every frame, combined so a missing process zeroes the score (Hronsky & Groves
//! 2008's multiplicative ranking), gated by the secular clocks, and integrated over the history.
//!
//! - A **term** is `curve(field)^weight`, or the maximum of its `any_of` alternatives (fuzzy OR).
//! - A **component** (source, driver, pathway, trap) is the product of its terms; an empty
//!   component scores 1.
//! - The **host** score is `min(1, Σ w_c · frac_c / full_at)` over the `deeptime:host/<class>`
//!   fractions with the type's host weights and `hosts.full_at` (default 1; 1 when the history has
//!   no host fields: no lithology opinion).
//! - The **clock** score is the product of the type's clock gates.
//! - `F_t = combine(source, driver, pathway, trap) · host · clock`, with `combine` the product,
//!   the minimum or the geometric mean.
//! - **Integration** over frames: `F = 1 − Π_t (1 − F_t)^(Δt/τ)`, τ the type's `integration_myr`
//!   (a probabilistic OR in time: a long favourable window counts more than a short one, and
//!   `F_t = 1` for any Δt gives 1).
//! - The **limiting component** is the smallest of the six scores at the cell's best frame: the
//!   frame of highest `F_t`, or for a barren cell the frame of highest mean score — "which
//!   component limited barren neighbours" (plan §5.5; `ore_genesis_modeling.md` Q3).

use std::collections::BTreeMap;

use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use super::fields::{FieldFrame, HOST_PREFIX, Values};
use super::types::{Combine, DepositType, Term};

/// Scores quantised to u8 (0–255 ↔ 0–1) for the accumulators.
pub fn q8(x: f64) -> u8 {
    (x.clamp(0.0, 1.0) * 255.0).round() as u8
}

pub fn dq8(x: u8) -> f64 {
    f64::from(x) / 255.0
}

/// Per-cell accumulator of one type.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CellAcc {
    /// Σ (Δt/τ) · ln(1 − F_t); the integrated favourability is 1 − exp(this).
    pub log_survival: f32,
    /// Highest F_t seen.
    pub f_max: f32,
    /// The ranking key of the best frame (F_t, or a small multiple of the mean score).
    pub best_key: f32,
    /// Frame index of the best frame (u16::MAX before any).
    pub k_best: u16,
    /// Component scores at the best frame, quantised: source, driver, pathway, trap, host, clock.
    pub scores: [u8; 6],
    /// Host class index (into the type's host list) at the best frame, 255 for none.
    pub host: u8,
}

impl CellAcc {
    pub fn fresh() -> CellAcc {
        CellAcc {
            k_best: u16::MAX,
            host: 255,
            ..CellAcc::default()
        }
    }

    pub fn favourability(&self) -> f64 {
        1.0 - f64::from(self.log_survival).exp()
    }

    /// Index into `types::COMPONENTS` of the smallest score.
    pub fn limiting(&self) -> usize {
        let mut best = 0;
        for k in 1..6 {
            if self.scores[k] < self.scores[best] {
                best = k;
            }
        }
        best
    }
}

/// A type's rules resolved for fast evaluation.
pub struct Compiled {
    pub combine: Combine,
    pub components: [Vec<Term>; 4],
    pub gates: Vec<String>,
    /// (field name, weight) per host class, and the class names in the same order.
    pub hosts: Vec<(String, f64)>,
    pub host_classes: Vec<String>,
    /// The weighted host share at which the host score is 1 (`hosts.full_at`).
    pub host_full_at: f64,
    pub tau_myr: f64,
}

impl Compiled {
    pub fn new(t: &DepositType) -> Compiled {
        let weights = t.hosts.weights().unwrap_or_default();
        let hosts: Vec<(String, f64)> = weights
            .iter()
            .map(|(c, w)| (format!("{HOST_PREFIX}{c}"), *w))
            .collect();
        Compiled {
            combine: t.components.combine,
            components: [
                t.components.source.clone(),
                t.components.driver.clone(),
                t.components.pathway.clone(),
                t.components.trap.clone(),
            ],
            gates: t.clock_gates.clone(),
            host_classes: weights.keys().cloned().collect(),
            hosts,
            host_full_at: t.hosts.full_at,
            tau_myr: t.integration_myr,
        }
    }
}

/// A term with its field resolved in one frame.
enum Resolved<'a> {
    Leaf { values: Values<'a>, term: &'a Term },
    AnyOf(Vec<Resolved<'a>>),
}

impl Resolved<'_> {
    fn score(&self, cell: usize) -> f64 {
        match self {
            Resolved::Leaf { values, term } => {
                let m = term.curve_at(f64::from(values.at(cell)));
                if term.weight == 1.0 {
                    m
                } else {
                    m.powf(term.weight)
                }
            }
            Resolved::AnyOf(alts) => alts.iter().map(|a| a.score(cell)).fold(0.0, f64::max),
        }
    }
}

fn resolve<'a>(t: &'a Term, frame: &'a FieldFrame) -> Option<Resolved<'a>> {
    match &t.field {
        Some(f) => Some(Resolved::Leaf {
            values: frame.values(f)?,
            term: t,
        }),
        None => Some(Resolved::AnyOf(
            t.any_of
                .iter()
                .map(|a| resolve(a, frame))
                .collect::<Option<Vec<_>>>()?,
        )),
    }
}

/// One cell's scores in one frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Scores {
    /// source, driver, pathway, trap, host, clock.
    pub parts: [f64; 6],
    pub f: f64,
    pub host: u8,
}

/// Evaluates a compiled type over one frame.
pub struct FrameEval<'a> {
    rules: &'a Compiled,
    components: [Vec<Resolved<'a>>; 4],
    hosts: Option<Vec<(Values<'a>, f64)>>,
    clock: f64,
}

impl<'a> FrameEval<'a> {
    /// `None` when the frame lacks a field the rules read.
    pub fn new(rules: &'a Compiled, frame: &'a FieldFrame) -> Option<FrameEval<'a>> {
        let mut components: [Vec<Resolved<'a>>; 4] = Default::default();
        for (k, terms) in rules.components.iter().enumerate() {
            for t in terms {
                components[k].push(resolve(t, frame)?);
            }
        }
        let any_host = frame.fields.keys().any(|k| k.starts_with(HOST_PREFIX));
        let hosts = if any_host && !rules.hosts.is_empty() {
            Some(
                rules
                    .hosts
                    .iter()
                    .map(|(name, w)| (frame.values(name).unwrap_or(Values::Uniform(0.0)), *w))
                    .collect(),
            )
        } else {
            None
        };
        let mut clock = 1.0;
        for g in &rules.gates {
            clock *= f64::from(frame.values(g)?.at(0)).clamp(0.0, 1.0);
        }
        Some(FrameEval {
            rules,
            components,
            hosts,
            clock,
        })
    }

    pub fn at(&self, cell: usize) -> Scores {
        let mut parts = [1.0f64; 6];
        for (k, comp) in self.components.iter().enumerate() {
            parts[k] = comp.iter().map(|t| t.score(cell)).product();
        }
        let mut host = 255u8;
        if let Some(hosts) = &self.hosts {
            let mut sum = 0.0;
            let mut best = 0.0;
            for (k, (v, w)) in hosts.iter().enumerate() {
                let s = f64::from(v.at(cell)).max(0.0) * w;
                sum += s;
                if s > best {
                    best = s;
                    host = k as u8;
                }
            }
            parts[4] = (sum / self.rules.host_full_at).min(1.0);
        }
        parts[5] = self.clock;
        let core = match self.rules.combine {
            Combine::Product => parts[..4].iter().product(),
            Combine::Min => parts[..4].iter().copied().fold(1.0, f64::min),
            Combine::GeometricMean => parts[..4].iter().product::<f64>().powf(0.25),
        };
        Scores {
            parts,
            f: (core * parts[4] * parts[5]).clamp(0.0, 1.0),
            host,
        }
    }
}

/// Folds one frame into a type's accumulators (Rayon over cells, on the caller's pool).
pub fn accumulate(acc: &mut [CellAcc], eval: &FrameEval<'_>, k: u16, dt_myr: f64) {
    let tau = eval.rules.tau_myr;
    acc.par_iter_mut().enumerate().for_each(|(cell, a)| {
        let s = eval.at(cell);
        let f = s.f.min(0.999);
        a.log_survival += ((dt_myr / tau) * (1.0 - f).ln()) as f32;
        a.f_max = a.f_max.max(s.f as f32);
        let mean = s.parts.iter().sum::<f64>() / 6.0;
        let key = if s.f > 0.0 { s.f } else { 1.0e-6 * mean };
        if a.k_best == u16::MAX || key as f32 >= a.best_key {
            a.best_key = key as f32;
            a.k_best = k;
            a.scores = s.parts.map(q8);
            a.host = s.host;
        }
    });
}

/// The names a type reads that a frame lacks.
pub fn missing_names(rules: &Compiled, names: &[String]) -> Vec<String> {
    let mut want = std::collections::BTreeSet::new();
    for comp in &rules.components {
        for t in comp {
            t.fields(&mut want);
        }
    }
    want.extend(rules.gates.iter().cloned());
    want.into_iter()
        .filter(|n| !names.iter().any(|m| m == n))
        .collect()
}

/// Summary statistics a census reports per type.
pub fn stats(acc: &[CellAcc]) -> BTreeMap<&'static str, f64> {
    let n = acc.len().max(1) as f64;
    let mean = acc.iter().map(CellAcc::favourability).sum::<f64>() / n;
    let max = acc.iter().map(CellAcc::favourability).fold(0.0, f64::max);
    [("mean_f", mean), ("max_f", max)].into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deposits::types::{Components, builtin};

    fn frame(values: &[(&str, f32)], clocks: &[(&str, f32)]) -> FieldFrame {
        FieldFrame {
            t_years: 0.0,
            dt_years: 1.0e7,
            fields: values
                .iter()
                .map(|(k, v)| ((*k).to_owned(), vec![*v, 0.0]))
                .collect(),
            clocks: clocks.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect(),
        }
    }

    fn term(field: &str, curve: &[[f64; 2]]) -> Term {
        Term {
            field: Some(field.into()),
            curve: curve.to_vec(),
            weight: 1.0,
            any_of: vec![],
            note: None,
        }
    }

    /// A missing process zeroes the score and is named as the limiting component.
    #[test]
    fn product_zeroes_and_names_the_limit() {
        let mut t = builtin().unwrap().remove(0);
        t.components = Components {
            combine: Combine::Product,
            source: vec![term("deeptime:a", &[[0.0, 0.0], [1.0, 1.0]])],
            driver: vec![term("deeptime:b", &[[0.0, 0.0], [1.0, 1.0]])],
            pathway: vec![],
            trap: vec![],
        };
        t.clock_gates = vec!["deeptime:clock/x".into()];
        t.hosts = Default::default();
        let rules = Compiled::new(&t);
        let f = frame(
            &[("deeptime:a", 0.8), ("deeptime:b", 0.5)],
            &[("deeptime:clock/x", 1.0)],
        );
        let e = FrameEval::new(&rules, &f).unwrap();
        let s = e.at(0);
        assert!((s.f - 0.4).abs() < 1e-6);
        let s0 = e.at(1);
        assert_eq!(s0.f, 0.0);
        let mut acc = vec![CellAcc::fresh(); 2];
        accumulate(&mut acc, &e, 0, 10.0);
        assert_eq!(acc[1].favourability(), 0.0);
        assert_eq!(super::super::types::COMPONENTS[acc[1].limiting()], "source");
        assert_eq!(super::super::types::COMPONENTS[acc[0].limiting()], "driver");
        // Integration over τ: two frames of F = 0.4 for τ each give 1 − 0.6².
        accumulate(&mut acc, &e, 1, 10.0);
        assert!((acc[0].favourability() - (1.0 - 0.36)).abs() < 1e-6);
        // A closed clock zeroes everything.
        let shut = frame(
            &[("deeptime:a", 1.0), ("deeptime:b", 1.0)],
            &[("deeptime:clock/x", 0.0)],
        );
        assert_eq!(FrameEval::new(&rules, &shut).unwrap().at(0).f, 0.0);
        // A frame without the field cannot evaluate the type.
        assert!(FrameEval::new(&rules, &frame(&[("deeptime:a", 1.0)], &[])).is_none());
        assert_eq!(
            missing_names(&rules, &["deeptime:a".to_string()]),
            vec!["deeptime:b".to_string(), "deeptime:clock/x".to_string()]
        );
    }

    #[test]
    fn any_of_takes_the_best_alternative_and_hosts_weight() {
        let mut t = builtin().unwrap().remove(0);
        t.components = Components {
            combine: Combine::Min,
            source: vec![Term {
                field: None,
                curve: vec![],
                weight: 1.0,
                any_of: vec![
                    term("deeptime:a", &[[0.0, 0.0], [1.0, 1.0]]),
                    term("deeptime:b", &[[0.0, 0.0], [1.0, 1.0]]),
                ],
                note: None,
            }],
            ..Components::default()
        };
        t.clock_gates.clear();
        t.hosts.any_of = vec!["dolostone".into()];
        t.hosts.host_bias = [("serpentinite".to_string(), 2.0)].into_iter().collect();
        let rules = Compiled::new(&t);
        let f = frame(
            &[
                ("deeptime:a", 0.3),
                ("deeptime:b", 0.9),
                ("deeptime:host/dolostone", 0.2),
                ("deeptime:host/serpentinite", 0.3),
            ],
            &[],
        );
        let s = FrameEval::new(&rules, &f).unwrap().at(0);
        assert!((s.parts[0] - 0.9).abs() < 1e-6);
        assert!((s.parts[4] - 0.8).abs() < 1e-6, "0.2·1 + 0.3·2");
        assert_eq!(rules.host_classes[s.host as usize], "serpentinite");
        assert!((s.f - 0.9 * 0.8).abs() < 1e-6);
    }
}
