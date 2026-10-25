// SPDX-License-Identifier: LGPL-3.0-or-later
//! The front trace (the front lane, 2026-10-25; measurement only, off by default): two records a
//! run writes into the folder [`ENV`] names, for questions the maps cannot answer. No rule reads
//! either, and the world a run leaves is the same with the trace on or off (the test
//! `the_front_trace_changes_nothing`; on the pool, `pool.sh alias` of the commit that added it).
//!
//! # The dip trace
//! The ocean-continent front's dip after a resampling (`docs/science/plates.md`, "The
//! ocean-continent dip on base10's maps"): late in a lattice's cycle a band of overriding samples
//! behind the real front is classed a front through lattice neighbours of the other plate. The
//! maps hold visible samples only, so they cannot say whether those neighbours are hidden. At the
//! step a resampling ends (the last fronts on the old lattice), the step after it (the first on
//! the new one) and the step halfway through the next cycle, [`dip_rows`] writes one row per
//! visible sample with a lattice neighbour of another plate ([`DIP_FIELDS`]): its plate and class,
//! the front `subduction::fronts_begin` found for it (that neighbour hidden or not, how far it is
//! now, the convergence, the contact and its slab), the neighbour the boundary mix's
//! (`dynamics::boundary_mix`) front reads, and how far the nearest visible sample of another plate
//! stands now. Files `dip_s<step>_p<phase>.bin` (little-endian f32 rows) and `dip_index.txt`.
//!
//! # The thickening attribution
//! Which process last took a continental column past [`THICK_M`] (45 km), and what thickened it
//! since: the step's processes are bracketed as the gap budget brackets them (`gap_budget::Part`),
//! and at each bracket a visible continental sample that crossed 45 km upwards records the part,
//! the time, its thickness before and its orogeny's tag before and after; while it stays thick it
//! records the last part that changed its thickness and the gross gain of each group of parts
//! ([`GROUP_NAMES`]). A resampling hands each new sample the record of its blend's major corner,
//! then books the blend and the crust accreted under it (the resampling's stack). Every
//! `GAP_SNAPSHOT_MYR` it writes the visible continental samples ([`ATT_FIELDS`]) to
//! `att_t<Myr>.bin`.

use std::cell::RefCell;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU8, Ordering};

use glam::{DQuat, DVec3};
use rayon::prelude::*;

use super::Env;
use super::gap_budget::{Part, Trace};
use super::geometry::{Geo, Tree};
use super::kinematics::{quat, tangent};
use super::shortening::{boundary_normal, is_transform};
use super::state::{CONTINENTAL, PlateState, Sample};

/// The folder the trace writes into; unset or empty, the trace is off.
pub const ENV: &str = "DEEPTIME_FRONT_TRACE";

/// The folder [`ENV`] names, read once.
pub fn env_dir() -> Option<PathBuf> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        std::env::var_os(ENV)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    })
    .clone()
}

/// The thickness the attribution follows, m (a measurement threshold, not physics: the lower
/// edge of the crust budget's 45-55 km bin; game-choice, the front lane 2026-10-25; ledger: PLT-580).
pub const THICK_M: f32 = 45_000.0;

/// The attribution's groups of parts.
pub const GROUPS: usize = 9;
/// Their names, in order.
pub const GROUP_NAMES: [&str; GROUPS] = [
    "founder",
    "collapse",
    "flow",
    "stack",
    "collision",
    "arc",
    "rift",
    "surface",
    "other",
];

/// A part's group.
pub fn group_of(part: Part) -> usize {
    match part {
        Part::Founder => 0,
        Part::Collapse => 1,
        Part::Flow => 2,
        Part::ResampleBlend | Part::ResampleAccrete => 3,
        Part::Belts | Part::Shortening | Part::Collision => 4,
        Part::Arcs | Part::Subduction | Part::Forearc => 5,
        Part::Transition | Part::Rifting => 6,
        Part::Erosion
        | Part::Rebound
        | Part::Water
        | Part::Flexure
        | Part::Roots
        | Part::Released
        | Part::FlexRest
        | Part::Spread
        | Part::Later
        | Part::Carry => 7,
        _ => 8,
    }
}

