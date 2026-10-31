// SPDX-License-Identifier: LGPL-3.0-or-later
//! The plate model's persistent state (plan §8.1), kept in `WorldState::plates` so checkpoints
//! carry it and later stages (surface, deposits) can read the L0 crust.
//!
//! **Frames.** The samples are the vertices of the current *lattice*: a Fibonacci sphere turned by
//! the quaternion `lattice` (a fresh random turn at every global resampling). Each sample belongs to
//! a rigid *group*; a group belongs to a plate and carries the rotation the plate has applied to
//! it since the last resampling. A sample's position now is `groups[g].rot · lattice point`, and its
//! vectors (`ridge`, `fold`) are stored in the same group frame. A plate usually has one group; a
//! rift gives each fragment copies of the parent's groups, and a collision moves a terrane into a
//! new group of the receiving plate, so no sample ever needs re-meshing between resamplings. A
//! triangle of the lattice is part of a plate's surface only while its three vertices share one
//! group (they have moved rigidly together).

use deeptime_format::EventId;
use serde::{Deserialize, Serialize};

use crate::sphere::SplitMix64;

/// Crust types (Cortial Table 1, x_C).
pub const OCEANIC: u8 = 0;
pub const CONTINENTAL: u8 = 1;

/// Orogeny types (Cortial Table 1, o).
pub const OROGENY_NONE: u8 = 0;
/// Subduction (Andean) orogeny.
pub const OROGENY_ANDEAN: u8 = 1;
/// Collision (Himalayan) orogeny.
pub const OROGENY_HIMALAYAN: u8 = 2;

/// Sample flags.
pub const HIDDEN: u8 = 1;
/// On a subducting plate at the trench this step.
pub const TRENCH: u8 = 2;
/// Received arc magmatism this step.
pub const ARC: u8 = 4;
/// Under a hotspot this step.
pub const HOTSPOT: u8 = 8;
/// At a divergent plate boundary this step.
pub const DIVERGENT: u8 = 16;
/// Converted from oceanic to continental crust this step (an emerged arc).
pub const EMERGED: u8 = 32;
/// Within the subduction distance of a front this step.
pub const SUBDUCTION_ZONE: u8 = 64;
/// On the incoming plate of an active collision, beyond the wedge and within the reach of its
/// foreland basin this step (`belts.rs`): where the structure stage looks for foreland folds.
pub const FORELAND: u8 = 128;

/// Tectonic regime codes (the `plates/regime` L1 layer; `EVENTS.md` lists them too).
pub mod regime {
    /// Oceanic plate interior.
    pub const OCEAN_BASIN: u8 = 0;
    /// Mid-ocean ridge (young oceanic crust, divergent oceanic boundary).
    pub const RIDGE: u8 = 1;
    /// Trench: the subducting plate at its front.
    pub const TRENCH: u8 = 2;
    /// Magmatic arc of an active margin.
    pub const ARC: u8 = 3;
    /// Overriding plate behind the arc (back-arc, foreland), within the subduction distance.
    pub const BACK_ARC: u8 = 4;
    /// Collision orogen.
    pub const COLLISION: u8 = 5;
    /// Continental rift (continental crust at a divergent boundary).
    pub const RIFT: u8 = 6;
    /// Hotspot volcanism.
    pub const HOTSPOT: u8 = 7;
    /// Stable continental interior.
    pub const CRATON: u8 = 8;
    /// Continental crust beside oceanic crust of the same plate.
    pub const PASSIVE_MARGIN: u8 = 9;
    /// Human-readable names, indexed by code.
    pub const NAMES: [&str; 10] = [
        "ocean_basin",
        "ridge",
        "trench",
        "arc",
        "back_arc",
        "collision",
        "rift",
        "hotspot",
        "craton",
        "passive_margin",
    ];
}

/// Rock pools per sample, metres of rock (tectonics.js `RockColumn`, CC BY 4.0; see `pools.rs`).
pub mod pool {
    pub const SEDIMENT: usize = 0;
    pub const SEDIMENTARY: usize = 1;
    pub const METAMORPHIC: usize = 2;
    pub const FELSIC_PLUTONIC: usize = 3;
    pub const FELSIC_VOLCANIC: usize = 4;
    pub const MAFIC_VOLCANIC: usize = 5;
    pub const MAFIC_PLUTONIC: usize = 6;
    pub const COUNT: usize = 7;
    /// The conserved (felsic-family) pools: never subducted, accreted instead.
    pub const CONSERVED: usize = 5;
    pub const NAMES: [&str; COUNT] = [
        "sediment",
        "sedimentary",
        "metamorphic",
        "felsic_plutonic",
        "felsic_volcanic",
        "mafic_volcanic",
        "mafic_plutonic",
    ];
}

