// SPDX-License-Identifier: LGPL-3.0-or-later
//! The lithosphere's thermal state (gap 4 of `docs/science/physics-gap-audit.md`; the page is
//! `docs/science/lithosphere-thermal-state.md`).
//!
//! Each sample carries the depth of its thermal lithosphere's base, L
//! (`Sample::lithosphere_km`), and the thickness of its depleted keel, K (`Sample::keel_km`).
//! Arcs, back-arcs, hotspots and continental rifts heat it towards a thin, hot lithosphere over
//! [`HEATING_MYR`]; elsewhere it cools towards an old craton's over McKenzie's (1978) thermal time
//! for that depth. A keel resists the convective heating of a back-arc or a plume (Jordan 1978's
//! tectosphere); a rift stretches it with the lithosphere. The state gives continental heat flow
//! ([`heat_flow_mw_m2`]), a thermal term of the Airy height ([`uplift_m`]) and a thermal age
//! ([`thermal_age_myr`]) for the thin sheet's viscosity and the flexure's Te.
//!
//! `tectonics.thermal_state` and `thermal_strength` are on by default since 2026-10-23 (Ben's
//! rule of that day, "Always, fix V1 elsewhere": physics goes on once built and correct); off,
//! for tests that isolate one law, there is no state and heat flow is the old proxy. The thermal
//! height (`thermal_isostasy_m`, an open fault: no frame for its reference depth is right while
//! the model's sea is not) and the backstop (`thermal_backstop`, a stand-in) stay off.
//!
//! Stage 3: a collision belt's thickened mantle lithosphere drops off by Rayleigh-Taylor
//! instability, so its lithosphere heats to the Moho, or to its keel where it has one
//! ([`ROOT_REMOVAL_MYR`]). Stage 2 thins a rifted margin's lithosphere with its crust in
//! `transition.rs` ([`stretch`]); the cooling here is then McKenzie's thermal subsidence.

use rayon::prelude::*;

use super::Env;
use super::state::{
    ARC, CONTINENTAL, DIVERGENT, HOTSPOT, OCEANIC, OROGENY_HIMALAYAN, PlateState, SUBDUCTION_ZONE,
    Sample,
};
use crate::params::TectonicParams;

/// A hot back-arc's lithosphere, km: "lithosphere thicknesses are 50-60 km" (Hyndman, Currie &
/// Mazzotti 2005); "1200°C at 60 km depth" (Currie & Hyndman 2006).
pub const L_HOT_KM: f64 = 60.0; // ledger: PLT-461

/// An old craton's lithosphere, km: "relatively uniform, 225 km" (Sleep 2003, abstract); 200 to
/// 250 km (Jaupart et al. 1998 as Rolandone et al. 2002 cite it).
pub const L_COLD_KM: f64 = 225.0; // ledger: PLT-462

/// The keel under the start's continents, km (the default of `thermal_keel_km`): lithosphere
/// under old cratons "unlikely to be smaller than 200 km" (Jaupart & Mareschal 1999 as Rolandone
/// et al. 2002 cite it).
pub const KEEL_KM: f64 = 200.0; // ledger: PLT-463

/// The heating's e-folding time, Myr: convective thinning "may require less than 10 Myr if the
/// viscosity is less than about 10^21 Pa s" (Houseman & Molnar 1997, p. 131); removal of lower
/// back-arc lithosphere "on timescales of 5-10 Ma" (Currie, Huismans & Beaumont 2008).
pub const HEATING_MYR: f64 = 10.0; // ledger: PLT-464

/// Rayleigh-Taylor removal of a thickened mantle lithosphere (stage 3), e-folding time, Myr:
/// "between 50 (for rheological parameters representative of dry olivine) and 75 per cent (for wet
/// olivine) of the lithospheric mantle would be involved in an instability whose duration is of the
/// order of 10 Myr" (Houseman & Molnar 1997, conclusions); 1 − e^(−1), 63 per cent, goes in the
/// first 10 Myr here.
pub const ROOT_REMOVAL_MYR: f64 = 10.0; // ledger: PLT-477

