// SPDX-License-Identifier: LGPL-3.0-or-later
//! Carrying the surface with the plates (plan §8.2: deep time on L1, coupled to the plate step).
//!
//! L1 is fixed on the planet while the crust moves over it, so each step the surface's own state
//! — its elevation, the regolith and the strata log — moves with the plate under each cell:
//! semi-Lagrangian advection. A cell on plate p after the step takes what was, before the step,
//! at its position turned back by p's rotation over the step, R(−Ω_p·Δt)·q (the same finite
//! rotation the plate model applied, `plates::kinematics`). Continuous fields interpolate
//! bilinearly there; the strata log and categorical fields take the nearest cell (a column is
//! copied whole, so the log keeps its invariants).
//!
//! The elevation moves with its own crust ([`carried_elevation`]): only source cells that held the
//! same crust count, the plate model's uplift of that crust is added, and where the crust is new
//! at a place the plate model's elevation is taken. (Until 2026-10-01 the state was the offset
//! from the plate model's elevation, advected whatever crust it came from and added to the plate
//! model's elevation under the cell. That elevation is sampled anew every step and can change by
//! kilometres under a cell with no uplift at all, when the cell reads the high side of a margin
//! instead of the low side; the offset built against the low side was added on top, 5 to 8 km in
//! a step.) The bilinear smoothing of repeated advection blurs detail that erosion re-cuts every
//! step.

use glam::{DQuat, DVec3};
use rayon::prelude::*;

use deeptime_format::GridDesc;

use crate::plates::kinematics::rotation;
use crate::plates::state::{TRENCH, regime};
use crate::sphere::BlockFrame;

/// Where every cell's material was before the step: fractional (i, j) in the grid.
pub fn sources(grid: &GridDesc, plate_id: &[f32], omega: &[[f64; 3]], dt_yr: f64) -> Vec<[f64; 2]> {
    let frame = BlockFrame::new(grid.circumference() as u32);
    let proj = frame.projection();
    let w = grid.w as usize;
    let r = f64::from(grid.r);
    let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
    // A cell's unit vector is (cos φ cos λ, cos φ sin λ, sin φ) with φ from its row and λ from its
    // column, so the sines and cosines are taken once per row and column, by the expressions
    // `BlockFrame::unit` uses; and each plate's rotation once (P1-19: both were per cell).
    let lat: Vec<(f64, f64)> = (0..grid.h as i64)
        .map(|j| {
            let z = grid.block_z(j) as f64;
            (proj.sin_lat(z), proj.cos_lat(z))
        })
        .collect();
    let lon: Vec<(f64, f64)> = (0..w as i64)
        .map(|i| proj.lon_deg(grid.block_x(i) as f64).to_radians().sin_cos())
        .collect();
    let rot: Vec<Option<DQuat>> = omega
        .iter()
        .map(|&om| (om != [0.0; 3]).then(|| rotation(DVec3::from_array(om), -dt_yr)))
        .collect();
    (0..grid.cells())
        .into_par_iter()
        .map(|c| {
            let (i, j) = (c % w, c / w);
            let p = plate_id[c];
            let rot = if p.is_finite() && p >= 0.0 {
                rot.get(p as usize).copied().flatten()
            } else {
                None
            };
            let Some(rot) = rot else {
                return [i as f64, j as f64];
            };
            let ((sp, cp), (sl, cl)) = (lat[j], lon[i]);
            let q = DVec3::new(cp * cl, cp * sl, sp);
            let back = rot * q;
            let (x, z) = frame.block(back);
            // x in [−C/2, C/2) maps to i = (x − origin_x) / r, wrapped by the samplers.
            [(x - ox) / r, (z - oz) / r]
        })
        .collect()
}

/// [`sources`] for chosen cells, each moved with the plate given beside it (ISO-133: a
/// collision's footprint under the plate that carries it). A plate without a rotation leaves the
/// cell where it is.
pub fn sources_with(
    grid: &GridDesc,
    cells: &[(usize, f32)],
    omega: &[[f64; 3]],
    dt_yr: f64,
) -> Vec<[f64; 2]> {
    let frame = BlockFrame::new(grid.circumference() as u32);
    let proj = frame.projection();
    let w = grid.w as usize;
    let r = f64::from(grid.r);
    let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
    cells
        .par_iter()
        .map(|&(c, p)| {
            let (i, j) = (c % w, c / w);
            let om = if p.is_finite() && p >= 0.0 {
                omega.get(p as usize).copied().filter(|om| *om != [0.0; 3])
            } else {
                None
            };
            let Some(om) = om else {
                return [i as f64, j as f64];
            };
            let rot = rotation(DVec3::from_array(om), -dt_yr);
            let z = grid.block_z(j as i64) as f64;
            let (sp, cp) = (proj.sin_lat(z), proj.cos_lat(z));
            let (sl, cl) = proj
                .lon_deg(grid.block_x(i as i64) as f64)
                .to_radians()
                .sin_cos();
            let q = DVec3::new(cp * cl, cp * sl, sp);
            let (x, z) = frame.block(rot * q);
            [(x - ox) / r, (z - oz) / r]
        })
        .collect()
}

/// Bilinear sample of `v` at fractional (fi, fj): x wraps, rows clamp.
pub fn bilinear(w: usize, h: usize, v: &[f32], fi: f64, fj: f64) -> f32 {
    let fj = fj.clamp(0.0, (h - 1) as f64);
    let i0 = fi.floor();
    let j0 = fj.floor();
    let (ti, tj) = (fi - i0, fj - j0);
    let wi = |i: f64| (i as i64).rem_euclid(w as i64) as usize;
    let j0u = j0 as usize;
    let j1u = (j0u + 1).min(h - 1);
    let (a, b) = (wi(i0), wi(i0 + 1.0));
    let v00 = f64::from(v[j0u * w + a]);
    let v10 = f64::from(v[j0u * w + b]);
    let v01 = f64::from(v[j1u * w + a]);
    let v11 = f64::from(v[j1u * w + b]);
    ((v00 * (1.0 - ti) + v10 * ti) * (1.0 - tj) + (v01 * (1.0 - ti) + v11 * ti) * tj) as f32
}

