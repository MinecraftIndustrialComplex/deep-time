// SPDX-License-Identifier: LGPL-3.0-or-later
//! The "Geologic provinces" view layer (decisions log 2026-09-29): a USGS-style classification of
//! every L1 cell, drawn as one more keyframe layer (`provinces`), so the workbench and the static
//! viewer show it at every time step, like the USGS map "Geologic provinces of the world" (shield,
//! platform, orogen, basin, large igneous province, extended crust, oceanic crust by age).
//!
//! **The view derives, the bake bakes.** The layer is derived from fields the stages already wrote,
//! at render time, into a scratch field: nothing is written to `WorldState`. Since P1-10 (decisions
//! log 2026-09-29, "Switch the world's own provinces to the USGS classes? Yes, for P1-10") the bake
//! runs the same classifier ([`derive_with`], with the preset's [`ProvinceParams`]) and cuts its
//! classes into contiguous provinces (`bake/meta.rs`, `meta/province`), which drive TerraBlender's
//! regions in the game. The view always uses the default thresholds.
//!
//! # Rules (plan §8.1 note; the first that applies wins)
//! 1. **Large igneous province**: a pile of mafic volcanic rock (`plates/pool/mafic_volcanic_m`)
//!    beyond what the column's crust carries (the 2 km of basalt of oceanic crust, told by its
//!    gabbro lower crust, `plates/pool/mafic_plutonic_m`) of at least a fifth of the LIP thickness
//!    the run's `deeptime:large_igneous_province` events recorded (`lip_thickness_m`, 1.5 km by
//!    default): a plume-head province's footprint, which rides its plate and stays until the crust
//!    is subducted. Hotspot piles of that size count too. Without a LIP event, none.
//! 2. **Oceanic crust** (`plates/crust_type` = 0), in the USGS map's three age classes, 0–20 Ma,
//!    20–65 Ma and over 65 Ma (`crust_age`); an island arc (regime arc on crust thickened above
//!    9 km, `plates/thickness_m`) is an **orogen**.
//! 3. **Orogen** (continental crust): an active collision or arc (`regime`); crust a collision
//!    orogeny (Himalayan type) touched within the last 500 Myr, which stays an orogen after it
//!    stops rising, as the Appalachians and Urals do (`plates/orogeny_type`,
//!    `plates/orogeny_age_myr`); or a range an Andean orogeny lifted to over 1.5 km within the
//!    last 50 Myr (`surface/elevation_m`). Andean orogeny alone is not enough: the plate model
//!    stamps every cell within the subduction distance of a trench.
//! 4. **Extended crust**: continental crust in a rift (`regime`) or thinned below 20 km
//!    (`plates/thickness_m`): rifts, the Basin-and-Range kind of province; and, after the rules,
//!    the rifted margins: a band `rifted_margin_deg` wide along every quiet coast ([`rifted_margins`]).
//! 5. **Basin**: continental crust under at least 3 km of sediment and sedimentary rock (the
//!    strata log `surface/sediment_m` or the plate model's sediment pools, whichever is thicker),
//!    or subsiding at over 0.03 mm/yr (`plates/uplift_mm_yr`) under at least 1 km.
//! 6. **Shield**: the rest of the old continental crust where crystalline basement is at the
//!    surface: under 0.2 km of cover, or a surface rock (`structure/rock`, else `surface/class`)
//!    that is neither sedimentary nor loose.
//! 7. **Platform**: the rest: old continental crust under a sedimentary cover thinner than a
//!    basin's.
//!
//! The continental classes are then generalised like a printed map ([`generalise`]): a cell takes
//! the majority class of the continental cells within half the map's smallest feature
//! (`feature_deg`, the USGS map's 150 km), and continental patches smaller than that feature
//! squared take the sea floor's class around them ([`drop_specks`]; island-arc specks).
//!
//! Continental crust has no formation age in the plate model, only its last orogeny; whatever no
//! rule 3 caught is taken to be old (Precambrian) crust.

use deeptime_format::ParamValue;

use deeptime_format::GridDesc;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::CoreError;
use crate::plates::state::regime;
use crate::world::{Field, WorldState};

/// The keyframe layer's name (`ramps.json` colours it, its `labels` are [`CLASSES`]).
pub const LAYER: &str = "provinces";

/// The classes, by code (the order of the ramp's `labels`).
pub const CLASSES: [&str; 9] = [
    "Shield",
    "Platform",
    "Orogen",
    "Basin",
    "Large igneous province",
    "Extended crust",
    "Oceanic crust 0–20 Ma",
    "Oceanic crust 20–65 Ma",
    "Oceanic crust over 65 Ma",
];

pub const SHIELD: u8 = 0;
pub const PLATFORM: u8 = 1;
pub const OROGEN: u8 = 2;
pub const BASIN: u8 = 3;
pub const LIP: u8 = 4;
pub const EXTENDED: u8 = 5;
pub const OCEAN_YOUNG: u8 = 6;
pub const OCEAN_MID: u8 = 7;
pub const OCEAN_OLD: u8 = 8;

/// How far the generalisation looks, in L1 cells (a cell is about 39 km on an Earth-sized planet
/// at 1024 columns).
pub const GENERALISE_CELLS: usize = 1;

/// Orogeny types of `plates/orogeny_type` (`plates::state::OROGENY_*`).
const ANDEAN: u8 = crate::plates::state::OROGENY_ANDEAN;
const HIMALAYAN: u8 = crate::plates::state::OROGENY_HIMALAYAN;

