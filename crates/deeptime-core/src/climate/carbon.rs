// SPDX-License-Identifier: LGPL-3.0-or-later
//! The long-term carbon cycle and the brightening star (deep-time climate, decisions log
//! 2026-09-29 "Greenhouse & sun over time" and "snowball earth": emergent, not scripted).
//!
//! **Reservoirs.** The exogenic carbon is COPSE's atmosphere–ocean reservoir, 3.193·10¹⁸ mol at
//! 280 ppm (Bergman, Lenton & Watson 2004, "COPSE: a new model of biogeochemical cycling over
//! Phanerozoic time", Am. J. Sci. 304: 397–437), split into the atmosphere A (4.96·10¹⁶ mol at
//! 280 ppm: 1.773·10²⁰ mol of air; pCO₂ = 280 ppm · A/A₀) and the ocean O. With open water the two
//! stay in equilibrium, A/A₀ = (O/O₀)^R with an effective buffer factor R = 5 (the Revelle factor
//! of ~10 lowered by sea-floor CaCO₃ neutralization over 10⁴–10⁵ yr: ~7–8 % of an added pulse stays
//! airborne; Archer et al. 2009, "Atmospheric lifetime of fossil fuel carbon dioxide", Annu. Rev.
//! Earth Planet. Sci. 37: 117–134). COPSE itself takes pCO₂ linear in the reservoir (R = 1); the
//! buffer changes only how fast CO₂ moves, not where degassing and weathering balance. Under sea
//! ice the air–sea exchange shuts in proportion to the ice cover, so in a snowball the degassing
//! builds up in the atmosphere alone: ~350× today's CO₂ in ~3 Myr at today's degassing, as
//! Hoffman et al. (1998, "A Neoproterozoic snowball Earth", Science 281: 1342–1346) estimate
//! (~4 Myr at 5.4·10¹² mol/yr); once the sea opens the ocean takes up part of it.
//!
//! d(A + O)/dt = D − W_sil (the Walker–Hays–Kasting / GEOCARB balance). Carbonate weathering and
//! burial cancel over the long term and organic burial is left out, so the silicate weathering flux
//! is also the CO₂ that ends as carbonate on the sea floor (Urey's reaction: CaSiO₃ + CO₂ → CaCO₃
//! + SiO₂; the surface stage deposits the carbonates, `surface/carbonate.rs`).
//!
//! **Degassing** D (volcanic and metamorphic): Earth's present D₀ = 6.65·10¹² mol C/yr (COPSE's
//! present carbonate degassing, from Berner's GEOCARB; equal to silicate weathering in steady state) ×
//! `degassing_scale` × the plates' activity against Earth's today (GEOCARB's f_SR, Berner 1994):
//! half from the ridges — sea-floor production, the true area of oceanic crust younger than
//! 20 Myr per year (Earth ≈ 3.4 km²/yr, Rowley 2002, "Rate of plate creation and destruction:
//! 180 Ma to present", GSA Bull. 114: 927–933) — and half from the arcs and collision belts —
//! the ocean floor consumed at trenches, production minus the ocean floor's growth, smoothed over
//! 10 Myr (Earth's destruction balances its production, ~3.4 km²/yr; Bird 2003's 51,000 km of
//! subduction zones at 62 mm/yr) — plus the CO₂ of each large igneous province (0.5 wt % of its
//! basalt, spread over 1 Myr; Self et al. 2006, "Volatile fluxes during flood basalt eruptions
//! and potential effects on the global environment", EPSL 248: 518–532). Both rates are areas of
//! crust, so they do not depend on the grid's resolution.
//!
//! **Silicate weathering** (Walker, Hays & Kasting 1981, "A negative feedback mechanism for the
//! long-term stabilization of Earth's surface temperature", JGR 86(C10): 9776–9782; Berner 1994,
//! GEOCARB II, Am. J. Sci. 294: 56–91, and Berner & Kothavala 2001, GEOCARB III, Am. J. Sci. 301:
//! 182–204, eq. 2): per land cell, runoff × e^(k_T·(T − 15 °C)), summed over the
//! planet's true (Earth-normalized) land area, so warm, wet tropical land weathers most — a
//! supercontinent in the tropics draws CO₂ down. Between snapshots each latitude band's weathering
//! follows the EBM's temperature change there: × e^(k_T·ΔT)·max(0, 1 + k_r·ΔT)^0.65 (GEOCARB's
//! runoff term) × (1 − the band's ice-covered land); and globally × (CO₂/280 ppm)^β. Normalized so
//! Earth's present relief under the model's present climate at 280 ppm weathers exactly the land's
//! share of D₀ ([`KERNEL_EARTH`], measured with `deeptime climate-dem`), times
//! `weathering_strength`. Under full ice cover the land is frozen and dry: weathering stops,
//! degassing builds CO₂ up.
//!
//! **Supply limit** (decisions log 2026-09-30, "Yes, supply-limited"): each band's kinetic
//! weathering K saturates at the Ca + Mg its denudation exposes, S·(1 − e^(−K/S)) (the kinetic
//! and supply-limited regimes of West, Galy & Bickle 2005, "Tectonic and climatic controls on
//! silicate weathering", EPSL 235: 211–228, and West 2012, Geology 40: 811–814). S is the surface
//! stage's erosion (at least `weathering_min_denudation_m_myr`) × 2,700 kg/m³ × 1.26 mol/kg
//! (Rudnick & Gao 2003) × `weathering_supply_scale` (the L1 grid's erosion against Earth's ~20
//! Gt/yr of pre-human denudation, Syvitski et al. 2005), rising with the runoff (1 + k_r·ΔT, as
//! the erosion does). A hothouse after a snowball draws CO₂ down at the supply's pace, millions of
//! years, not at its kinetics' pace.
//!
//! **Seafloor weathering** (`seafloor_weathering_share`, 20 % of today's; Lenton et al. 2018's
//! COPSE reloaded): young ocean crust's basalt, ∝ sea-floor production × e^(E_a/R·(1/T₀ − 1/T))
//! on the bottom water (E_a 75 kJ/mol) × (CO₂/280 ppm)^0.23 (Krissansen-Totton & Catling 2017).
//! It needs neither land nor erosion, so a land-poor planet keeps a thermostat under the supply
//! limit (without it the 8 %-land GameTest planet ran to 90,000 ppm); Foley 2015 (ApJ 812: 36)
//! shows the land-fraction sensitivity of supply-limited weathering it buffers.
//!
//! **Forcing** (P1-26 T6, `co2.rs`): a radiative-convective model's clear-sky OLR table
//! (Kopparapu et al. 2013 as tabulated by HEXTOR) at every CO₂ level up to 10 bar and every surface
//! temperature, scaled so a doubling gives Myhre et al. 1998's 5.35 ln 2 W/m². It replaced the
//! Myhre/Byrne–Goldblatt fit, which was stated only to 10,000 ppm and extrapolated beyond 50,000.
//! The energy balances subtract it from each tile's longwave at the tile's temperature (North et
//! al.'s A = 203.3 W/m² is Earth's at 280 ppm).
//!
//! **The star.** L(t) = L₀ / (1 + 0.4·(1 − t/t₀)) (Gough 1981, "Solar interior structure and
//! luminosity variations", Solar Phys. 74: 21–34), t the star's age, t₀ `present_age_gyr`.