/// One L0 sample: Cortial's Table 1 plus rock pools and the accumulators the deposit stage reads.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    pub group: u32,
    /// [`OCEANIC`] or [`CONTINENTAL`].
    pub kind: u8,
    /// [`OROGENY_NONE`], [`OROGENY_ANDEAN`] or [`OROGENY_HIMALAYAN`].
    pub orogeny: u8,
    /// A [`regime`] code, as of the last step.
    pub regime: u8,
    /// [`HIDDEN`], [`TRENCH`], … as of the last step.
    pub flags: u8,
    /// z, metres above sea level.
    pub elevation_m: f32,
    /// e, crust thickness in metres (the sum of the pools).
    pub thickness_m: f32,
    /// Oceanic crust: when it formed at a ridge, Myr relative to world creation (age = t − this).
    pub formed_myr: f32,
    /// When the last orogenic uplift happened here, Myr (a_c = t − this).
    pub orogeny_myr: f32,
    /// When this sample's current collision orogeny began ([`Sample::collision_onset_myr`]): in
    /// half-Myr steps after −16,384 Myr, 0 where it has had none; and its last collision stamp
    /// ([`Sample::collision_myr`]): in 4-Myr steps after the onset, rounded (±2 Myr), at most 255
    /// (1,020 Myr; an orogeny stamped for longer has its onset moved up, which the melt window
    /// does not see). Only a collision's stamp sets them ([`Sample::stamp_collision`]): an arc that
    /// later relabels the sample Andean (and renews `orogeny_myr`) leaves them, and the melt
    /// window reads them ([`Sample::collision_melt`]). Packed into the 3 bytes the struct's
    /// alignment left free: the full 16k preset's estimate (three copies of every sample) fits
    /// its 5 GiB cap with about 1 MiB to spare (`surface::young`'s estimate test), so a `Sample`
    /// cannot grow. ledger: PLT-734
    #[serde(default)]
    pub collision_onset: u16,
    #[serde(default)]
    pub collision_span: u8,
    /// r, the local ridge direction (oceanic), a tangent vector in the group frame.
    pub ridge: [f32; 3],
    /// f, the local fold (shortening) direction (continental), a tangent vector in the group frame.
    pub fold: [f32; 3],
    /// Rock pools, metres of rock ([`pool`]).
    pub pools: [f32; pool::COUNT],
    /// Arc magma emplaced here over time, metres: continental arcs …
    pub arc_continental_m: f32,
    /// … and oceanic (island) arcs.
    pub arc_oceanic_m: f32,
    /// Continental crust made from ocean floor (an island arc matured, crust stacked onto the
    /// floor; [`Sample::make_continental`]): the ocean's own crust under it, m. It stays in the
    /// column's mafic pools (it is there, and the column's height counts it: `settle.rs`), but it
    /// is no continental crust: the crust budget leaves it out, and a collision sends it down
    /// with the slab instead of stacking it into the belt (`belts.rs`). 0 on ocean floor and on
    /// the continents' own crust (ledger: PLT-331).
    #[serde(default)]
    pub basement_m: f32,
    /// Flux-weighted magma oxidation proxy in [0, 1] (older, sediment-laden slabs → more oxidised).
    pub oxidation: f32,
    /// At the nearest trench within the subduction distance, when last inside it: convergence
    /// rate (mm/yr, Earth-normalized) …
    pub convergence_mm_yr: f32,
    /// … obliquity (degrees between convergence and the front's normal) …
    pub obliquity_deg: f32,
    /// … and the subducting slab's age (Myr).
    pub slab_age_myr: f32,
    /// Surface heat flow proxy, mW/m².
    pub heat_flow_mw_m2: f32,
    /// Net elevation change over the last step, mm/yr (uplift positive).
    pub uplift_mm_yr: f32,
    /// In the footprint of an active collision this step (its belt behind the front, its wedge,
    /// and on the incoming plate the foreland [`FORELAND`] flags): the convergence across the
    /// front, mm/yr, and the shortening of the collision so far, km. Both 0 elsewhere
    /// (`belts.rs`; the structure stage reads them as `plates/collision_mm_yr` and
    /// `plates/shortening`).
    pub collision_mm_yr: f32,
    pub shortening_km: f32,
    /// Continental crust hidden under the overriding plate of a contact in collision
    /// (`subduction::resolve`): the collision stacks its volume into the belt and the wedge, or
    /// sends it down with the slab (`belts.rs`), so a resampling consumes it instead of accreting
    /// it a second time to the column on top (`ridges::resample`; ledger PLT-276). It stays set
    /// while the crust stays hidden, after the collision ends too, and a terrane transfer or weld
    /// leaves such crust where it is (ledger PLT-291).
    pub stacked: bool,
    /// The part of `uplift_mm_yr` that is the column's own crust, mm/yr: the change of its pools'
    /// Airy height (`collapse::iso_height`) over the last step, crust gained (belts, arcs, the
    /// collapse's and the flow's receivers, accretion) or lost. The rest of the uplift moved a
    /// height alone (the settling, a height rule, the rate limit's hold). The surface's flexure
    /// takes only this part as crust thickened or thinned (ledger: ISO-091).
    #[serde(default)]
    pub crust_uplift_mm_yr: f32,
    /// The depth of the thermal lithosphere's base, km, and the thickness of the depleted keel
    /// under the crust, km (`thermal.rs`; 0 = no state: `tectonics.thermal_state` off, or set at
    /// the next step). ledger: PLT-460
    #[serde(default)]
    pub lithosphere_km: f32,
    #[serde(default)]
    pub keel_km: f32,
    /// With `tectonics.collision_underthrust`, in an active collision's footprint (its belt and
    /// its wedge, `belts.rs`): the plate whose lithosphere is under the column, plus 1 (0: none,
    /// the column's own), and that lithosphere's depth, km (`thermal.rs`'s L: the median of the
    /// incoming plate's foreland). Stamped again each step the collision has a front, like
    /// `collision_mm_yr`. ledger: PLT-546
    #[serde(default)]
    pub carrier: u32,
    #[serde(default)]
    pub carrier_lithosphere_km: f32,
}