/// [`bilinear`] over the corners that hold the same crust only: `same(cell)` says whether a
/// corner cell held, before the step, the crust that is now at the destination. The weights of
/// the corners that did are renormalised; `None` when none did (the crust is new at this place:
/// another plate's edge arrived, and what was here before belongs to the plate it overrode).
///
/// Without this the offset of one plate's surface is carried onto another plate's crust wherever
/// a boundary crosses a cell: a trench's sediment fill (plate elevation −6 km, offset +4 km)
/// lands on the overriding margin, and a range's offset on the next range. The 20 km "mountains"
/// of 2026-10-01 were built that way, a few km a step.
pub fn bilinear_same(
    w: usize,
    h: usize,
    v: &[f32],
    fi: f64,
    fj: f64,
    same: impl Fn(usize) -> bool,
) -> Option<f32> {
    let fj = fj.clamp(0.0, (h - 1) as f64);
    let i0 = fi.floor();
    let j0 = fj.floor();
    let (ti, tj) = (fi - i0, fj - j0);
    let wi = |i: f64| (i as i64).rem_euclid(w as i64) as usize;
    let j0u = j0 as usize;
    let j1u = (j0u + 1).min(h - 1);
    let (a, b) = (wi(i0), wi(i0 + 1.0));
    let corners = [
        (j0u * w + a, (1.0 - ti) * (1.0 - tj)),
        (j0u * w + b, ti * (1.0 - tj)),
        (j1u * w + a, (1.0 - ti) * tj),
        (j1u * w + b, ti * tj),
    ];
    let (mut sum, mut weight) = (0.0f64, 0.0f64);
    for (c, wt) in corners {
        if wt > 0.0 && same(c) {
            sum += wt * f64::from(v[c]);
            weight += wt;
        }
    }
    (weight > 1e-9).then(|| (sum / weight) as f32)
}

/// The blur the carry owes a long step, per cell: the variance, in cells² along i and along j,
/// that `k = dt / ref` carries of a k-th of the step's displacement each would have spread a value
/// over, less what the step's one carry did; `None` when the step is no longer than `ref_yr`.
///
/// A bilinear sample at a fractional offset t is a two-point average of variance t(1 − t), so
/// every carry blurs the carried fields, by about a sixth of a cell² along each axis whatever the
/// plates' speed (the offset's fraction is anything); a step twice as long carries half as often
/// and blurs half as much per Myr. On the 16k planets that blur is a diffusion of 0.25 to 1 m²/yr,
/// beside the hillslope's 2 (`kd_land_m2_yr`), and it is the most the summits lose to anything
/// but the rivers: at 4-Myr plate steps without this the highest ground stood 2 to 3 km higher
/// (P1-19 #27). The top-up keeps the blur per Myr what it was at the reference step (ERO-185).
/// A step of displacement D (in cells, along one axis) owes `k·v(D/k) − v(D)`, v(x) the variance
/// of x's fraction: at k = 2, 2·min(f, 1 − f)² for f the fraction of D/2, at most a half.
pub fn blur_owed(w: usize, src: &[[f64; 2]], dt_yr: f64, ref_yr: f64) -> Option<Vec<[f32; 2]>> {
    if ref_yr.is_nan() || ref_yr <= 0.0 || dt_yr <= ref_yr * (1.0 + 1.0e-9) {
        return None;
    }
    let k = dt_yr / ref_yr;
    let var = |x: f64| {
        let f = x - x.floor();
        f * (1.0 - f)
    };
    let owed = |d: f64| (k * var(d / k) - var(d)).max(0.0) as f32;
    let wf = w as f64;
    Some(
        src.par_iter()
            .enumerate()
            .map(|(c, &[fi, fj])| {
                // The displacement, the shorter way round the wrapped axis.
                let di = fi - (c % w) as f64;
                let di = di - wf * (di / wf).round();
                let dj = fj - (c / w) as f64;
                [owed(di), owed(dj)]
            })
            .collect(),
    )
}

/// Spreads `v` by the variances `var` (cells² along i and along j, [`blur_owed`]): a cell takes
/// var/2 of its difference to each neighbour along that axis, x wrapping and rows ending at the
/// poles, from neighbours that `same(c, n)` says hold the same crust and whose value is finite (the
/// others count as the cell itself, so nothing crosses a plate's edge, as `bilinear_same` keeps it).
/// Variances past a half are spread in as many passes as keep each pass's below it, so no weight is
/// negative.
pub fn blur(
    w: usize,
    h: usize,
    var: &[[f32; 2]],
    v: &mut [f32],
    same: impl Fn(usize, usize) -> bool + Sync,
) {
    let most = var
        .par_iter()
        .map(|s| s[0].max(s[1]))
        .reduce(|| 0.0, f32::max);
    if most <= 0.0 {
        return;
    }
    let passes = (f64::from(most) / 0.5).ceil().max(1.0) as usize;
    let share = 0.5 / passes as f64;
    for _ in 0..passes {
        let old = crate::alloc::copied(v);
        v.par_iter_mut().enumerate().for_each(|(c, out)| {
            let [si, sj] = var[c];
            let x0 = old[c];
            if (si <= 0.0 && sj <= 0.0) || !x0.is_finite() {
                return;
            }
            let (i, j) = (c % w, c / w);
            let mut d = 0.0f64;
            let mut take = |n: usize, s: f32| {
                if s > 0.0 && old[n].is_finite() && same(c, n) {
                    d += share * f64::from(s) * f64::from(old[n] - x0);
                }
            };
            take(j * w + (i + w - 1) % w, si);
            take(j * w + (i + 1) % w, si);
            if j > 0 {
                take((j - 1) * w + i, sj);
            }
            if j + 1 < h {
                take((j + 1) * w + i, sj);
            }
            *out = (f64::from(x0) + d) as f32;
        });
    }
}