/// No part (a column already thick when the trace began, or new crust).
pub const NONE: u8 = u8::MAX;

/// One sample's attribution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rec {
    /// The part that last took the column from under [`THICK_M`] to it or over (a
    /// `gap_budget::Part` as u8), [`NONE`] if none did.
    pub cross_part: u8,
    pub cross_t: f32,
    /// Its thickness before that part, m.
    pub cross_from_m: f32,
    /// Its orogeny's tag before and after that part.
    pub oro_before: u8,
    pub oro_after: u8,
    /// The last part that changed its thickness by 1 m or more while it was thick, its change, m,
    /// and when.
    pub touch_part: u8,
    pub touch_dm: f32,
    pub touch_t: f32,
    /// The thickest it has been since it crossed, m.
    pub peak_m: f32,
    /// What each group gave it since it crossed (gains only), m.
    pub gain_m: [f32; GROUPS],
}

impl Default for Rec {
    fn default() -> Self {
        Rec {
            cross_part: NONE,
            cross_t: f32::NAN,
            cross_from_m: f32::NAN,
            oro_before: NONE,
            oro_after: NONE,
            touch_part: NONE,
            touch_dm: f32::NAN,
            touch_t: f32::NAN,
            peak_m: f32::NAN,
            gain_m: [0.0; GROUPS],
        }
    }
}

impl Rec {
    /// Books one part's change of a visible continental column, from `t0` m and tag `o0` to `t1`
    /// and `o1`, at `t_myr`.
    pub fn note(&mut self, part: Part, t0: f32, t1: f32, o0: u8, o1: u8, t_myr: f32) {
        let d = t1 - t0;
        let p = part as u8;
        if t0 < THICK_M && t1 >= THICK_M {
            let mut gain_m = [0.0; GROUPS];
            gain_m[group_of(part)] = d;
            *self = Rec {
                cross_part: p,
                cross_t: t_myr,
                cross_from_m: t0,
                oro_before: o0,
                oro_after: o1,
                touch_part: p,
                touch_dm: d,
                touch_t: t_myr,
                peak_m: t1,
                gain_m,
            };
            return;
        }
        if t1 < THICK_M && t0 < THICK_M {
            return;
        }
        if d.abs() >= 1.0 {
            self.touch_part = p;
            self.touch_dm = d;
            self.touch_t = t_myr;
            if d > 0.0 {
                self.gain_m[group_of(part)] += d;
            }
        }
        if self.peak_m.is_nan() || t1 > self.peak_m {
            self.peak_m = t1;
        }
    }
}

/// The attribution over the samples (see the module docs).
#[derive(Clone, Debug)]
pub struct Attribution {
    /// Each sample's thickness and tag at the last booking.
    thick: Vec<f32>,
    oro: Vec<u8>,
    pub rec: Vec<Rec>,
}

impl Attribution {
    pub fn start(samples: &[Sample]) -> Self {
        Attribution {
            thick: samples.iter().map(|s| s.thickness_m).collect(),
            oro: samples.iter().map(|s| s.orogeny).collect(),
            rec: vec![Rec::default(); samples.len()],
        }
    }

    /// Books every change since the last booking to `part`.
    pub fn book(&mut self, samples: &[Sample], part: Part, t_myr: f64) {
        if samples.len() != self.thick.len() {
            // Not a step of the same lattice: start again.
            *self = Attribution::start(samples);
            return;
        }
        let t = t_myr as f32;
        self.thick
            .par_iter_mut()
            .zip(self.oro.par_iter_mut())
            .zip(self.rec.par_iter_mut())
            .zip(samples.par_iter())
            .for_each(|(((th, or), r), s)| {
                let (t0, o0) = (*th, *or);
                *th = s.thickness_m;
                *or = s.orogeny;
                if s.is_hidden() || s.kind != CONTINENTAL || t0.to_bits() == th.to_bits() {
                    return;
                }
                r.note(part, t0, *th, o0, *or, t);
            });
    }