/// The packed collision times' origin, Myr (world time runs from minus the history's length to
/// 0), and the step of the last stamp after the onset, Myr ([`Sample::collision_onset`]).
const COLLISION_T0_MYR: f64 = -16_384.0;
const COLLISION_SPAN_MYR: f64 = 4.0;

impl Sample {
    /// A blank oceanic sample.
    pub fn blank(group: u32) -> Sample {
        Sample {
            group,
            kind: OCEANIC,
            orogeny: OROGENY_NONE,
            regime: regime::OCEAN_BASIN,
            flags: 0,
            elevation_m: 0.0,
            thickness_m: 0.0,
            formed_myr: 0.0,
            orogeny_myr: f32::NEG_INFINITY,
            collision_onset: 0,
            collision_span: 0,
            ridge: [0.0; 3],
            fold: [0.0; 3],
            pools: [0.0; pool::COUNT],
            arc_continental_m: 0.0,
            arc_oceanic_m: 0.0,
            basement_m: 0.0,
            oxidation: 0.0,
            convergence_mm_yr: 0.0,
            obliquity_deg: 0.0,
            slab_age_myr: 0.0,
            heat_flow_mw_m2: 0.0,
            uplift_mm_yr: 0.0,
            collision_mm_yr: 0.0,
            shortening_km: 0.0,
            stacked: false,
            crust_uplift_mm_yr: 0.0,
            lithosphere_km: 0.0,
            keel_km: 0.0,
            carrier: 0,
            carrier_lithosphere_km: 0.0,
        }
    }

    pub fn is_hidden(&self) -> bool {
        self.flags & HIDDEN != 0
    }

    pub fn is_continental(&self) -> bool {
        self.kind == CONTINENTAL
    }

    /// A collision stamps this sample at `t_myr` (the belt, the wedge, the interior's share:
    /// `belts.rs`, ledger PLT-247): a Himalayan orogeny, renewed now. It is a new collision
    /// orogeny, whose onset is now, where the sample had none or its last collision stamp is more
    /// than `melt_end_myr` (`collision_melt_end_myr`) ago: crust still hot from a collision that
    /// is thickened again goes on melting. ledger: PLT-734
    pub fn stamp_collision(&mut self, t_myr: f64, melt_end_myr: f64) {
        // With no collision before, the last stamp is −∞ and its age +∞.
        if t_myr - self.collision_myr() > melt_end_myr {
            let half = (2.0 * (t_myr - COLLISION_T0_MYR)).round();
            self.collision_onset = half.clamp(1.0, f64::from(u16::MAX)) as u16;
        }
        // A collision orogeny stamped for more than the span's 1,020 Myr: its onset moves up to
        // keep the last stamp in range. Only `t − onset ≥ collision_melt_after_myr` reads the
        // onset, which holds for either (PR #43's review).
        let longest = COLLISION_SPAN_MYR * f64::from(u8::MAX);
        if t_myr - self.collision_onset_myr() > longest {
            let half = (2.0 * (t_myr - longest - COLLISION_T0_MYR)).round();
            self.collision_onset = half.clamp(1.0, f64::from(u16::MAX)) as u16;
        }
        // The span from the exact onset, so rounding never accumulates over a belt's stamps.
        let span = (t_myr - self.collision_onset_myr()) / COLLISION_SPAN_MYR;
        self.collision_span = span.round().clamp(0.0, f64::from(u8::MAX)) as u8;
        self.orogeny = OROGENY_HIMALAYAN;
        self.orogeny_myr = t_myr as f32;
    }

    /// When the current collision orogeny began, Myr (−∞: none).
    pub fn collision_onset_myr(&self) -> f64 {
        if self.collision_onset == 0 {
            f64::NEG_INFINITY
        } else {
            COLLISION_T0_MYR + 0.5 * f64::from(self.collision_onset)
        }
    }

    /// The last collision stamp, Myr, to ±2 Myr (−∞: none).
    pub fn collision_myr(&self) -> f64 {
        self.collision_onset_myr() + COLLISION_SPAN_MYR * f64::from(self.collision_span)
    }

    /// Whether this sample's crust melts after a collision at `t_myr`: continental, its
    /// collision orogeny begun `after_myr` (`collision_melt_after_myr`) or more ago and last
    /// stamped `end_myr` (`collision_melt_end_myr`) or less ago. Thickened crust heats up for tens
    /// of Myr after the thickening and stays hot for tens more (England & Thompson 1984; Thompson
    /// & England 1984, Fig. 8): the collision granites, the granite stocks and the burial of
    /// regional metamorphism come in this window (the structure stage reads it as
    /// `plates/collision_melt`), not while the belt is thrust (the `collision` regime).
    /// ledger: PLT-735, PLT-736
    pub fn collision_melt(&self, t_myr: f64, after_myr: f64, end_myr: f64) -> bool {
        self.kind == CONTINENTAL
            && t_myr - self.collision_onset_myr() >= after_myr
            && t_myr - self.collision_myr() <= end_myr
    }

