// SPDX-License-Identifier: LGPL-3.0-or-later
//! What a geologist would object to in a rock column, as a check (`deeptime strata-check`, and
//! the property tests on baked columns). Phase 1 has no thrusts and no folds (P2-01), so a column
//! is a conformable pile cut by unconformities, and three things cannot happen in it:
//!
//! 1. **A metamorphic grade inversion.** Every regional-metamorphic class records the least
//!    temperature it formed at ([`metamorphism::formation_temp_c`]), and every class with a
//!    metamorphic path records the temperature it never reached (its path's first step: shale
//!    would be slate above 250 °C). Rock lower in a pile was at least as hot as the rock above it
//!    whenever both were there, so a horizon whose class was never as hot as a horizon above it
//!    formed at is an inversion: schist (≥ 450 °C) over shale (< 250 °C). The reverse — shale over
//!    schist — is cover on basement across an unconformity and is fine. Intrusive rock is never the
//!    lower member (a pluton can be younger than the metamorphism of its roof), and steps below
//!    [`METAMORPHIC_FROM_C`] (gypsum → anhydrite) are diagenesis, not grade.
//! 2. **A thin sliver of metamorphic rock** inside the pile: a horizon of a regional-metamorphic
//!    class thinner than [`Limits::sliver_blocks`] with other rock above and below it. One cut by
//!    the ground or by the world's floor is not a sliver (erosion and the floor may cut anything).
//!    High-grade slivers (schist, amphibolite, gneiss and the high-pressure rocks) are counted on
//!    their own.
//! 3. **Plutonic rock lying in the pile like a bed**: an intrusive class with rock of another
//!    family under it (a *floored* intrusion), and one thinner than [`Limits::sheet_blocks`]
//!    between other rocks (a *sheet*). Plutons are discordant bodies; how the columns show the
//!    crust's plutonic share before P2-01 is Ben's to decide, so this one is measured, not
//!    asserted.

use std::collections::BTreeMap;

use serde_json::{Value, json};

use super::bake_horizons::Horizons;
use super::metamorphism::{self, Paths};
use crate::surface::lithology;
use crate::synth::EVERY_CLASS;

/// Path steps colder than this are diagenesis (gypsum → anhydrite at 50 °C), not metamorphism.
pub const METAMORPHIC_FROM_C: f64 = 200.0;

/// What the check needs to know of each class of a world's class list.
#[derive(Clone, Debug)]
pub struct Rules {
    pub names: Vec<String>,
    /// The least temperature the class formed at, °C (0 for rock that is not metamorphic).
    pub formed_at_c: Vec<f64>,
    /// The temperature the class never reached, °C (∞ where nothing says).
    pub below_c: Vec<f64>,
    /// `metamorphism::grade` (0–4).
    pub grade: Vec<u8>,
    /// Regional-metamorphic classes (the ones with a formation temperature).
    pub metamorphic: Vec<bool>,
    /// Plutonic rock (the intrusive family less the mantle's peridotite).
    pub plutonic: Vec<bool>,
    /// The mantle's rock.
    pub mantle: Vec<bool>,
}

impl Rules {
    /// The rules for a world whose `strata/classes.json` lists `names`; a class this build does
    /// not know is neutral (never part of a finding).
    pub fn for_classes<S: AsRef<str>>(names: &[S]) -> Rules {
        let table = lithology::table();
        let paths = Paths::get();
        let n = names.len();
        let mut r = Rules {
            names: names.iter().map(|s| s.as_ref().to_owned()).collect(),
            formed_at_c: vec![0.0; n],
            below_c: vec![f64::INFINITY; n],
            grade: vec![0; n],
            metamorphic: vec![false; n],
            plutonic: vec![false; n],
            mantle: vec![false; n],
        };
        for (i, name) in names.iter().enumerate() {
            let name = name.as_ref();
            let Some(k) = EVERY_CLASS.iter().position(|&c| c == name) else {
                continue;
            };
            let c = &table[k];
            let intrusive = c.family == "intrusive_igneous" || c.family == "special_host";
            r.mantle[i] = name == "peridotite";
            r.plutonic[i] = c.family == "intrusive_igneous" && !r.mantle[i];
            r.grade[i] = metamorphism::grade(name);
            if c.family == "metamorphic"
                && let Some(t) = metamorphism::formation_temp_c(name)
                && t >= METAMORPHIC_FROM_C
            {
                r.formed_at_c[i] = t;
                r.metamorphic[i] = true;
            }
            if !intrusive
                && let Some(&(_, t)) = paths
                    .steps(k as u16)
                    .iter()
                    .find(|s| s.1 >= METAMORPHIC_FROM_C)
            {
                r.below_c[i] = t;
            }
        }
        r
    }

