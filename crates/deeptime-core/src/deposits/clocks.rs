// SPDX-License-Identifier: LGPL-3.0-or-later
//! The secular clocks (plan §8.6 item 1; `fields::CLOCKS`): global switches that open and close
//! deposit types over Earth's history. Each clock is dated in Earth's history, in Ga before
//! present, and applied to a planet by its age: a planet `a` Gyr old stands where Earth stood
//! [`EARTH_AGE_GYR`] − `a` Ga ago. The planet's age at time `t` (years, negative = past) is
//! `climate.present_age_gyr + t / 10⁹` (the same age the brightening star uses, `climate::clock`).
//! With the default present age (4.57 Gyr, Earth's) a frame at `t` is simply −t/10⁹ Ga ago.
//!
//! | clock | value | Earth's dates |
//! |---|---|---|
//! | `oxygenated_oceans` | 1 after [`ZN_PB_GA`] | sediment-hosted Zn–Pb (MVT and clastic-dominated) first appear at 2.02 Ga (Leach et al. 2010, Econ. Geol. 105: 593) |
//! | `land_plants` | 1 after [`LAND_PLANTS_GA`] | coal occurs by the Early Devonian (Nelsen et al. 2016, PNAS 113: 2442); the base of the Devonian is 419.2 ± 3.2 Ma (ICS chart 2023/09) |
//! | `bif_acme` | 1 inside [`BIF_ACME_GA`] | Superior-type BIF first appears ca. 2.6 Ga and iron formations disappear at ca. 1.85 Ga (Bekker et al. 2010, Econ. Geol. 105: 467, abstract) |
//!
//! A frame's clock is the share of its span inside the window (frames are time averages), so a
//! frame that straddles a date carries a fraction. Before stage 2 (WP-9/10, 2026-10-02) the
//! recorded pipeline published the fixed stand-ins 1, 1, 0 and never dated a frame: no BIF
//! source on any history, and no date limit on coal or sediment-hosted Zn–Pb on long runs.
//!
//! **Inherited BIF** ([`inherited_bif_share`]): when the run starts after (part of) the acme, the
//! stable continental interiors inherit banded iron formation in their basement (plan §8.6 item 7:
//! "inherited cratonic basement where the timeline starts after the acme"). The share is the part
//! of the acme window that lies before the run's start, times the preset's
//! `deposits.inherited_bif_share` (0 in the built-in presets until the owner decides how much of a
//! stable interior holds it; ledger ORE-261); `surface::fields` multiplies it by the frame's craton
//! share, and `structure` lets the basement count as the BIF host where it is.

/// Earth's age today, Gyr (the planet's age at which a clock's "Ga before present" is today).
pub const EARTH_AGE_GYR: f64 = 4.57; // ledger: ORE-258
/// Sediment-hosted Zn–Pb deposits (MVT and clastic-dominated/SEDEX) appear after this, Ga.
pub const ZN_PB_GA: f64 = 2.02; // ledger: ORE-014
/// Coal-forming land plants exist after this, Ga (the base of the Devonian).
pub const LAND_PLANTS_GA: f64 = 0.419; // ledger: ORE-015
/// The banded-iron-formation acme, Ga (older, younger).
pub const BIF_ACME_GA: (f64, f64) = (2.6, 1.85); // ledger: ORE-016

/// Clock names, as `fields::CLOCKS` lists them.
pub const OXYGENATED_OCEANS: &str = "deeptime:clock/oxygenated_oceans";
pub const LAND_PLANTS: &str = "deeptime:clock/land_plants";
pub const BIF_ACME: &str = "deeptime:clock/bif_acme";

/// Where a run sits in Earth's history.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Epoch {
    /// The planet's age at t = 0 (the end of the run), Gyr (`climate.present_age_gyr`).
    pub present_age_gyr: f64,
    /// The run's first instant, years (negative).
    pub run_start_years: f64,
    /// The preset's `deposits.inherited_bif_share` (0–1).
    pub inherited_bif: f64,
}

impl Epoch {
    /// The epoch of a preset's run: its climate's present age (the default's without a climate
    /// section) and its timeline's start.
    pub fn of_preset(p: &crate::params::PlanetPreset) -> Epoch {
        Epoch {
            present_age_gyr: p.climate.as_ref().map_or_else(
                || crate::climate::ClimateParams::default().present_age_gyr,
                |c| c.present_age_gyr,
            ),
            run_start_years: p.t_start_years(),
            inherited_bif: p.deposits.as_ref().map_or(0.0, |d| d.inherited_bif_share),
        }
    }

    /// Earth's "Ga before present" matching time `t_years` of this planet.
    pub fn ga(&self, t_years: f64) -> f64 {
        // ledger: ORE-259
        EARTH_AGE_GYR - (self.present_age_gyr + t_years / 1.0e9)
    }

    /// Whether coal-forming land plants exist at `t_years`.
    pub fn land_plants_at(&self, t_years: f64) -> bool {
        self.ga(t_years) <= LAND_PLANTS_GA
    }