/// Whether the plate model pulls the crust at cell `c` to its trench: the sample the cell reads
/// is a subducting plate's at its trench (`plates::state::TRENCH`, as a flag in `plates/flags` or
/// as the regime in `plates/regime`). ledger: ISO-097
pub fn at_trench(flags: Option<&[f32]>, regime: Option<&[f32]>, c: usize) -> bool {
    flags.is_some_and(|f| {
        f.get(c)
            .is_some_and(|&v| (v as u32) & u32::from(TRENCH) != 0)
    }) || regime.is_some_and(|r| r.get(c).is_some_and(|&v| v as u8 == regime::TRENCH))
}

/// The cells whose offset over the plate model's height the surface holds at the end of a step
/// rather than hand to the plate model, 1 or 0 (`surface.held_offsets`): the cells in a gap
/// between plates (`plates/gap`) and on a trench row ([`at_trench`]). The next step's carry
/// keeps their offset over the plate model's height ([`carried_elevation`]), so the plate model
/// must not take it as well: the feedback took it every step, each time again, and the plate
/// model's samples beside a gap rose by it step after step (the oceanic peaks, 10 to 16 km) and
/// those under a trench sank by it. ledger: ISO-125
pub fn held_cells(
    gap: Option<&[f32]>,
    flags: Option<&[f32]>,
    regime: Option<&[f32]>,
    n: usize,
) -> Vec<f32> {
    (0..n)
        .into_par_iter()
        .map(|c| {
            if flagged(gap, c) || at_trench(flags, regime, c) {
                1.0
            } else {
                0.0
            }
        })
        .collect()
}

/// The part of the last step's offset the surface held (`prev_offset` on the cells of
/// [`held_cells`], 0 on the rest), for [`Carry::prev_held_offset`]; `None` without the mask of a
/// step before (the first step, a world from before the switch): the whole offset then.
/// ledger: ISO-125
pub fn held_offset(prev_offset: &[f32], held: Option<&[f32]>) -> Option<Vec<f32>> {
    let held = held.filter(|h| h.len() == prev_offset.len())?;
    Some(
        prev_offset
            .par_iter()
            .zip(held.par_iter())
            .map(|(&o, &k)| if k > 0.5 { o } else { 0.0 })
            .collect(),
    )
}

/// What [`carried_elevation`] reads.
pub struct Carry<'a> {
    pub w: usize,
    pub h: usize,
    /// [`sources`] of the step.
    pub src: &'a [[f64; 2]],
    /// The surface's elevation after the last step, m.
    pub prev_elevation: &'a [f32],
    /// The plate and rigid group under each cell after the last step (`surface/crust_plate`,
    /// `surface/crust_group`); without the plates every source counts as the same crust.
    pub prev_plate: Option<&'a [f32]>,
    pub prev_group: Option<&'a [f32]>,
    /// `plates/plate_id`, `plates/group_id`, `plates/elevation_m`, `plates/uplift_mm_yr` now.
    pub plate: &'a [f32],
    pub group: Option<&'a [f32]>,
    pub plate_elevation: &'a [f32],
    pub uplift_mm_yr: &'a [f32],
    /// `plates/gap` now and after the last step (`surface/crust_gap`): 1 in a gap between plates
    /// (`plates::l1::GAP_FIELD`). `None`: no cell is or was in a gap.
    pub gap: Option<&'a [f32]>,
    pub prev_gap: Option<&'a [f32]>,
    /// The surface's offset over the plate model's elevation after the last step
    /// (`SurfaceState::offset_m`), which a cell that lay in a gap keeps and a trench carries
    /// ([`Carry::at_trench`]).
    pub prev_offset: Option<&'a [f32]>,
    /// The part of `prev_offset` the surface held after the last step, not handed to the plate
    /// model (`surface.held_offsets`: the offset on the cells of `surface/offset_held`, 0 on the
    /// rest, whose offset the plate model took). With it a trench row carries only that; `None`:
    /// the whole offset, as ISO-097 first had it. ledger: ISO-125
    pub prev_held_offset: Option<&'a [f32]>,
    /// `plates/flags` and `plates/regime` now: where the plate model pulls a subducting plate to
    /// its trench ([`Carry::at_trench`]). `None`: no trench.
    pub flags: Option<&'a [f32]>,
    pub regime: Option<&'a [f32]>,
    /// Each plate's parent, and how many plates existed before the step (a plate past that count
    /// was born in it, and its crust was its parent's).
    pub parents: &'a [Option<u32>],
    pub known_plates: usize,
    pub dt_yr: f64,
    /// The step the carry's blur is kept to per Myr (`surface.numerics.carry_blur_ref_myr`, in
    /// years; 0: none added): [`carried_elevation`] tops the blur up ([`blur_owed`]); the other
    /// carries ([`carried_fields`]) do not.
    pub blur_ref_yr: f64,
}