/// The thresholds of the rules above.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    /// Oceanic crust younger than this is the first age class, Myr (USGS: 20 Ma).
    pub ocean_young_myr: f32,
    /// … younger than this the second, Myr (USGS: 65 Ma).
    pub ocean_mid_myr: f32,
    /// An arc zone on oceanic crust is an island arc (an orogen) once the crust is this thick, m
    /// (ordinary oceanic crust is 7 km).
    pub island_arc_min_thickness_m: f32,
    /// A collision belt touched within this many Myr is an orogen.
    pub orogen_max_age_myr: f32,
    /// An Andean range is an orogen while it was lifted within this many Myr …
    pub andean_max_age_myr: f32,
    /// … and stands at least this high, m.
    pub andean_min_elevation_m: f32,
    /// Continental crust thinner than this is extended, m.
    pub thinned_crust_m: f32,
    /// A passive (rifted) margin is extended crust this far in from the continent's edge,
    /// degrees of arc (P1-27; 1.35° is 150 km on an Earth-sized sphere, the thinned crust under
    /// the shelf and slope; the plate model's one-sample `passive_margin` regime made a dashed
    /// line instead).
    pub rifted_margin_deg: f32,
    /// The smallest feature the map shows, degrees of arc (the USGS map's 150 km, 1.35°): the
    /// generalisation looks half as far, and continental crust in a patch smaller than this
    /// squared (an island-arc speck) takes the sea floor's class around it.
    pub feature_deg: f32,
    /// A basin holds at least this much sediment and sedimentary rock, m …
    pub basin_cover_m: f32,
    /// … or at least [`Rules::subsiding_cover_m`] while sinking faster than this, mm/yr.
    pub subsiding_mm_yr: f32,
    pub subsiding_cover_m: f32,
    /// Basement under less cover than this is at the surface, m.
    pub shield_cover_m: f32,
    /// A LIP is a mafic pile of this fraction of the recorded LIP thickness.
    pub lip_fraction: f32,
    /// The basalt an oceanic crust carries (`plates::pools::oceanic_column`), m.
    pub oceanic_basalt_m: f32,
}

impl Default for Rules {
    fn default() -> Rules {
        Rules {
            ocean_young_myr: 20.0,               // ledger: PRV-002 (abstract-only)
            ocean_mid_myr: 65.0,                 // ledger: PRV-003 (abstract-only)
            island_arc_min_thickness_m: 9_000.0, // ledger: PRV-005 (calibrated)
            orogen_max_age_myr: 500.0,           // ledger: PRV-011 (uncited)
            andean_max_age_myr: 50.0,            // ledger: PRV-012 (calibrated)
            andean_min_elevation_m: 1500.0,      // ledger: PRV-012
            thinned_crust_m: 20_000.0,           // ledger: PRV-013 (calibrated)
            rifted_margin_deg: 1.35,             // ledger: PRV-014 (uncited)
            feature_deg: 1.35,                   // ledger: PRV-021 (abstract-only)
            basin_cover_m: 3_000.0,              // ledger: PRV-016 (uncited)
            subsiding_mm_yr: 0.03,               // ledger: PRV-017 (uncited)
            subsiding_cover_m: 1_000.0,          // ledger: PRV-017
            shield_cover_m: 200.0,               // ledger: PRV-018 (uncited)
            lip_fraction: 0.2,                   // ledger: PRV-006 (uncited)
            oceanic_basalt_m: crate::plates::pools::oceanic_column()
                [crate::plates::state::pool::MAFIC_VOLCANIC],
        }
    }
}

/// The `provinces` section of a preset (every key optional): what the bake's provinces are, and so
/// how big the areas are that one TerraBlender region covers. The defaults are the view layer's
/// rules and the bake's minimum of before P1-10.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct ProvinceParams {
    /// A province is at least this large, km²: smaller connected regions of one class are absorbed
    /// by the nearest larger one. Bigger values give fewer, larger biome regions.
    pub min_area_km2: f64,
    /// Continental crust a collision touched within this many Myr stays an orogen after it stops
    /// rising (the Appalachians, the Urals). The plate model collides continents about twice as
    /// often as the Earth did, so lowering this brings the orogen share (0.40 of the continents,
    /// the Earth's about 0.17) down.
    pub orogen_max_age_myr: f64,
    /// An Andean (subduction) range counts as an orogen while it was lifted within this many Myr …
    pub andean_max_age_myr: f64,
    /// … and stands at least this high, m.
    pub andean_min_elevation_m: f64,
}

impl Default for ProvinceParams {
    fn default() -> Self {
        let r = Rules::default();
        ProvinceParams {
            min_area_km2: 50_000.0, // ledger: PRV-029 (game-choice)
            orogen_max_age_myr: f64::from(r.orogen_max_age_myr), // ledger: PRV-011 (uncited)
            andean_max_age_myr: f64::from(r.andean_max_age_myr), // ledger: PRV-012 (calibrated)
            andean_min_elevation_m: f64::from(r.andean_min_elevation_m), // ledger: PRV-012 (calibrated)
        }
    }
}

impl ProvinceParams {
    /// Range checks, each naming its key.
    pub fn validate(&self) -> Result<(), CoreError> {
        for (v, lo, hi, key) in [
            (self.min_area_km2, 1_000.0, 5_000_000.0, "min_area_km2"),
            (self.orogen_max_age_myr, 0.0, 4_000.0, "orogen_max_age_myr"),
            (self.andean_max_age_myr, 0.0, 4_000.0, "andean_max_age_myr"),
            (
                self.andean_min_elevation_m,
                0.0,
                9_000.0,
                "andean_min_elevation_m",
            ),
        ] {
            if !(v.is_finite() && (lo..=hi).contains(&v)) {
                return Err(CoreError::preset(
                    format!("$.provinces.{key}"),
                    format!("must be in [{lo}, {hi}]"),
                ));
            }
        }
        Ok(())
    }