/// CO₂ at which the EBM's longwave A is North et al.'s, ppm (pre-industrial).
pub const CO2_REF_PPM: f64 = 280.0;
/// COPSE's present atmosphere–ocean reservoir, mol C.
pub const RESERVOIR_REF_MOL: f64 = 3.193e18;
/// The atmosphere's CO₂ at 280 ppm, mol (1.773·10²⁰ mol of air).
pub const ATM_REF_MOL: f64 = 280.0e-6 * 1.773e20;
/// The ocean's share of the reference reservoir, mol.
pub const OCEAN_REF_MOL: f64 = RESERVOIR_REF_MOL - ATM_REF_MOL;
/// Effective buffer factor R of the atmosphere–ocean equilibrium A/A₀ = (O/O₀)^R.
pub const BUFFER_R: f64 = 5.0;
/// Air–sea exchange time of an open ocean, years.
pub const EXCHANGE_YEARS: f64 = 1000.0;
/// Earth's present volcanic and metamorphic degassing, mol C/yr (GEOCARB III).
pub const DEGASSING_EARTH_MOL_YR: f64 = 6.65e12;
/// Earth's present sea-floor production, km²/yr (Rowley 2002).
pub const RIDGE_EARTH_KM2_YR: f64 = 3.4;
/// Earth's present consumption of ocean floor at trenches, km²/yr (= its production).
pub const SUBDUCTION_EARTH_KM2_YR: f64 = 3.4;
/// Smoothing of the consumption rate, years.
pub const SUBDUCTION_SMOOTH_YEARS: f64 = 1.0e7;
/// Young crust counts as this many Myr of production.
pub const RIDGE_WINDOW_MYR: f64 = 20.0;
/// CO₂ of a large igneous province, mol C per km³ of basalt (2,900 kg/m³, 0.5 wt % CO₂).
pub const LIP_MOL_PER_KM3: f64 = 2.9e12 * 0.005 / 0.044;
/// Volume of a mole of calcite, m³ (100.09 g/mol at 2,710 kg/m³).
pub const CALCITE_M3_PER_MOL: f64 = 0.10009 / 2710.0;
/// A province degasses over this long, years.
pub const LIP_YEARS: f64 = 1.0e6;
/// The weathering kernel Σ a·r·e^(0.09·(T − 15)) (km² · m/yr) of Earth's present land under the
/// model's present climate (`deeptime climate-dem` on Earth's 1° relief, 2026-09-29: 2.845·10⁷ over
/// 1.476·10⁸ km² of land — a mean runoff-weighted 0.19 m/yr at the 15 °C reference).
pub const KERNEL_EARTH: f64 = 2.845e7;
/// Reference temperature of the kernel's Arrhenius term, °C.
pub const KERNEL_T_REF_C: f64 = 15.0;