    /// After a resampling: each new sample takes the record of its blend's major corner, then
    /// books the blend (from that corner's thickness to the blend's) and the crust accreted under
    /// it (to the new sample's). `traces` holds one per new sample (`None`: new crust).
    pub fn resampled(&mut self, samples: &[Sample], traces: &[Option<Trace>], t_myr: f64) {
        let t = t_myr as f32;
        let n_old = self.thick.len();
        let (thick, oro, rec) = (&self.thick, &self.oro, &self.rec);
        let rows: Vec<(f32, u8, Rec)> = samples
            .par_iter()
            .enumerate()
            .map(|(i, s)| {
                let tr = traces
                    .get(i)
                    .copied()
                    .flatten()
                    .filter(|tr| tr.tri.iter().all(|&c| (c as usize) < n_old));
                let Some(tr) = tr else {
                    return (s.thickness_m, s.orogeny, Rec::default());
                };
                let k = (0..3)
                    .max_by(|&a, &b| tr.w[a].total_cmp(&tr.w[b]).then(b.cmp(&a)))
                    .unwrap_or(0);
                let m = tr.tri[k] as usize;
                let mut r = rec[m];
                if !s.is_hidden() && s.kind == CONTINENTAL {
                    let blend: f32 = tr.pools.iter().sum();
                    r.note(Part::ResampleBlend, thick[m], blend, oro[m], s.orogeny, t);
                    r.note(
                        Part::ResampleAccrete,
                        blend,
                        s.thickness_m,
                        s.orogeny,
                        s.orogeny,
                        t,
                    );
                }
                (s.thickness_m, s.orogeny, r)
            })
            .collect();
        self.thick = rows.iter().map(|r| r.0).collect();
        self.oro = rows.iter().map(|r| r.1).collect();
        self.rec = rows.into_iter().map(|r| r.2).collect();
    }

    /// The visible continental samples' rows ([`ATT_FIELDS`]), in sample order.
    pub fn rows(
        &self,
        samples: &[Sample],
        pos: &[DVec3],
        plate_of: impl Fn(usize) -> u32,
        t_myr: f64,
    ) -> Vec<[f32; ATT_FIELDS]> {
        let mut out = Vec::new();
        for (i, s) in samples.iter().enumerate() {
            if s.is_hidden() || s.kind != CONTINENTAL || i >= self.rec.len() || i >= pos.len() {
                continue;
            }
            let r = &self.rec[i];
            let (lat, lon) = lat_lon(pos[i]);
            let part = |p: u8| if p == NONE { -1.0 } else { f32::from(p) };
            let mut row = [0.0f32; ATT_FIELDS];
            let head = [
                lat,
                lon,
                s.thickness_m,
                s.elevation_m,
                f32::from(s.orogeny),
                (t_myr - f64::from(s.orogeny_myr)) as f32,
                s.lithosphere_km,
                s.keel_km,
                plate_of(i) as f32,
                part(r.cross_part),
                r.cross_t,
                r.cross_from_m,
                part(r.oro_before),
                part(r.oro_after),
                part(r.touch_part),
                r.touch_dm,
                r.touch_t,
                r.peak_m,
            ];
            row[..head.len()].copy_from_slice(&head);
            row[head.len()..].copy_from_slice(&r.gain_m);
            out.push(row);
        }
        out
    }
}

/// The attribution's row: lat, lon (degrees), thickness, elevation (m), orogeny tag, Myr since
/// the orogeny's stamp, lithosphere and keel (km), plate, cross part (-1 none), cross time (Myr),
/// thickness before the crossing (m), tag before and after it (-1 none), last part that changed
/// it (-1 none), that change (m), its time, the peak since the crossing (m), then each group's
/// gain since the crossing (m, [`GROUP_NAMES`]).
pub const ATT_FIELDS: usize = 18 + GROUPS;