    /// The classifier's thresholds with these settings.
    pub fn rules(&self) -> Rules {
        Rules {
            orogen_max_age_myr: self.orogen_max_age_myr as f32,
            andean_max_age_myr: self.andean_max_age_myr as f32,
            andean_min_elevation_m: self.andean_min_elevation_m as f32,
            ..Rules::default()
        }
    }
}

/// What the classifier reads of one cell. `NaN` means "not known" and never triggers a rule.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub continental: bool,
    /// A `plates::state::regime` code.
    pub regime: u8,
    /// Oceanic crust age, Myr.
    pub age_myr: f32,
    /// Last orogeny: 0 none, 1 Andean, 2 Himalayan, and its age, Myr.
    pub orogeny: u8,
    pub orogeny_age_myr: f32,
    /// Surface height above the sea, m.
    pub elevation_m: f32,
    pub thickness_m: f32,
    /// Sediment and sedimentary rock on top of the basement, m.
    pub cover_m: f32,
    /// Sinking rate, mm/yr (positive down).
    pub subsidence_mm_yr: f32,
    /// Mafic volcanic rock beyond what the column's crust carries, m.
    pub mafic_excess_m: f32,
    /// Whether the surface rock is crystalline (`None`: not known).
    pub crystalline_surface: Option<bool>,
}

/// The class of one cell; `lip_min_m` is the pile a LIP needs (`None`: the run recorded no LIP).
pub fn classify(c: &Cell, lip_min_m: Option<f32>, r: &Rules) -> u8 {
    if let Some(min) = lip_min_m
        && c.mafic_excess_m >= min
    {
        return LIP;
    }
    if !c.continental {
        if c.regime == regime::ARC && c.thickness_m >= r.island_arc_min_thickness_m {
            return OROGEN;
        }
        return if c.age_myr < r.ocean_young_myr {
            OCEAN_YOUNG
        } else if c.age_myr < r.ocean_mid_myr {
            OCEAN_MID
        } else {
            OCEAN_OLD
        };
    }
    let collision_belt = c.orogeny == HIMALAYAN && c.orogeny_age_myr < r.orogen_max_age_myr;
    let andean_range = c.orogeny == ANDEAN
        && c.orogeny_age_myr < r.andean_max_age_myr
        && c.elevation_m >= r.andean_min_elevation_m;
    if c.regime == regime::COLLISION || c.regime == regime::ARC || collision_belt || andean_range {
        return OROGEN;
    }
    if c.regime == regime::RIFT || c.thickness_m < r.thinned_crust_m {
        return EXTENDED;
    }
    if c.cover_m >= r.basin_cover_m
        || (c.cover_m >= r.subsiding_cover_m && c.subsidence_mm_yr > r.subsiding_mm_yr)
    {
        return BASIN;
    }
    if c.cover_m < r.shield_cover_m || c.crystalline_surface == Some(true) {
        SHIELD
    } else {
        PLATFORM
    }
}

/// Generalises the continental classes like a printed map: each continental cell (shield,
/// platform, orogen, basin, extended crust) takes the majority class among the continental cells
/// within `radius` cells (x wraps, y clamps), keeping its own on a tie for first place, so specks
/// of a few cells vanish. Large igneous provinces, the sea floor and gaps stay as they are.
pub fn generalise(classes: &[f32], w: usize, h: usize, radius: usize) -> Vec<f32> {
    let continental = |v: f32| v.is_finite() && (v as u8) <= EXTENDED && (v as u8) != LIP;
    let mut out = classes.to_vec();
    let r = radius as i64;
    // Each cell reads `classes` only: the rows run side by side.
    out.par_chunks_mut(w.max(1))
        .enumerate()
        .for_each(|(j, row)| {
            for (i, o) in row.iter_mut().enumerate() {
                let own = classes[j * w + i];
                if !continental(own) {
                    continue;
                }
                let mut count = [0u32; 6];
                for dj in -r..=r {
                    let jj = (j as i64 + dj).clamp(0, h as i64 - 1) as usize;
                    for di in -r..=r {
                        let ii = (i as i64 + di).rem_euclid(w as i64) as usize;
                        let v = classes[jj * w + ii];
                        if continental(v) {
                            count[v as usize] += 1;
                        }
                    }
                }
                let best = *count.iter().max().expect("six classes");
                if count[own as usize] < best {
                    let k = count.iter().position(|&c| c == best).expect("a maximum");
                    *o = k as f32;
                }
            }
        });
    out
}

/// The pile a LIP needs, from the LIP events recorded so far (`None` without one).
pub fn lip_min_m(world: &WorldState, r: &Rules) -> Option<f32> {
    let mut thinnest: Option<f64> = None;
    for e in world
        .events
        .iter()
        .filter(|e| e.kind == crate::plates::events::LARGE_IGNEOUS_PROVINCE)
    {
        let t = match e.params.get("thickness_m") {
            Some(ParamValue::Float(v)) if *v > 0.0 => *v,
            _ => 1500.0,
        };
        thinnest = Some(thinnest.map_or(t, |x| x.min(t)));
    }
    thinnest.map(|t| (t as f32) * r.lip_fraction)
}

/// The layer for `world`'s current state, on the plate model's L1 grid; `None` when the plate
/// model's crust type, regime and age are not there yet.
pub fn derive(world: &WorldState) -> Option<Field> {
    derive_with(world, &Rules::default())
}

