// SPDX-License-Identifier: LGPL-3.0-or-later
//! The pelagic rain (2026-10-25): what settles on the sea floor out of the air and the water,
//! beyond the neritic factory's carbonate and the rivers' and the sea's mud. It replaced the
//! plates' unsourced 5 m/Myr on oceanic crust (PLT-103, retired) and the fixed 4 m/Myr of chalk
//! (ERO-191), and it closes the dissolved load's leak (ERO-205): the rock the land's weathering
//! takes away in solution comes back to the floor. Three parts, each from a measured flux, none
//! fitted to a sediment thickness:
//!
//! - **Aeolian dust** (ERO-208, ERO-209). Earth's arid land sends 450 Tg of mineral dust a year to
//!   the oceans (Jickells et al. 2005, Fig. 2). A planet sends that flux per km² of Earth's arid
//!   and hyper-arid land (aridity index below 0.2, UNEP's arid limit: BIO-038) times its own such
//!   land, taken off that land evenly and laid evenly on the sea. Jickells gives the ocean basins'
//!   shares (North Atlantic 43 %, Indian 25 %, ...), which follow Earth's deserts and winds and are
//!   no law a planet can use, so the dust falls evenly: downwind transport is a limit.
//! - **Biogenic opal** (ERO-211, ERO-212, ERO-213). The dissolved load's silica (the SiO₂ share of
//!   its mass, West et al. 2005 Table 2, the craton ERO-206 takes) is buried as the modern ocean
//!   buries it, a steady state (Tréguer et al. 2021: inputs 14.8, outputs 15.6 Tmol Si/yr): the
//!   floor deeper than 1 km takes the open ocean's 5.5 of the 15.6 (Hayes et al. 2021's burial
//!   below 1 km), the shallower floor the rest (the coastal and continental-margin zone's burial
//!   3.7, reverse weathering 4.7, sponges 1.7).
//! - **Carbonate** (ERO-214, ERO-191, ERO-192). Each mol of the dissolved Ca + Mg is buried as a
//!   mol of calcite, less the part the climate's drawdown carbonate already laid (the cap
//!   carbonate, `climate/cap_carbonate_mol`). The planktic calcifiers' share (`deposits::clocks`)
//!   rains as chalk on the floor above the carbonate compensation depth, each cell's share in
//!   proportion to what survives dissolution at its depth ([`pelagic_share`]; the CCD RCK-015):
//!   what dissolves below the lysocline goes back into the sea and is laid where it survives, so
//!   the whole supply is laid. The rest (before the calcifiers, all of it) is laid as the cap
//!   carbonate is, evenly on the floor 0 to [`super::CARBONATE_FLOOR_M`] deep (ERO-067).
//!
//! The rain is a plate step's supply, laid once per step after its last substep, on the floor
//! as the substeps leave it (so its arrays live only in the step's last phase); cells count as
//! equal area, as everywhere in the surface's mass budget. A cell takes no more than the water
//! above it: what a full cell cannot hold settles on the cells of its floor that can
//! ([`Rain::fill`]), so the supply is laid whole while the sea has room; the dust taken off the
//! land is what the sea got. The pile is solid, as every pool and layer of the model (no pore water):
//! the comparison with GlobSed's in-place thickness decompacts it (the unit test).

use rayon::prelude::*;

use crate::climate::carbon::{CALCITE_M3_PER_MOL, ROCK_KG_M3};

/// Earth's mineral dust that reaches the oceans, kg/yr: "Total atmospheric dust inputs to the
/// oceans = 450 Tg year⁻¹" (Jickells et al. 2005, Fig. 2 caption).
pub const DUST_TO_SEA_EARTH_KG_YR: f64 = 450.0e9; // ledger: ERO-208
/// Earth's arid and hyper-arid land (aridity index below 0.2), km²: hyper-arid 9.8 and arid 15.7
/// × 10⁶ km² (Millennium Ecosystem Assessment, Safriel et al. 2005, Table 22.1, after Middleton
/// and Thomas 1997).
pub const ARID_EARTH_KM2: f64 = 25.5e6; // ledger: ERO-209
/// The grains' density, kg/m³: the dust's volume taken off the land and laid on the sea. Silt's
/// grain density in Kominz et al. 2011's table (as pyBacktrack's lithology table gives it; clay
/// 2,735).
pub const DUST_KG_M3: f64 = 2661.0; // ledger: ERO-210