/// McKenzie's (1978) plate: a = 125 km, τ = 62.8 My (Table 1), τ = a²/(π²κ) (eq. 6).
pub const PLATE_KM: f64 = 125.0; // ledger: PLT-465
pub const PLATE_TAU_MYR: f64 = 62.8; // ledger: PLT-465

/// The 90 % isotherm of a cooling half-space, 2.32 √(κt) (from-memory), with κ from McKenzie's τ
/// and a: κ = a² / (π² τ), in km² per Myr.
pub const HALF_SPACE: f64 = 2.32; // ledger: PLT-466 (from-memory)

/// The crust's part of continental heat flow, mW/m², and the mantle's, B / L in mW/m² with L in
/// km: 41 mW/m² over Archean crust (Nyblade & Pollack 1993, as cited) of which 15 crosses the base
/// of a 225 km lithosphere (11 to 17: Rolandone et al. 2002; Jaupart & Mareschal 1999 as Sleep 2003
/// cites them).
pub const CRUST_HEAT_MW_M2: f64 = 26.0; // ledger: PLT-467
pub const MANTLE_HEAT_MW_M2_KM: f64 = 3375.0; // ledger: PLT-467

/// The old proxy's magmatic terms, kept for an arc and a hotspot (PLT-137, uncited); a rift's
/// thinning is in L now.
pub const ARC_HEAT_MW_M2: f64 = 40.0; // ledger: PLT-468
pub const HOTSPOT_HEAT_MW_M2: f64 = 40.0; // ledger: PLT-468

/// The cooling's e-folding time at the craton's depth, Myr: τ = a²/(π²κ) scales with a².
pub fn cooling_myr() -> f64 {
    PLATE_TAU_MYR * (L_COLD_KM / PLATE_KM).powi(2) // ledger: PLT-465
}

/// κ in km² per Myr, from McKenzie's τ and a.
fn kappa_km2_myr() -> f64 {
    PLATE_KM * PLATE_KM / (std::f64::consts::PI.powi(2) * PLATE_TAU_MYR)
}

/// The ocean floor's lithosphere at `age_myr`, km: 2.32 √(κt), at most McKenzie's plate.
// ledger: PLT-466
pub fn ocean_km(age_myr: f64) -> f64 {
    (HALF_SPACE * (kappa_km2_myr() * age_myr.max(0.0)).sqrt()).min(PLATE_KM)
}

/// Continental heat flow from the lithosphere's depth alone (no magmatic term), mW/m².
// ledger: PLT-467
pub fn conductive_mw_m2(l_km: f64) -> f64 {
    CRUST_HEAT_MW_M2 + MANTLE_HEAT_MW_M2_KM / l_km.max(1.0)
}

/// Continental heat flow of a sample with flags `flags` over a lithosphere `l_km` deep, mW/m².
// ledger: PLT-467, PLT-468
pub fn heat_flow_mw_m2(l_km: f64, flags: u8) -> f64 {
    let mut q = conductive_mw_m2(l_km);
    if flags & ARC != 0 {
        q += ARC_HEAT_MW_M2;
    }
    if flags & HOTSPOT != 0 {
        q += HOTSPOT_HEAT_MW_M2;
    }
    q
}

/// The thermal term of a continental column's Airy height, m: z_th = E (L_ref − L) / (L_cold −
/// L_hot), 0 with E 0 (off).
// ledger: ISO-110, ISO-111
pub fn uplift_m(k: &TectonicParams, l_km: f32) -> f64 {
    Switches::of(k).uplift_m(l_km)
}

/// The state's switches as the thin sheet keeps them (`collapse::Sheet` is `Copy`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Switches {
    /// E, m: 0 unless `thermal_state` and `thermal_isostasy_m` (ledger: ISO-110).
    isostasy_m: f64,
    /// L_ref, km (ledger: ISO-111).
    reference_km: f64,
    /// `thermal_state` and `thermal_strength` (ledger: ISO-112).
    pub strength: bool,
}

impl Switches {
    pub fn of(k: &TectonicParams) -> Self {
        Switches {
            isostasy_m: if k.thermal_state {
                k.thermal_isostasy_m
            } else {
                0.0
            },
            reference_km: k.thermal_reference_km,
            strength: k.thermal_state && k.thermal_strength,
        }
    }

