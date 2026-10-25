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
/// model's present climate, measured with `deeptime climate-dem` on Earth's relief (ledger:
/// CLM-061): 2.845·10⁷ under the banded climate (2026-09-29); 2.475·10⁷ over 1.473·10⁸ km² of land
/// under the calibrated per-cell climate on Lambert's equal-area grid (P1-26 T9, 2026-10-03);
/// 5.039·10⁷ over 1.481·10⁸ km² since the belts' rain is scaled to Earth's land rain
/// (`belts_rain_scale` 1.42, CLM-201, 2026-10-16), a mean of 0.34 m/yr at the 15 °C reference;
/// 5.147·10⁷ since the plateaus' warmth (CLM-225 to CLM-228, 2026-10-21) warms Earth's high land.
/// It changes whenever the climate's defaults change the rain or the temperature on Earth's land.
pub const KERNEL_EARTH: f64 = 5.147e7;
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

    /// The kinetic weathering of one unit of kernel (km²·m/yr) at `ppm`, mol C/yr: Earth's land
    /// share of D₀ over [`KERNEL_EARTH`] × `strength` × (CO₂/280 ppm)^β.
    pub fn kinetic_per_kernel(&self, ppm: f64) -> f64 {
        DEGASSING_EARTH_MOL_YR * self.strength * (1.0 - self.seafloor) / KERNEL_EARTH
            * (ppm.max(1e-6) / CO2_REF_PPM).powf(self.beta)
    }

    /// Band `b`'s silicate weathering, mol C/yr (`scale`: [`Weathering::kinetic_per_kernel`]).
    #[inline]
    fn band_flux(
        &self,
        k: &Kernel,
        b: usize,
        kb: f64,
        t_land: &[f64],
        ice: &[f64],
        scale: f64,
    ) -> f64 {
        if kb <= 0.0 {
            return 0.0;
        }
        let dt =
            t_land.get(b).copied().unwrap_or(0.0) - k.ref_t_land.get(b).copied().unwrap_or(0.0);
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
    }

    /// Silicate weathering, mol C/yr, of kernel `k` with each band's land at `t_land` °C and
    /// `ice` of its land under ice, at `ppm`: each band's kinetic flux, limited by its supply of
    /// fresh rock where the kernel has one (`supply_limited`).
    pub fn flux(&self, k: &Kernel, t_land: &[f64], ice: &[f64], ppm: f64) -> f64 {
        let scale = self.kinetic_per_kernel(ppm);
        k.band
            .iter()
            .enumerate()
            .map(|(b, &kb)| self.band_flux(k, b, kb, t_land, ice, scale))
            .sum()
    }

    /// Each band's silicate weathering, mol C/yr: the terms [`Weathering::flux`] adds, in its
    /// order (their sum is `flux` to the bit).
    pub fn flux_by_band(&self, k: &Kernel, t_land: &[f64], ice: &[f64], ppm: f64) -> Vec<f64> {
        let scale = self.kinetic_per_kernel(ppm);
        k.band
            .iter()
            .enumerate()
            .map(|(b, &kb)| self.band_flux(k, b, kb, t_land, ice, scale))
            .collect()
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

/// One cell's part in placing its band's dissolved load ([`place_dissolved`]). Sea cells have
/// `area_km2` 0.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DissolvedCell {
    /// The cell's EBM band.
    pub band: usize,
    /// True area, km² (Earth-normalized: the kernel's and the supply's).
    pub area_km2: f64,
    /// The cell's kinetic weathering at the step's end climate, mol C/yr
    /// ([`Weathering::kinetic_per_kernel`] × its kernel term).
    pub kin: f64,
    /// Its supply of fresh rock, mol C/yr (unused without a supply limit).
    pub supply: f64,
    /// Under land ice at the step's end.
    pub iced: bool,
}