impl Carry<'_> {
    /// Whether the plate model pulls the crust at cell `c` to its trench this step: the sample
    /// the cell reads is a subducting plate's at its trench (`plates::state::TRENCH`, as a flag
    /// or as the regime). ledger: ISO-097
    pub fn at_trench(&self, c: usize) -> bool {
        at_trench(self.flags, self.regime, c)
    }

    /// Whether cells `c` and `n` hold the same crust now: the same plate or the same rigid group.
    pub fn same_now(&self, c: usize, n: usize) -> bool {
        self.plate[c] == self.plate[n] || self.group.is_some_and(|g| g[c] == g[n])
    }

    /// Whether source cell `s` held, before the step, the crust that is at cell `c` now: the same
    /// rigid group, the same plate, or the parent of a plate born this step.
    pub fn same(&self, c: usize, s: usize) -> bool {
        let x = self;
        let Some(pp) = x.prev_plate else {
            return true;
        };
        let now = x.plate[c];
        if pp[s] == now {
            return true;
        }
        if let (Some(pg), Some(g)) = (x.prev_group, x.group)
            && pg[s] == g[c]
        {
            return true;
        }
        if !(now.is_finite() && now >= 0.0) {
            return false;
        }
        let mut k = now as usize;
        for _ in 0..64 {
            if k < x.known_plates {
                break;
            }
            match x.parents.get(k).copied().flatten() {
                Some(pk) => {
                    if pp[s] == pk as f32 {
                        return true;
                    }
                    k = pk as usize;
                }
                None => break,
            }
        }
        false
    }
}

/// Any other field of the surface carried with its crust, by [`carried_elevation`]'s rule: the
/// bilinear sample of `v` over the source corners that held the same crust, `None` where none did
/// (the crust is new at the place).
pub fn carried_field(x: &Carry<'_>, v: &[f32]) -> Vec<Option<f32>> {
    x.src
        .par_iter()
        .enumerate()
        .map(|(c, &[fi, fj])| bilinear_same(x.w, x.h, v, fi, fj, |s| x.same(c, s)))
        .collect()
}

/// Several fields carried at once (the corners and their weights are found once per cell).
pub fn carried_fields<const K: usize>(x: &Carry<'_>, v: [&[f32]; K]) -> Vec<Option<[f32; K]>> {
    let (w, h) = (x.w, x.h);
    x.src
        .par_iter()
        .enumerate()
        .map(|(c, &[fi, fj])| {
            let fj = fj.clamp(0.0, (h - 1) as f64);
            let (i0, j0) = (fi.floor(), fj.floor());
            let (ti, tj) = (fi - i0, fj - j0);
            let wi = |i: f64| (i as i64).rem_euclid(w as i64) as usize;
            let j0u = j0 as usize;
            let j1u = (j0u + 1).min(h - 1);
            let (a, b) = (wi(i0), wi(i0 + 1.0));
            let corners = [
                (j0u * w + a, (1.0 - ti) * (1.0 - tj)),
                (j0u * w + b, ti * (1.0 - tj)),
                (j1u * w + a, (1.0 - ti) * tj),
                (j1u * w + b, ti * tj),
            ];
            let mut sum = [0.0f64; K];
            let mut weight = 0.0f64;
            for (s, wt) in corners {
                if wt > 0.0 && x.same(c, s) {
                    for (acc, f) in sum.iter_mut().zip(&v) {
                        *acc += wt * f64::from(f[s]);
                    }
                    weight += wt;
                }
            }
            (weight > 1e-9).then(|| sum.map(|y| (y / weight) as f32))
        })
        .collect()
}

/// Whether cell `c` is flagged in an optional 0/1 field.
fn flagged(f: Option<&[f32]>, c: usize) -> bool {
    f.is_some_and(|g| g.get(c).is_some_and(|&v| v > 0.5))
}

/// Holds in place the cells that lay in a gap between plates after the last step
/// (`surface/crust_gap`): the floor a ridge makes there belongs to no plate until the next
/// resampling, so it does not move with one, and neither does what lies on it. ledger: PLT-354
pub fn hold_gaps(w: usize, src: &mut [[f64; 2]], prev_gap: Option<&[f32]>) {
    let Some(g) = prev_gap.filter(|g| g.len() == src.len()) else {
        return;
    };
    src.par_iter_mut().enumerate().for_each(|(c, s)| {
        if g[c] > 0.5 {
            *s = [(c % w) as f64, (c / w) as f64];
        }
    });
}

/// Crust new at the place this step, which starts bare: crust younger than the step, and a cell
/// that entered a gap between plates; never a cell that lay in a gap already (its floor and what
/// lies on it stay, [`carried_elevation`]). ledger: PLT-354
pub fn newborn(
    crust_age: &[f32],
    dt_myr: f64,
    gap: Option<&[f32]>,
    prev_gap: Option<&[f32]>,
) -> Vec<bool> {
    crust_age
        .par_iter()
        .enumerate()
        .map(|(c, &age)| {
            !flagged(prev_gap, c) && ((age >= 0.0 && age < dt_myr as f32) || flagged(gap, c))
        })
        .collect()
}