    /// [`uplift_m`] with these switches.
    // ledger: ISO-110, ISO-111
    pub fn uplift_m(&self, l_km: f32) -> f64 {
        let e = self.isostasy_m;
        if !(e > 0.0 && l_km > 0.0) {
            return 0.0;
        }
        e * (self.reference_km - f64::from(l_km)) / (L_COLD_KM - L_HOT_KM)
    }
}

/// The thermal age a lithosphere `l_km` deep stands for, Myr: the time the cooling law takes from
/// L_hot to it, τ_c ln((L_cold − L_hot) / (L_cold − L)); 0 at or above the hot depth, infinite at
/// the craton's.
// ledger: ISO-112
pub fn thermal_age_myr(l_km: f64) -> f64 {
    if l_km <= L_HOT_KM {
        0.0
    } else if l_km >= L_COLD_KM {
        f64::INFINITY
    } else {
        cooling_myr() * ((L_COLD_KM - L_HOT_KM) / (L_COLD_KM - l_km)).ln()
    }
}

/// The lithosphere a thermal age stands for, km: the cooling law's depth after `age_myr` from
/// L_hot, L_cold − (L_cold − L_hot) e^(−t/τ_c), the inverse of [`thermal_age_myr`] (L_hot at or
/// under 0, NaN for NaN).
// ledger: ISO-112, PLT-465
pub fn lithosphere_km_of_age(age_myr: f64) -> f64 {
    if age_myr.is_nan() {
        f64::NAN
    } else if age_myr <= 0.0 {
        L_HOT_KM
    } else {
        L_COLD_KM - (L_COLD_KM - L_HOT_KM) * (-age_myr / cooling_myr()).exp()
    }
}

/// The mantle's temperature at the base of the thermal lithosphere, °C: "T(a,t)=Tm=1350°C" at
/// the thermal bottom (Burov & Diament 1995, appendix).
pub const MANTLE_C: f64 = 1350.0; // ledger: ISO-138

/// The depth scale of the crust's radiogenic heat production, km: "hr≈10 km is the depth scale
/// for the decrease in radiogenic heat production" (Burov & Diament 1995, appendix).
pub const RADIOGENIC_DEPTH_KM: f64 = 10.0; // ledger: ISO-138

/// The steady conductive geotherm of a continental lithosphere `l_km` deep, as the plate model's
/// heat flow makes it, as a share of T_m at depth `z_km` (0 at the surface, 1 at L): the crust's
/// heat q_c ([`CRUST_HEAT_MW_M2`]) from sources decaying as e^(−z/h_r), the mantle's q_m = B / L
/// ([`MANTLE_HEAT_MW_M2_KM`]) through the whole lithosphere, and T_m at its base, so
/// T(z) / T_m = (q_m z + q_c h_r (1 − e^(−z/h_r))) / (q_m L + q_c h_r (1 − e^(−L/h_r))). The
/// conductivity this implies, (q_m L + q_c h_r (1 − e^(−L/h_r))) / T_m, is 2.69 W/m/K at every
/// L from L_hot on (q_m L is B whatever L is), within the 2.0 to 3.5 of Burov & Diament's
/// appendix (lower crust, mantle).
// ledger: ISO-138, PLT-467
pub fn geotherm_share(l_km: f64, z_km: f64) -> f64 {
    let l = l_km.max(1.0);
    let qm = MANTLE_HEAT_MW_M2_KM / l;
    let hr = RADIOGENIC_DEPTH_KM;
    let heat = |z: f64| qm * z + CRUST_HEAT_MW_M2 * hr * (1.0 - (-z / hr).exp());
    heat(z_km) / heat(l)
}

/// The depth of the isotherm `t_c` (°C, from 0 to T_m) in that geotherm, km. Newton's method from
/// the depth the surface heat flow's gradient gives, which is never deeper than it (the geotherm
/// is concave), so every step stays on the shallow side and the depths rise to it.
// ledger: ISO-138
pub fn isotherm_km(l_km: f64, t_c: f64) -> f64 {
    let l = l_km.max(1.0);
    let qm = MANTLE_HEAT_MW_M2_KM / l;
    let (qc, hr) = (CRUST_HEAT_MW_M2, RADIOGENIC_DEPTH_KM);
    let heat = |z: f64| qm * z + qc * hr * (1.0 - (-z / hr).exp());
    let target = (t_c / MANTLE_C).clamp(0.0, 1.0) * heat(l);
    let mut z = target / (qm + qc);
    for _ in 0..60 {
        let step = (target - heat(z)) / (qm + qc * (-z / hr).exp());
        z += step;
        if step.abs() < 1.0e-9 {
            break;
        }
    }
    z
}