    /// The three clocks over the frame [`t0_years`, `t1_years`]: each the share of the frame's
    /// span inside its window (the instant's value when the span is empty).
    pub fn clocks(&self, t0_years: f64, t1_years: f64) -> [(&'static str, f32); 3] {
        // ledger: ORE-260
        let (a, b) = (self.ga(t1_years), self.ga(t0_years)); // a ≤ b: younger, older
        let share = |lo: f64, hi: f64| -> f32 {
            if b - a <= 0.0 {
                return f32::from(u8::from((lo..=hi).contains(&a)));
            }
            ((b.min(hi) - a.max(lo)).max(0.0) / (b - a)) as f32
        };
        [
            (OXYGENATED_OCEANS, share(f64::NEG_INFINITY, ZN_PB_GA)),
            (LAND_PLANTS, share(f64::NEG_INFINITY, LAND_PLANTS_GA)),
            (BIF_ACME, share(BIF_ACME_GA.1, BIF_ACME_GA.0)),
        ]
    }

    /// The share of the BIF acme that lies before the run's start (1 when the run starts after the
    /// acme, 0 when it starts before it: the run then has the acme in its own clock), times the
    /// preset's inherited-BIF setting.
    pub fn inherited_bif_share(&self) -> f64 {
        // ledger: ORE-020
        let start = self.ga(self.run_start_years);
        let (old, young) = BIF_ACME_GA;
        ((old - start.max(young)).max(0.0) / (old - young)).min(1.0) * self.inherited_bif
    }
}

impl Default for Epoch {
    /// Earth's present age and a run of no length.
    fn default() -> Self {
        Epoch {
            present_age_gyr: EARTH_AGE_GYR,
            run_start_years: 0.0,
            inherited_bif: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earth(start_myr: f64) -> Epoch {
        Epoch {
            present_age_gyr: EARTH_AGE_GYR,
            run_start_years: -start_myr * 1.0e6,
            inherited_bif: 1.0,
        }
    }

    fn clock(e: &Epoch, t0_myr: f64, t1_myr: f64, name: &str) -> f32 {
        e.clocks(t0_myr * 1.0e6, t1_myr * 1.0e6)
            .iter()
            .find(|c| c.0 == name)
            .unwrap()
            .1
    }

    /// Earth's dates: no coal-forming plants before the Devonian, no sediment-hosted Zn–Pb before
    /// 2.02 Ga, BIF only inside 2.6–1.85 Ga. The old stand-ins (1, 1, 0 at every frame) fail all
    /// three tests below.
    #[test]
    fn the_clocks_follow_earths_dates() {
        let e = earth(3000.0);
        // A Phanerozoic frame: oxygenated, plants, no acme.
        assert_eq!(clock(&e, -300.0, -250.0, LAND_PLANTS), 1.0);
        assert_eq!(clock(&e, -300.0, -250.0, OXYGENATED_OCEANS), 1.0);
        assert_eq!(clock(&e, -300.0, -250.0, BIF_ACME), 0.0);
        // A Cambrian frame (before 419 Ma): no coal-forming plants yet.
        assert_eq!(clock(&e, -520.0, -470.0, LAND_PLANTS), 0.0);
        // Inside the acme (2.4–2.35 Ga): BIF, no sediment-hosted Zn–Pb.
        assert_eq!(clock(&e, -2400.0, -2350.0, BIF_ACME), 1.0);
        assert_eq!(clock(&e, -2400.0, -2350.0, OXYGENATED_OCEANS), 0.0);
        assert_eq!(clock(&e, -2400.0, -2350.0, LAND_PLANTS), 0.0);
        // A frame straddling 1.85 Ga is in the acme for 3/5 of its span.
        assert!((clock(&e, -1880.0, -1830.0, BIF_ACME) - 0.6).abs() < 1e-6);
        assert!((clock(&e, -2040.0, -1990.0, OXYGENATED_OCEANS) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn a_younger_planet_is_earlier_in_earths_history() {
        // A planet 2.57 Gyr old at the end of its run stands where Earth stood 2.0 Ga ago.
        let e = Epoch {
            present_age_gyr: 2.57,
            run_start_years: -500.0e6,
            inherited_bif: 1.0,
        };
        assert!((e.ga(0.0) - 2.0).abs() < 1e-12);
        assert_eq!(clock(&e, -10.0, 0.0, OXYGENATED_OCEANS), 1.0);
        assert_eq!(clock(&e, -10.0, 0.0, LAND_PLANTS), 0.0);
        assert_eq!(clock(&e, -10.0, 0.0, BIF_ACME), 1.0);
        assert!(!e.land_plants_at(0.0));
    }

    #[test]
    fn inherited_bif_needs_a_run_that_starts_after_the_acme() {
        // Every run shorter than 1.85 Gyr on an Earth-aged planet starts after the acme.
        assert_eq!(earth(1000.0).inherited_bif_share(), 1.0);
        assert_eq!(earth(500.0).inherited_bif_share(), 1.0);
        // A run that starts at 3 Ga holds the acme itself.
        assert_eq!(earth(3000.0).inherited_bif_share(), 0.0);
        // A run that starts at 2.225 Ga inherits the older half.
        assert!((earth(2225.0).inherited_bif_share() - 0.5).abs() < 1e-9);
        // The built-in presets' interim setting 0 inherits none.
        let none = Epoch {
            inherited_bif: 0.0,
            ..earth(1000.0)
        };
        assert_eq!(none.inherited_bif_share(), 0.0);
        let half = Epoch {
            inherited_bif: 0.5,
            ..earth(1000.0)
        };
        assert_eq!(half.inherited_bif_share(), 0.5);
        assert!(earth(100.0).land_plants_at(-100.0e6));
        assert!(!earth(1000.0).land_plants_at(-1000.0e6));
    }
}