    /// The rules for the build's own class list (`synth::EVERY_CLASS`, what the bake writes).
    pub fn builtin() -> Rules {
        Rules::for_classes(EVERY_CLASS.as_slice())
    }

    fn name(&self, class: u8) -> &str {
        self.names.get(usize::from(class)).map_or("?", |s| s)
    }
}

/// The thicknesses the check uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// A metamorphic horizon thinner than this inside the pile is a sliver, blocks.
    pub sliver_blocks: i32,
    /// A plutonic horizon thinner than this between other rocks is a sheet, blocks.
    pub sheet_blocks: i32,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            sliver_blocks: 8,
            sheet_blocks: 16,
        }
    }
}

/// One horizon of a column below the ground: (top block y, blocks, class).
pub type Visible = (i32, i32, u8);

/// What is wrong with one column.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Findings {
    /// (class above, class below, top of the lower one): the lower horizon was never as hot as
    /// the upper one formed at.
    pub inversions: Vec<(u8, u8, i32)>,
    /// Metamorphic slivers: (class, top, blocks).
    pub slivers: Vec<Visible>,
    /// Plutonic sheets: (class, top, blocks) as [`Visible`].
    pub sheets: Vec<Visible>,
    /// Plutonic horizons with rock of another family under them.
    pub floored: Vec<Visible>,
}

impl Findings {
    pub fn clean(&self) -> bool {
        self.inversions.is_empty() && self.slivers.is_empty()
    }
}

/// The horizons of a column the game shows: `horizons` (top block y, class; top-down, horizon h
/// fills from its top down to the next one's top + 1, the last down to `floor_y`) under a ground
/// whose top block is `ground_y`.
pub fn visible(horizons: &[(i32, u8)], ground_y: i32, floor_y: i32, out: &mut Vec<Visible>) {
    out.clear();
    for (h, &(top, class)) in horizons.iter().enumerate() {
        let top = top.min(ground_y);
        let below = horizons
            .get(h + 1)
            .map_or(floor_y - 1, |n| n.0.max(floor_y - 1));
        let blocks = top - below;
        if blocks <= 0 {
            continue;
        }
        match out.last_mut() {
            Some(last) if last.2 == class => last.1 += blocks,
            _ => out.push((top, blocks, class)),
        }
    }
}

/// Checks one column ([`visible`] horizons, top-down).
pub fn check(column: &[Visible], rules: &Rules, limits: &Limits, out: &mut Findings) {
    out.inversions.clear();
    out.slivers.clear();
    out.sheets.clear();
    out.floored.clear();
    let n = column.len();
    // The hottest formation temperature above, and whose it is.
    let mut hot = (0.0f64, 0u8);
    for (h, &(top, blocks, class)) in column.iter().enumerate() {
        let k = usize::from(class);
        if k >= rules.names.len() {
            continue;
        }
        if hot.0 > rules.below_c[k] {
            out.inversions.push((hot.1, class, top));
        }
        if rules.formed_at_c[k] > hot.0 {
            hot = (rules.formed_at_c[k], class);
        }
        let inside = h > 0 && h + 1 < n;
        if inside && rules.metamorphic[k] && blocks < limits.sliver_blocks {
            out.slivers.push((top, blocks, class));
        }
        if rules.plutonic[k] {
            let other_below = column[h + 1..].iter().any(|&(_, _, c)| {
                let c = usize::from(c);
                c < rules.names.len() && !rules.plutonic[c] && !rules.mantle[c]
            });
            if other_below {
                out.floored.push((top, blocks, class));
                if h > 0 && blocks < limits.sheet_blocks {
                    out.sheets.push((top, blocks, class));
                }
            }
        }
    }
}