    /// Ocean floor becomes continental crust (an island arc matured, `finish_samples`; crust
    /// stacked onto it, `subduction::resolve` and `collapse::spread`): its own ocean crust, the
    /// ocean column's 7 km or the mafic rock it has, stays under the crust on top as its
    /// [`basement_m`](Sample::basement_m). ledger: PLT-331
    pub fn make_continental(&mut self) {
        if self.kind == CONTINENTAL {
            return;
        }
        self.kind = CONTINENTAL;
        let ocean = super::pools::oceanic_column();
        let own = ocean[pool::MAFIC_VOLCANIC] + ocean[pool::MAFIC_PLUTONIC];
        self.basement_m = own.min(self.mafic_m());
    }

    /// Continental crust made from ocean floor is ocean floor again (a sediment margin outside
    /// the transition's zone, `transition::step`): its pools stay as they are, its basement is
    /// the floor's own crust once more. The inverse of [`make_continental`](Sample::make_continental).
    /// ledger: PLT-667
    pub fn make_oceanic(&mut self) {
        if self.kind == OCEANIC {
            return;
        }
        self.kind = OCEANIC;
        self.basement_m = 0.0;
    }

    /// The column's mafic rock, m.
    fn mafic_m(&self) -> f32 {
        self.pools[pool::MAFIC_VOLCANIC].max(0.0) + self.pools[pool::MAFIC_PLUTONIC].max(0.0)
    }

    /// The ocean crust under continental crust made from ocean floor, m: its
    /// [`basement_m`](Sample::basement_m), no more than the column's mafic rock; 0 on ocean floor.
    pub fn basement(&self) -> f32 {
        if self.kind == CONTINENTAL {
            self.basement_m.min(self.mafic_m()).max(0.0)
        } else {
            0.0
        }
    }

    /// The continental crust's own thickness, m: the column less its [`basement`](Sample::basement).
    pub fn own_thickness_m(&self) -> f32 {
        (self.thickness_m - self.basement()).max(0.0)
    }

    /// The column's pools less its [`basement`](Sample::basement), taken from the mafic pools in
    /// the ocean column's proportion (lavas 2 : gabbro 5) as far as each holds.
    pub fn own_pools(&self) -> [f32; pool::COUNT] {
        let mut p = self.pools;
        let b = self.basement();
        if b > 0.0 {
            let (v, g) = (pool::MAFIC_VOLCANIC, pool::MAFIC_PLUTONIC);
            let ocean = super::pools::oceanic_column();
            let gabbro = (b * ocean[g] / (ocean[v] + ocean[g])).min(p[g].max(0.0));
            let lava = (b - gabbro).min(p[v].max(0.0));
            let gabbro = (b - lava).min(p[g].max(0.0));
            p[v] -= lava;
            p[g] -= gabbro;
        }
        p
    }

    /// Oceanic crust age at `t_myr` (0 for continental crust).
    pub fn age_myr(&self, t_myr: f64) -> f64 {
        if self.kind == OCEANIC {
            (t_myr - f64::from(self.formed_myr)).max(0.0)
        } else {
            0.0
        }
    }
}

/// A rigid group of samples (see the module docs).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub plate: u32,
    /// Quaternion (x, y, z, w): lattice frame → current position.
    pub rot: [f64; 4],
}

/// A tectonic plate. Ids are indices into [`PlateState::plates`] and are never reused.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plate {
    pub id: u32,
    pub alive: bool,
    /// Angular velocity Ω = ω·w in the mantle frame, rad/yr (surface velocity s(p) = Ω × p · R).
    pub omega: [f64; 3],
    pub born_myr: f64,
    pub died_myr: Option<f64>,
    /// The plate_birth event (rift-born plates).
    pub birth_event: Option<EventId>,
    pub parent: Option<u32>,
    /// Rifting is boosted until this time by a large igneous province …
    pub lip_until_myr: f64,
    /// … this one.
    pub lip_event: Option<EventId>,
    /// Samples (visible or hidden) as of the last count.
    pub samples: u32,
    /// Unit centroid of the visible samples, as of the last count.
    pub centroid: [f64; 3],
    /// Fraction of the plate's visible samples that are continental, as of the last count.
    pub continental_frac: f64,
}