/// Whether a continental sample with these flags is heated, and whether by a rift (which thins a
/// keel too) rather than by convection under it.
fn heating(flags: u8) -> Option<bool> {
    if flags & DIVERGENT != 0 {
        Some(true)
    } else if flags & (ARC | SUBDUCTION_ZONE | HOTSPOT) != 0 {
        Some(false)
    } else {
        None
    }
}

/// The crust, km, of a continental sample whose thickened mantle lithosphere is unstable (stage
/// 3): a collision belt (a Himalayan orogeny) whose crust is thicker than the model's thickened
/// crust, `collapse_thickness_km` (PLT-256: the lithosphere thickened with it by about the crust's
/// factor, 1.5 or more, Houseman, McKenzie & Molnar 1981's thickened boundary layer); `None`
/// elsewhere.
// ledger: PLT-478
pub fn root_crust_km(k: &TectonicParams, s: &Sample) -> Option<f64> {
    let crust_km = f64::from(s.thickness_m) / 1000.0;
    (s.orogeny == OROGENY_HIMALAYAN && crust_km > k.collapse_thickness_km).then_some(crust_km)
}

/// One step of a continental sample's state over `dt_myr`: (L, K) after it. `flags` are the
/// step's.
// ledger: PLT-460, PLT-464, PLT-465
pub fn advance(l_km: f64, keel_km: f64, flags: u8, dt_myr: f64) -> (f64, f64) {
    advance_with_root(l_km, keel_km, flags, None, dt_myr)
}

/// [`advance`] with stage 3: `root` is the crust, km, of a column whose thickened mantle
/// lithosphere is unstable ([`root_crust_km`]). Its lithosphere drops off towards the Moho over
/// [`ROOT_REMOVAL_MYR`] unless a keel holds it (Jordan's depleted tectosphere is buoyant): the
/// target is max(L_hot, K, crust). A rift or an arc's heating comes first.
// ledger: PLT-460, PLT-464, PLT-465, PLT-477
pub fn advance_with_root(
    l_km: f64,
    keel_km: f64,
    flags: u8,
    root: Option<f64>,
    dt_myr: f64,
) -> (f64, f64) {
    let (target, tau, rift) = match (heating(flags), root) {
        (Some(true), _) => (L_HOT_KM, HEATING_MYR, true),
        (Some(false), _) => (L_HOT_KM.max(keel_km), HEATING_MYR, false),
        (None, Some(crust_km)) => (L_HOT_KM.max(keel_km).max(crust_km), ROOT_REMOVAL_MYR, false),
        (None, None) => (L_COLD_KM, cooling_myr(), false),
    };
    // A convective target never lies below the keel; a lithosphere already thinner than its
    // target (a rift's, cooling) moves up towards it.
    let l = target + (l_km - target) * (-dt_myr / tau).exp();
    let keel = if rift { keel_km.min(l) } else { keel_km };
    (l, keel.min(l).max(0.0))
}

/// Sets the start's continents' state (L at the frame's depth, a keel) on a fresh planet.
pub fn start(state: &mut PlateState, k: &TectonicParams) {
    if !k.thermal_state {
        return;
    }
    let l = k.thermal_reference_km;
    let keel = k.thermal_keel_km.min(l); // ledger: PLT-463
    for s in &mut state.samples {
        if s.kind == CONTINENTAL {
            s.lithosphere_km = l as f32;
            s.keel_km = keel as f32;
        }
    }
}