/// What [`place_dissolved`] took off the land.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Dissolved {
    /// Rock taken into solution, m (at [`ROCK_KG_M3`]), per cell.
    pub thickness_m: Vec<f64>,
    /// The weathered Ca + Mg placed, mol.
    pub placed_mol: f64,
    /// The weathered Ca + Mg of bands with no land at the step's end, mol (not placed).
    pub unplaced_mol: f64,
}

/// Chemical denudation: the rock the land's silicate weathering took into solution, cell by cell.
/// Each band's weathered Ca + Mg over the step (`mol_by_band`, the clock's
/// [`super::clock::Output::land_mol_by_band`]) is the mass the carbon cycle already weathered, so
/// the dissolved load is that mass and no new rate: `kg_per_mol` of rock leaves in solution per
/// mol of Ca + Mg. Within a band it goes where the weathering is, by each cell's own supply-limited
/// flux, S·(1 − e^(−K/S)) (`supply_limited`; its kinetic flux without a supply limit), over the
/// cells not under ice; a band whose land is all under ice spreads it over that land by the same
/// weights, then by area. The thickness of a cell is its mass over ρ_rock × its true area, so on
/// supply-limited ground it is (denudation) × `weathering_rock_mol_kg` × `kg_per_mol` ×
/// `weathering_supply_scale` whatever the area's normalization. Every placed mol is taken off:
/// Σ thickness × area × ρ_rock = `kg_per_mol` × Σ mol of the bands with land.
pub fn place_dissolved(
    mol_by_band: &[f64],
    cells: &[DissolvedCell],
    kg_per_mol: f64,
    limited: bool,
) -> Dissolved {
    let bands = mol_by_band.len();
    let weight = |c: &DissolvedCell| -> f64 {
        if limited {
            supply_limited(c.kin, c.supply)
        } else {
            c.kin.max(0.0)
        }
    };
    // Per band: the free cells' weights, all the land's weights, the land's area.
    let mut sums = vec![[0.0f64; 3]; bands];
    for c in cells.iter().filter(|c| c.area_km2 > 0.0 && c.band < bands) {
        let w = weight(c);
        let s = &mut sums[c.band];
        if !c.iced {
            s[0] += w;
        }
        s[1] += w;
        s[2] += c.area_km2;
    }
    let mut out = Dissolved {
        thickness_m: vec![0.0; cells.len()],
        ..Dissolved::default()
    };
    for (b, &m) in mol_by_band.iter().enumerate() {
        if m <= 0.0 {
            continue;
        }
        if sums[b][2] > 0.0 {
            out.placed_mol += m;
        } else {
            out.unplaced_mol += m;
        }
    }
    for (c, t) in cells.iter().zip(out.thickness_m.iter_mut()) {
        if c.area_km2 <= 0.0 || c.band >= bands || mol_by_band[c.band] <= 0.0 {
            continue;
        }
        let [free, all, area] = sums[c.band];
        let share = if free > 0.0 {
            if c.iced { 0.0 } else { weight(c) / free }
        } else if all > 0.0 {
            weight(c) / all
        } else {
            c.area_km2 / area
        };
        *t = mol_by_band[c.band] * share * kg_per_mol / (ROCK_KG_M3 * c.area_km2 * 1.0e6);
    }
    out
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

    #[test]
    fn the_bands_weathering_adds_up_to_the_flux_to_the_bit() {
        let wp = Weathering::from(&super::super::ClimateParams::default());
        let k = Kernel {
            band: vec![0.0, 3.0e6, 1.2e7, 5.0e5, 2.0e6],
            ref_t_land: vec![-20.0, 5.0, 25.0, 12.0, -3.0],
            supply: vec![1.0e11, 2.0e11, 4.0e12, 3.0e10, 1.0e11],
        };
        let ice = [0.9, 0.1, 0.0, 0.3, 1.0];
        for (dt, ppm) in [(0.0, 280.0), (4.0, 900.0), (-6.0, 180.0)] {
            let t: Vec<f64> = k.ref_t_land.iter().map(|x| x + dt).collect();
            let by = wp.flux_by_band(&k, &t, &ice, ppm);
            assert_eq!(by.len(), 5);
            assert_eq!(by[0], 0.0);
            assert_eq!(
                by.iter().sum::<f64>().to_bits(),
                wp.flux(&k, &t, &ice, ppm).to_bits()
            );
            let kin = Kernel {
                supply: Vec::new(),
                ..k.clone()
            };
            let by = wp.flux_by_band(&kin, &t, &ice, ppm);
            assert_eq!(
                by.iter().sum::<f64>().to_bits(),
                wp.flux(&kin, &t, &ice, ppm).to_bits()
            );
        }
    }

    /// Cells of four bands: two with free land (one partly under ice), one all under ice, one with
    /// no land; sea cells among them.
    fn dissolved_cells() -> Vec<DissolvedCell> {
        (0..400)
            .map(|c| {
                let band = c % 4;
                if band == 3 || c % 7 == 0 {
                    return DissolvedCell {
                        band,
                        ..DissolvedCell::default()
                    };
                }
                DissolvedCell {
                    band,
                    area_km2: 1000.0 + (c as f64 * 37.0) % 900.0,
                    kin: 1.0e6 * (1.0 + (c % 13) as f64),
                    supply: 3.0e6 * (1.0 + (c % 5) as f64),
                    iced: band == 2 || (band == 1 && c % 3 == 0),
                }
            })
            .collect()
    }

    #[test]
    fn the_dissolved_load_is_the_weathered_mass() {
        // Every placed mol leaves as kg_per_mol of rock (Σ thickness × area × ρ), on land only,
        // not under ice while the band has free land; a band all under ice still loses its
        // weathered mass; a band with no land keeps it unplaced.
        let cells = dissolved_cells();
        let mol = [2.0e15, 7.0e14, 3.0e14, 5.0e14];
        let kg_per_mol = 0.15;
        for limited in [true, false] {
            let d = place_dissolved(&mol, &cells, kg_per_mol, limited);
            assert_eq!(d.thickness_m.len(), cells.len());
            assert_eq!(d.placed_mol, mol[0] + mol[1] + mol[2]);
            assert_eq!(d.unplaced_mol, mol[3]);
            for (b, &m) in mol.iter().enumerate() {
                let kg: f64 = d
                    .thickness_m
                    .iter()
                    .zip(&cells)
                    .filter(|(_, c)| c.band == b)
                    .map(|(t, c)| t * c.area_km2 * 1.0e6 * ROCK_KG_M3)
                    .sum();
                let want = if b == 3 { 0.0 } else { m * kg_per_mol };
                assert!(
                    (kg - want).abs() <= 1e-12 * want.max(1.0),
                    "band {b}: {kg} kg, {want}"
                );
            }
            for (t, c) in d.thickness_m.iter().zip(&cells) {
                let none = c.area_km2 == 0.0 || (c.band == 1 && c.iced);
                assert_eq!(*t == 0.0, none, "{c:?}: {t}");
            }
        }
        // Nothing weathered, nothing taken.
        let d = place_dissolved(&[0.0; 4], &cells, kg_per_mol, true);
        assert!(d.thickness_m.iter().all(|&t| t == 0.0) && d.placed_mol == 0.0);
    }

    #[test]
    fn the_dissolved_load_goes_where_the_weathering_is() {
        // Two free cells of one band, equal area: one supply-limited (ample kinetics), one
        // kinetic (ample supply). Each loses its own flux's share.
        let a = DissolvedCell {
            band: 0,
            area_km2: 100.0,
            kin: 1.0e9,
            supply: 1.0e6,
            iced: false,
        };
        let b = DissolvedCell {
            kin: 2.0e6,
            supply: 1.0e12,
            ..a
        };
        let d = place_dissolved(&[1.0e12], &[a, b], 0.15, true);
        let (wa, wb) = (
            supply_limited(a.kin, a.supply),
            supply_limited(b.kin, b.supply),
        );
        assert!((d.thickness_m[0] / d.thickness_m[1] - wa / wb).abs() < 1e-9);
        // Without a supply limit, by the kinetics alone.
        let d = place_dissolved(&[1.0e12], &[a, b], 0.15, false);
        assert!((d.thickness_m[0] / d.thickness_m[1] - a.kin / b.kin).abs() < 1e-9);
    }

    #[test]
    fn a_craton_sinks_at_west_2005s_chemical_rates() {
        // West, Galy & Bickle 2005 (EPSL 235: 211), Table 2, the five cratons: mean annual
        // temperature (°C), runoff (mm/yr), total denudation TDR and silicate cations SCDR plus
        // dissolved SiO₂ (t/km²/yr); the silicate chemical denudation is (SCDR + SiO₂)/ρ_rock.
        // ledger: ERO-207
        let west = [
            ("Slave", 4.0, 100.0, 3.2, 0.25 + 0.18),
            ("Siberia", 2.0, 270.0, 15.0, 0.77 + 1.61),
            ("Africa", 24.1, 250.0, 10.0, 0.80 + 2.96),
            ("Grenville", 4.5, 580.0, 13.0, 1.14 + 2.63),
            ("Guyana", 25.0, 1680.0, 35.0, 2.40 + 10.18),
        ];
        let p = super::super::ClimateParams::default();
        let wp = Weathering::from(&p);
        let area = 1.0e4;
        let m_myr = |t_km2_yr: f64| t_km2_yr * 1.0e3 / ROCK_KG_M3; // t/km²/yr → m/Myr
        let lo = west
            .iter()
            .map(|r| m_myr(r.4))
            .fold(f64::INFINITY, f64::min);
        let hi = west.iter().map(|r| m_myr(r.4)).fold(0.0, f64::max);
        // Each craton's sink, m/Myr, through the model's chain: its kernel and supply (eroding at
        // `e` m/Myr), its band's weathering at 280 ppm over 1 Myr, placed on the one cell.
        let sink = |t: f64, runoff_mm: f64, e: f64| -> f64 {
            let k = Kernel {
                band: vec![cell_kernel(
                    area,
                    runoff_mm / 1000.0,
                    t,
                    p.weathering_temp_per_k,
                )],
                ref_t_land: vec![t],
                supply: vec![
                    cell_supply(
                        area,
                        e,
                        p.weathering_min_denudation_m_myr,
                        p.weathering_rock_mol_kg,
                    ) * p.weathering_supply_scale,
                ],
            };
            let mol = wp.flux_by_band(&k, &[t], &[0.0], CO2_REF_PPM)[0] * 1.0e6;
            let cell = DissolvedCell {
                band: 0,
                area_km2: area,
                kin: wp.kinetic_per_kernel(CO2_REF_PPM) * k.band[0],
                supply: k.supply[0],
                iced: false,
            };
            place_dissolved(&[mol], &[cell], p.dissolved_kg_per_mol, true).thickness_m[0]
        };
        for (name, t, runoff, tdr, chem) in west {
            // Eroding at the craton's own denudation: within a factor of 2.5 of its measured rate.
            let (model, want) = (sink(t, runoff, m_myr(tdr)), m_myr(chem));
            assert!(
                model > want / 2.5 && model < want * 2.5,
                "{name}: {model:.2} m/Myr against West's {want:.2}"
            );
            // Flat and not eroding (the supply's least denudation): inside the cratons' range.
            let flat = sink(t, runoff, 0.0);
            assert!(
                flat >= lo && flat <= hi,
                "{name} flat: {flat:.2} m/Myr, West {lo:.2} to {hi:.2}"
            );
        }
    }
}