/// A convergent contact between two plates, with its subduction polarity (Cortial §4.1–4.2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contact {
    /// The subducting plate.
    pub under: u32,
    /// The overriding plate.
    pub over: u32,
    pub onset_myr: f64,
    /// The step it was first seen.
    pub onset_step: u64,
    /// Its subduction_onset event, written once the contact has lasted `CONFIRM_MYR` (short
    /// flickers at transform boundaries never get one).
    pub onset_event: Option<EventId>,
    /// Slab age at the onset, Myr.
    pub onset_slab_age_myr: f64,
    /// Convergence at the onset, mm/yr.
    pub onset_convergence_mm_yr: f64,
    /// 0 ocean–ocean, 1 ocean–continent, 2 continent–continent (forced subduction).
    pub setting: u8,
    /// Step at which overlap was last seen.
    pub last_step: u64,
    /// Samples in the overlap at `last_step`.
    pub samples: u32,
    /// The collision this contact is in, while continental crust of the subducting plate stands
    /// at the front (`shortening.rs`).
    pub collision: Option<Collision>,
    /// Largest uplift rate seen on the overriding plate, mm/yr.
    pub max_uplift_mm_yr: f64,
    /// Whether the overriding plate's continental crust was uplifted (an Andean orogeny).
    pub andean: bool,
    /// Unit centroid of the overlap, as last seen.
    pub centroid: [f64; 3],
    /// Measurement only (`dynamics.rs`; no rule reads it): continental samples of the subducting
    /// plate that went under since the contact began or last collided, each counted once, when
    /// it is first hidden. Counted only while the dynamics record is on.
    pub cont_in: u32,
    /// Measurement only (no rule reads it): this polarity came from reversing a confirmed
    /// subduction zone (`subduction::reverse`), and is not confirmed yet itself.
    #[serde(default)]
    pub reversed: bool,
    /// The contact was active when the lattice was last rebuilt and no overlap has been seen
    /// since (`tectonics.contact_carry`, PLT-550): a resampling gives every plate fresh,
    /// unoverlapped crust, so the overlap that makes a contact active takes a few steps to form
    /// again, but the subduction zone goes on. Set at a resampling ([`super::subduction::carry`]),
    /// cleared when overlap is seen again (`subduction::resolve`) or the contact has no front
    /// (`subduction::fronts_finish`).
    #[serde(default)]
    pub carried: bool,
    /// When the contact's arc began, Myr: the first step one of its oceanic fronts was past
    /// incipient (its slab past PLT-611's 100 km, [`super::forces::incipient`]; with
    /// `incipient_subduction` off, its first oceanic front). `None` until then, and again after a
    /// polarity reversal (a new slab); a split's copies keep it with the slab (`inherit.rs`). The
    /// arc's age, from it, sets the island arcs' early pulse (`tectonics.arc_front_loaded`) and the
    /// young fronts' measure. ledger: PLT-669
    #[serde(default)]
    pub arc_onset_myr: Option<f64>,
    /// Whether the contact's arc began intra-oceanic: at the step its onset was set, none of its
    /// oceanic fronts had visible continental crust of the overriding plate within
    /// `arc_distance_km` (PLT-370's test), so the arc stood on ocean floor. Set with
    /// `arc_onset_myr` and cleared with it; with `tectonics.island_arc_own_crust` the arc keeps
    /// the island law on the crust it made ([`super::subduction::island_arc_crust`]).
    /// ledger: PLT-760
    #[serde(default)]
    pub arc_intra_oceanic: bool,
}

impl Contact {
    /// Whether the contact counts as a working subduction zone at `step`: its overlap was seen at
    /// this step, or it is carried through a resampling ([`Contact::carried`]). The rules that
    /// act on the overlap itself (the hiding, a collision's onset) read `last_step` instead.
    pub fn active(&self, step: u64) -> bool {
        self.last_step == step || self.carried
    }
}

/// A collision: the state of a [`Contact`] from the step continental crust of the subducting
/// plate first stands at the overriding plate's edge and converges on it, until the crust is
/// transferred or welded, the two part, or the contact closes or reverses (`shortening.rs`; design
/// `docs/design/plate-force-balance.md`, task 2). Lengths are Earth-normalized km.
///
/// The belt (`belts.rs`, task 3) stacks each step's crust behind the front and keeps its state in
/// `belt`; the resistance and the weld on stall read `convergence_mm_yr`, `belt` and
/// `stalled_since_myr` (task 4); the slab's break-off is to count from `onset_myr` (task 5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collision {
    /// End of the first step of the collision, Myr, and that step.
    pub onset_myr: f64,
    pub onset_step: u64,
    /// The last step continental crust stood at the front, and its time, Myr.
    pub last_step: u64,
    pub last_myr: f64,
    /// Shortening so far, km: the convergence across the front (the flux of the relative velocity
    /// through the front over the front's length; parting steps add nothing), integrated over the
    /// collision.
    pub shortening_km: f64,
    /// Crust that crossed the front so far, km²: each step's shortening times its front length.
    pub area_km2: f64,
    /// Convergence across the front in the last step it was seen, mm/yr (negative: parting), at
    /// the onset, and the largest.
    pub convergence_mm_yr: f64,
    pub onset_convergence_mm_yr: f64,
    pub peak_convergence_mm_yr: f64,
    /// Convergence across the front `shortening::SLOWDOWN_AFTER_MYR` after the onset (the first
    /// step with a front from then on); `None` while the collision is younger.
    pub later_convergence_mm_yr: Option<f64>,
    /// Length of the front in the last step it was seen, km (the straight distance between the
    /// ends of the stretch of the overriding plate's edge that continental crust of the subducting
    /// plate stands at), and the longest it has been.
    pub front_km: f64,
    pub peak_front_km: f64,
    /// The front's unit centroid, and its mean outward normal there: a unit tangent pointing from
    /// the overriding plate towards the subducting one.
    pub centroid: [f64; 3],
    pub normal: [f64; 3],
    /// Share of the front where the overriding plate's crust is continental too.
    pub continental_over: f64,
    /// Since when the convergence across the front has stayed below `weld_speed_mm_yr`, Myr (the
    /// first such step's end); `None` while it converges faster. Steps with no front leave it.
    pub stalled_since_myr: Option<f64>,
    /// The collision's shortening when its contact's slab was first found broken off, km: the
    /// subduction has failed and from then on the plate pushes the continent (transference,
    /// `collision::transference_due`; set only with `tectonics.transference` or
    /// `tectonics.collision_backarc_reversal`, ledger PLT-516, PLT-737).
    /// Always written: the checkpoint's postcard has no field names, so a skipped field cannot
    /// be read back.
    #[serde(default)]
    pub jam_km: Option<f64>,
    /// The belt this collision has built (`belts.rs`).
    pub belt: Belt,
}