/// The SiO₂ share of the dissolved load's mass: West et al. 2005, Table 2, Grenville (the craton
/// ERO-206 takes), SiO₂ 2.63 of SCDR 1.14 + SiO₂ 2.63 t/km²/yr.
pub const SILICA_SHARE: f64 = 2.63 / (1.14 + 2.63); // ledger: ERO-211
/// Molar mass of SiO₂, kg/mol (28.086 + 2 × 15.999 g/mol).
pub const SIO2_KG_PER_MOL: f64 = 0.060_08; // ledger: ERO-213
/// The deep sea's floor starts this deep, m: Hayes et al. 2021 integrate the open ocean's burial
/// over "water depths > 1000 m" (Tréguer et al. 2021: "open-ocean locations > 1 km in depth").
pub const DEEP_FLOOR_M: f64 = 1000.0; // ledger: ERO-212
/// The deep sea's share of the buried silica: the open ocean's 5.5 of the 15.6 Tmol Si/yr of
/// outputs (Tréguer et al. 2021, sec. 3.1 and 3.4).
pub const DEEP_SILICA_SHARE: f64 = 5.5 / 15.6; // ledger: ERO-212
/// Opal's grain density, kg/m³ (radiolarians 1,850 to 2,000, sponge spicules 1,950 to 2,100,
/// diatoms 1,700 to 1,950: the review in Frontiers in Marine Science 2023,
/// doi 10.3389/fmars.2023.1150518).
pub const OPAL_KG_M3: f64 = 2000.0; // ledger: ERO-213
/// Opal's water, mass share: about 10 % (SiO₂·0.4H₂O, Mortlock and Froelich 1989, read at second
/// hand).
pub const OPAL_WATER: f64 = 0.10; // ledger: ERO-213

/// Calcite buried per mol of the Ca + Mg the land's silicate weathering dissolved.
pub const CALCITE_PER_CATION: f64 = 1.0; // ledger: ERO-214

/// The lysocline lies this far above the CCD: the ooze's dissolution grows linearly from none at
/// the lysocline to all of it at the CCD. "Within a further 1000 m [below the saturation horizon]
/// sediments are often completely devoid of carbonate particles (the CCD)" (Ridgwell and Zeebe
/// 2005, sec. 1.1); the linear taper is the model's.
pub const LYSOCLINE_ABOVE_CCD_M: f64 = 1000.0; // ledger: ERO-192

/// The share of the carbonate rain that survives at water depth `z` (m) over a CCD at `ccd_m`.
pub fn pelagic_share(z: f64, ccd_m: f64) -> f64 {
    if z <= 0.0 {
        return 0.0;
    }
    ((ccd_m - z) / LYSOCLINE_ABOVE_CCD_M).clamp(0.0, 1.0)
}

/// Earth's dust to the sea per m² of its arid land, kg/m²/yr.
pub fn dust_kg_m2_yr() -> f64 {
    DUST_TO_SEA_EARTH_KG_YR / (ARID_EARTH_KM2 * 1.0e6)
}

/// Volume of a mol of silica laid as opal, m³ (its water included).
pub fn opal_m3_per_mol() -> f64 {
    SIO2_KG_PER_MOL / (1.0 - OPAL_WATER) / OPAL_KG_M3
}

/// GlobSed's eq. 2a (Straume et al. 2019, sec. 3.3): the sediment on ocean floor `age_myr` old at
/// latitude `lat_deg`, more than 200 km from the margins, in place (pore water included), m:
/// Z = √τ (52 − 2.46 |φ| + 0.045 φ²), fitted on floor younger than 82 Myr below 72°.
pub fn globsed_eq_2a_m(age_myr: f64, lat_deg: f64) -> f64 {
    let phi = lat_deg.abs();
    age_myr.max(0.0).sqrt() * (52.0 - 2.46 * phi + 0.045 * phi * phi) // ledger: ERO-215
}

/// Kominz et al. 2011's average ocean-floor sediment, as pyBacktrack's lithology table gives it:
/// the porosity at the sea floor, and the depth over which it falls by a factor e, m.
pub const OCEAN_SEDIMENT_POROSITY: f64 = 0.66; // ledger: ERO-215
pub const OCEAN_SEDIMENT_DECAY_M: f64 = 1333.0; // ledger: ERO-215

/// The rock in a pile `z_m` thick in place under that porosity (φ₀ e^(−depth/λ)), m: the pile
/// less its pore water, z − φ₀ λ (1 − e^(−z/λ)).
pub fn solid_of_in_place_m(z_m: f64) -> f64 {
    let (phi0, lam) = (OCEAN_SEDIMENT_POROSITY, OCEAN_SEDIMENT_DECAY_M);
    let z = z_m.max(0.0);
    z - phi0 * lam * (1.0 - (-z / lam).exp())
}

/// The open ocean's pile on floor `age_myr` old at `lat_deg`, as rock the way the model's pools
/// hold it, m: GlobSed's eq. 2a less its pore water. The line PLT-661 draws between floor with the
/// pelagic rain alone and floor holding the continents' rock (the rain lies 14 to 42 m in place
/// under eq. 2a, ERO-215). Past the fit's 82 Myr and 72° it is eq. 2a carried on.
pub fn open_floor_solid_m(age_myr: f64, lat_deg: f64) -> f64 {
    solid_of_in_place_m(globsed_eq_2a_m(age_myr, lat_deg))
}