/// [`derive`] with the thresholds of `r` (the bake's, from the preset's `provinces` section).
pub fn derive_with(world: &WorldState, r: &Rules) -> Option<Field> {
    let (cells, lip, grid) = cells(world, r)?;
    let raw: Vec<f32> = cells
        .par_iter()
        .map(|c| c.map_or(f32::NAN, |c| f32::from(classify(&c, lip, r))))
        .collect();
    let (w, h) = (grid.w as usize, grid.h as usize);
    if w * h != raw.len() {
        return Some(Field { grid, data: raw });
    }
    let cell_deg = 360.0 / w as f32;
    let regimes: Vec<u8> = cells
        .iter()
        .map(|c| c.map_or(u8::MAX, |c| c.regime))
        .collect();
    let banded = rifted_margins(
        &raw,
        &regimes,
        w,
        h,
        (r.rifted_margin_deg / cell_deg).round() as usize,
    );
    let radius = ((0.5 * r.feature_deg / cell_deg).round() as usize).max(GENERALISE_CELLS);
    let general = generalise(&banded, w, h, radius);
    let min_cells = ((r.feature_deg / cell_deg).powi(2)).round() as usize;
    Some(Field {
        grid,
        data: drop_specks(&general, w, h, min_cells),
    })
}

/// Continental cells within `cells` (4-neighbour steps over continental cells) of the sea floor
/// whose class is a craton's (shield, platform, basin) and whose regime is not orogenic (no arc,
/// back-arc, collision or trench near) become extended crust: the rifted margins as a continuous
/// band (P1-27).
pub fn rifted_margins(
    classes: &[f32],
    regimes: &[u8],
    w: usize,
    h: usize,
    cells: usize,
) -> Vec<f32> {
    let continental = |v: f32| v.is_finite() && (v as u8) <= EXTENDED;
    let mut out = classes.to_vec();
    if cells == 0 {
        return out;
    }
    let mut dist = vec![usize::MAX; w * h];
    let mut queue = std::collections::VecDeque::new();
    let neighbours = |p: usize| {
        let (i, j) = (p % w, p / w);
        let mut nb = [usize::MAX; 4];
        nb[0] = j * w + (i + 1) % w;
        nb[1] = j * w + (i + w - 1) % w;
        if j > 0 {
            nb[2] = (j - 1) * w + i;
        }
        if j + 1 < h {
            nb[3] = (j + 1) * w + i;
        }
        nb
    };
    for p in 0..w * h {
        if continental(classes[p])
            && neighbours(p)
                .iter()
                .any(|&q| q != usize::MAX && classes[q].is_finite() && !continental(classes[q]))
        {
            dist[p] = 1;
            queue.push_back(p);
        }
    }
    while let Some(p) = queue.pop_front() {
        if dist[p] >= cells {
            continue;
        }
        for q in neighbours(p) {
            if q != usize::MAX && dist[q] == usize::MAX && continental(classes[q]) {
                dist[q] = dist[p] + 1;
                queue.push_back(q);
            }
        }
    }
    for p in 0..w * h {
        let cratonic =
            matches!(classes[p] as u8, SHIELD | PLATFORM | BASIN) && classes[p].is_finite();
        let quiet = matches!(
            regimes[p],
            regime::CRATON | regime::PASSIVE_MARGIN | regime::RIFT | regime::HOTSPOT
        );
        if dist[p] <= cells && cratonic && quiet {
            out[p] = f32::from(EXTENDED);
        }
    }
    out
}

/// Continental patches (any continental class, 4-neighbour, x wraps) smaller than `min_cells`
/// take the commonest sea-floor class along their edge: below the map's smallest feature an
/// island-arc speck is not drawn (the USGS map generalises to 150 km).
pub fn drop_specks(classes: &[f32], w: usize, h: usize, min_cells: usize) -> Vec<f32> {
    let continental = |v: f32| v.is_finite() && (v as u8) <= EXTENDED;
    let mut out = classes.to_vec();
    let mut seen = vec![false; w * h];
    let mut stack = Vec::new();
    for start in 0..w * h {
        if seen[start] || !continental(classes[start]) {
            continue;
        }
        let mut members = Vec::new();
        let mut sea = [0usize; CLASSES.len()];
        seen[start] = true;
        stack.push(start);
        while let Some(p) = stack.pop() {
            members.push(p);
            let (i, j) = (p % w, p / w);
            let mut nb = vec![j * w + (i + 1) % w, j * w + (i + w - 1) % w];
            if j > 0 {
                nb.push((j - 1) * w + i);
            }
            if j + 1 < h {
                nb.push((j + 1) * w + i);
            }
            for q in nb {
                let v = classes[q];
                if continental(v) {
                    if !seen[q] {
                        seen[q] = true;
                        stack.push(q);
                    }
                } else if v.is_finite() {
                    sea[v as usize] += 1;
                }
            }
        }
        if members.len() >= min_cells {
            continue;
        }
        if let Some(k) = (0..CLASSES.len())
            .filter(|&k| sea[k] > 0)
            .max_by_key(|&k| (sea[k], std::cmp::Reverse(k)))
        {
            for p in members {
                out[p] = k as f32;
            }
        }
    }
    out
}