/// A collision's belt: what the stacking of `belts.rs` has built so far, in Earth-normalized km
/// and real metres.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Belt {
    /// Mean height above sea level of the overriding plate within half the belt's width of the
    /// front, m, as of the last step with a front: the height the resistance reads.
    pub height_m: f64,
    /// The mean elevation within half the belt's width of the front before the collision's first
    /// stacking, m, and the belt's relief over it as of the last step with a front: how much the
    /// collision has thickened the crust there (what the resistance reads, so a belt built on
    /// a submarine edge resists too).
    pub base_m: Option<f64>,
    pub relief_m: f64,
    /// The highest `height_m` so far, and the step it was reached.
    pub peak_height_m: f64,
    pub peak_step: u64,
    /// Width of the wedge on the incoming plate, km: how far from the front its profile meets the
    /// ground (last step with a front, and at the peak).
    pub wedge_km: f64,
    pub peak_wedge_km: f64,
    /// The foreland basin in front of the wedge at the peak, read from the surface's flexure
    /// (`surface/flexure_m`): width, km, and depth, m. `None` when there was no flexure field.
    pub foredeep_km: Option<f64>,
    pub foredeep_m: Option<f64>,
    /// Crust stacked so far (belt and wedge): volume, km² × m (Earth-normalized area, real
    /// thickness), and the share the wedge took.
    pub stacked_km2_m: f64,
    pub wedge_km2_m: f64,
    /// The uplift at the front summed over the steps (the largest uplift of each step), m: the
    /// most any ground has risen by the collision.
    pub uplift_m: f64,
    /// The most samples one step raised by more than 10 m and by more than 100 m, and the most
    /// it raised from below the sea datum to above it.
    pub lifted10: u32,
    pub lifted100: u32,
    pub emerged: u32,
    /// Where the belt and wedge rose across the front, weighted by the uplift: Σ dz, Σ dz·x and
    /// Σ dz·x², with x the distance from the front, km (negative behind it on the overriding
    /// plate, positive in front of it on the incoming one). The belt's width is √(12·variance) of
    /// x (exact for a uniform strip, and blind to how the front curves).
    pub weight: f64,
    pub across_sum: f64,
    pub across_sq: f64,
}

/// A mantle-fixed hotspot (Cortial §7.5's suggested extension).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hotspot {
    pub id: u32,
    /// Unit vector, mantle frame (never moves).
    pub pos: [f64; 3],
    pub born_myr: f64,
    pub dies_myr: f64,
    /// Uplift at the centre, mm/yr.
    pub uplift_mm_yr: f64,
    /// The large igneous province its plume head made, if any.
    pub lip_event: Option<EventId>,
    /// Plates it has sat under, in order.
    pub plates: Vec<u32>,
    /// Length of its track on the plates, km (Earth-normalized).
    pub track_km: f64,
    /// Its hotspot_track event was written (it died, or the run ended).
    pub closed: bool,
}

/// A spreading ridge between two plates, recorded at its ridge_birth event.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ridge {
    pub a: u32,
    pub b: u32,
    pub event: EventId,
    /// The resampling that last filled crust between them.
    pub last_resampling: u32,
}

/// Two plates a rift separated (the cause of the ridge that opens between them).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RiftLink {
    pub a: u32,
    pub b: u32,
    pub event: EventId,
    /// Since when the two plates have moved together (relative speed along their whole shared
    /// boundary under `weld_speed_mm_yr`), Myr: a rift that did not open heals once this has
    /// lasted `weld_after_myr` (`rifting::heal`).
    #[serde(default)]
    pub quiet_since_myr: Option<f64>,
}

/// The slab hanging from a subduction zone (`forces.rs`, docs/design/plate-force-balance.md
/// §3.3): known by its two plates, so a contact that flickers off and on keeps its slab and a
/// reversed contact starts a new one. Its length, km, grows by the convergence at the oceanic
/// front each step up to `slab_depth_km`; `blocked_myr` is how long continental crust has held at
/// least half of its front (it breaks off after `slab_breakoff_myr`), `idle_myr` how long no
/// oceanic front has fed it: it hangs and pulls along the plates' shared boundary meanwhile and
/// is gone after `slab_breakoff_myr` (ledger PLT-271). A subduction initiation plants one
/// (`initiation_slab_km`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SlabTrack {
    pub under: u32,
    pub over: u32,
    pub km: f64,
    pub blocked_myr: f64,
    pub idle_myr: f64,
    /// A copy a split handed to a piece of a plate (`inherit.rs`, ledger PLT-534) that has had no
    /// front of its own yet: it pulls only at that piece's fronts and does not hang, so a piece
    /// that borders the other plate off the trench holds none of the slab, and it is gone after
    /// `slab_breakoff_myr` with no front. Its first front makes it a slab like any other.
    pub latent: bool,
}