/// The surface's elevation at the start of a step, m, and the number of cells where another
/// plate's crust arrived: the last elevation of the same crust at its source (the corners that
/// held the same rigid group, the same plate, or the parent of a plate born this step), plus the
/// plate model's uplift of that crust over the step; where no corner held it (another plate's
/// edge arrived over a trench, a small plate was absorbed), the plate model's elevation: what the
/// surface had built there belongs to the crust that left. A group keeps its number when a
/// collision hands it to another plate, so a sutured terrane keeps its surface.
///
/// The floor a ridge makes in a gap between plates (`plates/gap`, [`hold_gaps`]) belongs to no
/// plate until the next resampling: a cell that enters a gap takes the plate model's elevation
/// there (new crust, bare), and a cell that lay in a gap after the last step stays where it is
/// and keeps what the surface added over the plate model's floor there (`prev_offset`), while the
/// floor under it deepens as the plate model's gap does, with its age (`plates::l1`), and through
/// the resampling that turns the gap into samples of that age. Carried with the nearest plate and
/// its zero uplift, the gap kept the edge it was pulled from (the crest of the last resampling, a
/// rifted continent's margin) and never sank under GDH1's rate form, which sinks only samples:
/// the young floor stood 200 to 450 m too high, for good (ledger: PLT-354). A step longer than
/// `blur_ref_yr` has the blur it owes ([`blur_owed`]) spread into the carried surface before the
/// uplift, over the same crust only.
///
/// Where the plate model pulls a subducting plate to its trench ([`Carry::at_trench`]) the cell
/// carries its offset over the plate model's elevation instead (the same corners, the same
/// crust): the plate's bending is the plate model's, and the surface keeps only what it added
/// (ledger: ISO-097). The uplift there is a trench sample's pull towards the trench depth; the
/// cell's elevation is read across the triangle of samples around it, whose trench corner is
/// soon subducted, so the two part. Carried as a height, the pull went in each step and the
/// plate model's elevation under the cell rose: a row one cell wide stood 5 to 9 km under the
/// plate model's trench, 15 to 16 km deep, which the plate model's feedback (sample means) never
/// took back (`docs/science/flexure.md`, "The trench rows"). Earth's deepest floor, the
/// Challenger Deep, is about 10.9 km down, within the bending of the plate (Zhang et al. 2014).
pub fn carried_elevation(x: &Carry<'_>) -> (Vec<f32>, u64) {
    let same = |c: usize, s: usize| x.same(c, s);
    let lift = x.dt_yr * 1.0e-3;
    let was_gap = |c: usize| flagged(x.prev_gap, c);
    let in_gap = |c: usize| flagged(x.gap, c);
    // NaN where the crust is new at the place (and where the gap's own rule sets it, below).
    let mut h: Vec<f32> = x
        .src
        .par_iter()
        .enumerate()
        .map(|(c, &[fi, fj])| {
            if was_gap(c) || in_gap(c) {
                return f32::NAN;
            }
            bilinear_same(x.w, x.h, x.prev_elevation, fi, fj, |s| same(c, s)).unwrap_or(f32::NAN)
        })
        .collect();
    if let Some(var) = blur_owed(x.w, x.src, x.dt_yr, x.blur_ref_yr) {
        blur(x.w, x.h, &var, &mut h, |c, n| x.same_now(c, n));
    }
    let foreign = h
        .par_iter()
        .enumerate()
        .filter(|&(c, z)| z.is_nan() && !was_gap(c) && !in_gap(c))
        .count() as u64;
    h.par_iter_mut().enumerate().for_each(|(c, z)| {
        *z = if was_gap(c) {
            let kept = x.prev_offset.map_or(0.0, |o| o[c]);
            x.plate_elevation[c] + kept
        } else if z.is_nan() {
            x.plate_elevation[c]
        } else if let Some(o) = x.prev_offset.filter(|_| x.at_trench(c)) {
            // ledger: ISO-097, ISO-125
            let o = x.prev_held_offset.unwrap_or(o);
            let [fi, fj] = x.src[c];
            match bilinear_same(x.w, x.h, o, fi, fj, |s| same(c, s)) {
                Some(kept) => x.plate_elevation[c] + kept,
                None => x.plate_elevation[c],
            }
        } else {
            (f64::from(*z) + f64::from(x.uplift_mm_yr[c]) * lift) as f32
        };
    });
    (h, foreign)
}