/// The exogenic carbon: atmosphere and ocean, mol C.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Carbon {
    pub atm: f64,
    pub ocean: f64,
}

impl Carbon {
    /// Atmosphere and ocean in equilibrium at `ppm`.
    pub fn at_ppm(ppm: f64) -> Carbon {
        let x = (ppm / CO2_REF_PPM).max(1e-9);
        Carbon {
            atm: ATM_REF_MOL * x,
            ocean: OCEAN_REF_MOL * x.powf(1.0 / BUFFER_R),
        }
    }

    pub fn ppm(&self) -> f64 {
        CO2_REF_PPM * self.atm / ATM_REF_MOL
    }

    pub fn total(&self) -> f64 {
        self.atm + self.ocean
    }

    /// The equilibrium split of `total` mol: O + A₀·(O/O₀)^R = total (bisection).
    pub fn equilibrium(total: f64) -> Carbon {
        let atm_of = |o: f64| ATM_REF_MOL * (o / OCEAN_REF_MOL).max(0.0).powf(BUFFER_R);
        let (mut lo, mut hi) = (0.0, total.max(0.0));
        for _ in 0..200 {
            let mid = 0.5 * (lo + hi);
            if mid + atm_of(mid) > total {
                hi = mid;
            } else {
                lo = mid;
            }
            if hi - lo <= 1e-13 * total.max(1.0) {
                break;
            }
        }
        let ocean = 0.5 * (lo + hi);
        Carbon {
            atm: (total - ocean).max(0.0),
            ocean,
        }
    }