/// The findings of many columns, summed.
#[derive(Clone, Debug, Default)]
pub struct Census {
    pub columns: u64,
    pub horizons: u64,
    pub with_inversion: u64,
    pub with_sliver: u64,
    pub with_high_grade_sliver: u64,
    pub with_sheet: u64,
    pub with_floored: u64,
    /// Columns holding any regional-metamorphic rock.
    pub with_metamorphic: u64,
    /// "upper over lower" → horizons.
    pub inversion_pairs: BTreeMap<String, u64>,
    pub sliver_classes: BTreeMap<String, u64>,
    pub sheet_classes: BTreeMap<String, u64>,
    pub floored_classes: BTreeMap<String, u64>,
}

impl Census {
    pub fn add(&mut self, column: &[Visible], f: &Findings, rules: &Rules) {
        self.columns += 1;
        self.horizons += column.len() as u64;
        self.with_inversion += u64::from(!f.inversions.is_empty());
        self.with_sliver += u64::from(!f.slivers.is_empty());
        self.with_high_grade_sliver += u64::from(
            f.slivers
                .iter()
                .any(|s| rules.grade.get(usize::from(s.2)).is_some_and(|&g| g >= 3)),
        );
        self.with_sheet += u64::from(!f.sheets.is_empty());
        self.with_floored += u64::from(!f.floored.is_empty());
        self.with_metamorphic += u64::from(
            column
                .iter()
                .any(|v| rules.metamorphic.get(usize::from(v.2)) == Some(&true)),
        );
        for &(up, low, _) in &f.inversions {
            let key = format!("{} over {}", rules.name(up), rules.name(low));
            *self.inversion_pairs.entry(key).or_insert(0) += 1;
        }
        for (list, map) in [
            (&f.slivers, &mut self.sliver_classes),
            (&f.sheets, &mut self.sheet_classes),
            (&f.floored, &mut self.floored_classes),
        ] {
            for v in list {
                *map.entry(rules.name(v.2).to_owned()).or_insert(0) += 1;
            }
        }
    }

    pub fn merge(&mut self, o: &Census) {
        self.columns += o.columns;
        self.horizons += o.horizons;
        self.with_inversion += o.with_inversion;
        self.with_sliver += o.with_sliver;
        self.with_high_grade_sliver += o.with_high_grade_sliver;
        self.with_sheet += o.with_sheet;
        self.with_floored += o.with_floored;
        self.with_metamorphic += o.with_metamorphic;
        for (mine, theirs) in [
            (&mut self.inversion_pairs, &o.inversion_pairs),
            (&mut self.sliver_classes, &o.sliver_classes),
            (&mut self.sheet_classes, &o.sheet_classes),
            (&mut self.floored_classes, &o.floored_classes),
        ] {
            for (k, v) in theirs {
                *mine.entry(k.clone()).or_insert(0) += v;
            }
        }
    }

    pub fn share(&self, n: u64) -> f64 {
        if self.columns == 0 {
            0.0
        } else {
            n as f64 / self.columns as f64
        }
    }

    pub fn to_json(&self) -> Value {
        let top = |m: &BTreeMap<String, u64>| -> Value {
            let mut v: Vec<(&String, &u64)> = m.iter().collect();
            v.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
            Value::Array(v.into_iter().take(12).map(|(k, n)| json!([k, n])).collect())
        };
        json!({
            "columns": self.columns,
            "mean_horizons": if self.columns == 0 { 0.0 } else { self.horizons as f64 / self.columns as f64 },
            "share": {
                "grade_inversion": self.share(self.with_inversion),
                "metamorphic_sliver": self.share(self.with_sliver),
                "high_grade_sliver": self.share(self.with_high_grade_sliver),
                "plutonic_sheet": self.share(self.with_sheet),
                "plutonic_with_a_floor": self.share(self.with_floored),
                "any_metamorphic_rock": self.share(self.with_metamorphic),
            },
            "inversions": top(&self.inversion_pairs),
            "slivers": top(&self.sliver_classes),
            "plutonic_sheets": top(&self.sheet_classes),
            "plutonic_with_a_floor": top(&self.floored_classes),
        })
    }
}