/// The nearest cell to fractional (fi, fj).
pub fn nearest(w: usize, h: usize, fi: f64, fj: f64) -> usize {
    let i = (fi.round() as i64).rem_euclid(w as i64) as usize;
    let j = (fj.round().clamp(0.0, (h - 1) as f64)) as usize;
    j * w + i
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sources as they were before P1-19: the unit vector and the rotation per cell.
    fn sources_reference(
        grid: &GridDesc,
        plate_id: &[f32],
        omega: &[[f64; 3]],
        dt_yr: f64,
    ) -> Vec<[f64; 2]> {
        let frame = BlockFrame::new(grid.circumference() as u32);
        let w = grid.w as usize;
        let r = f64::from(grid.r);
        let (ox, oz) = (grid.origin_block_x as f64, grid.origin_block_z as f64);
        (0..grid.cells())
            .map(|c| {
                let (i, j) = ((c % w) as i64, (c / w) as i64);
                let p = plate_id[c];
                let om = if p.is_finite() && p >= 0.0 {
                    omega.get(p as usize).copied().unwrap_or([0.0; 3])
                } else {
                    [0.0; 3]
                };
                if om == [0.0; 3] {
                    return [i as f64, j as f64];
                }
                let q = frame.unit(grid.block_x(i) as f64, grid.block_z(j) as f64);
                let back = rotation(DVec3::from_array(om), -dt_yr) * q;
                let (x, z) = frame.block(back);
                [(x - ox) / r, (z - oz) / r]
            })
            .collect()
    }

    /// P1-19: the row and column tables and the rotation per plate give the per-cell sources bit
    /// for bit: plates with and without motion, ids past the list, negative and not finite.
    #[test]
    fn the_tabled_sources_equal_the_per_cell_ones() {
        let grid = GridDesc::for_planet("L1", 4096, 32).unwrap();
        let n = grid.cells();
        let mut rng = crate::sphere::SplitMix64::new(7);
        let omega: Vec<[f64; 3]> = (0..6)
            .map(|k| {
                if k == 2 {
                    [0.0; 3]
                } else {
                    std::array::from_fn(|_| (rng.next_f64() - 0.5) * 4.0e-8)
                }
            })
            .collect();
        let ids: Vec<f32> = (0..n)
            .map(|c| match c % 11 {
                0 => f32::NAN,
                1 => -1.0,
                2 => 9.0,
                k => (k % 6) as f32,
            })
            .collect();
        let got = sources(&grid, &ids, &omega, 2.0e6);
        let want = sources_reference(&grid, &ids, &omega, 2.0e6);
        let bits = |v: &[[f64; 2]]| {
            v.iter()
                .map(|p| [p[0].to_bits(), p[1].to_bits()])
                .collect::<Vec<_>>()
        };
        assert_eq!(bits(&got), bits(&want));
    }

    #[test]
    fn a_rotation_about_the_pole_shifts_columns() {
        // Ω along +Z (the pole): a turn by one cell's longitude per step moves material east.
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let w = grid.w as usize;
        let dlon = std::f64::consts::TAU / w as f64;
        let dt = 1.0e6;
        let omega = vec![[0.0, 0.0, dlon / dt]];
        let ids = vec![0.0f32; grid.cells()];
        let src = sources(&grid, &ids, &omega, dt);
        let c = 10 * w + 20;
        let [fi, fj] = src[c];
        assert!((fi - 19.0).abs() < 1e-6, "{fi}");
        assert!((fj - 10.0).abs() < 1e-6, "{fj}");
        let v: Vec<f32> = (0..grid.cells()).map(|k| (k % w) as f32).collect();
        assert!((bilinear(w, grid.h as usize, &v, 19.5, 10.0) - 19.5).abs() < 1e-5);
        assert_eq!(nearest(w, grid.h as usize, -0.4, 3.0), 3 * w);
        // Only the corners of the same crust count; none of them, no value.
        let h = grid.h as usize;
        assert_eq!(bilinear_same(w, h, &v, 19.5, 10.0, |_| true), Some(19.5));
        assert_eq!(
            bilinear_same(w, h, &v, 19.5, 10.0, |c| c % w == 19),
            Some(19.0)
        );
        assert_eq!(
            bilinear_same(w, h, &v, 19.5, 10.0, |c| c % w == 20),
            Some(20.0)
        );
        assert_eq!(bilinear_same(w, h, &v, 19.5, 10.0, |_| false), None);
        // A corner with no weight does not make the crust "the same".
        assert_eq!(bilinear_same(w, h, &v, 19.0, 10.0, |c| c % w == 20), None);
    }

    /// P1-19 #27: one carry of a long step with the blur it owes spreads a spike as far as two
    /// carries of half the step do (the variances add), with its middle in the same place; a step
    /// no longer than the reference owes nothing; the blur stops at another crust.
    #[test]
    fn a_long_step_owes_the_short_steps_blur() {
        let grid = GridDesc::for_planet("L1", 1024, 16).unwrap();
        let (w, h) = (grid.w as usize, grid.h as usize);
        let dlon = std::f64::consts::TAU / w as f64;
        // 2.6 cells along i over the long step.
        let dt = 4.0e6;
        let omega = vec![[0.0, 0.0, 2.6 * dlon / dt]];
        let ids = vec![0.0f32; grid.cells()];
        let row = h / 2;
        let mut v0 = vec![0.0f32; grid.cells()];
        v0[row * w + 40] = 1.0;
        let carry = |v: &[f32], src: &[[f64; 2]]| -> Vec<f32> {
            src.iter()
                .map(|&[fi, fj]| bilinear(w, h, v, fi, fj))
                .collect()
        };
        let half = sources(&grid, &ids, &omega, 0.5 * dt);
        let short = carry(&carry(&v0, &half), &half);
        let full = sources(&grid, &ids, &omega, dt);
        assert!(blur_owed(w, &full, dt, dt).is_none());
        assert!(blur_owed(w, &full, dt, 0.0).is_none());
        let owed = blur_owed(w, &full, dt, 0.5 * dt).unwrap();
        let mut long = carry(&v0, &full);
        let before = long.clone();
        blur(w, h, &owed, &mut long, |_, _| true);
        let moments = |v: &[f32]| {
            let r = &v[row * w..(row + 1) * w];
            let m0: f64 = r.iter().map(|&x| f64::from(x)).sum();
            let m1: f64 = r
                .iter()
                .enumerate()
                .map(|(i, &x)| i as f64 * f64::from(x))
                .sum();
            let mean = m1 / m0;
            let var: f64 = r
                .iter()
                .enumerate()
                .map(|(i, &x)| (i as f64 - mean).powi(2) * f64::from(x))
                .sum::<f64>()
                / m0;
            (m0, mean, var)
        };
        let (s0, s_mean, s_var) = moments(&short);
        let (l0, l_mean, l_var) = moments(&long);
        let (_, _, b_var) = moments(&before);
        assert!(
            (s0 - 1.0).abs() < 1e-5 && (l0 - 1.0).abs() < 1e-5,
            "{s0} {l0}"
        );
        assert!((s_mean - l_mean).abs() < 1e-4, "{s_mean} {l_mean}");
        // 2 · 0.3 · 0.7 for the two short carries; 0.6 · 0.4 for the long one alone.
        assert!((s_var - 0.42).abs() < 1e-4, "{s_var}");
        assert!((b_var - 0.24).abs() < 1e-4, "{b_var}");
        assert!((l_var - s_var).abs() < 1e-4, "{l_var} against {s_var}");
        // Nothing crosses to another crust: a wall west of column 41 keeps the spike's west side.
        let mut walled = before.clone();
        blur(w, h, &owed, &mut walled, |c, n| {
            (c % w > 41) == (n % w > 41)
        });
        let r = row * w;
        let west: f32 = walled[r..r + 42].iter().sum();
        let west0: f32 = before[r..r + 42].iter().sum();
        assert!((west - west0).abs() < 1e-6, "{west} {west0}");
    }

    /// The 2026-10-01 defect in small: plate 0 (low, a trench with 4 km of fill on it) west of
    /// column 8, plate 1 (a 9 km margin) from it east; nothing moves and nothing is uplifted, but
    /// the boundary steps one column west, so column 7 now reads plate 1's crust. The surface
    /// there is plate 1's elevation, not plate 1's elevation plus plate 0's fill; crust that
    /// stayed keeps its surface and takes the uplift; a sutured group keeps its surface under
    /// its new plate; a plate born this step carries its parent's.
    #[test]
    fn the_surface_moves_with_its_own_crust() {
        let (w, h) = (16usize, 4usize);
        let n = w * h;
        let src: Vec<[f64; 2]> = (0..n).map(|c| [(c % w) as f64, (c / w) as f64]).collect();
        let col = |c: usize| c % w;
        let prev_plate: Vec<f32> = (0..n).map(|c| if col(c) < 8 { 0.0 } else { 1.0 }).collect();
        let prev_group = prev_plate.clone();
        // Before: plate 0 at −6 km with 4 km of fill (surface −2 km), plate 1 at 9 km.
        let prev_elevation: Vec<f32> = (0..n)
            .map(|c| if col(c) < 8 { -2000.0 } else { 9000.0 })
            .collect();
        let plate: Vec<f32> = (0..n).map(|c| if col(c) < 7 { 0.0 } else { 1.0 }).collect();
        let plate_elevation: Vec<f32> = (0..n)
            .map(|c| if col(c) < 7 { -6000.0 } else { 9000.0 })
            .collect();
        let mut uplift = vec![0.0f32; n];
        uplift[3] = 0.5; // 0.5 mm/yr over 2 Myr: 1 km
        let carry = |plate: &[f32], group: &[f32], parents: &[Option<u32>]| {
            carried_elevation(&Carry {
                w,
                h,
                src: &src,
                prev_elevation: &prev_elevation,
                prev_plate: Some(&prev_plate),
                prev_group: Some(&prev_group),
                plate,
                group: Some(group),
                plate_elevation: &plate_elevation,
                uplift_mm_yr: &uplift,
                gap: None,
                prev_gap: None,
                prev_offset: None,
                prev_held_offset: None,
                flags: None,
                regime: None,
                parents,
                known_plates: 2,
                dt_yr: 2.0e6,
                blur_ref_yr: 2.0e6,
            })
        };
        let (z, foreign) = carry(&plate, &plate, &[None, None]);
        assert_eq!(foreign, h as u64, "column 7 of every row");
        assert_eq!(
            z[7], 9000.0,
            "the margin's own elevation, not 9 km + 4 km of fill"
        );
        assert_eq!(z[6], -2000.0);
        assert_eq!(z[8], 9000.0);
        assert_eq!(z[3], -1000.0, "the same crust, lifted 1 km");
        // A suture: columns 4..8 keep their group (0) and now belong to plate 1.
        let sutured: Vec<f32> = (0..n).map(|c| if col(c) < 4 { 0.0 } else { 1.0 }).collect();
        let (z, foreign) = carry(&sutured, &prev_group, &[None, None]);
        assert_eq!((foreign, z[5]), (0, -2000.0));
        // A rift: columns 0..4 are plate 2, born this step from plate 0.
        let rifted: Vec<f32> = (0..n)
            .map(|c| match col(c) {
                0..4 => 2.0,
                4..8 => 0.0,
                _ => 1.0,
            })
            .collect();
        let (z, foreign) = carry(&rifted, &rifted, &[None, None, Some(0)]);
        assert_eq!((foreign, z[1], z[3]), (0, -2000.0, -1000.0));
        // An unrelated plate 2 there instead: its crust is new at the place.
        let (z, foreign) = carry(&rifted, &rifted, &[None, None, None]);
        assert_eq!((foreign, z[1]), (4 * h as u64, -6000.0));
    }
    /// The floor of a gap between plates (PLT-354): a cell that lay in a gap stays where it is
    /// and keeps what the surface added over the gap's floor, on the floor as it deepened; a
    /// cell that enters a gap is bare new floor; a cell that lay in one is not new crust again.
    #[test]
    fn a_gap_holds_its_floor_in_place() {
        let (w, h) = (8usize, 2usize);
        let n = w * h;
        let col = |c: usize| c % w;
        // Plate 0 west of column 4, plate 1 east; columns 3 and 4 were a gap after the last
        // step, and column 5 opens into it this step.
        let prev_gap: Vec<f32> = (0..n).map(|c| f32::from(matches!(col(c), 3 | 4))).collect();
        let gap: Vec<f32> = (0..n).map(|c| f32::from(matches!(col(c), 3..=5))).collect();
        // The plates moved a column apart: every source is one column west (east of the
        // gap: east), which a held gap cell does not follow.
        let mut src: Vec<[f64; 2]> = (0..n)
            .map(|c| {
                let i = col(c) as f64;
                [if col(c) < 4 { i + 1.0 } else { i - 1.0 }, (c / w) as f64]
            })
            .collect();
        hold_gaps(w, &mut src, Some(&prev_gap));
        assert_eq!(src[3], [3.0, 0.0]);
        assert_eq!(src[w + 4], [4.0, 1.0]);
        assert_eq!(src[6], [5.0, 0.0], "plate crust still moves");
        let plate: Vec<f32> = (0..n).map(|c| if col(c) < 4 { 0.0 } else { 1.0 }).collect();
        // The surface stood 30 m of sediment above the gap's floor; the floor sank 100 m.
        let prev_elevation: Vec<f32> = (0..n)
            .map(|c| match col(c) {
                3 | 4 => -2970.0,
                _ => -4000.0,
            })
            .collect();
        let prev_offset: Vec<f32> = (0..n)
            .map(|c| if matches!(col(c), 3 | 4) { 30.0 } else { 0.0 })
            .collect();
        let plate_elevation: Vec<f32> = (0..n)
            .map(|c| match col(c) {
                3 | 4 => -3100.0,
                5 => -2600.0,
                _ => -4000.0,
            })
            .collect();
        let uplift = vec![0.0f32; n];
        let (z, foreign) = carried_elevation(&Carry {
            w,
            h,
            src: &src,
            prev_elevation: &prev_elevation,
            prev_plate: Some(&plate),
            prev_group: Some(&plate),
            plate: &plate,
            group: Some(&plate),
            plate_elevation: &plate_elevation,
            uplift_mm_yr: &uplift,
            gap: Some(&gap),
            prev_gap: Some(&prev_gap),
            prev_offset: Some(&prev_offset),
            prev_held_offset: None,
            flags: None,
            regime: None,
            parents: &[None, None],
            known_plates: 2,
            dt_yr: 2.0e6,
            blur_ref_yr: 2.0e6,
        });
        assert_eq!(z[3], -3070.0, "the deepened floor and the 30 m on it");
        assert_eq!(z[w + 4], -3070.0);
        assert_eq!(z[5], -2600.0, "a cell entering the gap is bare new floor");
        assert_eq!(z[0], -4000.0);
        assert_eq!(foreign, 0, "gap floor is not another plate's crust");
        // Ages: the gap's floor is 1 Myr old, the plates' 50 Myr.
        let age: Vec<f32> = (0..n)
            .map(|c| if gap[c] > 0.5 { 1.0 } else { 50.0 })
            .collect();
        let fresh = newborn(&age, 2.0, Some(&gap), Some(&prev_gap));
        assert!(fresh[5] && fresh[w + 5], "entering a gap");
        assert!(
            !fresh[3] && !fresh[4],
            "already in the gap: what lies on it stays"
        );
        assert!(!fresh[0] && !fresh[7]);
    }

    /// At a trench the cell carries its offset over the plate model's elevation, not its height
    /// (ISO-097): the plate model's pull of its trench samples, which the cell's elevation read
    /// across their triangle does not follow, is not added again, and the surface keeps what it
    /// added. Elsewhere the height moves with the uplift.
    #[test]
    fn a_trench_carries_its_offset() {
        let (w, h) = (8usize, 4usize);
        let n = w * h;
        let col = |c: usize| c % w;
        let src: Vec<[f64; 2]> = (0..n)
            .map(|c| [col(c) as f64 + 0.5, (c / w) as f64])
            .collect();
        let plate = vec![0.0f32; n];
        // The floor stood at −6.5 km with 500 m of the surface's own on it; the plate model now
        // pulls its trench samples down 2.4 km in the step while the elevation it reads under
        // the cells rises 300 m.
        let prev_elevation = vec![-6000.0f32; n];
        let prev_offset = vec![500.0f32; n];
        let plate_elevation = vec![-6200.0f32; n];
        let uplift = vec![-1.2f32; n];
        let flags: Vec<f32> = (0..n)
            .map(|c| if col(c) == 3 { f32::from(TRENCH) } else { 0.0 })
            .collect();
        let reg: Vec<f32> = (0..n)
            .map(|c| {
                f32::from(if col(c) == 5 {
                    regime::TRENCH
                } else {
                    regime::OCEAN_BASIN
                })
            })
            .collect();
        let carry = |flags: Option<&[f32]>, reg: Option<&[f32]>, held: Option<&[f32]>| {
            carried_elevation(&Carry {
                w,
                h,
                src: &src,
                prev_elevation: &prev_elevation,
                prev_plate: Some(&plate),
                prev_group: None,
                plate: &plate,
                group: None,
                plate_elevation: &plate_elevation,
                uplift_mm_yr: &uplift,
                gap: None,
                prev_gap: None,
                prev_offset: Some(&prev_offset),
                prev_held_offset: held,
                flags,
                regime: reg,
                parents: &[None],
                known_plates: 1,
                dt_yr: 2.0e6,
                blur_ref_yr: 0.0,
            })
            .0
        };
        let z = carry(Some(&flags), Some(&reg), None);
        assert_eq!(
            z[3], -5700.0,
            "the trench flag: the plate's floor and the 500 m on it"
        );
        assert_eq!(z[w + 5], -5700.0, "the trench regime");
        assert_eq!(
            z[1], -8400.0,
            "away from the trench the height takes the uplift"
        );
        let plain = carry(None, None, None);
        assert!(plain.iter().all(|&v| v == -8400.0), "no trench: {plain:?}");

        // ISO-125: the surface held the offset of column 3 at the last step (a trench row then)
        // and handed the rest to the plate model. Column 3 reads half its source from column 3
        // and half from column 4: half the 500 m. Column 5 reads columns 5 and 6, whose offset
        // the plate model took: the plate's floor alone.
        let mask: Vec<f32> = (0..n)
            .map(|c| if col(c) == 3 { 1.0 } else { 0.0 })
            .collect();
        let kept = held_offset(&prev_offset, Some(&mask)).expect("a mask of the grid's size");
        let z = carry(Some(&flags), Some(&reg), Some(&kept));
        assert_eq!(z[3], -5950.0, "half the held 500 m");
        assert_eq!(z[w + 5], -6200.0, "nothing held: the plate's floor");
        assert_eq!(z[1], -8400.0, "away from the trench as before");
        assert!(held_offset(&prev_offset, Some(&mask[1..])).is_none());
        assert!(held_offset(&prev_offset, None).is_none());
    }

    /// ISO-125: the surface holds the offset on the cells in a gap between plates and on a trench
    /// row (the flag or the regime), and nowhere else.
    #[test]
    fn the_held_cells_are_the_gaps_and_the_trench_rows() {
        let n = 6;
        let gap = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0f32];
        let flags = [0.0, 0.0, f32::from(TRENCH | 16), 16.0, 0.0, 0.0];
        let reg = [0.0, 0.0, 0.0, 0.0, f32::from(regime::TRENCH), 1.0];
        let held = held_cells(Some(&gap), Some(&flags), Some(&reg), n);
        assert_eq!(held, vec![0.0, 1.0, 1.0, 0.0, 1.0, 0.0]);
        assert_eq!(held_cells(None, None, None, n), vec![0.0; n]);
    }
}