    /// One step of `dt` years: degassing `d` into the air, weathering `w(ppm)` out of it (implicit:
    /// solved for the end of the step), and air–sea exchange over the open share `open` of the
    /// ocean. With open water (the exchange time ≪ the step) the step is taken on the total in
    /// equilibrium; under full ice on the air alone. Carbon is conserved: the total changes by
    /// exactly Δt·(D − W) at the step's end state.
    pub fn step(&self, d: f64, w: &dyn Fn(f64) -> f64, open: f64, dt: f64) -> Carbon {
        // Share of the disequilibrium the exchange removes within the step.
        let e = 1.0 - (-open.clamp(0.0, 1.0) * dt / EXCHANGE_YEARS).exp();
        let solve = |x0: f64, ppm_of: &dyn Fn(f64) -> f64| -> f64 {
            // x − x0 − Δt·(D − W(ppm(x))) = 0, increasing in x; bisection on [0, x0 + Δt·D].
            let hi0 = x0 + dt * d.max(0.0);
            let g = |x: f64| x - x0 - dt * (d - w(ppm_of(x)));
            if g(hi0) <= 0.0 {
                return hi0;
            }
            let (mut lo, mut hi) = (0.0f64, hi0);
            for _ in 0..200 {
                let mid = 0.5 * (lo + hi);
                if g(mid) > 0.0 {
                    hi = mid;
                } else {
                    lo = mid;
                }
                if hi - lo <= 1e-12 * hi.max(1.0) {
                    break;
                }
            }
            0.5 * (lo + hi)
        };
        if e > 0.5 {
            // Open: the total, the air in equilibrium with the sea throughout; whatever
            // disequilibrium the air still held relaxes by e.
            let dis = self.atm - Carbon::equilibrium(self.total()).atm;
            let keep = dis * (1.0 - e);
            let t = solve(self.total(), &|x| {
                CO2_REF_PPM * (Carbon::equilibrium(x).atm + keep) / ATM_REF_MOL
            });
            let eq = Carbon::equilibrium(t);
            Carbon {
                atm: (eq.atm + keep).max(0.0),
                ocean: eq.ocean - keep,
            }
        } else {
            // Sealed: the air alone, then the partial exchange toward equilibrium.
            let a = solve(self.atm, &|x| CO2_REF_PPM * x / ATM_REF_MOL);
            let c = Carbon {
                atm: a,
                ocean: self.ocean,
            };
            let eq = Carbon::equilibrium(c.total());
            Carbon {
                atm: c.atm + e * (eq.atm - c.atm),
                ocean: c.ocean + e * (eq.ocean - c.ocean),
            }
        }
    }
}

/// Radiative forcing of `ppm` against 280 ppm at Earth's 15 °C, W/m² (`co2.rs`; the energy
/// balances take it at each tile's own temperature). For reports and first guesses.
pub fn forcing(ppm: f64) -> f64 {
    super::co2::Forcing::new(ppm).at(15.0)
}

/// The star's luminosity at age `age_gyr` against its luminosity at `present_gyr` (Gough 1981).
pub fn gough(age_gyr: f64, present_gyr: f64) -> f64 {
    1.0 / (1.0 + 0.4 * (1.0 - age_gyr / present_gyr))
}

/// The plates' activity for degassing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Activity {
    /// Sea-floor production, Earth-normalized km²/yr.
    pub ridge_km2_yr: f64,
    /// Ocean floor consumed at trenches, Earth-normalized km²/yr (smoothed).
    pub subduction_km2_yr: f64,
}

/// Degassing, mol C/yr, before large igneous provinces.
pub fn degassing(a: &Activity, scale: f64) -> f64 {
    DEGASSING_EARTH_MOL_YR
        * scale
        * (0.5 * a.ridge_km2_yr / RIDGE_EARTH_KM2_YR
            + 0.5 * a.subduction_km2_yr / SUBDUCTION_EARTH_KM2_YR)
}

/// GEOCARB's temperature–runoff factor for a warming of `dt` K: e^(k_T·ΔT)·max(0, 1 + k_r·ΔT)^0.65.
pub fn climate_factor(dt: f64, k_t: f64, k_r: f64) -> f64 {
    (k_t * dt).exp() * (1.0 + k_r * dt).max(0.0).powf(0.65)
}

/// The weathering of one plate step's geography: each EBM band's kernel (the cells' Σ a·r·e^(k_T(T
/// − 15)), km²·m/yr), the band's land temperature when the cells' climate was computed, and the
/// band's supply of fresh rock (mol C/yr its erosion could take up if every eroded grain weathered
/// completely; empty: no supply limit).
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Kernel {
    pub band: Vec<f64>,
    pub ref_t_land: Vec<f64>,
    pub supply: Vec<f64>,
}