/// Checks every `stride`-th cell's own stack of `hz` (not the column rule's blend of sixteen):
/// each stack under its own first top.
pub fn census_of_cells(hz: &Horizons, rules: &Rules, limits: &Limits, stride: usize) -> Census {
    let mut census = Census::default();
    let (mut raw, mut col, mut f) = (Vec::new(), Vec::new(), Findings::default());
    for c in (0..hz.cells()).step_by(stride.max(1)) {
        let (tops, classes, _) = hz.cell(c);
        raw.clear();
        raw.extend(
            tops.iter()
                .zip(classes)
                .map(|(&t, &k)| (i32::from(t).div_euclid(16), k)),
        );
        let Some(&(ground, _)) = raw.first() else {
            continue;
        };
        visible(&raw, ground, hz.min_y, &mut col);
        check(&col, rules, limits, &mut f);
        census.add(&col, &f, rules);
    }
    census
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::lithology::index;

    fn column(units: &[(&str, i32)], ground: i32) -> Vec<Visible> {
        let mut raw = Vec::new();
        let mut y = ground;
        for &(name, blocks) in units {
            raw.push((y, index(name) as u8));
            y -= blocks;
        }
        let mut out = Vec::new();
        visible(&raw, ground, y + 1, &mut out);
        out
    }

    fn findings(units: &[(&str, i32)]) -> Findings {
        let mut f = Findings::default();
        check(
            &column(units, 100),
            &Rules::builtin(),
            &Limits::default(),
            &mut f,
        );
        f
    }

    #[test]
    fn cover_on_basement_is_fine_and_schist_over_shale_is_not() {
        // Unmetamorphosed cover on a metamorphic basement, grade rising downwards.
        let ok = findings(&[
            ("sandstone", 40),
            ("shale", 30),
            ("slate", 20),
            ("phyllite", 20),
            ("schist", 30),
            ("gneiss", 50),
            ("granite", 100),
        ]);
        assert!(ok.clean(), "{ok:?}");
        assert!(ok.floored.is_empty());
        // Ben's column: schist at the surface over unmetamorphosed shale.
        let bad = findings(&[("schist", 63), ("shale", 57), ("granite", 30)]);
        assert_eq!(bad.inversions.len(), 1, "{bad:?}");
        let (up, low, _) = bad.inversions[0];
        assert_eq!((up, low), (index("schist") as u8, index("shale") as u8));
        // Schist (450 °C) over greenstone (which would be amphibolite at 500 °C) is no
        // inversion; schist over basalt (greenstone at 300 °C) is; a pluton below is exempt.
        assert!(findings(&[("schist", 30), ("greenstone", 30), ("gneiss", 30)]).clean());
        assert_eq!(
            findings(&[("schist", 30), ("basalt", 30)]).inversions.len(),
            1
        );
        assert!(findings(&[("gneiss", 30), ("granite", 30), ("gabbro", 30)]).clean());
        // Anhydrite over gypsum is diagenesis.
        assert!(findings(&[("anhydrite", 30), ("gypsum", 30)]).clean());
    }

    #[test]
    fn slivers_and_sheets_are_inside_the_pile() {
        // A 3-block gneiss between greenstone and granodiorite; granodiorite and gabbro as beds
        // over greenstone.
        let f = findings(&[
            ("basalt", 45),
            ("greenstone", 3),
            ("gneiss", 3),
            ("granodiorite", 60),
            ("gabbro", 10),
            ("greenstone", 40),
        ]);
        assert_eq!(f.slivers.len(), 2, "{f:?}");
        assert_eq!(f.floored.len(), 2, "{f:?}");
        assert_eq!(f.sheets.len(), 1, "{f:?}");
        // Cut by the ground or the floor: not a sliver.
        let f = findings(&[("schist", 2), ("granite", 80), ("gneiss", 3)]);
        assert!(f.slivers.is_empty(), "{f:?}");
        // A ground below the first top hides what is above it.
        let mut out = Vec::new();
        visible(&[(100, 1), (90, 2), (50, 3)], 80, 0, &mut out);
        assert_eq!(out, [(80, 30, 2), (50, 51, 3)]);
    }
}