/// Counters for the run summary.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Counters {
    pub rifts: u32,
    pub plate_births: u32,
    pub plate_deaths: u32,
    pub subduction_onsets: u32,
    pub collisions: u32,
    /// Collision states begun (`shortening.rs`): each ends in a transfer or weld, a parting, or
    /// is open at the end of the run.
    pub collision_onsets: u32,
    /// Terrane transfers of every size (collisions included).
    pub accretions: u32,
    /// Subduction initiations at old oceanic lithosphere.
    pub initiations: u32,
    /// Subduction polarity reversals: every flip of a contact's polarity.
    pub reversals: u32,
    /// Reversals a geologist could map (measurement only, ledger PLT-514): a confirmed
    /// subduction zone whose reversed polarity was confirmed in turn (`subduction::confirm`).
    #[serde(default)]
    pub reversals_confirmed: u32,
    /// Collisions that sutured the subducting plate whole to the receiving one (P1-27).
    pub sutures: u32,
    /// Continents moved alone to the overriding plate by transference (ledger PLT-516).
    #[serde(default)]
    pub transferences: u32,
    /// Rifts that did not open and healed (`rifting::heal`).
    #[serde(default)]
    pub healed: u32,
    /// Island-arc samples that emerged and became continental crust (P1-27).
    pub emerged: u32,
    /// Ocean-floor samples at a plate contact too thick to subduct, made continental crust
    /// (`subduction::resolve`, ledger PLT-289).
    #[serde(default)]
    pub buoyant: u32,
    pub ridges: u32,
    pub hotspots: u32,
    pub lips: u32,
    pub resamplings: u32,
    /// Margin propagations begun (law 2, `propagation.rs`, ledger PLT-656): each cut a
    /// passive margin off its continent and made a new plate.
    #[serde(default)]
    pub propagation_starts: u32,
    /// Margin propagation events: the starts and every later advance along the margin.
    #[serde(default)]
    pub propagations: u32,
    /// Propagations that passed the force test but whose cut or advance failed a geometry check
    /// (one piece each side, the plate sizes, the cut along the margin).
    #[serde(default)]
    pub propagation_stalls: u32,
    /// Subduction initiations whose seed lay within PLT-064's old clearance (1,000 km) of a
    /// trench: possible only with law 2's one-spacing clearance (measurement, PLT-064).
    #[serde(default)]
    pub initiations_near_fronts: u32,
    /// Subduction initiations whose new plate's fracture came within 250, 500, 1,000 and 1,500
    /// km of a trench that was there before it (measurement, PLT-690: Crameri et al. 2020's
    /// distance classes, Table 1's freshness code).
    #[serde(default)]
    pub initiations_reach_250: u32,
    #[serde(default)]
    pub initiations_reach_500: u32,
    #[serde(default)]
    pub initiations_reach_1000: u32,
    #[serde(default)]
    pub initiations_reach_1500: u32,
    /// Subduction initiations drawn but not made: no seed beyond PLT-503's margin, or a fracture
    /// leaving fewer than 32 samples on a side (`rifting::maybe_initiate_subduction`), so the
    /// realized rate reads against PLT-692's hazard.
    #[serde(default)]
    pub initiations_refused: u32,
    /// Back-arc rifts made (`rifting::maybe_rift_backarc`, ledger PLT-710): each split a forearc
    /// and arc sliver off an overriding plate.
    #[serde(default)]
    pub backarc_rifts: u32,
    /// Back-arc rifts drawn but not made: the sliver or the rest of the plate under 32 samples.
    #[serde(default)]
    pub backarc_refused: u32,
    /// Collisions ended by a new subduction zone behind the overriding block: the block moved to
    /// the incoming plate (`tectonics.collision_backarc_reversal`, ledger PLT-737).
    #[serde(default)]
    pub backarc_reversals: u32,
}

/// A back-arc sliver (`tectonics.backarc_rifting`, ledger PLT-710): plate `plate`, split off
/// `parent`'s forearc and arc at `born_myr`. Kept below `min_plate_area_frac` (PLT-091) while it
/// holds a subducting front (PLT-715); the entry goes when it holds none or dies.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BackArc {
    pub plate: u32,
    pub parent: u32,
    pub born_myr: f64,
}

/// A subduction zone propagating along a passive margin (law 2, `tectonics.margin_propagation`,
/// `propagation.rs`, ledger PLT-656): plate `under` (U) subducts at a trench that reached its own
/// continental margin, and the margin's floor was cut off the continent, which with the floor
/// beyond the cut became plate `over` (K'). Each critical convergence across the margin past the
/// cut, under the force test, moves the next stretch of K''s margin floor to U.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Propagation {
    pub under: u32,
    pub over: u32,
    /// The propagating end of the cut, a unit position: where the next step will find it (the
    /// position at the step it was set, carried by K''s rotation for one step).
    pub tip: [f64; 3],
    /// Convergence of U on K' across the margin at the tip since the last advance, km.
    pub converged_km: f64,
    /// Advances since the start.
    pub advances: u32,
    pub born_myr: f64,
    /// The time of the last advance (the start's at first), Myr.
    pub last_myr: f64,
}