/// Density of the eroded rock, kg/m³.
pub const ROCK_KG_M3: f64 = 2700.0; // ledger: CLM-154

/// The supply of one land cell, mol C/yr: true area (km², Earth-normalized) × denudation (m/Myr,
/// at least `min_m_myr`) × ρ_rock × `mol_kg` (the Ca + Mg of the rock, each ion finally burying
/// one carbonate).
pub fn cell_supply(area_km2: f64, erosion_m_myr: f64, min_m_myr: f64, mol_kg: f64) -> f64 {
    area_km2 * 1.0e6 * erosion_m_myr.max(min_m_myr) * 1.0e-6 * ROCK_KG_M3 * mol_kg
}

/// Weathering limited by its supply: the kinetic flux `kin` where the supply `s` is ample, the
/// supply where the kinetics would outrun it, s·(1 − e^(−kin/s)) between (the kinetic and
/// supply-limited regimes of West, Galy & Bickle 2005 and West 2012).
pub fn supply_limited(kin: f64, s: f64) -> f64 {
    if s <= 0.0 || kin <= 0.0 {
        return 0.0;
    }
    let x = kin / s;
    if x < 1.0e-6 { kin } else { s * -(-x).exp_m1() }
}

/// Activation energy of seafloor (basalt) weathering, J/mol (Krissansen-Totton & Catling 2017).
pub const SEAFLOOR_EA_J_MOL: f64 = 75_000.0; // ledger: CLM-159
/// CO₂ dependence of seafloor weathering, (CO₂/280 ppm)^this (Krissansen-Totton & Catling 2017).
pub const SEAFLOOR_CO2_EXPONENT: f64 = 0.23; // ledger: CLM-159

/// Bottom-water temperature for a global mean `t_global`, °C: 2 °C today, following the surface
/// one for one (Hansen et al. 2013, "Climate sensitivity, sea level and atmospheric carbon
/// dioxide", Phil. Trans. R. Soc. A 371: 20120294), never below sea water's freezing point.
pub fn deep_ocean_c(t_global: f64) -> f64 {
    (2.0 + (t_global - 15.0)).max(-1.8) // ledger: CLM-160
}

/// Weathering parameters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weathering {
    pub strength: f64,
    pub k_t: f64,
    pub k_r: f64,
    pub beta: f64,
    /// Seafloor weathering's share of Earth's present silicate weathering.
    pub seafloor: f64,
    /// The most the sea floor can take up, × today's degassing at Earth's spreading (0: no limit).
    pub seafloor_cap: f64,
}

/// The sea floor's supply limit, × today's degassing at Earth's spreading rate. Carbonating all
/// the Ca + Mg of the upper 500 m of new crust (3.4 km²/yr × 500 m × 2,900 kg/m³ × ~5.5 mol/kg)
/// would take ~4× today's degassing; the most carbonated old crust drilled (Mesozoic, ~3 wt % CO₂
/// in the upper 300 m; Coogan & Gillis 2013, G³ 14: 1771–1786) took ~0.3×. 1 lies between.
pub const SEAFLOOR_SUPPLY_REL: f64 = 1.0; // ledger: CLM-161

impl Weathering {
    pub fn from(p: &super::ClimateParams) -> Weathering {
        Weathering {
            strength: p.weathering_strength,
            k_t: p.weathering_temp_per_k,
            k_r: p.runoff_temp_per_k,
            beta: p.weathering_co2_exponent,
            seafloor: p.seafloor_weathering_share,
            seafloor_cap: if p.weathering_supply {
                SEAFLOOR_SUPPLY_REL
            } else {
                0.0
            },
        }
    }