/// Per front-trace sample, the front `subduction::fronts_begin` found: the other-plate neighbour,
/// the contact, the relative velocity (mm/yr), the lattice edge's direction and the boundary's
/// normal.
pub type FbFront = Option<(u32, usize, DVec3, DVec3, DVec3)>;

/// The dip trace's row (see the module docs). Fields, in order: step, phase (0 the step a
/// resampling ends, 1 the next, 2 mid-cycle), steps since the last resampling, sample, lat, lon,
/// plate, group, continental, the mix's class + 1 (0 none), trench (a hidden neighbour of its own
/// group), lattice neighbours of another plate, of them hidden, of them on an active contact this
/// sample overrides, of those visible, the nearest visible sample of another plate now (km, NaN
/// beyond 8 spacings) and its plate; `fronts_begin`'s candidate before the transform test (-1
/// none), hidden, why it was dropped (0 kept, 1 a transform, 2 not converging on the normal); the
/// front kept (-1 none), hidden, continental, its distance now (km), the relative speed (mm/yr),
/// its part on the normal (mm/yr), the contact (-1), under, over, collision, carried, slab (km),
/// the contact's age (Myr); the mix's front neighbour (-1), hidden, its distance now (km); the
/// step's time (Myr); the kept front's edge convergence (mm/yr along the edge).
pub const DIP_FIELDS: usize = 38;