/// What the classifier reads of every L1 cell (`None` where the plate model has nothing), the
/// pile a LIP needs and the grid.
fn cells(world: &WorldState, r: &Rules) -> Option<(Vec<Option<Cell>>, Option<f32>, GridDesc)> {
    let regime_f = world.fields.get("regime")?;
    let n = regime_f.data.len();
    let get = |name: &str| {
        world
            .fields
            .get(name)
            .filter(|f| f.data.len() == n)
            .map(|f| f.data.as_slice())
    };
    let kind = get("plates/crust_type")?;
    let age = get("crust_age")?;
    let orogeny = get("plates/orogeny_type");
    let orogeny_age = get("plates/orogeny_age_myr");
    let elevation = get("surface/elevation_m").or_else(|| get("plates/elevation_m"));
    let thick = get("plates/thickness_m");
    let uplift = get("plates/uplift_mm_yr");
    let sediment_pool = get("plates/pool/sediment_m");
    let sedimentary_pool = get("plates/pool/sedimentary_m");
    let mafic_volcanic = get("plates/pool/mafic_volcanic_m");
    let mafic_plutonic = get("plates/pool/mafic_plutonic_m");
    let log = get("surface/sediment_m");
    let rock = get("structure/rock").or_else(|| get("surface/class"));
    let crystalline: Vec<bool> = crate::surface::lithology::table()
        .iter()
        .map(|c| c.source_family != 2)
        .collect();
    let lip = lip_min_m(world, r);
    let at = |v: Option<&[f32]>, c: usize| v.map_or(f32::NAN, |v| v[c]);
    let cells: Vec<Option<Cell>> = (0..n)
        .into_par_iter()
        .map(|c| {
            let continental = kind[c] >= 0.5;
            if !regime_f.data[c].is_finite() || (!continental && !age[c].is_finite()) {
                return None;
            }
            let pools = at(sediment_pool, c) + at(sedimentary_pool, c);
            let cover = match (log.map(|l| l[c]), pools.is_finite()) {
                (Some(l), true) if l.is_finite() => l.max(pools),
                (Some(l), false) if l.is_finite() => l,
                (_, true) => pools,
                _ => f32::NAN,
            };
            // Crust with a gabbro lower crust (5 km, against a continent's 4.5) is oceanic or was:
            // it carries its basalt, which is no province.
            let basalt = if mafic_plutonic.is_some() {
                r.oceanic_basalt_m * ((at(mafic_plutonic, c) - 4500.0) / 500.0).clamp(0.0, 1.0)
            } else if continental {
                0.0
            } else {
                r.oceanic_basalt_m
            };
            let cell = Cell {
                continental,
                regime: regime_f.data[c].round().max(0.0) as u8,
                age_myr: age[c],
                orogeny: at(orogeny, c).round().max(0.0) as u8,
                orogeny_age_myr: at(orogeny_age, c),
                elevation_m: at(elevation, c),
                thickness_m: at(thick, c),
                cover_m: cover,
                subsidence_mm_yr: -at(uplift, c),
                mafic_excess_m: at(mafic_volcanic, c) - basalt,
                crystalline_surface: rock.and_then(|f| {
                    let k = f[c];
                    (k.is_finite() && k >= 0.0)
                        .then(|| crystalline.get(k as usize).copied())
                        .flatten()
                }),
            };
            Some(cell)
        })
        .collect();
    Some((cells, lip, regime_f.grid.clone()))
}

/// Why continental cells are orogens (before the generalisation), by the first rule that makes
/// them one: an active arc or collision (`regime`), a collision belt from the last
/// `orogen_max_age_myr`, an Andean range; and how many continental cells there are.
pub fn orogen_reasons(world: &WorldState) -> Option<[usize; 4]> {
    orogen_reasons_with(world, &Rules::default())
}

/// [`orogen_reasons`] with the thresholds of `r`.
pub fn orogen_reasons_with(world: &WorldState, r: &Rules) -> Option<[usize; 4]> {
    let (cells, lip, _) = cells(world, r)?;
    let mut out = [0usize; 4];
    for c in cells.iter().flatten().filter(|c| c.continental) {
        out[3] += 1;
        if classify(c, lip, r) != OROGEN {
            continue;
        }
        if c.regime == regime::COLLISION || c.regime == regime::ARC {
            out[0] += 1;
        } else if c.orogeny == HIMALAYAN && c.orogeny_age_myr < r.orogen_max_age_myr {
            out[1] += 1;
        } else {
            out[2] += 1;
        }
    }
    Some(out)
}

/// Same-class patches smaller than this many cells are specks (P1-27; about 40,000 km² at 1024
/// columns on an Earth-sized planet).
pub const SPECK_CELLS: usize = 25;

/// How coherent the continental provinces are (P1-27, "large coherent cratons with platform
/// cover"): the continental classes' shares (true area), the share of neighbouring continental
/// cell pairs whose classes differ (edge density: low for broad provinces, high for a mosaic), and
/// the share of continental cells in same-class patches smaller than [`SPECK_CELLS`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Coherence {
    pub continental_cells: usize,
    /// Shield, platform, orogen, basin, LIP, extended, as shares of the continental area.
    pub shares: [f64; 6],
    pub edge_density: f64,
    pub speck_share: f64,
}

/// [`Coherence`] of a `provinces` layer on a W × H grid (rows from the north pole) whose rows
/// weigh `row_weight` each by true area (`GridDesc::row_weights`).
pub fn coherence(classes: &[f32], w: usize, h: usize, row_weight: &[f64]) -> Coherence {
    let continental = |v: f32| v.is_finite() && (v as u8) <= EXTENDED;
    let mut c = Coherence::default();
    let mut area = 0.0;
    let (mut pairs, mut differ) = (0usize, 0usize);
    for j in 0..h {
        let wgt = row_weight[j];
        for i in 0..w {
            let v = classes[j * w + i];
            if !continental(v) {
                continue;
            }
            c.continental_cells += 1;
            c.shares[v as usize] += wgt;
            area += wgt;
            for (ii, jj) in [((i + 1) % w, j), (i, j + 1)] {
                if jj < h && continental(classes[jj * w + ii]) {
                    pairs += 1;
                    if classes[jj * w + ii] != v {
                        differ += 1;
                    }
                }
            }
        }
    }
    for x in &mut c.shares {
        *x /= area.max(1e-12);
    }
    c.edge_density = differ as f64 / pairs.max(1) as f64;
    // Same-class patches (4-neighbour, x wraps).
    let mut seen = vec![false; w * h];
    let mut specks = 0usize;
    let mut stack = Vec::new();
    for start in 0..w * h {
        if seen[start] || !continental(classes[start]) {
            continue;
        }
        let k = classes[start];
        let mut members = 0usize;
        seen[start] = true;
        stack.push(start);
        while let Some(p) = stack.pop() {
            members += 1;
            let (i, j) = (p % w, p / w);
            let mut nb = vec![j * w + (i + 1) % w, j * w + (i + w - 1) % w];
            if j > 0 {
                nb.push((j - 1) * w + i);
            }
            if j + 1 < h {
                nb.push((j + 1) * w + i);
            }
            for q in nb {
                if !seen[q] && classes[q] == k {
                    seen[q] = true;
                    stack.push(q);
                }
            }
        }
        if members < SPECK_CELLS {
            specks += members;
        }
    }
    c.speck_share = specks as f64 / c.continental_cells.max(1) as f64;
    c
}

