// SPDX-License-Identifier: LGPL-3.0-or-later
//! Regional metamorphism of the strata log (plan §8.5, Phase 1): every step, every layer steps
//! along its class's `metamorphic_path` (shale → slate → phyllite → schist → gneiss, limestone →
//! marble, basalt → greenstone → amphibolite, …) as far as the temperature at its mid-depth
//! allows. Classes only move forward, so a metamorphic rock keeps its grade when erosion brings it
//! back up — which is how metamorphic belts reach the surface.
//!
//! **Where.** On continental crust (sediment on oceanic crust is subducted, or accreted and then
//! continental, before it is buried deep enough).
//!
//! **Temperature.** A linear geotherm T(z) = T_s + (q / k)·z_eff, with q the plate model's heat
//! flow proxy (`plates/heat_flow_mw_m2`, at most `max_heat_flow_mw_m2`), k = `conductivity_w_mk`
//! and T_s the climate's mean temperature (or `surface_temp_c`). **Regime history** enters through z_eff: under an active
//! collision (the `collision` regime, or a Himalayan orogeny younger than 30 Myr) the crust is being
//! thickened and everything in it is buried with it, so depths scale by the thickening factor
//! β = clamp(e / e₀, 1, `max_burial_factor`) (homogeneous pure shear; e the crust's thickness,
//! e₀ `reference_crust_m`). Elsewhere β = 1: burial metamorphism of deep basins with the heat flow
//! they have. This is the "collision zones step shale → slate → … along each class's
//! metamorphic_path" of plan §8.5; the full P–T–t treatment (restored burial and heat history,
//! high-pressure series) is Phase 2's (P2-02, open question Q8), so blueschist and eclogite are not
//! formed here, nor contact metamorphism (the plutons' aureoles, `plutons.rs`).
//!
//! **Formation temperatures** (°C; the regional sequence's textbook boundaries, rounded — Winter
//! 2010, *Principles of Igneous and Metamorphic Petrology*, 2nd ed., ch. 21 and 25; Yardley 1989,
//! *An Introduction to Metamorphic Petrology*, ch. 3): slate 250 (the anchizone–epizone,
//! lowest greenschist), phyllite 350, quartzite, marble and greenstone 300 (greenschist facies),
//! schist 450 (upper greenschist to lower amphibolite, the garnet zone), amphibolite 500, gneiss
//! 650 (upper amphibolite, the sillimanite zone and migmatites). Gypsum → anhydrite at 50 °C, the
//! dehydration of buried gypsum within about the first kilometre (Warren 2016, *Evaporites*, 2nd
//! ed., ch. 7). A path element without a temperature (serpentinite, eclogite) is never reached.

use crate::surface::lithology;
use crate::surface::strata_log::{ColumnMut, Layer};

/// The temperature (°C) at which `name` forms from the class before it in a metamorphic path.
pub fn formation_temp_c(name: &str) -> Option<f64> {
    Some(match name {
        "anhydrite" => 50.0,
        "slate" => 250.0,
        "quartzite" | "marble" | "greenstone" => 300.0,
        "phyllite" => 350.0,
        "schist" => 450.0,
        "amphibolite" => 500.0,
        "gneiss" => 650.0,
        _ => return None,
    })
}

/// A class's metamorphic grade for maps: 0 none, 1 very low (slate), 2 low (phyllite,
/// greenschist-facies quartzite, marble, greenstone, serpentinite, contact rocks), 3 medium
/// (schist, amphibolite, blueschist), 4 high (gneiss, eclogite).
pub fn grade(name: &str) -> u8 {
    match name {
        "slate" => 1,
        "phyllite" | "quartzite" | "marble" | "greenstone" | "serpentinite" | "hornfels"
        | "skarn" => 2,
        "schist" | "amphibolite" | "blueschist" => 3,
        "gneiss" | "eclogite" => 4,
        _ => 0,
    }
}

/// Per class: the path (class, formation temperature) as far as it has temperatures.
#[derive(Clone, Debug)]
pub struct Paths {
    steps: Vec<Vec<(u16, f64)>>,
    pub grades: Vec<u8>,
}

impl Paths {
    pub fn get() -> Paths {
        let t = lithology::table();
        Paths {
            steps: t
                .iter()
                .map(|c| {
                    c.metamorphic_path
                        .iter()
                        .map_while(|&k| {
                            formation_temp_c(t[usize::from(k)].name).map(|temp| (k, temp))
                        })
                        .collect()
                })
                .collect(),
            grades: t.iter().map(|c| grade(c.name)).collect(),
        }
    }

    /// `class`'s path as (class, formation temperature °C), as far as it has temperatures.
    pub fn steps(&self, class: u16) -> &[(u16, f64)] {
        &self.steps[usize::from(class)]
    }

    /// The class `class` becomes at temperature `temp_c` (itself when too cold).
    pub fn at(&self, class: u16, temp_c: f64) -> u16 {
        let mut out = class;
        for &(k, t) in &self.steps[usize::from(class)] {
            if temp_c >= t {
                out = k;
            } else {
                break;
            }
        }
        out
    }
}