/// A plate step's supply to the rain.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Supply {
    /// Mineral dust blown off the arid land, m³ of grains.
    pub dust_m3: f64,
    /// Calcite the dissolved Ca + Mg makes beyond the cap carbonate, mol.
    pub calcite_mol: f64,
    /// The dissolved load's silica, mol Si.
    pub silica_mol: f64,
}

impl Supply {
    /// The supply of a step `years` long whose weathering took `dissolved_m3` of rock away in
    /// solution (at ρ_rock, CLM-154), `kg_per_mol` of it per mol of Ca + Mg (ERO-206), of which
    /// the share `cap_share` was already laid as the climate's drawdown carbonate, with `arid_m2`
    /// of arid land.
    pub fn new(
        dissolved_m3: f64,
        kg_per_mol: f64,
        cap_share: f64,
        arid_m2: f64,
        years: f64,
    ) -> Supply {
        let mass = dissolved_m3.max(0.0) * ROCK_KG_M3;
        let cations = if kg_per_mol > 0.0 {
            mass / kg_per_mol
        } else {
            0.0
        };
        Supply {
            dust_m3: dust_kg_m2_yr() * arid_m2.max(0.0) * years.max(0.0) / DUST_KG_M3,
            calcite_mol: cations * CALCITE_PER_CATION * (1.0 - cap_share).clamp(0.0, 1.0),
            silica_mol: mass * SILICA_SHARE / SIO2_KG_PER_MOL,
        }
    }

    /// The volume the supply lays when every part finds its floor and its room, m³.
    pub fn volume_m3(&self) -> f64 {
        self.dust_m3.max(0.0)
            + self.silica_mol.max(0.0) * opal_m3_per_mol()
            + self.calcite_mol.max(0.0) * CALCITE_M3_PER_MOL
    }
}

/// Cells per chunk of the rain's sums: fixed, so the thread count does not change them.
const CHUNK: usize = 4096;

/// The sea floor the rain falls on, in cells of equal area, from each cell's water depth before
/// the rain.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Floor {
    /// Sea floor (deeper than 0).
    pub sea: f64,
    /// Deeper than [`DEEP_FLOOR_M`].
    pub deep: f64,
    /// [`pelagic_share`] summed over the sea floor: where chalk survives.
    pub calcite: f64,
    /// 0 to [`super::CARBONATE_FLOOR_M`] deep (the cap carbonate's floor).
    pub shelf: f64,
}

impl Floor {
    /// Counts a cell `z` m deep.
    pub fn add(&mut self, z: f64, ccd_m: f64) {
        if z <= 0.0 {
            return;
        }
        self.sea += 1.0;
        if z > DEEP_FLOOR_M {
            self.deep += 1.0;
        }
        self.calcite += pelagic_share(z, ccd_m);
        if z <= super::CARBONATE_FLOOR_M {
            self.shelf += 1.0;
        }
    }

    /// Both counts together.
    pub fn merge(self, o: &Floor) -> Floor {
        Floor {
            sea: self.sea + o.sea,
            deep: self.deep + o.deep,
            calcite: self.calcite + o.calcite,
            shelf: self.shelf + o.shelf,
        }
    }

    /// The floor of cells `z` m deep (0 or less: not sea).
    pub fn of(z: &[f64], ccd_m: f64) -> Floor {
        z.par_chunks(CHUNK)
            .map(|zs| {
                let mut f = Floor::default();
                for &x in zs {
                    f.add(x, ccd_m);
                }
                f
            })
            .collect::<Vec<_>>()
            .iter()
            .fold(Floor::default(), |a, b| a.merge(b))
    }
}

/// What the rain lays, m (on one cell) or m × cells (summed).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Laid {
    /// The calcifiers' chalk (the surface's pelagic part).
    pub chalk: f64,
    /// The rest of the returned carbonate (the surface's carbonate part).
    pub carbonate: f64,
    pub opal: f64,
    pub dust: f64,
}

impl Laid {
    pub fn total(&self) -> f64 {
        self.chalk + self.carbonate + self.opal + self.dust
    }
}

/// The rain's parts, in the order they are laid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    Chalk,
    /// The returned carbonate beside the chalk, laid as the cap carbonate is.
    Carbonate,
    /// Opal on the floor no deeper than [`DEEP_FLOOR_M`].
    OpalMargin,
    /// Opal on the floor deeper than [`DEEP_FLOOR_M`].
    OpalDeep,
    Dust,
}

impl Part {
    pub const ALL: [Part; 5] = [
        Part::Chalk,
        Part::Carbonate,
        Part::OpalMargin,
        Part::OpalDeep,
        Part::Dust,
    ];