/// The rows of the step's dip trace at the positions `pos`, with `fb` the fronts
/// `fronts_begin` found (one per sample) and `classes` the boundary mix's.
pub fn dip_rows(
    st: &PlateState,
    geo: &Geo,
    pos: &[DVec3],
    env: &Env,
    fb: &[FbFront],
    classes: Option<&[AtomicU8]>,
) -> Vec<[f32; DIP_FIELDS]> {
    let samples = &st.samples;
    let n = samples.len();
    if fb.len() != n || pos.len() != n {
        return Vec::new();
    }
    let norm = env.norm;
    let step = env.step;
    let plate_of: Vec<u32> = (0..n).map(|i| st.plate_of(i)).collect();
    let omega: Vec<DVec3> = st
        .plates
        .iter()
        .map(|p| DVec3::from_array(p.omega))
        .collect();
    let rots: Vec<DQuat> = st.groups.iter().map(|g| quat(g.rot)).collect();
    let pts = geo.points();
    let active: std::collections::BTreeMap<(u32, u32), usize> = st
        .contacts
        .iter()
        .enumerate()
        .filter(|(_, c)| c.active(step))
        .map(|(i, c)| ((c.under, c.over), i))
        .collect();
    let slab_km: std::collections::BTreeMap<(u32, u32), f64> =
        st.slabs.iter().map(|s| ((s.under, s.over), s.km)).collect();
    let visible = Tree::over(pos, |i| !samples[i].is_hidden());
    let spacing = norm.spacing_rad();
    let since = st.step.saturating_sub(st.resample_step) + 1;
    let km_between = |a: usize, b: usize| norm.km(pos[a].angle_between(pos[b])) as f32;
    let idx = |x: Option<u32>| x.map_or(-1.0, |u| u as f32);
    let rows: Vec<Option<[f32; DIP_FIELDS]>> = (0..n)
        .into_par_iter()
        .map(|v| {
            let sv = &samples[v];
            if sv.is_hidden() {
                return None;
            }
            let pv = plate_of[v];
            let x = pos[v];
            let rot = rots[sv.group as usize];
            let (mut n_other, mut n_hidden, mut n_active, mut n_active_vis) = (0, 0, 0, 0);
            let mut trench = false;
            let mut cand: Option<(u32, DVec3)> = None;
            let mut best = 0.0;
            let mut mix_best = f64::NEG_INFINITY;
            let mut mix_u: Option<u32> = None;
            let mut normals: Vec<(u32, DVec3)> = Vec::new();
            for (u, _) in geo.edges(v as u32) {
                let su = &samples[u as usize];
                if su.group == sv.group && su.is_hidden() {
                    trench = true;
                }
                let pu = plate_of[u as usize];
                if pu == pv {
                    continue;
                }
                n_other += 1;
                n_hidden += usize::from(su.is_hidden());
                let on_contact = active.contains_key(&(pu, pv));
                if on_contact {
                    n_active += 1;
                    n_active_vis += usize::from(!su.is_hidden());
                }
                let out = tangent(x, rot * (pts[u as usize] - pts[v])).normalize_or_zero();
                let rel = norm.velocity_mm_yr(omega[pu as usize] - omega[pv as usize], x);
                let along = rel.dot(out);
                // `fronts_begin`'s candidate (before its transform test).
                if along <= 0.0 && on_contact && -along > best {
                    best = -along;
                    cand = Some((u, rel));
                }
                // The boundary mix's front (`dynamics::front_index`).
                let nrm = match normals.iter().find(|(p, _)| *p == pu) {
                    Some(&(_, n)) => n,
                    None => {
                        let n = boundary_normal(geo, pos, &rots, samples, &plate_of, v, pu);
                        normals.push((pu, n));
                        n
                    }
                };
                let across = if is_transform(rel, nrm) {
                    0.0
                } else {
                    rel.dot(nrm)
                };
                if across < 0.0 && on_contact && -along > mix_best {
                    mix_best = -along;
                    mix_u = Some(u);
                }
            }
            if n_other == 0 {
                return None;
            }
            let (near_km, near_plate) = visible
                .within(x, 8.0 * spacing)
                .into_iter()
                .filter(|&(j, _)| plate_of[j as usize] != pv)
                .map(|(j, _)| (km_between(v, j as usize), plate_of[j as usize]))
                .min_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)))
                .map_or((f32::NAN, -1.0), |(d, p)| (d, p as f32));
            let (cand_hidden, cand_reason) = match cand {
                Some((u, rel)) => {
                    let nrm = normals
                        .iter()
                        .find(|(p, _)| *p == plate_of[u as usize])
                        .map_or(DVec3::ZERO, |&(_, n)| n);
                    let reason = if is_transform(rel, nrm) {
                        1.0
                    } else if rel.dot(nrm) >= 0.0 {
                        2.0
                    } else {
                        0.0
                    };
                    (f32::from(u8::from(samples[u as usize].is_hidden())), reason)
                }
                None => (-1.0, -1.0),
            };
            let mut row = [f32::NAN; DIP_FIELDS];
            let (lat, lon) = lat_lon(x);
            let class = classes.map_or(0.0, |c| f32::from(c[v].load(Ordering::Relaxed)));
            let head = [
                step as f32,
                -1.0,
                since as f32,
                v as f32,
                lat,
                lon,
                pv as f32,
                sv.group as f32,
                f32::from(u8::from(sv.kind == CONTINENTAL)),
                class,
                f32::from(u8::from(trench)),
                n_other as f32,
                n_hidden as f32,
                n_active as f32,
                n_active_vis as f32,
                near_km,
                near_plate,
                idx(cand.map(|c| c.0)),
                cand_hidden,
                cand_reason,
            ];
            row[..head.len()].copy_from_slice(&head);
            match fb[v] {
                Some((u, c, rel, out, nrm)) => {
                    let su = &samples[u as usize];
                    let contact = &st.contacts[c];
                    let key = (contact.under, contact.over);
                    row[20..33].copy_from_slice(&[
                        u as f32,
                        f32::from(u8::from(su.is_hidden())),
                        f32::from(u8::from(su.kind == CONTINENTAL)),
                        km_between(v, u as usize),
                        rel.length() as f32,
                        rel.dot(nrm) as f32,
                        c as f32,
                        contact.under as f32,
                        contact.over as f32,
                        f32::from(u8::from(contact.collision.is_some())),
                        f32::from(u8::from(contact.carried)),
                        slab_km.get(&key).copied().unwrap_or(0.0) as f32,
                        (env.t_myr - contact.onset_myr) as f32,
                    ]);
                    row[37] = rel.dot(out) as f32;
                }
                None => row[20] = -1.0,
            }
            match mix_u {
                Some(u) => {
                    row[33] = u as f32;
                    row[34] = f32::from(u8::from(samples[u as usize].is_hidden()));
                    row[35] = km_between(v, u as usize);
                }
                None => row[33] = -1.0,
            }
            row[36] = env.t_myr as f32;
            Some(row)
        })
        .collect();
    rows.into_iter().flatten().collect()
}