/// The thermal state of one cell for one step.
#[derive(Clone, Copy, Debug)]
pub struct Thermal {
    pub surface_c: f64,
    /// Geotherm, °C per metre.
    pub gradient_c_m: f64,
    /// Burial factor β ≥ 1.
    pub burial: f64,
}

impl Thermal {
    pub fn temp_at(&self, depth_m: f64) -> f64 {
        self.surface_c + self.gradient_c_m * depth_m * self.burial
    }
}

/// Steps the column's layers along their paths (`above_m` of regolith on top); merges
/// neighbouring layers that end up in one class (the older base age, the thicker one's
/// provenance). Returns the thickness that changed class, m.
pub fn metamorphose(col: &mut ColumnMut<'_>, above_m: f64, th: &Thermal, paths: &Paths) -> f64 {
    let n = usize::from(*col.count);
    if n == 0 {
        return 0.0;
    }
    let mut changed = 0.0;
    let mut depth = above_m;
    for i in (0..n).rev() {
        let l = &mut col.layers[i];
        let t = f64::from(l.thickness_m);
        let to = paths.at(l.class, th.temp_at(depth + 0.5 * t));
        if to != l.class {
            l.class = to;
            changed += t;
        }
        depth += t;
    }
    if changed > 0.0 {
        *col.count = merge_same(&mut col.layers[..n]) as u8;
    }
    changed
}

/// Merges runs of one class in `col` (bottom first); returns the new count.
fn merge_same(col: &mut [Layer]) -> usize {
    let mut m = 0usize;
    for i in 0..col.len() {
        let l = col[i];
        if m > 0 && col[m - 1].class == l.class {
            let below = &mut col[m - 1];
            if l.thickness_m > below.thickness_m {
                below.provenance = l.provenance;
            }
            below.thickness_m += l.thickness_m;
            below.age_myr = below.age_myr.min(l.age_myr);
        } else {
            col[m] = l;
            m += 1;
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::lithology::index;
    use crate::surface::strata_log::StrataLog;

    #[test]
    fn layers_step_along_their_paths_with_depth_and_never_back() {
        let paths = Paths::get();
        let shale = index("shale");
        assert_eq!(paths.at(shale, 100.0), shale);
        assert_eq!(paths.at(shale, 260.0), index("slate"));
        assert_eq!(paths.at(shale, 460.0), index("schist"));
        assert_eq!(paths.at(shale, 700.0), index("gneiss"));
        assert_eq!(paths.at(index("slate"), 360.0), index("phyllite"));
        assert_eq!(paths.at(index("limestone"), 320.0), index("marble"));
        assert_eq!(paths.at(index("basalt"), 520.0), index("amphibolite"));
        assert_eq!(paths.at(index("gypsum"), 60.0), index("anhydrite"));
        assert_eq!(paths.at(index("coal"), 900.0), index("coal"), "no path");
        assert_eq!(paths.at(index("blueschist"), 900.0), index("blueschist"));

        // A 12-km pile of shale over sandstone in a collision zone: 25 °C/km, β 2.
        let mut log = StrataLog::new(1, 8);
        log.deposit(0, -100.0, 4000.0, index("sandstone"), 0);
        log.deposit(0, -90.0, 6000.0, index("shale"), 0);
        log.deposit(0, -80.0, 2000.0, index("limestone"), 0);
        let hot = Thermal {
            surface_c: 10.0,
            gradient_c_m: 0.025,
            burial: 2.0,
        };
        let changed = metamorphose(&mut log.column_mut(0), 0.0, &hot, &paths);
        assert!(changed > 0.0);
        let col: Vec<u16> = log.column(0).iter().map(|l| l.class).collect();
        // Limestone at 1 km mid-depth: 10 + 25·2 = 60 °C, unchanged; shale at 5 km: 260 °C →
        // slate; sandstone at 10 km: 510 °C → quartzite.
        assert_eq!(
            col,
            [index("quartzite"), index("slate"), index("limestone")]
        );
        log.check(0, crate::synth::EVERY_CLASS.len()).unwrap();
        // Cooled and exhumed: nothing goes back.
        let cold = Thermal {
            burial: 1.0,
            gradient_c_m: 0.0,
            ..hot
        };
        assert_eq!(
            metamorphose(&mut log.column_mut(0), 0.0, &cold, &paths),
            0.0
        );
        assert_eq!(log.column(0)[1].class, index("slate"));
        // Two layers reaching one class merge (older base age, summed thickness).
        let mut log = StrataLog::new(1, 8);
        log.deposit(0, -50.0, 100.0, index("mudstone"), 0);
        log.deposit(0, -40.0, 100.0, index("shale"), 0);
        let warm = Thermal {
            surface_c: 10.0,
            gradient_c_m: 0.05,
            burial: 1.0,
        };
        // Under 5 km of cover both reach ~265 °C: slate.
        metamorphose(&mut log.column_mut(0), 5000.0, &warm, &paths);
        let c = log.column(0);
        assert_eq!(c.len(), 1, "{c:?}");
        assert_eq!(c[0].class, index("slate"));
        assert_eq!(c[0].age_myr, -50.0);
        assert!((c[0].thickness_m - 200.0).abs() < 1e-3);
        log.check(0, crate::synth::EVERY_CLASS.len()).unwrap();
    }
}