    /// Seafloor weathering, mol C/yr: the basalt of young ocean crust altered by warm, acid pore
    /// water (Brady & Gíslason 1997; Coogan & Gillis 2013), its share of today's degassing at
    /// Earth's spreading (`ridge_rel` × Earth's), × e^(E_a/R·(1/T₀ − 1/T)) on the bottom water
    /// × (CO₂/280 ppm)^0.23 (Krissansen-Totton & Catling 2017, "Constraining climate sensitivity
    /// and continental versus seafloor weathering using an inverse geological carbon cycle
    /// model", Nat. Commun. 8: 15423). A sink that needs no land and no erosion: a land-poor
    /// planet keeps a thermostat. The clock counts only the ice-free ocean's share of it against
    /// the atmosphere: under a snowball's ice the ocean is sealed, and the CO₂ the volcanoes add
    /// to the air builds up as before (with the whole sink on the air, a frozen tropical
    /// supercontinent stood at 0.6 bar of CO₂ after 70 Myr and had not melted).
    ///
    /// With the supply limit on (`seafloor_cap` > 0) the sink cannot outrun the fresh basalt the
    /// ridges make, `seafloor_cap` × today's degassing × `ridge_rel`, by the same law as the land's
    /// (`supply_limited`); the kinetic rate is scaled so today's sink is still its share.
    pub fn seafloor_flux(&self, ridge_rel: f64, t_global: f64, ppm: f64) -> f64 {
        if self.seafloor <= 0.0 {
            return 0.0;
        }
        let (t0, t) = (275.15, deep_ocean_c(t_global) + 273.15);
        let climate = (SEAFLOOR_EA_J_MOL / 8.314 * (1.0 / t0 - 1.0 / t)).exp()
            * (ppm.max(1e-6) / CO2_REF_PPM).powf(SEAFLOOR_CO2_EXPONENT);
        let unit = DEGASSING_EARTH_MOL_YR * ridge_rel.max(0.0);
        if self.seafloor_cap > self.seafloor {
            // kin₀ with cap·(1 − e^(−kin₀/cap)) = share.
            let kin0 = -self.seafloor_cap * (1.0 - self.seafloor / self.seafloor_cap).ln();
            supply_limited(
                unit * self.strength * kin0 * climate,
                unit * self.seafloor_cap,
            )
        } else {
            unit * self.strength * self.seafloor * climate
        }
    }

    /// Silicate weathering, mol C/yr, of kernel `k` with each band's land at `t_land` °C and
    /// `ice` of its land under ice, at `ppm`: each band's kinetic flux, limited by its supply of
    /// fresh rock where the kernel has one (`supply_limited`).
    pub fn flux(&self, k: &Kernel, t_land: &[f64], ice: &[f64], ppm: f64) -> f64 {
        let scale = DEGASSING_EARTH_MOL_YR * self.strength * (1.0 - self.seafloor) / KERNEL_EARTH
            * (ppm.max(1e-6) / CO2_REF_PPM).powf(self.beta);
        k.band
            .iter()
            .enumerate()
            .map(|(b, &kb)| {
                if kb <= 0.0 {
                    return 0.0;
                }
                let dt = t_land.get(b).copied().unwrap_or(0.0)
                    - k.ref_t_land.get(b).copied().unwrap_or(0.0);
                let free = 1.0 - ice.get(b).copied().unwrap_or(0.0).clamp(0.0, 1.0);
                let kin = scale * kb * climate_factor(dt, self.k_t, self.k_r) * free;
                match k.supply.get(b) {
                    // Fresh rock comes with the erosion, and the erosion with the runoff: the
                    // supply follows GEOCARB's runoff term, 1 + k_r·ΔT (×2.5 at +40 K), not the
                    // kinetics' exponential.
                    Some(&s) => {
                        // ledger: CLM-157
                        supply_limited(kin, s * free * (1.0 + self.k_r * dt).clamp(0.1, 10.0))
                    }
                    None => kin,
                }
            })
            .sum()
    }

    /// The supply of fresh rock of the land not under ice, mol C/yr (0 without a supply limit).
    pub fn supply(k: &Kernel, ice: &[f64]) -> f64 {
        k.supply
            .iter()
            .enumerate()
            .map(|(b, &s)| s * (1.0 - ice.get(b).copied().unwrap_or(0.0).clamp(0.0, 1.0)))
            .sum()
    }
}