/// A unit position's latitude and longitude, degrees.
fn lat_lon(p: DVec3) -> (f32, f32) {
    let p = p.normalize_or_zero();
    (
        p.z.clamp(-1.0, 1.0).asin().to_degrees() as f32,
        p.y.atan2(p.x).to_degrees() as f32,
    )
}

/// Writes `rows` to `dir/name` as little-endian f32 (a failure is reported and ignored).
fn write_rows<const F: usize>(dir: &Path, name: &str, rows: &[[f32; F]]) {
    let mut buf: Vec<u8> = Vec::with_capacity(rows.len() * F * 4);
    for r in rows {
        for x in r {
            buf.extend_from_slice(&x.to_le_bytes());
        }
    }
    let path = dir.join(name);
    if let Err(e) = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(&path, &buf)) {
        eprintln!("front_trace: cannot write {}: {e}", path.display());
    }
}

/// Appends a line to `dir/name` (a failure is reported and ignored).
fn append_line(dir: &Path, name: &str, line: &str) {
    let path = dir.join(name);
    let r = std::fs::create_dir_all(dir).and_then(|()| {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .and_then(|mut f| writeln!(f, "{line}"))
    });
    if let Err(e) = r {
        eprintln!("front_trace: cannot write {}: {e}", path.display());
    }
}

/// The trace a plate stage keeps while it is on.
#[derive(Clone, Debug)]
pub struct FrontTrace {
    pub dir: PathBuf,
    pub att: Option<Attribution>,
    /// The step's dip rows, written or dropped when the step ends.
    pub dip: Vec<[f32; DIP_FIELDS]>,
    /// The last lattice cycle's length, steps (0 before the first resampling).
    pub cycle_steps: u64,
    /// Resamplings seen.
    pub resamplings: u64,
}

impl FrontTrace {
    pub fn new(dir: PathBuf) -> Self {
        FrontTrace {
            dir,
            att: None,
            dip: Vec::new(),
            cycle_steps: 0,
            resamplings: 0,
        }
    }

    /// Books a bracket of the step to the attribution (started at the first).
    pub fn book(&mut self, samples: &[Sample], part: Part, t_myr: f64) {
        match self.att.as_mut() {
            Some(a) => a.book(samples, part, t_myr),
            None => self.att = Some(Attribution::start(samples)),
        }
    }

    /// After a resampling at step `step` whose lattice lasted `cycle` steps: the attribution
    /// through it (the traces [`stash_resample`] kept).
    pub fn resampled(&mut self, samples: &[Sample], t_myr: f64, cycle: u64) {
        let traces = take_resample().unwrap_or_default();
        if let Some(a) = self.att.as_mut() {
            a.resampled(samples, &traces, t_myr);
        }
        self.cycle_steps = cycle;
        self.resamplings += 1;
    }