    /// The part's weight on a cell `z` m deep: the share of the part's thickness it gets.
    pub fn weight(self, z: f64, ccd_m: f64) -> f64 {
        if z <= 0.0 {
            return 0.0;
        }
        let on = |b: bool| if b { 1.0 } else { 0.0 };
        match self {
            Part::Chalk => pelagic_share(z, ccd_m),
            Part::Carbonate => on(z <= super::CARBONATE_FLOOR_M),
            Part::OpalMargin => on(z <= DEEP_FLOOR_M),
            Part::OpalDeep => on(z > DEEP_FLOOR_M),
            Part::Dust => 1.0,
        }
    }
}

/// The surface's part arrays the rain adds to, m per cell.
pub struct Outs<'a> {
    pub chalk: &'a mut [f32],
    pub carbonate: &'a mut [f32],
    pub opal: &'a mut [f32],
    pub dust: &'a mut [f32],
}

/// A step's supply spread over its sea floor.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rain {
    /// Each part's volume over the cell area, m × cells, in [`Part::ALL`]'s order.
    pub volume: [f64; 5],
    /// Each part's weights summed over the floor ([`Part::weight`]), in the same order.
    pub floor: [f64; 5],
    pub ccd_m: f64,
}

impl Rain {
    /// Spreads `s` over `floor` (cells of `cell_m2`) with the calcifiers at `calcifiers` (a
    /// share, 0 to 1, of their full rain). A part whose floor is missing goes to the other floor
    /// of its kind (opal: deep and margin; carbonate: the chalk's and the shelf's); with neither
    /// it is not laid.
    pub fn spread(s: &Supply, calcifiers: f64, floor: &Floor, cell_m2: f64, ccd_m: f64) -> Rain {
        let margin = floor.sea - floor.deep;
        let deep_share = if floor.deep <= 0.0 {
            0.0
        } else if margin <= 0.0 {
            1.0
        } else {
            DEEP_SILICA_SHARE
        };
        let opal = s.silica_mol.max(0.0) * opal_m3_per_mol();
        let calcite = s.calcite_mol.max(0.0) * CALCITE_M3_PER_MOL;
        let c = calcifiers.clamp(0.0, 1.0);
        let (mut chalk, mut shelf) = (c * calcite, (1.0 - c) * calcite);
        if floor.calcite <= 0.0 {
            (chalk, shelf) = (0.0, calcite);
        } else if floor.shelf <= 0.0 {
            (chalk, shelf) = (calcite, 0.0);
        }
        let per = if cell_m2 > 0.0 { 1.0 / cell_m2 } else { 0.0 };
        Rain {
            volume: [
                chalk * per,
                shelf * per,
                opal * (1.0 - deep_share) * per,
                opal * deep_share * per,
                s.dust_m3.max(0.0) * per,
            ],
            floor: [floor.calcite, floor.shelf, margin, floor.deep, floor.sea],
            ccd_m,
        }
    }

    /// The part's thickness where its weight is 1, m (room enough everywhere).
    pub fn thickness(&self, p: Part) -> f64 {
        let i = p as usize;
        if self.floor[i] > 0.0 {
            self.volume[i] / self.floor[i]
        } else {
            0.0
        }
    }

    /// What it lays on a cell `z` m deep with room enough, m.
    pub fn lay(&self, z: f64) -> Laid {
        let t = |p: Part| self.thickness(p) * p.weight(z, self.ccd_m);
        Laid {
            chalk: t(Part::Chalk),
            carbonate: t(Part::Carbonate),
            opal: t(Part::OpalMargin) + t(Part::OpalDeep),
            dust: t(Part::Dust),
        }
    }

    /// Lays the rain on cells `z` m deep with `room` m of water left above each (taken down by
    /// what it gets), adding each part to its array in `out`; returns what was laid, m × cells.
    /// Each part in turn ([`Part::ALL`]'s order) is laid in proportion to its weight but at most
    /// the room a cell has left: what a full cell cannot hold settles on the cells that can (the
    /// thickness s · weight with the s that lays the part whole, [`fill`]). What the part's floor
    /// cannot hold at all goes on to its other floor (the chalk's to the shelf carbonate's, the
    /// margin opal's to the deep opal's), or is not laid.
    pub fn fill(&self, z: &[f64], room: &mut [f64], out: Outs<'_>) -> Laid {
        let Outs {
            chalk,
            carbonate,
            opal,
            dust,
        } = out;
        let mut laid = [0.0; 5];
        let mut carry = 0.0;
        for p in Part::ALL {
            let i = p as usize;
            let v = self.volume[i] + carry;
            let target: &mut [f32] = match p {
                Part::Chalk => chalk,
                Part::Carbonate => carbonate,
                Part::OpalMargin | Part::OpalDeep => opal,
                Part::Dust => dust,
            };
            laid[i] = fill(z, room, &|x| p.weight(x, self.ccd_m), v, target);
            carry = match p {
                Part::Chalk | Part::OpalMargin => (v - laid[i]).max(0.0),
                _ => 0.0,
            };
        }
        Laid {
            chalk: laid[0],
            carbonate: laid[1],
            opal: laid[2] + laid[3],
            dust: laid[4],
        }
    }
}

