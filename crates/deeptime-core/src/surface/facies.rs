// SPDX-License-Identifier: LGPL-3.0-or-later
//! Facies rules: the rock class of a step's deposit in a cell (plan §8.2). The rules are the
//! research's proposal (`surface_processes_climate.md` §4 — "a proposal, not a published scheme"):
//! coarse fluvial → sandstone or conglomerate, offshore or lacustrine fines → shale or mudstone,
//! photic-zone carbonate → limestone, arid closed basin → evaporite, humid subsiding swamp → coal,
//! duricrust → laterite (the regolith's kind, `weathering.rs`). Deep Time adds red beds (arid
//! warm floodplains), dolostone (arid tidal flats), black shale (restricted deep water: the
//! euxinic basins of the deposit engine), and splits evaporites into halite and gypsum by
//! aridity. Every number below is a design value, documented here and tuned in P1-07.
//! Pelagic chalk (2026-10-19): a deposit that is mostly the open sea's calcareous ooze
//! (`pelagic::Rain`) is chalk. Periplatform carbonate (2026-10-20) votes with the factory's
//! for the class but, laid mostly below the photic zone, is no platform (`Kind::Periplatform`);
//! euxinic water makes black shale of carbonate ooze too.
//! The pelagic rain's opal and dust (2026-10-26, `pelagic.rs`): a deposit that is mostly opal is
//! chert (siliceous ooze lithifies to chert), one that is mostly dust is shale (the deep sea's
//! clay), black shale in euxinic water as the other muds and oozes.
//!
//! A step's deposit takes one class: the process that laid down most of it decides (evaporite,
//! carbonate, lake, sea, river, glacier or the pelagic rain's chalk, opal or dust), then the
//! setting.

use super::carbonate::PHOTIC_M;
use super::lithology::Classes;

/// What a deposit counts as for the deposit engine's facies fields (`fields.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Clastic,
    Carbonate,
    /// Pelagic ooze, chalk or chert: no platform (the deposit engine's `carbonate_platform`
    /// field leaves it out).
    Pelagic,
    /// Periplatform limestone: carbonate the marine diffusion carried off a platform and laid
    /// below the photic zone, on its slope or the floor around it. Carbonate, but no platform,
    /// as [`Kind::Pelagic`].
    Periplatform,
    Evaporite,
    /// Organic-rich black shale in restricted deep water.
    Euxinic,
    /// Peat mire.
    Coal,
}

/// The setting of one cell over one step.
#[derive(Clone, Copy, Debug, Default)]
pub struct Setting {
    /// Under the sea at the end of the step.
    pub marine: bool,
    /// Water depth, m (positive below sea level).
    pub depth_m: f64,
    /// Local slope (m/m).
    pub slope: f64,
    pub t_c: f64,
    /// P / PET (below 0.2 hyper-arid, above 1 humid).
    pub aridity: f64,
    /// In a restricted sea (little open ocean around it).
    pub restricted: bool,
    /// Thickness laid down this step by rivers, lakes, the sea's clastic delivery, the carbonate
    /// factory and evaporation, m.
    pub fluvial: f64,
    pub lake: f64,
    pub marine_clastic: f64,
    /// The carbonate factory's own deposit, m.
    pub carbonate: f64,
    /// Periplatform carbonate (`carbonate::exported`): the factory's carbonate the marine
    /// diffusion moved here from another cell, m. It votes with `carbonate` for the class.
    pub periplatform: f64,
    pub evaporite: f64,
    /// Till and glaciomarine diamict (`glacial.rs`), m.
    pub glacial: f64,
    /// Pelagic carbonate ooze (the pelagic rain's chalk, `pelagic.rs`), m.
    pub pelagic: f64,
    /// The pelagic rain's opal and dust, m.
    pub siliceous: f64,
    pub dust: f64,
    /// Net deposition rate, m/Myr.
    pub rate_m_myr: f64,
    /// Land plants exist (the atmosphere clock; coal needs them).
    pub land_plants: bool,
}

