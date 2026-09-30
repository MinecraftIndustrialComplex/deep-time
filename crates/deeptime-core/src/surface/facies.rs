// SPDX-License-Identifier: LGPL-3.0-or-later
//! Facies rules: the rock class of a step's deposit in a cell (plan §8.2). The rules are the
//! research's proposal (`surface_processes_climate.md` §4 — "a proposal, not a published scheme"):
//! coarse fluvial → sandstone or conglomerate, offshore or lacustrine fines → shale or mudstone,
//! photic-zone carbonate → limestone, arid closed basin → evaporite, humid subsiding swamp → coal,
//! duricrust → laterite (the regolith's kind, `weathering.rs`). Deep Time adds red beds (arid
//! warm floodplains), dolostone (arid tidal flats), black shale (restricted deep water: the
//! euxinic basins of the deposit engine), and splits evaporites into halite and gypsum by
//! aridity. Every number below is a design value, documented here and tuned in P1-07.
//!
//! A step's deposit takes one class: the process that laid down most of it decides (evaporite,
//! carbonate, lake, sea or river), then the setting.

use super::lithology::Classes;

/// What a deposit counts as for the deposit engine's facies fields (`fields.rs`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Clastic,
    Carbonate,
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
    pub carbonate: f64,
    pub evaporite: f64,
    /// Till and glaciomarine diamict (`glacial.rs`), m.
    pub glacial: f64,
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
        (s.carbonate, 1),
        (s.lake, 2),
        (s.marine_clastic, 3),
        (s.fluvial, 4),
        (s.glacial, 5),
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
    match main {
        0 if total > 0.0 => evaporite(s.aridity),
        5 => (c.till, Kind::Clastic),
        1 => {
            if s.aridity < EVAPORITE_ARIDITY && s.depth_m < DOLOMITE_DEPTH_M {
                (c.dolostone, Kind::Carbonate)
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
            } else if s.restricted && s.depth_m > EUXINIC_DEPTH_M && s.t_c > EUXINIC_T_C {
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
    }
}