/// The share of the cordillera's strain a source behind the range takes (`subduction.rs`): 1,
/// or with `thermal_backstop` the thin sheet's η_0 / η at its lithosphere's thermal age,
/// e^(−t_eq / τ_s) (PLT-415's law, τ_s = `collapse_sheet_strength_myr`): a hot back-arc gives its
/// crust, a craton at its full depth gives none. Hyndman, Currie & Mazzotti (2005): back-arc
/// lithospheres "weaker than cratons by more than a factor of 10".
// ledger: PLT-470
pub fn backstop_weight(k: &TectonicParams, l_km: f32) -> f64 {
    let tau = k.collapse_sheet_strength_myr;
    if !(k.thermal_state && k.thermal_backstop && l_km > 0.0 && tau > 0.0) {
        return 1.0;
    }
    (-thermal_age_myr(f64::from(l_km)) / tau).exp()
}

/// The thin sheet's weakness at a lithosphere `l_km` deep, η_0 / η = e^(−t_eq / τ_s) with t_eq
/// its thermal age and τ_s `collapse_sheet_strength_myr` (`tau_myr`; PLT-415's law with ISO-112's
/// age, PLT-470's form): 1 for a hot lithosphere at or above L_hot, 0 for a craton at L_cold, and
/// 1 everywhere when τ_s ≤ 0 (no growth, as in the collapse's sheet). A sample with no state
/// (`l_km` ≤ 0) has no strength to read and takes no strain: 0. For the overriding plate's share
/// of a collision (`belts.rs`, PLT-652).
// ledger: PLT-652, PLT-415, ISO-112
pub fn sheet_weakness(tau_myr: f64, l_km: f32) -> f64 {
    if l_km <= 0.0 || l_km.is_nan() {
        return 0.0;
    }
    if tau_myr.is_nan() || tau_myr <= 0.0 {
        return 1.0;
    }
    (-thermal_age_myr(f64::from(l_km)) / tau_myr).exp()
}

/// The step's update (see the module docs): every oceanic sample's L from its age, every visible
/// continental sample's L and K from the step's flags. Runs after the hotspots, before the
/// settling.
pub fn step(state: &mut PlateState, env: &Env) {
    let k = env.k;
    if !k.thermal_state {
        return;
    }
    let (t, dt) = (env.t_myr, env.dt_myr);
    let reference = k.thermal_reference_km as f32;
    state.samples.par_iter_mut().for_each(|s| {
        if s.kind == OCEANIC {
            s.lithosphere_km = ocean_km(s.age_myr(t)) as f32;
            s.keel_km = 0.0;
            return;
        }
        if s.lithosphere_km <= 0.0 {
            // Continental crust with no state yet (made mid-run from nothing): the frame's depth,
            // no keel.
            s.lithosphere_km = reference;
            s.keel_km = 0.0;
        }
        if s.is_hidden() {
            return;
        }
        let (l, keel) = advance_with_root(
            f64::from(s.lithosphere_km),
            f64::from(s.keel_km),
            s.flags,
            root_crust_km(k, s),
            dt,
        );
        s.lithosphere_km = l as f32;
        s.keel_km = keel as f32;
    });
}

/// Stage 2: a continental column whose crystalline crust the transition thins
/// to `keep` of what it was (a stretch by β = 1 / `keep`) has its lithosphere and keel thinned by
/// the same factor, L / β and K / β (McKenzie 1978's uniform stretching of the whole lithosphere,
/// pure shear); a sample with no state keeps none.
// ledger: PLT-476
pub fn stretch(s: &mut Sample, keep: f64) {
    if s.kind != CONTINENTAL || s.lithosphere_km <= 0.0 || !(0.0..1.0).contains(&keep) {
        return;
    }
    s.lithosphere_km = (f64::from(s.lithosphere_km) * keep).max(1.0) as f32;
    s.keel_km = (f64::from(s.keel_km) * keep).min(f64::from(s.lithosphere_km)) as f32;
}

/// The heat flow of a continental sample as the state gives it, mW/m² (`finish_samples`).
pub fn sample_heat_flow(l_km: f32, flags: u8) -> f32 {
    heat_flow_mw_m2(f64::from(l_km), flags) as f32
}