/// Design thresholds of the rules.
pub mod limits {
    /// Shoreface depth: shallower marine clastics are sands, m.
    pub const SHOREFACE_M: f64 = 20.0;
    /// Restricted water deeper than this makes black shale, m …
    pub const EUXINIC_DEPTH_M: f64 = 100.0;
    /// … and warmer than this, °C.
    pub const EUXINIC_T_C: f64 = 10.0;
    /// P/PET below which evaporites form (restricted seas, closed lakes) …
    pub const EVAPORITE_ARIDITY: f64 = 0.3;
    /// … and below which they are halite rather than gypsum.
    pub const HALITE_ARIDITY: f64 = 0.15;
    /// Tidal-flat carbonates shallower than this in arid climates become dolostone, m.
    pub const DOLOMITE_DEPTH_M: f64 = 10.0;
    /// Coal: humid (P/PET above), warm enough, nearly flat, and accommodation in this band (m/Myr).
    pub const COAL_ARIDITY: f64 = 1.2;
    pub const COAL_T_C: f64 = 5.0;
    pub const COAL_SLOPE: f64 = 0.002;
    pub const COAL_RATE_M_MYR: (f64, f64) = (5.0, 150.0);
    /// Fans: steeper fluvial deposits are conglomerate.
    pub const CONGLOMERATE_SLOPE: f64 = 0.02;
    /// Red beds: arid-to-semi-arid warm floodplains.
    pub const RED_BED_ARIDITY: f64 = 0.5;
    pub const RED_BED_T_C: f64 = 10.0;
}

/// Whether the peat-mire conditions hold (coal facies), whatever the deposit's class.
pub fn peat_mire(s: &Setting) -> bool {
    use limits::*;
    !s.marine
        && s.land_plants
        && s.aridity > COAL_ARIDITY
        && s.t_c > COAL_T_C
        && s.slope < COAL_SLOPE
        && (COAL_RATE_M_MYR.0..=COAL_RATE_M_MYR.1).contains(&s.rate_m_myr)
}

