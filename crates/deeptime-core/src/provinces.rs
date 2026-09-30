// SPDX-License-Identifier: LGPL-3.0-or-later
//! The "Geologic provinces" view layer (decisions log 2026-09-29): a USGS-style classification of
//! every L1 cell, drawn as one more keyframe layer (`provinces`), so the workbench and the static
//! viewer show it at every time step, like the USGS map "Geologic provinces of the world" (shield,
//! platform, orogen, basin, large igneous province, extended crust, oceanic crust by age).
//!
//! **View only.** The layer is derived from fields the stages already wrote, at render time, into
//! a scratch field: nothing is written to `WorldState`, so a run's world files (the bake, its
//! `meta/province`, the deposits) are what they were. The bake's own provinces
//! (`bake/meta.rs`: settings of the plate regime, which drive TerraBlender's regions) are a
//! different thing and stay as they are.
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
//! 4. **Extended crust**: continental crust in a rift or on a passive margin (`regime`), or
//!    thinned below 20 km (`plates/thickness_m`): rifts, the Basin-and-Range kind of province and
//!    the thinned margins.
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
//! the majority class of the continental cells around it, so single-cell specks vanish.
//!
//! Continental crust has no formation age in the plate model, only its last orogeny; whatever no
//! rule 3 caught is taken to be old (Precambrian) crust.

use deeptime_format::ParamValue;

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
            ocean_young_myr: 20.0,
            ocean_mid_myr: 65.0,
            island_arc_min_thickness_m: 9_000.0,
            orogen_max_age_myr: 500.0,
            andean_max_age_myr: 50.0,
            andean_min_elevation_m: 1500.0,
            thinned_crust_m: 20_000.0,
            basin_cover_m: 3_000.0,
            subsiding_mm_yr: 0.03,
            subsiding_cover_m: 1_000.0,
            shield_cover_m: 200.0,
            lip_fraction: 0.2,
            oceanic_basalt_m: crate::plates::pools::oceanic_column()
                [crate::plates::state::pool::MAFIC_VOLCANIC],
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
    if c.regime == regime::RIFT
        || c.regime == regime::PASSIVE_MARGIN
        || c.thickness_m < r.thinned_crust_m
    {
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
    for j in 0..h {
        for i in 0..w {
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
                out[j * w + i] = k as f32;
            }
        }
    }
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
    let r = Rules::default();
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
    let lip = lip_min_m(world, &r);
    let at = |v: Option<&[f32]>, c: usize| v.map_or(f32::NAN, |v| v[c]);
    let raw: Vec<f32> = (0..n)
        .map(|c| {
            let continental = kind[c] >= 0.5;
            if !regime_f.data[c].is_finite() || (!continental && !age[c].is_finite()) {
                return f32::NAN;
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
            f32::from(classify(&cell, lip, &r))
        })
        .collect();
    let (w, h) = (regime_f.grid.w as usize, regime_f.grid.h as usize);
    let data = if w * h == n {
        generalise(&raw, w, h, GENERALISE_CELLS)
    } else {
        raw
    };
    Some(Field {
        grid: regime_f.grid.clone(),
        data,
    })
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
        let margin = Cell {
            regime: regime::PASSIVE_MARGIN,
            ..land()
        };
        assert_eq!(k(&margin), EXTENDED);
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
        assert_eq!(f.data[0], f32::from(BASIN));
        assert_eq!(f.data[n - 1], f32::from(OCEAN_MID));
        // A recorded LIP of 1.5 km makes 500 m of basalt on a continent, or 500 m above the
        // ordinary 2 km on the sea floor, a province; the sea floor's ordinary basalt is not.
        put(&mut world, "plates/pool/mafic_volcanic_m", 500.0, 2500.0);
        assert_eq!(
            f32::from(BASIN),
            derive(&world).unwrap().data[0],
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
        assert_eq!(f.data[0], f32::from(LIP));
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
}