/// The final pass after the step's resampling (ledger: PLT-469). Crust that became continental
/// after the step set its heat flow (the rate guard's spread onto ocean floor after the
/// resampling, `collapse::spread`) has none, since its floor's went with the floor's kind
/// ([`Sample::make_continental`]): it gets its law's, as `finish_samples` sets it
/// ([`continental_heat_flow`](super::continental_heat_flow)), with its floor's lithosphere under the
/// state. Each other continental sample gets at least its law's conductive value: the state's with
/// `thermal_state`, else the old proxy's (55 + 35 e^(−t/60), PLT-137).
pub fn refresh_heat_flow(state: &mut PlateState, k: &TectonicParams, t_myr: f64) {
    if !(k.heat_flow_refresh || k.thermal_state) {
        return;
    }
    let thermal = k.thermal_state;
    let reference = k.thermal_reference_km;
    state.samples.par_iter_mut().for_each(|s| {
        if s.kind != CONTINENTAL {
            return;
        }
        if s.heat_flow_mw_m2 <= 0.0 {
            s.heat_flow_mw_m2 = super::continental_heat_flow(s, t_myr, thermal, reference as f32);
            return;
        }
        let floor = if thermal {
            let l = if s.lithosphere_km > 0.0 {
                f64::from(s.lithosphere_km)
            } else {
                reference
            };
            conductive_mw_m2(l)
        } else {
            let since = (t_myr - f64::from(s.orogeny_myr)).max(0.0);
            55.0 + 35.0 * (-since / 60.0).exp() // ledger: PLT-137
        };
        s.heat_flow_mw_m2 = s.heat_flow_mw_m2.max(floor as f32);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_heat_flow_law_meets_its_anchors() {
        // 41 over the craton's 225 km, 53 over McKenzie's 125 km plate, 82 over a back-arc's 60.
        assert!((conductive_mw_m2(L_COLD_KM) - 41.0).abs() < 1e-9);
        assert!((conductive_mw_m2(PLATE_KM) - 53.0).abs() < 1e-9);
        assert!((conductive_mw_m2(L_HOT_KM) - 82.25).abs() < 1e-9);
        assert!((heat_flow_mw_m2(L_HOT_KM, ARC | HOTSPOT) - 162.25).abs() < 1e-9);
        // A rift's heat is in its depth now, not in a term.
        assert_eq!(heat_flow_mw_m2(100.0, DIVERGENT), conductive_mw_m2(100.0));
    }

    #[test]
    fn mckenzies_time_scales_with_the_square_of_the_plate() {
        assert!((cooling_myr() - 62.8 * 1.8 * 1.8).abs() < 1e-9);
        // κ back from τ and a: 0.80 × 10⁻⁶ m²/s.
        let kappa_m2_s = kappa_km2_myr() * 1.0e6 / (1.0e6 * 365.25 * 86400.0);
        assert!((kappa_m2_s - 0.80e-6).abs() < 0.01e-6, "{kappa_m2_s}");
        // The floor's lithosphere reaches the plate after about 115 Myr and stays there.
        assert!((ocean_km(100.0) - 116.6).abs() < 0.5, "{}", ocean_km(100.0));
        assert_eq!(ocean_km(500.0), PLATE_KM);
    }

    #[test]
    fn a_back_arc_heats_new_crust_in_tens_of_myr_and_it_cools_over_hundreds() {
        // New continental crust from 125 km floor behind a trench, 2 Myr steps.
        let (mut l, mut keel) = (PLATE_KM, 0.0);
        for _ in 0..20 {
            (l, keel) = advance(l, keel, SUBDUCTION_ZONE, 2.0);
        }
        // 40 Myr is four of its e-folding times: 60 + 65 e^-4 = 61.2 km.
        assert!((l - L_HOT_KM).abs() < 1.5, "{l}");
        // 100 Myr after the subduction stops it is still hot (CH 2006: "most rapid cooling ... in
        // the first 50-100 Myr", decay "over a timescale of about 300 Myr").
        for _ in 0..50 {
            (l, keel) = advance(l, keel, 0, 2.0);
        }
        assert!(l > 100.0 && l < 130.0, "{l}");
        for _ in 0..150 {
            (l, keel) = advance(l, keel, 0, 2.0);
        }
        assert!(l > 200.0 && l < L_COLD_KM, "{l}");
        assert_eq!(keel, 0.0);
    }

    /// The savanna-pit case (flexure.md, "The release's pace"): a craton an Andean arc's
    /// back-arc reaches. Its keel holds: after 40 Myr behind the trench the lithosphere is the
    /// keel's 200 km, not a back-arc's 60, and its thermal age keeps the sheet and the flexure
    /// stiff (the orogeny's new date made its Te fall from 80 to 5 km in one step).
    #[test]
    fn a_keeled_craton_behind_an_arc_stays_cold_and_stiff() {
        let (mut l, mut keel) = (L_COLD_KM, KEEL_KM);
        for _ in 0..20 {
            (l, keel) = advance(l, keel, SUBDUCTION_ZONE, 2.0);
        }
        assert!((KEEL_KM..201.0).contains(&l), "{l}");
        assert_eq!(keel, KEEL_KM);
        let age = thermal_age_myr(l);
        assert!(age > 350.0, "{age}");
        // The flexure's continental law (ISO-050, ISO-051): 80 (1 − e^(−t/300)).
        let te = 80.0 * (1.0 - (-age / 300.0).exp());
        assert!(te > 55.0, "{te}");
        // The sheet's viscosity (PLT-415): more than 250 times a fresh orogen's.
        assert!((age / PLATE_TAU_MYR).exp() > 250.0);
        // Heat flow stays cratonic.
        assert!(conductive_mw_m2(l) < 43.0);
        // A rift does thin it, keel and all.
        for _ in 0..20 {
            (l, keel) = advance(l, keel, DIVERGENT, 2.0);
        }
        assert!(l < 65.0 && keel <= l, "{l} {keel}");
    }

    /// Stage 3 (PLT-477): a collision belt with no keel loses half to three quarters of its
    /// mantle lithosphere in 10 Myr (Houseman & Molnar 1997) and its lithosphere ends at its Moho;
    /// a keel holds it (the target is the keel); a rift or an arc's heating comes first.
    #[test]
    fn a_belts_thickened_mantle_lithosphere_drops_off_unless_a_keel_holds_it() {
        let crust = 65.0;
        let (mut l, mut keel) = (L_COLD_KM, 0.0);
        for _ in 0..5 {
            (l, keel) = advance_with_root(l, keel, 0, Some(crust), 2.0);
        }
        let removed = (L_COLD_KM - l) / (L_COLD_KM - crust);
        assert!((0.5..=0.75).contains(&removed), "{removed}");
        for _ in 0..50 {
            (l, keel) = advance_with_root(l, keel, 0, Some(crust), 2.0);
        }
        assert!((l - crust).abs() < 0.1, "{l}");
        assert_eq!(keel, 0.0);
        // Its heat flow is near a back-arc's, its thermal age young (6.3 Myr at 65 km).
        assert!(conductive_mw_m2(l) > 75.0 && thermal_age_myr(l) < 10.0);
        // A keel holds it: the lithosphere stops at the keel.
        let (mut l, mut keel) = (L_COLD_KM, KEEL_KM);
        for _ in 0..50 {
            (l, keel) = advance_with_root(l, keel, 0, Some(crust), 2.0);
        }
        assert!((l - KEEL_KM).abs() < 0.1 && keel == KEEL_KM, "{l} {keel}");
        // A rift's heating comes first; with no root it cools as before.
        let rift = advance_with_root(L_COLD_KM, KEEL_KM, DIVERGENT, Some(crust), 2.0);
        assert_eq!(rift, advance(L_COLD_KM, KEEL_KM, DIVERGENT, 2.0));
        assert_eq!(
            advance_with_root(100.0, 0.0, 0, None, 2.0),
            advance(100.0, 0.0, 0, 2.0)
        );
    }

    /// The root is unstable only in a collision belt thickened past the collapse threshold.
    #[test]
    fn only_a_thickened_collision_belt_loses_its_root() {
        let k = TectonicParams::default();
        let mut s = Sample::blank(0);
        s.kind = CONTINENTAL;
        s.orogeny = OROGENY_HIMALAYAN;
        s.thickness_m = 60_000.0;
        assert_eq!(root_crust_km(&k, &s), Some(60.0));
        s.thickness_m = 45_000.0;
        assert_eq!(root_crust_km(&k, &s), None, "not thickened");
        s.thickness_m = 60_000.0;
        s.orogeny = crate::plates::state::OROGENY_ANDEAN;
        assert_eq!(root_crust_km(&k, &s), None, "an arc's heating is its own");
    }

    /// Stage 2 (PLT-476): stretching by β thins the lithosphere and the keel by β; ocean floor and
    /// crust with no state keep theirs.
    #[test]
    fn stretching_thins_the_lithosphere_and_its_keel_by_beta() {
        let mut s = Sample::blank(0);
        s.kind = CONTINENTAL;
        s.lithosphere_km = 225.0;
        s.keel_km = 200.0;
        stretch(&mut s, 0.5);
        assert_eq!((s.lithosphere_km, s.keel_km), (112.5, 100.0));
        stretch(&mut s, 1.0);
        assert_eq!((s.lithosphere_km, s.keel_km), (112.5, 100.0));
        let mut o = Sample::blank(0);
        o.kind = OCEANIC;
        o.lithosphere_km = 80.0;
        stretch(&mut o, 0.5);
        assert_eq!(o.lithosphere_km, 80.0);
        let mut n = Sample::blank(0);
        n.kind = CONTINENTAL;
        stretch(&mut n, 0.5);
        assert_eq!(n.lithosphere_km, 0.0);
        // McKenzie's cooling then brings it back: a β = 2 margin's lithosphere is two thirds back
        // after one τ_c (the relaxation's 1 − e^(−1) of the way).
        let (mut l, mut keel) = (f64::from(s.lithosphere_km), f64::from(s.keel_km));
        let steps = (cooling_myr() / 2.0).round() as usize;
        for _ in 0..steps {
            (l, keel) = advance(l, keel, 0, 2.0);
        }
        let back = (l - 112.5) / (L_COLD_KM - 112.5);
        assert!(
            (back - (1.0 - (-1.0f64).exp())).abs() < 0.01,
            "{back} {keel}"
        );
    }

    #[test]
    fn a_cold_craton_is_a_backstop_and_a_hot_back_arc_gives_its_crust() {
        let mut k = TectonicParams::default();
        assert_eq!(backstop_weight(&k, 60.0), 1.0, "off by default");
        k.thermal_state = true;
        k.thermal_backstop = true;
        assert_eq!(backstop_weight(&k, L_HOT_KM as f32), 1.0);
        assert_eq!(backstop_weight(&k, L_COLD_KM as f32), 0.0);
        // The keeled craton behind an arc (200.5 km): about 1/480 of a back-arc's share.
        let w = backstop_weight(&k, 200.46);
        assert!(w > 1.0 / 600.0 && w < 1.0 / 400.0, "{w}");
        // A back-arc 100 Myr after its arc stopped: about a fifth.
        let w = backstop_weight(&k, 124.8);
        assert!(w > 0.15 && w < 0.25, "{w}");
    }

    #[test]
    fn the_thermal_age_inverts_the_cooling() {
        let (mut l, mut keel) = (L_HOT_KM, 0.0);
        for _ in 0..50 {
            (l, keel) = advance(l, keel, 0, 2.0);
        }
        assert!(
            (thermal_age_myr(l) - 100.0).abs() < 1e-6,
            "{}",
            thermal_age_myr(l)
        );
        assert_eq!(thermal_age_myr(L_HOT_KM), 0.0);
        assert!(thermal_age_myr(L_COLD_KM).is_infinite());
        let _ = keel;
    }

    #[test]
    fn the_thermal_height_is_zero_at_the_frame_and_spans_its_e() {
        let mut k = TectonicParams::default();
        assert_eq!(uplift_m(&k, 60.0), 0.0, "off by default");
        k.thermal_state = true;
        k.thermal_isostasy_m = 1600.0;
        k.thermal_reference_km = L_COLD_KM;
        assert_eq!(uplift_m(&k, L_COLD_KM as f32), 0.0);
        assert!((uplift_m(&k, L_HOT_KM as f32) - 1600.0).abs() < 1e-9);
        k.thermal_reference_km = 187.0;
        let span = uplift_m(&k, L_HOT_KM as f32) - uplift_m(&k, L_COLD_KM as f32);
        assert!((span - 1600.0).abs() < 1e-9);
    }
}