/// One-way subduction (`tectonics.one_way_subduction`, ledger PLT-542), per sample of the lattice
/// since the last resampling.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OneWay {
    /// Ocean floor that went down a trench: hidden until the resampling consumes it
    /// (`subduction::resolve`, `ridges::resample`).
    pub held: Vec<bool>,
    /// Visible crust within `arc_distance_km` (PLT-042) of a subduction front as the last step's
    /// fronts left it (`subduction::fronts_and_uplift`): the forearc and the arc, under which the
    /// slab goes down. Empty until a step's fronts have been found.
    pub forearc: Vec<bool>,
}

impl OneWay {
    /// Whether sample `i` is held crust: still hidden (a collision that brought it up let it go)
    /// and still ocean floor (crust a rule turned continental since is judged as continental
    /// crust).
    pub fn holds(&self, i: usize, s: &Sample) -> bool {
        self.held.get(i).copied().unwrap_or(false) && s.is_hidden() && s.kind != CONTINENTAL
    }
}

/// Everything the plate model keeps between steps.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlateState {
    /// L0 samples.
    pub n: u32,
    /// Quaternion (x, y, z, w) of the current lattice: the Fibonacci sphere turned by it.
    pub lattice: [f64; 4],
    pub samples: Vec<Sample>,
    pub groups: Vec<Group>,
    pub plates: Vec<Plate>,
    pub contacts: Vec<Contact>,
    pub hotspots: Vec<Hotspot>,
    pub ridges: Vec<Ridge>,
    pub rift_links: Vec<RiftLink>,
    pub rng: SplitMix64,
    /// Plate steps done.
    pub step: u64,
    /// Simulated time, Myr relative to world creation.
    pub t_myr: f64,
    /// When the history starts, Myr.
    pub start_myr: f64,
    /// Step and time of the last global resampling.
    pub resample_step: u64,
    pub resample_myr: f64,
    /// A_0: the average initial plate area, km² (Earth-normalized), for rifting and collisions.
    pub a0_km2: f64,
    pub counters: Counters,
    /// The slabs of the subduction zones (`forces.rs`), which pull the plates.
    pub slabs: Vec<SlabTrack>,
    /// The dynamics record (`dynamics.rs`): measurement only, absent when switched off.
    pub dynamics: Option<super::dynamics::Dynamics>,
    /// Each sample's distance from its group's outline (`level_set.rs`, ledger PLT-536): set at
    /// every resampling while `tectonics.level_set_membership` is on, absent while it is off.
    #[serde(default)]
    pub level_set: Option<super::level_set::LevelSet>,
    /// The crust shown again (`shown.rs`): measurement only, kept while the dynamics record is.
    #[serde(default)]
    pub shown: Option<super::shown::Track>,
    /// One-way subduction's state (`tectonics.one_way_subduction`, ledger PLT-542); absent while
    /// the switch is off.
    #[serde(default)]
    pub one_way: Option<OneWay>,
    /// Subduction zones propagating along passive margins (law 2, `propagation.rs`, ledger
    /// PLT-656).
    #[serde(default)]
    pub propagations: Vec<Propagation>,
    /// Back-arc slivers holding a subducting front (`rifting::maybe_rift_backarc`, ledger
    /// PLT-710, PLT-715); empty while `tectonics.backarc_rifting` is off.
    #[serde(default)]
    pub backarcs: Vec<BackArc>,
    /// The land's gap budget (`gap_budget.rs`): measurement only, absent unless switched on; not
    /// kept in checkpoints.
    #[serde(skip)]
    pub gap_budget: Option<super::gap_budget::GapBudget>,
    /// The step's thickening limits (`rate.rs`): set when a step begins, cleared when it ends,
    /// so never in a checkpoint.
    #[serde(skip)]
    pub limits: Option<super::rate::Limits>,
}

impl PlateState {
    /// The plate of sample `i`.
    pub fn plate_of(&self, i: usize) -> u32 {
        self.groups[self.samples[i].group as usize].plate
    }

    /// Ids of the plates alive now.
    pub fn alive(&self) -> impl Iterator<Item = &Plate> {
        self.plates.iter().filter(|p| p.alive)
    }

    /// Recounts every plate's samples, centroid and continental fraction (visible samples) from
    /// current positions.
    pub fn recount(&mut self, pos: &[glam::DVec3]) {
        let np = self.plates.len();
        let mut count = vec![0u32; np];
        let mut vis = vec![0u32; np];
        let mut cont = vec![0u32; np];
        let mut sum = vec![glam::DVec3::ZERO; np];
        for (i, s) in self.samples.iter().enumerate() {
            let p = self.groups[s.group as usize].plate as usize;
            count[p] += 1;
            if !s.is_hidden() {
                vis[p] += 1;
                sum[p] += pos[i];
                if s.is_continental() {
                    cont[p] += 1;
                }
            }
        }
        for (k, plate) in self.plates.iter_mut().enumerate() {
            plate.samples = count[k];
            if vis[k] > 0 {
                let c = sum[k].normalize_or_zero();
                if c != glam::DVec3::ZERO {
                    plate.centroid = c.to_array();
                }
                plate.continental_frac = f64::from(cont[k]) / f64::from(vis[k]);
            }
        }
    }
}