/// The class and kind of the deposit laid down in setting `s`.
pub fn classify(s: &Setting, c: &Classes) -> (u16, Kind) {
    use limits::*;
    let parts = [
        (s.evaporite, 0),
        (s.carbonate + s.periplatform, 1),
        (s.lake, 2),
        (s.marine_clastic, 3),
        (s.fluvial, 4),
        (s.glacial, 5),
        (s.pelagic, 6),
        (s.siliceous, 7),
        (s.dust, 8),
    ];
    let total: f64 = parts.iter().map(|p| p.0.max(0.0)).sum();
    let main = parts
        .iter()
        .copied()
        .max_by(|a, b| a.0.total_cmp(&b.0).then(b.1.cmp(&a.1)))
        .map_or(4, |p| p.1);
    let evaporite = |a: f64| {
        if a < HALITE_ARIDITY {
            (c.halite, Kind::Evaporite)
        } else {
            (c.gypsum, Kind::Evaporite)
        }
    };
    // Restricted, deep, warm water (ledger ERO-088): its anoxic floor keeps the organic matter
    // whatever the rain is, mud or carbonate ooze, so the deposit is black shale even where the
    // pelagic or periplatform carbonate outweighs the mud (ledger ERO-196).
    let euxinic = s.restricted && s.depth_m > EUXINIC_DEPTH_M && s.t_c > EUXINIC_T_C;
    match main {
        0 if total > 0.0 => evaporite(s.aridity),
        5 => (c.till, Kind::Clastic),
        6 if euxinic => (c.black_shale, Kind::Euxinic), // ledger: ERO-196
        6 => (c.chalk, Kind::Pelagic),                  // ledger: ERO-194
        7 | 8 if euxinic => (c.black_shale, Kind::Euxinic), // ledger: ERO-216
        7 => (c.chert, Kind::Pelagic),                      // ledger: ERO-216
        8 => (c.shale, Kind::Clastic),                      // ledger: ERO-216
        1 => {
            if s.aridity < EVAPORITE_ARIDITY && s.depth_m < DOLOMITE_DEPTH_M {
                (c.dolostone, Kind::Carbonate)
            } else if euxinic {
                (c.black_shale, Kind::Euxinic) // ledger: ERO-196
            } else if s.periplatform > s.carbonate && s.depth_m > PHOTIC_M {
                // Mostly carbonate carried here, below the factory's photic zone: the slope's
                // or the basin's lime mud, not a platform (ledger ERO-197).
                (c.limestone, Kind::Periplatform)
            } else {
                (c.limestone, Kind::Carbonate)
            }
        }
        2 => {
            if s.aridity < EVAPORITE_ARIDITY {
                evaporite(s.aridity)
            } else if peat_mire(s) {
                (c.coal, Kind::Coal)
            } else {
                (c.mudstone, Kind::Clastic)
            }
        }
        3 => {
            if s.depth_m < SHOREFACE_M {
                (c.sandstone, Kind::Clastic)
            } else if euxinic {
                (c.black_shale, Kind::Euxinic)
            } else {
                (c.shale, Kind::Clastic)
            }
        }
        _ => {
            if s.marine {
                // Diffusion moved the sediment here: offshore fines.
                if s.depth_m < SHOREFACE_M {
                    (c.sandstone, Kind::Clastic)
                } else {
                    (c.shale, Kind::Clastic)
                }
            } else if peat_mire(s) {
                (c.coal, Kind::Coal)
            } else if s.slope > CONGLOMERATE_SLOPE {
                (c.conglomerate, Kind::Clastic)
            } else if s.aridity < RED_BED_ARIDITY && s.t_c > RED_BED_T_C {
                (c.red_bed, Kind::Clastic)
            } else {
                (c.sandstone, Kind::Clastic)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rules_pick_the_expected_classes() {
        let c = Classes::get();
        let base = Setting {
            t_c: 25.0,
            aridity: 1.0,
            land_plants: true,
            ..Setting::default()
        };
        let reef = Setting {
            marine: true,
            depth_m: 30.0,
            carbonate: 20.0,
            marine_clastic: 1.0,
            ..base
        };
        assert_eq!(classify(&reef, &c), (c.limestone, Kind::Carbonate));
        let sabkha = Setting {
            depth_m: 2.0,
            aridity: 0.1,
            ..reef
        };
        assert_eq!(classify(&sabkha, &c).0, c.dolostone);
        let salt = Setting {
            evaporite: 50.0,
            aridity: 0.1,
            ..reef
        };
        assert_eq!(classify(&salt, &c).0, c.halite);
        let offshore = Setting {
            marine: true,
            depth_m: 150.0,
            marine_clastic: 10.0,
            ..base
        };
        assert_eq!(classify(&offshore, &c).0, c.shale);
        let euxinic = Setting {
            restricted: true,
            ..offshore
        };
        assert_eq!(classify(&euxinic, &c), (c.black_shale, Kind::Euxinic));
        let swamp = Setting {
            fluvial: 30.0,
            slope: 0.0005,
            aridity: 1.6,
            rate_m_myr: 20.0,
            ..base
        };
        assert_eq!(classify(&swamp, &c), (c.coal, Kind::Coal));
        let fan = Setting {
            fluvial: 30.0,
            slope: 0.05,
            ..base
        };
        assert_eq!(classify(&fan, &c).0, c.conglomerate);
        let desert = Setting {
            fluvial: 30.0,
            slope: 0.001,
            aridity: 0.3,
            ..base
        };
        assert_eq!(classify(&desert, &c).0, c.red_bed);
        let playa = Setting {
            lake: 10.0,
            aridity: 0.1,
            ..base
        };
        assert_eq!(classify(&playa, &c).0, c.halite);
        // A moraine: melt-out debris outweighs the rivers' sand.
        let moraine = Setting {
            fluvial: 5.0,
            glacial: 40.0,
            t_c: -3.0,
            ..base
        };
        assert_eq!(classify(&moraine, &c), (c.till, Kind::Clastic));
        // The open sea floor: chalk where the ooze outweighs the mud, shale near the margin.
        let ooze = Setting {
            marine: true,
            depth_m: 3000.0,
            pelagic: 8.0,
            marine_clastic: 1.0,
            ..base
        };
        assert_eq!(classify(&ooze, &c), (c.chalk, Kind::Pelagic));
        let rise = Setting {
            marine_clastic: 20.0,
            ..ooze
        };
        assert_eq!(classify(&rise, &c).0, c.shale);
    }

    /// Euxinic water vetoes chalk and periplatform limestone: a restricted, deep, warm basin
    /// lays black shale whatever outweighs the mud (ERO-196). The open sea keeps its chalk; a
    /// cold restricted sea too.
    #[test]
    fn euxinic_water_makes_black_shale_of_carbonate_ooze() {
        let c = Classes::get();
        let basin = Setting {
            marine: true,
            depth_m: 800.0,
            t_c: 15.0,
            aridity: 1.0,
            restricted: true,
            pelagic: 3.0,
            marine_clastic: 0.5,
            ..Setting::default()
        };
        assert_eq!(classify(&basin, &c), (c.black_shale, Kind::Euxinic));
        let open = Setting {
            restricted: false,
            ..basin
        };
        assert_eq!(classify(&open, &c), (c.chalk, Kind::Pelagic));
        let cold = Setting { t_c: 4.0, ..basin };
        assert_eq!(classify(&cold, &c), (c.chalk, Kind::Pelagic));
        let shed = Setting {
            pelagic: 0.0,
            periplatform: 6.0,
            ..basin
        };
        assert_eq!(classify(&shed, &c), (c.black_shale, Kind::Euxinic));
    }

    /// The pelagic rain's opal makes chert and its dust shale where they outweigh the rest; in
    /// euxinic water both are black shale (ERO-216).
    #[test]
    fn the_rains_opal_is_chert_and_its_dust_shale() {
        let c = Classes::get();
        let floor = Setting {
            marine: true,
            depth_m: 5200.0,
            t_c: 2.0,
            aridity: 1.0,
            siliceous: 0.6,
            dust: 0.4,
            pelagic: 0.1,
            ..Setting::default()
        };
        assert_eq!(classify(&floor, &c), (c.chert, Kind::Pelagic));
        let red_clay = Setting {
            siliceous: 0.2,
            ..floor
        };
        assert_eq!(classify(&red_clay, &c), (c.shale, Kind::Clastic));
        let restricted = Setting {
            restricted: true,
            t_c: 15.0,
            depth_m: 800.0,
            ..floor
        };
        assert_eq!(classify(&restricted, &c), (c.black_shale, Kind::Euxinic));
        let chalk = Setting {
            pelagic: 2.0,
            ..floor
        };
        assert_eq!(classify(&chalk, &c), (c.chalk, Kind::Pelagic));
    }

    /// Periplatform carbonate votes with the factory's for the class (limestone either way), but
    /// a deposit that is mostly carbonate carried below the photic zone is no platform (ERO-197);
    /// on the platform top it stays one.
    #[test]
    fn periplatform_limestone_is_no_platform() {
        let c = Classes::get();
        let slope = Setting {
            marine: true,
            depth_m: 400.0,
            t_c: 20.0,
            aridity: 1.0,
            carbonate: 0.1,
            periplatform: 2.0,
            marine_clastic: 1.5,
            ..Setting::default()
        };
        // 2.1 m of carbonate outvote 1.5 m of mud only together.
        assert_eq!(classify(&slope, &c), (c.limestone, Kind::Periplatform));
        let top = Setting {
            depth_m: 30.0,
            ..slope
        };
        assert_eq!(classify(&top, &c), (c.limestone, Kind::Carbonate));
        let factory = Setting {
            carbonate: 3.0,
            ..slope
        };
        assert_eq!(classify(&factory, &c), (c.limestone, Kind::Carbonate));
    }
}