/// The coherence line of a world's provinces now (`None` without the plate fields).
pub fn coherence_line(world: &WorldState) -> Option<String> {
    coherence_line_with(world, &Rules::default())
}

/// [`coherence_line`] with the thresholds of `r`.
pub fn coherence_line_with(world: &WorldState, r: &Rules) -> Option<String> {
    let f = derive_with(world, r)?;
    let (w, h) = (f.grid.w as usize, f.grid.h as usize);
    if w * h != f.data.len() {
        return None;
    }
    let c = coherence(&f.data, w, h, &f.grid.row_weights());
    let why = orogen_reasons_with(world, r).unwrap_or_default();
    let of = |k: usize| why[k] as f64 / why[3].max(1) as f64;
    Some(format!(
        "provinces shield={:.3} platform={:.3} orogen={:.3} basin={:.3} lip={:.3} extended={:.3} \
         edge_density={:.3} speck_share={:.3} continental_cells={} \
         orogen_by(active/belt/andean)={:.3}/{:.3}/{:.3}",
        c.shares[0],
        c.shares[1],
        c.shares[2],
        c.shares[3],
        c.shares[4],
        c.shares[5],
        c.edge_density,
        c.speck_share,
        c.continental_cells,
        of(0),
        of(1),
        of(2)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use deeptime_format::GridDesc;

    fn land() -> Cell {
        Cell {
            continental: true,
            regime: regime::CRATON,
            age_myr: -1.0,
            orogeny: 0,
            orogeny_age_myr: f32::NAN,
            elevation_m: 300.0,
            thickness_m: 35_000.0,
            cover_m: 100.0,
            subsidence_mm_yr: 0.0,
            mafic_excess_m: 0.0,
            crystalline_surface: None,
        }
    }

    fn sea(age: f32) -> Cell {
        Cell {
            continental: false,
            regime: regime::OCEAN_BASIN,
            age_myr: age,
            elevation_m: -4000.0,
            thickness_m: 7_000.0,
            ..land()
        }
    }

    #[test]
    fn province_params_are_the_rules_and_the_builtins_carry_the_defaults() {
        let d = ProvinceParams::default();
        assert!(d.validate().is_ok());
        assert_eq!(
            d.rules(),
            Rules::default(),
            "the defaults are the view's rules"
        );
        let r = ProvinceParams {
            orogen_max_age_myr: 200.0,
            andean_max_age_myr: 25.0,
            andean_min_elevation_m: 2500.0,
            ..d
        }
        .rules();
        assert_eq!(
            (
                r.orogen_max_age_myr,
                r.andean_max_age_myr,
                r.andean_min_elevation_m
            ),
            (200.0, 25.0, 2500.0)
        );
        // Out-of-range values and unknown keys are refused, naming the key.
        let bad = ProvinceParams {
            min_area_km2: 10.0,
            ..d
        };
        assert!(
            bad.validate()
                .unwrap_err()
                .to_string()
                .contains("min_area_km2")
        );
        assert!(serde_json::from_str::<ProvinceParams>("{\"min_area\": 1}").is_err());
        // Every built-in preset names the defaults, so the editor shows them.
        for json in [
            crate::preset::builtin("earthlike_quick_16k"),
            crate::params::planet_builtin("pipeline_full_32k"),
        ] {
            let v: serde_json::Value = serde_json::from_str(json.expect("a built-in")).unwrap();
            assert_eq!(v["provinces"], serde_json::to_value(d).unwrap());
        }
    }

    #[test]
    fn every_class_has_a_rule() {
        let r = Rules::default();
        let k = |c: &Cell| classify(c, Some(300.0), &r);
        // Old crystalline craton under a thin veneer: a shield; under 1.5 km of sediment a
        // platform (a shield again where crystalline rock is at the surface); under 3 km, or
        // sinking under 1.2 km, a basin.
        assert_eq!(k(&land()), SHIELD);
        let platform = Cell {
            cover_m: 1500.0,
            ..land()
        };
        assert_eq!(k(&platform), PLATFORM);
        let crystalline = Cell {
            crystalline_surface: Some(true),
            ..platform
        };
        assert_eq!(k(&crystalline), SHIELD);
        let deep = Cell {
            cover_m: 3000.0,
            ..land()
        };
        assert_eq!(k(&deep), BASIN);
        let sinking = Cell {
            cover_m: 1200.0,
            subsidence_mm_yr: 0.1,
            ..land()
        };
        assert_eq!(k(&sinking), BASIN);
        let thin = Cell {
            cover_m: 100.0,
            ..sinking
        };
        assert_eq!(k(&thin), SHIELD);
        // Collision and arc are orogens whatever the cover; so is a collision belt from 300 Myr
        // ago but not one from 600 Myr ago; an Andean range only while young and high.
        let collision = Cell {
            regime: regime::COLLISION,
            cover_m: 4000.0,
            ..land()
        };
        assert_eq!(k(&collision), OROGEN);
        let arc = Cell {
            regime: regime::ARC,
            ..land()
        };
        assert_eq!(k(&arc), OROGEN);
        let belt = Cell {
            orogeny: HIMALAYAN,
            orogeny_age_myr: 300.0,
            ..land()
        };
        assert_eq!(k(&belt), OROGEN);
        let old_belt = Cell {
            orogeny_age_myr: 600.0,
            ..belt
        };
        assert_eq!(k(&old_belt), SHIELD);
        let andes = Cell {
            orogeny: ANDEAN,
            orogeny_age_myr: 20.0,
            elevation_m: 3000.0,
            ..land()
        };
        assert_eq!(k(&andes), OROGEN);
        let low = Cell {
            elevation_m: 800.0,
            ..andes
        };
        assert_eq!(k(&low), SHIELD, "a low back-arc is not a range");
        let stale = Cell {
            orogeny_age_myr: 90.0,
            ..andes
        };
        assert_eq!(k(&stale), SHIELD, "an old Andean stamp is not a range");
        // Rifts, passive margins and thinned crust are extended, ahead of basins.
        let rift = Cell {
            regime: regime::RIFT,
            cover_m: 4000.0,
            ..land()
        };
        assert_eq!(k(&rift), EXTENDED);
        // A passive-margin sample alone is no longer extended crust: the rifted margins are a
        // band drawn after the rules (`rifted_margins`).
        let margin = Cell {
            regime: regime::PASSIVE_MARGIN,
            ..land()
        };
        assert_eq!(k(&margin), SHIELD);
        let thinned = Cell {
            thickness_m: 15_000.0,
            ..land()
        };
        assert_eq!(k(&thinned), EXTENDED);
        // Oceanic crust by age, at the USGS edges; an island arc is an orogen.
        assert_eq!(k(&sea(0.0)), OCEAN_YOUNG);
        assert_eq!(k(&sea(19.9)), OCEAN_YOUNG);
        assert_eq!(k(&sea(20.0)), OCEAN_MID);
        assert_eq!(k(&sea(64.9)), OCEAN_MID);
        assert_eq!(k(&sea(65.0)), OCEAN_OLD);
        let island_arc = Cell {
            regime: regime::ARC,
            thickness_m: 12_000.0,
            ..sea(40.0)
        };
        assert_eq!(k(&island_arc), OROGEN);
        let bare_arc = Cell {
            thickness_m: 7_500.0,
            ..island_arc
        };
        assert_eq!(
            k(&bare_arc),
            OCEAN_MID,
            "an arc zone without a built arc is sea floor"
        );
        // A LIP wins over everything, on land and at sea, and only when the run recorded one.
        let lava = Cell {
            mafic_excess_m: 400.0,
            ..land()
        };
        assert_eq!(k(&lava), LIP);
        let plateau = Cell {
            mafic_excess_m: 400.0,
            ..sea(100.0)
        };
        assert_eq!(k(&plateau), LIP);
        assert_eq!(classify(&lava, None, &r), SHIELD);
        // An unknown value never triggers a rule.
        let unknown = Cell {
            orogeny_age_myr: f32::NAN,
            elevation_m: f32::NAN,
            thickness_m: f32::NAN,
            cover_m: f32::NAN,
            subsidence_mm_yr: f32::NAN,
            mafic_excess_m: f32::NAN,
            ..land()
        };
        assert_eq!(k(&unknown), PLATFORM);
    }

    #[test]
    fn generalise_removes_specks_and_keeps_the_rest() {
        // 8 × 5: a continent of platform (1) with one basin speck (3) inside and a 2-wide strip
        // of extended crust (5) on its east side, a LIP (4) and the sea (8) around.
        let (w, h) = (8, 5);
        let mut c = vec![f32::from(OCEAN_OLD); w * h];
        for j in 1..4 {
            for i in 1..6 {
                c[j * w + i] = f32::from(PLATFORM);
            }
            c[j * w + 4] = f32::from(EXTENDED);
            c[j * w + 5] = f32::from(EXTENDED);
        }
        c[2 * w + 2] = f32::from(BASIN);
        c[w + 1] = f32::from(LIP);
        let g = generalise(&c, w, h, 1);
        assert_eq!(
            g[2 * w + 2],
            f32::from(PLATFORM),
            "the speck joins its surroundings"
        );
        assert_eq!(
            g[2 * w + 4],
            f32::from(EXTENDED),
            "a strip two cells wide stays"
        );
        assert_eq!(g[2 * w + 5], f32::from(EXTENDED));
        assert_eq!(g[w + 1], f32::from(LIP), "a LIP is not generalised");
        assert_eq!(g[0], f32::from(OCEAN_OLD), "the sea is not generalised");
        let mut nan = c.clone();
        nan[0] = f32::NAN;
        assert!(generalise(&nan, w, h, 1)[0].is_nan());
        // A wrapped world: the strip across the seam is one region.
        let mut wrap = vec![f32::from(OCEAN_OLD); w * h];
        for j in 0..h {
            for i in [7usize, 0, 1] {
                wrap[j * w + i] = f32::from(SHIELD);
            }
        }
        wrap[2 * w] = f32::from(BASIN);
        assert_eq!(generalise(&wrap, w, h, 1)[2 * w], f32::from(SHIELD));
    }

    #[test]
    fn derive_reads_the_world_and_writes_nothing() {
        let mut world = WorldState::new(1, 1024, -1.0e6);
        assert!(derive(&world).is_none(), "no plate fields yet");
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let n = grid.cells();
        let w = grid.w as usize;
        let put = |world: &mut WorldState, name: &str, west: f32, east: f32| {
            let data = (0..n)
                .map(|c| if c % w < w / 2 { west } else { east })
                .collect();
            world.fields.insert(
                name.into(),
                Field {
                    grid: grid.clone(),
                    data,
                },
            );
        };
        // The western half is continental craton under 3 km of sedimentary rock, the eastern
        // half 50 Myr old sea floor.
        put(&mut world, "regime", 8.0, 0.0);
        put(&mut world, "plates/crust_type", 1.0, 0.0);
        put(&mut world, "crust_age", -1.0, 50.0);
        put(&mut world, "plates/thickness_m", 35_000.0, 7_000.0);
        put(&mut world, "plates/pool/sediment_m", 0.0, 0.0);
        put(&mut world, "plates/pool/sedimentary_m", 3000.0, 0.0);
        put(&mut world, "plates/pool/mafic_plutonic_m", 4500.0, 5000.0);
        put(&mut world, "plates/pool/mafic_volcanic_m", 0.0, 2000.0);
        let before = world.clone();
        let f = derive(&world).unwrap();
        assert_eq!(world, before, "the derivation reads only");
        let inland = (grid.h as usize / 2) * w + w / 4;
        assert_eq!(f.data[inland], f32::from(BASIN));
        assert_eq!(
            f.data[0],
            f32::from(BASIN),
            "at 5.6° a cell, the 1.35° rifted margin is less than a cell"
        );
        assert_eq!(f.data[n - 1], f32::from(OCEAN_MID));
        // A recorded LIP of 1.5 km makes 500 m of basalt on a continent, or 500 m above the
        // ordinary 2 km on the sea floor, a province; the sea floor's ordinary basalt is not.
        put(&mut world, "plates/pool/mafic_volcanic_m", 500.0, 2500.0);
        assert_eq!(
            f32::from(BASIN),
            derive(&world).unwrap().data[inland],
            "no LIP event yet"
        );
        let e = world
            .event(
                -1.0e6,
                crate::plates::events::LARGE_IGNEOUS_PROVINCE,
                "deeptime:test",
            )
            .param("thickness_m", 1500.0);
        world.emit(e).unwrap();
        let f = derive(&world).unwrap();
        assert_eq!(f.data[inland], f32::from(LIP));
        assert_eq!(f.data[n - 1], f32::from(LIP));
        put(&mut world, "plates/pool/mafic_volcanic_m", 0.0, 2000.0);
        assert_eq!(derive(&world).unwrap().data[n - 1], f32::from(OCEAN_MID));
    }

    #[test]
    fn ramp_labels_and_colours_match_the_classes() {
        let ramps = deeptime_format::Ramps::builtin();
        let (name, ramp) = ramps.for_layer(LAYER);
        assert_eq!(name, "provinces");
        let json: serde_json::Value =
            serde_json::from_str(deeptime_format::ramps::RAMPS_JSON).unwrap();
        let labels: Vec<&str> = json["ramps"]["provinces"]["labels"]
            .as_array()
            .expect("the ramp is labelled")
            .iter()
            .map(|l| l.as_str().unwrap())
            .collect();
        assert_eq!(labels, CLASSES);
        // Every class has its own colour, and NaN is transparent.
        let colours: std::collections::BTreeSet<[u8; 4]> =
            (0..CLASSES.len()).map(|k| ramp.rgba(k as f64)).collect();
        assert_eq!(colours.len(), CLASSES.len());
        assert_eq!(ramp.rgba(f64::NAN)[3], 0);
    }

    #[test]
    fn rifted_margins_are_a_band_on_quiet_coasts_only() {
        // 12 × 6: platform west of column 8, sea east of it; the northern half's coast is
        // quiet (craton regime), the southern half's faces a trench (back-arc).
        let (w, h) = (12, 6);
        let mut c = vec![f32::from(OCEAN_OLD); w * h];
        let mut reg = vec![regime::OCEAN_BASIN; w * h];
        for j in 0..h {
            for i in 0..8 {
                c[j * w + i] = f32::from(PLATFORM);
                reg[j * w + i] = if j < 3 {
                    regime::CRATON
                } else {
                    regime::BACK_ARC
                };
            }
            // The seam: column 0 borders column 11 (sea).
        }
        let b = rifted_margins(&c, &reg, w, h, 2);
        for j in 0..3 {
            assert_eq!(b[j * w + 7], f32::from(EXTENDED), "quiet coast, first cell");
            assert_eq!(b[j * w + 6], f32::from(EXTENDED), "second cell");
            assert_eq!(b[j * w + 5], f32::from(PLATFORM), "inland");
            assert_eq!(b[j * w], f32::from(EXTENDED), "across the seam");
        }
        for j in 3..h {
            assert_eq!(b[j * w + 7], f32::from(PLATFORM), "an active coast stays");
        }
        assert_eq!(b[11], f32::from(OCEAN_OLD), "the sea stays");
    }

    #[test]
    fn specks_below_the_feature_size_go_to_the_sea() {
        let (w, h) = (10, 6);
        let mut c = vec![f32::from(OCEAN_MID); w * h];
        // A 3-cell speck and a 12-cell landmass.
        for p in [2 * w + 1, 2 * w + 2, 3 * w + 1] {
            c[p] = f32::from(EXTENDED);
        }
        for j in 1..5 {
            for i in 5..8 {
                c[j * w + i] = f32::from(SHIELD);
            }
        }
        let d = drop_specks(&c, w, h, 10);
        assert_eq!(d[2 * w + 1], f32::from(OCEAN_MID));
        assert_eq!(d[3 * w + 1], f32::from(OCEAN_MID));
        assert_eq!(
            d[2 * w + 6],
            f32::from(SHIELD),
            "a landmass at the feature size stays"
        );
        let co = coherence(&d, w, h, &vec![1.0; h]);
        assert_eq!(co.continental_cells, 12);
        assert!((co.shares[SHIELD as usize] - 1.0).abs() < 1e-12);
        assert_eq!(co.edge_density, 0.0);
        assert_eq!(co.speck_share, 1.0, "12 cells is below SPECK_CELLS");
    }
}