    /// The step's end: the dip rows are written if the step resampled (phase 0), was the first
    /// on a new lattice (1) or stood halfway through the cycle (2), else dropped; the attribution
    /// is written when `snapshot` holds.
    #[allow(clippy::too_many_arguments)]
    pub fn end_step(
        &mut self,
        samples: &[Sample],
        pos: &[DVec3],
        plate_of: impl Fn(usize) -> u32,
        step: u64,
        t_myr: f64,
        resampled: bool,
        snapshot: bool,
    ) {
        let rows = std::mem::take(&mut self.dip);
        if let Some(since) = rows.first().map(|r| r[2] as u64) {
            let mid = (self.cycle_steps / 2).max(2);
            let phase = if resampled {
                Some(0)
            } else if since == 1 {
                Some(1)
            } else if since == mid {
                Some(2)
            } else {
                None
            };
            if let Some(phase) = phase {
                let mut rows = rows;
                for r in &mut rows {
                    r[1] = phase as f32;
                }
                let name = format!("dip_s{step:05}_p{phase}.bin");
                write_rows(&self.dir, &name, &rows);
                append_line(
                    &self.dir,
                    "dip_index.txt",
                    &format!(
                        "step={step} t={t_myr:.1} phase={phase} since={since} resamplings={} \
                         cycle={} rows={} file={name}",
                        self.resamplings,
                        self.cycle_steps,
                        rows.len()
                    ),
                );
            }
        }
        if snapshot && let Some(a) = self.att.as_ref() {
            let rows = a.rows(samples, pos, plate_of, t_myr);
            let name = format!("att_t{:04.0}.bin", t_myr.round());
            write_rows(&self.dir, &name, &rows);
            append_line(
                &self.dir,
                "att_index.txt",
                &format!("t={t_myr:.1} rows={} file={name}", rows.len()),
            );
        }
    }
}

thread_local! {
    /// Whether the next resampling on this thread keeps its traces for the trace.
    static WANT: RefCell<bool> = const { RefCell::new(false) };
    /// The last resampling's traces, one per new sample (`None`: new crust).
    static RESAMPLED: RefCell<Option<Vec<Option<Trace>>>> = const { RefCell::new(None) };
}

/// Whether the next resampling on this thread keeps its traces (`ridges::resample` asks).
pub fn wants_resample() -> bool {
    WANT.with(|w| *w.borrow())
}

/// Sets [`wants_resample`] for this thread.
pub fn want_resample(on: bool) {
    WANT.with(|w| *w.borrow_mut() = on);
}

/// Keeps a resampling's traces for [`FrontTrace::resampled`].
pub fn stash_resample(traces: Vec<Option<Trace>>) {
    RESAMPLED.with(|r| *r.borrow_mut() = Some(traces));
}