/// Lays `volume` (m × cells) on cells `z` m deep in proportion to `weight(z)`, each at most its
/// `room` (m, taken down by what it gets): the thickness s · weight, capped by the room, with the
/// s that lays the volume whole, or all the room where the volume is more than it. Adds what each
/// cell gets to `out` and returns what was laid, m × cells. The s is found by Newton's method
/// from below (the laid volume is concave in s, so each step stays at or below the root and
/// fills at least one more cell, or lands on it).
pub fn fill(
    z: &[f64],
    room: &mut [f64],
    weight: &(dyn Fn(f64) -> f64 + Sync),
    volume: f64,
    out: &mut [f32],
) -> f64 {
    // `volume` is NaN-safe here: anything not above 0 lays nothing.
    if volume.is_nan() || volume <= 0.0 {
        return 0.0;
    }
    // (laid, the weight of the cells not yet full) at s, summed in fixed chunks then in order.
    let at = |s: f64, room: &[f64]| -> (f64, f64) {
        z.par_chunks(CHUNK)
            .zip(room.par_chunks(CHUNK))
            .map(|(zs, rs)| {
                let mut acc = (0.0, 0.0);
                for (&x, &r) in zs.iter().zip(rs) {
                    let w = weight(x);
                    let r = r.max(0.0);
                    if w > 0.0 {
                        let t = s * w;
                        if t < r {
                            acc.0 += t;
                            acc.1 += w;
                        } else {
                            acc.0 += r;
                        }
                    }
                }
                acc
            })
            .collect::<Vec<_>>()
            .iter()
            .fold((0.0, 0.0), |a, b| (a.0 + b.0, a.1 + b.1))
    };
    let (capacity, _) = at(f64::INFINITY, room);
    let s = if capacity <= volume {
        f64::INFINITY
    } else {
        let mut s = 0.0;
        for _ in 0..200 {
            let (laid, open) = at(s, room);
            let short = volume - laid;
            if short <= 1.0e-12 * volume || open <= 0.0 {
                break;
            }
            s += short / open;
        }
        s
    };
    z.par_chunks(CHUNK)
        .zip(room.par_chunks_mut(CHUNK))
        .zip(out.par_chunks_mut(CHUNK))
        .map(|((zs, rs), os)| {
            let mut sum = 0.0;
            for ((&x, r), o) in zs.iter().zip(rs.iter_mut()).zip(os.iter_mut()) {
                let w = weight(x);
                if w > 0.0 && *r > 0.0 {
                    let t = (s * w).min(*r);
                    *r -= t;
                    *o += t as f32;
                    sum += t;
                }
            }
            sum
        })
        .collect::<Vec<_>>()
        .iter()
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structure::columns::CCD_M;

    /// A sea floor of `n` cells from 10 m to 6,500 m deep, the same depths every run.
    fn depths(n: usize) -> Vec<f64> {
        let mut seed = 0x2545_f491_4f6c_dd1du64;
        (0..n)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                10.0 + 6490.0 * ((seed >> 11) as f64 / (1u64 << 53) as f64)
            })
            .collect()
    }

    /// Lays `r` on `z` with the water above each cell as its room; (what was laid, the arrays).
    fn fill_all(r: &Rain, z: &[f64]) -> (Laid, [Vec<f32>; 4]) {
        let mut room: Vec<f64> = z.iter().map(|x| x.max(0.0)).collect();
        let mut a: [Vec<f32>; 4] = std::array::from_fn(|_| vec![0.0; z.len()]);
        let [chalk, carbonate, opal, dust] = &mut a;
        let laid = r.fill(
            z,
            &mut room,
            Outs {
                chalk,
                carbonate,
                opal,
                dust,
            },
        );
        (laid, a)
    }

    /// The dissolved load comes back whole: every mol of its Ca + Mg as a mol of calcite (the cap
    /// carbonate's share included), every mol of its silica as opal, and the dust laid is the
    /// dust supplied, whatever the calcifiers' clock, even with a tenth of the floor (platform
    /// tops) too shallow to hold its share: what they cannot hold settles where there is room, and
    /// no cell takes more than its water.
    #[test]
    fn the_dissolved_load_and_the_dust_come_back_whole() {
        let mut z = depths(4500);
        z.extend((0..500).map(|i| 0.001 + 0.0001 * i as f64));
        let floor = Floor::of(&z, CCD_M);
        let cell_m2 = 9.0e6;
        // 1 km³ of rock dissolved over 2 Myr, a fifth of its Ca + Mg already cap carbonate.
        let (dissolved_m3, kg_per_mol, cap_share) = (1.0e9, 0.26, 0.2);
        let s = Supply::new(dissolved_m3, kg_per_mol, cap_share, 4.0e11, 2.0e6);
        let cations = dissolved_m3 * ROCK_KG_M3 / kg_per_mol;
        let rel = |a: f64, b: f64| (a - b).abs() / b;
        for calcifiers in [0.0, 0.4, 1.0] {
            let r = Rain::spread(&s, calcifiers, &floor, cell_m2, CCD_M);
            // The platform tops would get more than their water with room enough.
            assert!(r.lay(0.05).total() > 0.05);
            let (sum, a) = fill_all(&r, &z);
            let calcite_mol = (sum.chalk + sum.carbonate) * cell_m2 / CALCITE_M3_PER_MOL;
            assert!(
                rel(calcite_mol + cap_share * cations, cations) < 1e-9,
                "calcite {calcite_mol} + cap {} against {cations} mol",
                cap_share * cations
            );
            let silica = sum.opal * cell_m2 / opal_m3_per_mol();
            let expect = dissolved_m3 * ROCK_KG_M3 * SILICA_SHARE / SIO2_KG_PER_MOL;
            assert!(rel(silica, expect) < 1e-9, "{silica} against {expect}");
            assert!(rel(sum.dust * cell_m2, s.dust_m3) < 1e-9);
            assert!(rel(sum.total() * cell_m2, s.volume_m3()) < 1e-9);
            // The arrays hold what was laid, and no cell more than its water.
            let arrays: f64 = a.iter().flatten().map(|&x| f64::from(x)).sum();
            assert!(rel(arrays, sum.total()) < 1e-6);
            for (c, &x) in z.iter().enumerate() {
                let got: f64 = a.iter().map(|v| f64::from(v[c])).sum();
                assert!(
                    got <= x * (1.0 + 1e-6),
                    "cell {c}: {got} m in {x} m of water"
                );
            }
            // The calcifiers' share is the chalk.
            let carb = sum.chalk + sum.carbonate;
            assert!((sum.chalk - calcifiers * carb).abs() <= 1e-9 * carb);
        }
        // The dust is Jickells's 450 Tg/yr from Earth's arid land.
        let earth = Supply::new(0.0, kg_per_mol, 0.0, ARID_EARTH_KM2 * 1.0e6, 1.0);
        assert!((earth.dust_m3 * DUST_KG_M3 - 450.0e9).abs() < 1e-3);
    }

    /// The fill: cells with room take the same thickness, full cells their water, and the volume
    /// is laid whole; a sea too full for it is filled to the brim and the rest is not laid.
    #[test]
    fn the_fill_lays_what_full_cells_cannot_hold_on_the_rest() {
        let z: [f64; 6] = [0.5, 0.2, 10.0, 10.0, 10.0, -3.0];
        let mut room: Vec<f64> = z.iter().map(|x| x.max(0.0)).collect();
        let mut out = vec![0.0f32; z.len()];
        let laid = fill(&z, &mut room, &|_| 1.0, 6.0, &mut out);
        assert!((laid - 6.0).abs() < 1e-9, "{laid}");
        // 6 m over five cells: the shallow two hold 0.7 m, the other three 1.766... m each.
        assert_eq!((out[0], out[1], out[5]), (0.5, 0.2, 0.0));
        let t = (6.0 - 0.7) / 3.0;
        for &x in &out[2..5] {
            assert!((f64::from(x) - t).abs() < 1e-5, "{x} against {t}");
        }
        let mut room: Vec<f64> = z.iter().map(|x| x.max(0.0)).collect();
        let mut out = vec![0.0f32; z.len()];
        let laid = fill(&z, &mut room, &|_| 1.0, 100.0, &mut out);
        assert!((laid - 30.7).abs() < 1e-9 && room.iter().all(|&r| r == 0.0));
    }

    /// Chalk survives above the CCD only, all of it above the lysocline and half of it half way
    /// down; the carbonate laid before the calcifiers keeps to the cap carbonate's floor; the opal
    /// splits at 1 km.
    #[test]
    fn the_ccd_decides_where_the_chalk_survives() {
        let r = Rain {
            volume: [3.0, 1.0, 2.0, 0.6, 0.5],
            floor: [1.0; 5],
            ccd_m: 4500.0,
        };
        let deep = r.lay(5000.0);
        assert_eq!((deep.chalk, deep.carbonate), (0.0, 0.0));
        assert_eq!((deep.opal, deep.dust), (0.6, 0.5));
        assert_eq!(r.lay(3000.0).chalk, 3.0);
        assert!((r.lay(4000.0).chalk - 1.5).abs() < 1e-12);
        assert_eq!(r.lay(4000.0).carbonate, 1.0);
        assert_eq!(r.lay(4200.0).carbonate, 0.0);
        assert_eq!(r.lay(800.0).opal, 2.0);
        assert_eq!(r.lay(-5.0), Laid::default());
        // A part whose floor is missing goes to its other floor: the carbonate to the chalk's
        // floor where none is shallow enough for the shelf's (even before the calcifiers), to the
        // shelf's where none is above the CCD (even with them); the opal to the margin with no
        // deep floor and to the deep floor with no margin. With neither floor, it is not laid.
        let s = Supply {
            dust_m3: 0.0,
            calcite_mol: 1.0e6,
            silica_mol: 1.0e6,
        };
        let one = |z: f64, ccd: f64| {
            let mut f = Floor::default();
            f.add(z, ccd);
            f
        };
        let has = |r: &Rain, p: Part| r.thickness(p) > 0.0;
        let r = Rain::spread(&s, 0.0, &one(4200.0, 4500.0), 1.0, 4500.0);
        assert!(has(&r, Part::Chalk) && !has(&r, Part::Carbonate));
        assert!(has(&r, Part::OpalDeep) && !has(&r, Part::OpalMargin));
        let r = Rain::spread(&s, 1.0, &one(3500.0, 3000.0), 1.0, 3000.0);
        assert!(!has(&r, Part::Chalk) && has(&r, Part::Carbonate));
        let r = Rain::spread(&s, 1.0, &one(500.0, 4500.0), 1.0, 4500.0);
        assert!(has(&r, Part::OpalMargin) && !has(&r, Part::OpalDeep));
        let r = Rain::spread(&s, 1.0, &one(6000.0, 4500.0), 1.0, 4500.0);
        assert!(!has(&r, Part::Chalk) && !has(&r, Part::Carbonate));
        let (laid, _) = fill_all(&r, &[6000.0]);
        assert_eq!(laid.chalk + laid.carbonate, 0.0);
        assert!(laid.opal > 0.0);
    }

    /// The rain on young ocean floor away from the margins against GlobSed's eq. 2a (Straume et
    /// al. 2019: Z = √τ (52 − 2.46 |φ| + 0.045 φ²) m in place, fitted for τ < 82 Myr and |φ| <
    /// 72°, more than 200 km from the margins; average thickness 267 m, average STD 140 m; ledger
    /// ERO-215). A synthetic Earth at its weathering balance: the land weathers the degassing less
    /// the sea floor's share (CLM-158) and the dissolved load takes ERO-206's mass per mol; the
    /// dust is Jickells's; the calcifiers in full. Earth's ocean floor by age is Seton et al.
    /// 2020's (the sediment diagnosis's bins, tools/review/seddiag/sedstock.py), at GDH1's depth
    /// (PLT-352). The rest of the ocean (361.9 × 10⁶ km², Eakins and Sharman 2010) is margin
    /// floor: 32.8 × 10⁶ km² shallower than 500 m (the ocean less Diesing 2020's 329.06 × 10⁶ km²
    /// of seabed deeper than 500 m), at 200 m, and the floor deeper than 500 m that the age grid
    /// leaves out, at either bound of its depth (500 m, where it takes chalk and margin opal, and
    /// 5,000 m, where it takes deep opal and no chalk). A floor `τ` old has gathered the rain at the
    /// depths it passed through; its solid pile is decompacted with Kominz et al. 2011's average
    /// ocean-floor sediment (φ₀ 0.66, λ 1,333 m, as pyBacktrack's lithology table gives it) and
    /// set against eq. 2a's mean over |φ| < 72° (area-weighted): within GlobSed's average STD at
    /// both bounds.
    #[test]
    fn the_rain_on_young_floor_matches_globsed_eq_2a() {
        use crate::climate::ClimateParams;
        use crate::climate::carbon::DEGASSING_EARTH_MOL_YR;
        use crate::plates::pools::gdh1_depth_m;
        // 10⁶ km² of ocean floor per 10-Myr bin, 0 to 180 Myr, then all older floor.
        const SETON_BINS: [f64; 19] = [
            30.3, 29.4, 30.1, 25.8, 21.3, 20.9, 19.2, 18.7, 15.4, 12.9, 15.7, 14.4, 11.4, 8.3, 6.0,
            6.5, 3.4, 1.5, 1.64,
        ];
        const OCEAN_KM2: f64 = 361.9e6;
        const BELOW_500_KM2: f64 = 329.062e6;
        const SHELF_Z_M: f64 = 200.0;
        let cp = ClimateParams::default();
        let land_mol_yr = DEGASSING_EARTH_MOL_YR * (1.0 - cp.seafloor_weathering_share);
        let dissolved_m3_yr = land_mol_yr * cp.dissolved_kg_per_mol / ROCK_KG_M3;
        let s = Supply::new(
            dissolved_m3_yr,
            cp.dissolved_kg_per_mol,
            0.0,
            ARID_EARTH_KM2 * 1.0e6,
            1.0,
        );
        // The floor in 1-Myr slices (each bin spread evenly over its ten), cells of 1 km².
        let slice = |age: f64| -> f64 {
            let b = ((age / 10.0) as usize).min(SETON_BINS.len() - 1);
            SETON_BINS[b] * 1.0e6 / if b + 1 == SETON_BINS.len() { 1.0 } else { 10.0 }
        };
        let add = |floor: &mut Floor, z: f64, km2: f64| {
            let mut one = Floor::default();
            one.add(z, CCD_M);
            *floor = floor.merge(&Floor {
                sea: one.sea * km2,
                deep: one.deep * km2,
                calcite: one.calcite * km2,
                shelf: one.shelf * km2,
            });
        };
        let mut aged = Floor::default();
        let mut crust_km2 = 0.0;
        for k in 0..180 {
            let km2 = slice(k as f64);
            add(&mut aged, gdh1_depth_m(k as f64 + 0.5), km2);
            crust_km2 += km2;
        }
        add(&mut aged, gdh1_depth_m(185.0), slice(185.0));
        crust_km2 += slice(185.0);
        add(&mut aged, SHELF_Z_M, OCEAN_KM2 - BELOW_500_KM2);
        let rest_km2 = BELOW_500_KM2 - crust_km2;
        assert!(rest_km2 > 0.0, "{rest_km2}");
        // Eq. 2a's latitude mean, area-weighted (cos φ), over |φ| < 72°.
        let (mut num, mut den) = (0.0, 0.0);
        for i in 0..720 {
            let phi = (i as f64 + 0.5) * 0.1;
            let w = phi.to_radians().cos();
            num += w * globsed_eq_2a_m(1.0, phi);
            den += w;
        }
        let lat_mean = num / den;
        // In-place height of a solid pile: the H with solid_of_in_place_m(H) = pile.
        let (phi0, lam) = (OCEAN_SEDIMENT_POROSITY, OCEAN_SEDIMENT_DECAY_M);
        let in_place = |pile: f64| -> f64 {
            let mut h = pile.max(1e-9);
            for _ in 0..60 {
                let f = solid_of_in_place_m(h) - pile;
                h = (h - f / (1.0 - phi0 * (-h / lam).exp())).max(1e-12);
            }
            h
        };
        let mut report = Vec::new();
        for rest_z in [500.0, 5000.0] {
            let mut floor = aged;
            add(&mut floor, rest_z, rest_km2);
            let r = Rain::spread(&s, 1.0, &floor, 1.0e6, CCD_M);
            let mut pile = 0.0;
            for k in 0..80 {
                // The slice's million years at its middle depth.
                let t = k as f64 + 0.5;
                let z = gdh1_depth_m(t);
                pile += r.lay(z).total() * 1.0e6;
                let tau = (k + 1) as f64;
                if (k + 1) % 10 == 0 {
                    let model = in_place(pile);
                    let globsed = tau.sqrt() * lat_mean;
                    report.push(format!(
                        "rest at {rest_z} m, {tau} Myr: {model:.0} m against {globsed:.0}"
                    ));
                    assert!(
                        (model - globsed).abs() <= 140.0,
                        "{tau} Myr, the rest at {rest_z} m: the rain's {model:.0} m in place \
                         ({pile:.1} m solid) against eq. 2a's {globsed:.0} m; {report:?}"
                    );
                }
            }
        }
        eprintln!("{report:?}");
    }

    /// The open floor's line (PLT-661): GlobSed's eq. 2a in place, less its pore water. A thin
    /// pile is (1 − φ₀) rock, a thick one loses φ₀ λ of water at most; eq. 2a's average young floor
    /// (267 m in place, Straume et al. 2019) is about 110 m of rock; the line grows with age and is
    /// least near 27° (the quadratic's low point, 2.46 / 0.09).
    #[test]
    fn the_open_floor_line_is_globsed_less_its_pore_water() {
        let thin = solid_of_in_place_m(1.0e-3);
        assert!((thin / 1.0e-3 - (1.0 - OCEAN_SEDIMENT_POROSITY)).abs() < 1e-6, "{thin}");
        let thick = 50_000.0 - solid_of_in_place_m(50_000.0);
        let most = OCEAN_SEDIMENT_POROSITY * OCEAN_SEDIMENT_DECAY_M;
        assert!((thick - most).abs() < 1e-6 * most, "{thick} {most}");
        let young = solid_of_in_place_m(267.0);
        assert!((100.0..120.0).contains(&young), "{young}");
        assert_eq!(globsed_eq_2a_m(100.0, -30.0), globsed_eq_2a_m(100.0, 30.0));
        assert!((globsed_eq_2a_m(100.0, 0.0) - 520.0).abs() < 1e-9);
        assert!(open_floor_solid_m(80.0, 10.0) > open_floor_solid_m(40.0, 10.0));
        let low = open_floor_solid_m(100.0, 27.3);
        for lat in [0.0, 20.0, 35.0, 60.0, 90.0] {
            assert!(open_floor_solid_m(100.0, lat) > low, "{lat}");
        }
    }
}