/// One cell's kernel term: true area (km², Earth-normalized) × runoff (m/yr) × e^(k_T (T − 15)).
pub fn cell_kernel(area_km2: f64, runoff_m_yr: f64, t_c: f64, k_t: f64) -> f64 {
    area_km2 * runoff_m_yr.max(0.0) * (k_t * (t_c - KERNEL_T_REF_C)).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_reservoirs_forcing_and_sun_have_their_published_forms() {
        let c = Carbon::at_ppm(280.0);
        assert!((c.total() / RESERVOIR_REF_MOL - 1.0).abs() < 1e-12);
        assert!((c.ppm() - 280.0).abs() < 1e-9);
        let c = Carbon::at_ppm(1234.0);
        let e = Carbon::equilibrium(c.total());
        assert!((e.ppm() / 1234.0 - 1.0).abs() < 1e-6, "{}", e.ppm());
        // The table's doubling at 15 °C is within a few per cent of Myhre's 3.71 W/m² (exact
        // at 288.15 K); none at 280 ppm; Byrne & Goldblatt's 38.1 W/m² at 50,000 ppm within 3.
        assert!((forcing(560.0) - 5.35 * 2f64.ln()).abs() < 0.05);
        assert_eq!(forcing(280.0), 0.0);
        assert!(
            (forcing(50_000.0) - 38.1).abs() < 3.0,
            "{}",
            forcing(50_000.0)
        );
        // Monotone.
        let mut prev = f64::MIN;
        for k in 0..400 {
            let f = forcing(10.0 * 1.03f64.powi(k));
            assert!(f > prev);
            prev = f;
        }
        // Gough: 1/(1 + 0.4) = 0.714 at birth, 1 today, 0.958 500 Myr ago.
        assert!((gough(0.0, 4.57) - 1.0 / 1.4).abs() < 1e-12);
        assert!((gough(4.57, 4.57) - 1.0).abs() < 1e-12);
        assert!((gough(4.07, 4.57) - 0.9582).abs() < 1e-3);
    }

    #[test]
    fn the_carbon_ode_balances() {
        // Weathering 2·D·(pCO₂/280)^0.8: the open-ocean steps converge on D = W and never overshoot,
        // and every step conserves carbon: Δ(A + O) = Δt·(D − W(end)).
        let d = DEGASSING_EARTH_MOL_YR;
        let w = |ppm: f64| 2.0 * d * (ppm / 280.0).powf(0.8);
        let mut c = Carbon::at_ppm(2000.0);
        let mut prev = c.ppm();
        for _ in 0..300 {
            let n = c.step(d, &w, 1.0, 1.0e5);
            let dm = n.total() - c.total();
            assert!(
                (dm - 1.0e5 * (d - w(n.ppm()))).abs() < 1e-6 * c.total(),
                "mass balance"
            );
            assert!(n.ppm() <= prev * (1.0 + 1e-9), "monotone approach");
            prev = n.ppm();
            c = n;
        }
        let want = 280.0 * 0.5f64.powf(1.0 / 0.8);
        assert!((c.ppm() / want - 1.0).abs() < 1e-4, "{} vs {want}", c.ppm());
        // Sealed by ice with no weathering: the degassing stays in the air (Hoffman et al. 1998:
        // ~350× today's CO₂ within a few Myr at today's degassing).
        let mut s = Carbon::at_ppm(100.0);
        let ocean = s.ocean;
        let mut myr = 0.0;
        while s.ppm() < 350.0 * 280.0 {
            s = s.step(d, &|_| 0.0, 0.0, 1.0e5);
            myr += 0.1;
        }
        assert_eq!(s.ocean, ocean);
        assert!((2.0..4.0).contains(&myr), "{myr} Myr");
        // The sea opens: the ocean takes up part of the air's excess, not most of it.
        let open = s.step(0.0, &|_| 0.0, 1.0, 1.0e5);
        assert!((open.total() - s.total()).abs() < 1e-6 * s.total());
        assert!(
            open.ppm() < s.ppm() && open.ppm() > 0.4 * s.ppm(),
            "{} → {}",
            s.ppm(),
            open.ppm()
        );
    }

    #[test]
    fn weathering_follows_temperature_ice_and_co2() {
        let mut w = Weathering {
            strength: 1.0,
            k_t: 0.09,
            k_r: 0.038,
            beta: 0.3,
            seafloor: 0.0,
            seafloor_cap: 0.0,
        };
        let mut k = Kernel {
            band: vec![KERNEL_EARTH * 0.5, KERNEL_EARTH * 0.5],
            ref_t_land: vec![10.0, 20.0],
            supply: Vec::new(),
        };
        let base = w.flux(&k, &[10.0, 20.0], &[0.0, 0.0], 280.0);
        assert!((base / DEGASSING_EARTH_MOL_YR - 1.0).abs() < 1e-12);
        assert!(w.flux(&k, &[15.0, 25.0], &[0.0, 0.0], 280.0) > 1.5 * base);
        assert!(
            w.flux(&k, &[10.0, 20.0], &[1.0, 1.0], 280.0) == 0.0,
            "ice stops it"
        );
        // A frozen, dry planet (−40 K) barely weathers.
        assert!(w.flux(&k, &[-30.0, -20.0], &[0.0, 0.0], 280.0) < 1e-3 * base);
        assert!(w.flux(&k, &[10.0, 20.0], &[0.0, 0.0], 2800.0) > 1.9 * base);
        // Supply-limited: ample supply changes little, and no warming lifts weathering past it
        // (the supply itself follows the runoff: × 1 + 0.038·40 = 2.52 at +40 K).
        k.supply = vec![20.0 * DEGASSING_EARTH_MOL_YR, 20.0 * DEGASSING_EARTH_MOL_YR];
        let limited = w.flux(&k, &[10.0, 20.0], &[0.0, 0.0], 280.0);
        assert!(
            limited < base && limited > 0.97 * base,
            "{limited} vs {base}"
        );
        let hot = w.flux(&k, &[50.0, 60.0], &[0.0, 0.0], 5.0e5);
        assert!(
            hot < 100.8 * DEGASSING_EARTH_MOL_YR && hot > 90.0 * DEGASSING_EARTH_MOL_YR,
            "{hot}"
        );
        assert_eq!(supply_limited(1.0, 0.0), 0.0);
        assert!((supply_limited(1.0e-9, 1.0) - 1.0e-9).abs() < 1e-18);
        // Seafloor weathering: its share of today's degassing on Earth's spreading at 15 °C and
        // 280 ppm, faster in a warm ocean, slower in a frozen one, and land weathering the rest.
        w.seafloor = 0.2;
        k.supply.clear();
        let sf = w.seafloor_flux(1.0, 15.0, 280.0);
        assert!((sf / DEGASSING_EARTH_MOL_YR - 0.2).abs() < 1e-9);
        assert!(w.seafloor_flux(1.0, 25.0, 280.0) > 2.0 * sf);
        assert!(
            w.seafloor_flux(1.0, -40.0, 280.0) > 0.5 * sf,
            "bottom water stays at −1.8 °C"
        );
        // Supply-limited too: the same share today, and never more than the ridges' fresh basalt.
        w.seafloor_cap = 1.0;
        assert!((w.seafloor_flux(1.0, 15.0, 280.0) / sf - 1.0).abs() < 1e-9);
        let hot_sf = w.seafloor_flux(1.0, 50.0, 1.0e5);
        assert!(
            hot_sf < DEGASSING_EARTH_MOL_YR && hot_sf > 0.9 * DEGASSING_EARTH_MOL_YR,
            "{hot_sf}"
        );
        assert!((w.seafloor_flux(2.0, 50.0, 1.0e5) / hot_sf - 2.0).abs() < 1e-6);
        w.seafloor_cap = 0.0;
        let land = w.flux(&k, &[10.0, 20.0], &[0.0, 0.0], 280.0);
        assert!((land + sf - DEGASSING_EARTH_MOL_YR).abs() < 1e-3 * DEGASSING_EARTH_MOL_YR);
        // 1 km² eroding 50 m/Myr of rock with 1.2 mol/kg of Ca + Mg: 162,000 mol/yr.
        assert!((cell_supply(1.0, 50.0, 2.0, 1.2) - 1.62e5).abs() < 1.0);
    }
}