fn take_resample() -> Option<Vec<Option<Trace>>> {
    RESAMPLED.with(|r| r.borrow_mut().take())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plates::state::{OCEANIC, OROGENY_HIMALAYAN, OROGENY_NONE, pool};

    fn column(t_m: f32) -> Sample {
        let mut s = Sample::blank(0);
        s.kind = CONTINENTAL;
        s.pools = [0.0; pool::COUNT];
        s.pools[pool::FELSIC_PLUTONIC] = t_m;
        s.thickness_m = t_m;
        s
    }

    /// A column that crosses 45 km records the part, its thickness before and its tags; a later
    /// part that thins it is its last touch, and gains add to their groups while it is thick.
    #[test]
    fn the_crossing_part_and_the_last_touch() {
        let mut samples = vec![column(40_000.0), column(44_000.0), column(30_000.0)];
        samples[2].kind = OCEANIC;
        let mut a = Attribution::start(&samples);
        samples[1].thickness_m = 50_000.0;
        samples[2].thickness_m = 50_000.0;
        a.book(&samples, Part::Founder, -500.0);
        let r = a.rec[1];
        assert_eq!(r.cross_part, Part::Founder as u8);
        assert_eq!(r.cross_from_m, 44_000.0);
        assert_eq!(r.cross_t, -500.0);
        assert_eq!(r.oro_before, OROGENY_NONE);
        assert_eq!(r.oro_after, OROGENY_NONE);
        assert_eq!(r.gain_m[group_of(Part::Founder)], 6_000.0);
        assert!(untouched(&a.rec[0]), "untouched");
        assert!(untouched(&a.rec[2]), "ocean floor is not followed");
        samples[1].thickness_m = 60_000.0;
        samples[1].orogeny = OROGENY_HIMALAYAN;
        a.book(&samples, Part::Belts, -498.0);
        samples[1].thickness_m = 55_000.0;
        a.book(&samples, Part::Flow, -498.0);
        let r = a.rec[1];
        assert_eq!(r.cross_part, Part::Founder as u8, "the crossing stays");
        assert_eq!(r.touch_part, Part::Flow as u8);
        assert_eq!(r.touch_dm, -5_000.0);
        assert_eq!(r.peak_m, 60_000.0);
        assert_eq!(r.gain_m[group_of(Part::Belts)], 10_000.0);
        assert_eq!(r.gain_m[group_of(Part::Flow)], 0.0, "losses are not gains");
        // Thinned under 45 km and thickened again: the new crossing replaces the record.
        samples[1].thickness_m = 40_000.0;
        a.book(&samples, Part::Collapse, -496.0);
        samples[1].thickness_m = 46_000.0;
        a.book(&samples, Part::Arcs, -494.0);
        let r = a.rec[1];
        assert_eq!(r.cross_part, Part::Arcs as u8);
        assert_eq!(r.oro_before, OROGENY_HIMALAYAN);
        assert_eq!(r.gain_m[group_of(Part::Belts)], 0.0);
    }

    /// A record no part has touched (its NaN times make `==` useless).
    fn untouched(r: &Rec) -> bool {
        r.cross_part == NONE
            && r.touch_part == NONE
            && r.peak_m.is_nan()
            && r.gain_m.iter().all(|&g| g == 0.0)
    }

    /// Through a resampling a new sample takes its major corner's record; crust accreted under
    /// the blend that takes it past 45 km is the stack's crossing.
    #[test]
    fn a_resampling_carries_the_records_and_books_its_stack() {
        let mut old = vec![column(44_000.0), column(52_000.0), column(30_000.0)];
        let mut a = Attribution::start(&old);
        old[1].thickness_m = 52_000.0;
        old[0].thickness_m = 50_000.0;
        a.book(&old, Part::Flow, -10.0);
        let mut new = vec![column(46_000.0), column(48_000.0), column(30_000.0)];
        new[2].thickness_m = 30_000.0;
        let blend = |tri: [u32; 3], w: [f64; 3], t: f32| {
            let mut pools = [0.0; pool::COUNT];
            pools[pool::FELSIC_PLUTONIC] = t;
            Some(Trace {
                tri,
                w,
                pools,
                z: 0.0,
            })
        };
        let traces = vec![
            // Mostly old sample 0 (crossed by the flow), blended to 46 km.
            blend([0, 2, 1], [0.6, 0.3, 0.1], 46_000.0),
            // Mostly old sample 2 (30 km): blended to 40 km, the stack adds 8 km.
            blend([0, 2, 1], [0.1, 0.8, 0.1], 40_000.0),
            None,
        ];
        a.resampled(&new, &traces, -8.0);
        assert_eq!(a.rec[0].cross_part, Part::Flow as u8);
        assert_eq!(a.rec[0].touch_part, Part::ResampleBlend as u8);
        assert_eq!(a.rec[1].cross_part, Part::ResampleAccrete as u8);
        assert_eq!(a.rec[1].cross_from_m, 40_000.0);
        assert_eq!(a.rec[1].gain_m[group_of(Part::ResampleAccrete)], 8_000.0);
        assert!(untouched(&a.rec[2]));
        // The next step books against the new lattice.
        new[1].thickness_m = 50_000.0;
        a.book(&new, Part::Founder, -6.0);
        assert_eq!(a.rec[1].touch_part, Part::Founder as u8);
        assert_eq!(a.rec[1].cross_part, Part::ResampleAccrete as u8);
    }
}
