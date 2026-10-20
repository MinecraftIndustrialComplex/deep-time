// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime run`, `deeptime stage` and `deeptime bench` (plan §3.3, §9.3): the CLI drives the same
//! `Session` the JNI wrapper drives, so `run` writes exactly the folder the game would.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::compute::ComputeMode;
use deeptime_core::session::{AdvanceStatus, Session, SessionOptions, SessionState, group_of};
use deeptime_format::keyframes::{KeyframeIndex, read_keyframe};
use deeptime_format::manifest::rfc3339_utc;
use serde_json::{Value, json};

use crate::alloc::{self, AllocGauge};

pub type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Budget of one `advance` call; the CLI only needs it to print progress between calls.
const ADVANCE_BUDGET: Duration = Duration::from_millis(250);

/// Options shared by the subcommands that run a session.
#[derive(Clone)]
pub struct Common {
    pub threads: usize,
    pub mem_cap: u64,
    pub verbose: bool,
    /// `--set path=value` overrides of the preset (`apply_overrides`).
    pub set: Vec<String>,
    /// `--history-mem`: the recorded history's frames held in memory at most, bytes
    /// (`Session::set_history_budget`).
    pub history_mem: Option<u64>,
    /// `--climate-json`: write the climate stage's note (its summary, the run's CO₂/temperature/
    /// ice/sea-level curve and the ice-age window's) here after a `stage` run.
    pub climate_json: Option<std::path::PathBuf>,
    /// `--compute`: where the kernels with a GPU version run (None: `DEEPTIME_COMPUTE`, else
    /// auto; `deeptime_core::compute`).
    pub compute: Option<ComputeMode>,
}

impl Common {
    pub fn apply_compute(&self, s: &Session) {
        if let Some(m) = self.compute {
            s.set_compute(m);
        }
    }
}

/// The recorded history's line: its size, and what went to the spill file (P1-17).
pub(crate) fn spill_line(s: &Session) -> CliResult<Option<String>> {
    let spill = s.spill();
    let (read, read_s, write_s) = spill.io_stats();
    Ok(s.with_world(|w| {
        w.history.as_ref().map(|h| {
            format!(
                "history frames={} history_mib={:.1} in_memory_mib={:.1} frames_on_disk={} spilled_mib={:.1} spilled_frames={} spill_read_mib={:.1} spill_read_s={:.2} spill_write_s={:.2} reserved_peak_mib={:.1} reserved_base_peak_mib={:.1}",
                h.frame_count(),
                h.bytes() as f64 / 1048576.0,
                h.resident_bytes() as f64 / 1048576.0,
                h.frames_on_disk(),
                spill.written_bytes() as f64 / 1048576.0,
                spill.written_frames(),
                read as f64 / 1048576.0,
                read_s,
                write_s,
                s.needed_peak() as f64 / 1048576.0,
                s.needed_base_peak() as f64 / 1048576.0,
            )
        })
    })?)
}

/// Applies `--set a.b.c=value` overrides to preset JSON (the value is JSON, else a string; missing
/// objects on the way are made). The result goes through the preset parser like any preset, so
/// an unknown key or an out-of-range value is refused naming its path.
pub(crate) fn apply_overrides(json: &str, set: &[String]) -> CliResult<String> {
    if set.is_empty() {
        return Ok(json.to_owned());
    }
    let mut v: Value = serde_json::from_str(json)?;
    for item in set {
        let (path, value) = item
            .split_once('=')
            .ok_or_else(|| format!("--set {item}: expected PATH=VALUE"))?;
        let value: Value = serde_json::from_str(value).unwrap_or_else(|_| json!(value));
        let mut at = &mut v;
        let keys: Vec<&str> = path.split('.').collect();
        for (n, key) in keys.iter().enumerate() {
            let obj = at
                .as_object_mut()
                .ok_or_else(|| format!("--set {item}: {} is not an object", keys[..n].join(".")))?;
            if n + 1 == keys.len() {
                obj.insert((*key).to_owned(), value);
                break;
            }
            at = obj.entry((*key).to_owned()).or_insert_with(|| json!({}));
        }
    }
    Ok(v.to_string())
}

pub(crate) fn read_text(path: &Path, what: &str) -> CliResult<String> {
    std::fs::read_to_string(path).map_err(|e| format!("{what} {}: {e}", path.display()).into())
}

pub(crate) fn session_options(out: &Path, c: &Common) -> SessionOptions {
    SessionOptions {
        out_dir: out.to_path_buf(),
        threads: c.threads,
        mem_cap: c.mem_cap,
    }
}

/// Prints the session's log lines to stderr (DEBUG only when verbose).
pub(crate) fn forward_log(s: &Session, verbose: bool) {
    for line in s.log().drain() {
        if verbose || !line.starts_with("DEBUG") {
            eprintln!("  {}", line.replacen('\t', " ", 1));
        }
    }
}

/// Advances until `stop` says so after a status; returns the last status.
fn drive(
    s: &Session,
    verbose: bool,
    mut stop: impl FnMut(&Session, AdvanceStatus) -> bool,
) -> CliResult<AdvanceStatus> {
    loop {
        let status = s.advance(ADVANCE_BUDGET);
        forward_log(s, verbose);
        let status = status?;
        if status == AdvanceStatus::Cancelled {
            return Err("the session was cancelled".into());
        }
        if stop(s, status) {
            return Ok(status);
        }
    }
}

/// Prints the deposit stage's census: sites per type next to the plan §8.6 expectation.
pub fn print_census(census: &Value) {
    println!(
        "deposits: province={} ({} provinces) count_mult={} max_formed_per_survivor={} brine_flow_km={} — target(§8.6) = count_mult · ρ/1e5 · Σ_tracts A_km2 survivors (ρ the mean of a measured density, else its p50); formed = target / preservation",
        census["province"].as_str().unwrap_or("?"),
        census["provinces"],
        census["count_mult"],
        census["max_formed_per_survivor"],
        census["brine_flow_km"],
    );
    if let Some(h) = census["history"].as_object() {
        println!(
            "deposits: history {} · {} frames · {} names",
            h["source"].as_str().unwrap_or("?"),
            h["frames"],
            h["names"].as_array().map_or(0, Vec::len),
        );
    }
    for s in census["skipped"].as_array().into_iter().flatten() {
        println!(
            "deposits SKIPPED {}: the history lacks {}",
            s["type"].as_str().unwrap_or("?"),
            s["missing"]
        );
    }
    for note in census["fallbacks"].as_array().into_iter().flatten() {
        println!("deposits FALLBACK: {}", note.as_str().unwrap_or("?"));
    }
    for t in census["types"].as_array().into_iter().flatten() {
        let f = |k: &str| t[k].as_f64().unwrap_or(0.0);
        println!(
            "deposits {:<30} sites={:>3} counted={:>3} density={:.1} target={:>7.2} expected={:>7.2} expected_sites={:>7.2} at_positions={:>7.2} preservation={:.3} formed_expected={:>7.2} drawn={:>3} capped={} tracts={:>3} permissive_km2={:>9.0} mean_F={:.3} buried={} truncated={} eroded={} too_deep={} children={} guaranteed={} dropped={} on_land={} seams={} seams_max={} measures_m={:.0} no_cover={} cover_capped={}",
            t["id"].as_str().unwrap_or("?"),
            f("sites"),
            f("counted"),
            f("density"),
            f("target"),
            f("expected"),
            f("expected_sites"),
            f("expected_at_positions"),
            f("preservation"),
            f("expected_formed"),
            f("drawn"),
            t["capped"].as_bool().unwrap_or(false),
            f("tracts"),
            f("permissive_km2"),
            f("mean_f"),
            f("buried"),
            f("truncated"),
            f("eroded"),
            f("too_deep"),
            f("children"),
            f("guaranteed"),
            f("dropped_separation"),
            f("on_land"),
            f("seams"),
            f("seams_max"),
            f("measures_m"),
            f("no_cover"),
            f("cover_capped"),
        );
        if let Some(b) = t["best_cell"].as_object() {
            let g = |k: &str| b[k].as_f64().unwrap_or(0.0);
            println!(
                "  best cell F={:.2} at lat {:.1} lon {:.1}, limited by {} (scores {})",
                g("favourability"),
                g("lat"),
                g("lon"),
                b["limiting"].as_str().unwrap_or("?"),
                b["scores"]
                    .as_object()
                    .map(|m| m
                        .iter()
                        .map(|(k, v)| format!("{k} {:.2}", v.as_f64().unwrap_or(0.0)))
                        .collect::<Vec<_>>()
                        .join(", "))
                    .unwrap_or_default()
            );
        }
    }
}

fn census_of(notes: &std::collections::BTreeMap<String, String>) -> Option<Value> {
    serde_json::from_str(notes.get(deeptime_core::deposits::STAGE_NAME)?).ok()
}

/// A preset argument: a built-in name, a file or a share code (`deeptime_core::preset::resolve`).
fn load_preset(arg: &str) -> CliResult<String> {
    Ok(deeptime_core::preset::resolve(arg)?)
}

/// A preset argument with the timeline choices applied (`--start`, `--length-myr`).
pub fn load_preset_with(
    arg: &str,
    start: Option<&str>,
    length_myr: Option<f64>,
) -> CliResult<String> {
    Ok(deeptime_core::preset::with_timeline(
        &load_preset(arg)?,
        start,
        length_myr,
    )?)
}

/// The plate model's summary line (plan P1-03: `plates_final=<n> ocean_age_p90_myr=<a> …`), when
/// the world has plates.
pub(crate) fn plates_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    use deeptime_core::plates::collapse;
    let st = world.plates.as_ref()?;
    // The orogenic collapse over the run, and the continental crust's thickness now (visible
    // samples), km.
    let global = |k: &str| world.globals.get(k).copied().unwrap_or(0.0);
    let mut thick: Vec<f64> = st
        .samples
        .iter()
        .filter(|s| s.is_continental() && !s.is_hidden())
        .map(|s| f64::from(s.thickness_m) / 1000.0)
        .collect();
    thick.sort_by(f64::total_cmp);
    let q = |f: f64| {
        thick
            .get(((thick.len().max(1) - 1) as f64 * f).round() as usize)
            .copied()
            .unwrap_or(0.0)
    };
    Some(
        format!(
            "{} collapse_km3={:.3e} collapse_gained_km2={:.3e} foundered_km3={:.3e} rate_capped_km3={:.3e} rate_lost_km3={:.3e} rate_held_km3={:.3e} collision_crossed_km3={:.3e} collision_kept_km3={:.3e} resample_accreted_km3={:.3e} resample_stacked_km3={:.3e} resample_accreted_continent_km3={:.3e} resample_accreted_visible_km3={:.3e} resample_accreted_rise_km3={:.3e} arc_continental_km3={:.3e} arc_oceanic_km3={:.3e} arc_front_continental_km_myr={:.3e} arc_front_oceanic_km_myr={:.3e} cordillera_shortened_km3={:.3e} hotspot_magma_km3={:.3e} cone_felsic_km3={:.3e} cone_mafic_km3={:.3e} continental_volume_km3={:.3e} continental_thickness_km_p10/50/90/99={:.1}/{:.1}/{:.1}/{:.1}",
            deeptime_core::plates::stats::summary(st).line(),
            global(collapse::MOVED_GLOBAL),
            global(collapse::GAINED_GLOBAL),
            global(collapse::FOUNDERED_GLOBAL),
            global(deeptime_core::plates::rate::CAPPED_GLOBAL),
            global(deeptime_core::plates::rate::LOST_GLOBAL),
            global(deeptime_core::plates::rate::HELD_GLOBAL),
            global(deeptime_core::plates::belts::CROSSED_GLOBAL),
            global(deeptime_core::plates::belts::KEPT_GLOBAL),
            global(deeptime_core::plates::ridges::ACCRETED_GLOBAL),
            global(deeptime_core::plates::ridges::STACKED_CONSUMED_GLOBAL),
            global(deeptime_core::plates::ridges::ACCRETED_HIDDEN_CONTINENT_GLOBAL),
            global(deeptime_core::plates::ridges::ACCRETED_VISIBLE_GLOBAL),
            global(deeptime_core::plates::ridges::ACCRETED_RISE_GLOBAL),
            global(deeptime_core::plates::subduction::ARC_CONTINENTAL_GLOBAL),
            global(deeptime_core::plates::subduction::ARC_OCEANIC_GLOBAL),
            global(deeptime_core::plates::subduction::ARC_FRONT_CONTINENTAL_GLOBAL),
            global(deeptime_core::plates::subduction::ARC_FRONT_OCEANIC_GLOBAL),
            global(deeptime_core::plates::subduction::CORDILLERA_GLOBAL),
            global(deeptime_core::plates::hotspots::MAGMA_GLOBAL),
            global(deeptime_core::plates::CONE_FELSIC_GLOBAL),
            global(deeptime_core::plates::CONE_MAFIC_GLOBAL),
            global(deeptime_core::plates::subduction::CONTINENTAL_VOLUME_GLOBAL),
            q(0.1),
            q(0.5),
            q(0.9),
            q(0.99),
        ) + &crust_bins(st)
            + &st
                .gap_budget
                .as_ref()
                .map(|gb| gb.lines())
                .unwrap_or_default()
            + world
                .notes
                .get(deeptime_core::plates::CRUST_BUDGET_NOTE)
                .map_or("", String::as_str),
    )
}

/// Diagnostic (belts-area): visible continental crust by thickness bin: share of the samples,
/// share of the crust volume, median elevation, median time since the last orogeny stamp, median
/// sediment share.
fn crust_bins(st: &deeptime_core::plates::state::PlateState) -> String {
    use deeptime_core::plates::state::pool;
    const EDGES: [f64; 7] = [0.0, 30.0, 40.0, 55.0, 70.0, 100.0, 1.0e9];
    let vis: Vec<_> = st
        .samples
        .iter()
        .filter(|s| s.is_continental() && !s.is_hidden())
        .collect();
    let vol: f64 = vis.iter().map(|s| f64::from(s.thickness_m)).sum();
    let med = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(f64::NAN)
    };
    let mut out = String::from("\ncrust_bins");
    for w in EDGES.windows(2) {
        let b: Vec<_> = vis
            .iter()
            .filter(|s| {
                let t = f64::from(s.thickness_m) / 1000.0;
                t >= w[0] && t < w[1]
            })
            .collect();
        let bv: f64 = b.iter().map(|s| f64::from(s.thickness_m)).sum();
        out += &format!(
            " [{}-{}km n={:.3} v={:.3} z={:.0} quiet={:.0} sed={:.2}]",
            w[0],
            w[1].min(999.0),
            b.len() as f64 / vis.len().max(1) as f64,
            bv / vol.max(1.0),
            med(b.iter().map(|s| f64::from(s.elevation_m)).collect()),
            med(b
                .iter()
                .map(|s| st.t_myr - f64::from(s.orogeny_myr))
                .collect()),
            med(b
                .iter()
                .map(
                    |s| f64::from(s.pools[pool::SEDIMENT] + s.pools[pool::SEDIMENTARY])
                        / f64::from(s.thickness_m).max(1.0)
                )
                .collect()),
        );
    }
    out
}

/// The continental margins over the run (P1-27, `notes.plates.margins`): the mean shares by
/// setting over the record after the first 100 Myr (the start's own margins are not the model's),
/// the last record, and the series of active / passive shares.
pub(crate) fn margins_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note: Value = serde_json::from_str(world.notes.get("plates")?).ok()?;
    let list = note["margins"].as_array()?;
    let rec: Vec<deeptime_core::plates::stats::Margins> = list
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();
    let (first, last) = (rec.first()?, rec.last()?);
    let after: Vec<_> = rec
        .iter()
        .filter(|m| m.t_myr - first.t_myr >= 90.0)
        .collect();
    let pool = if after.is_empty() {
        rec.iter().collect()
    } else {
        after
    };
    let mean = |f: &dyn Fn(&deeptime_core::plates::stats::Margins) -> f64| {
        pool.iter().map(|m| f(m)).sum::<f64>() / pool.len() as f64
    };
    let andean =
        |m: &deeptime_core::plates::stats::Margins| f64::from(m.andean) / f64::from(m.coast.max(1));
    let initiated = |m: &deeptime_core::plates::stats::Margins| {
        f64::from(m.initiated) / f64::from(m.coast.max(1))
    };
    let ratio = |a: u32, b: u32| f64::from(a) / f64::from(b.max(1));
    let at_boundary = mean(&|m| ratio(m.at_boundary, m.coast));
    let front_km = mean(&|m| {
        if m.front_km_p50.is_finite() {
            m.front_km_p50
        } else {
            0.0
        }
    });
    let front_cont = mean(&|m| ratio(m.front_over_continental, m.front_over));
    let front_len = mean(&|m| f64::from(m.front_over) / f64::from(m.coast.max(1)));
    let fragment_crust = mean(&|m| m.fragment_crust_share);
    let fragment_coast = mean(&|m| ratio(m.fragment_coast, m.coast + m.fragment_coast));
    let floor: Vec<f64> = pool
        .iter()
        .map(|m| m.passive_floor_age_p50_myr)
        .filter(|a| a.is_finite())
        .collect();
    let series: Vec<String> = rec
        .iter()
        .map(|m| {
            format!(
                "{:.0}:{:.2}/{:.2}",
                m.t_myr,
                m.active_share(),
                m.passive_share()
            )
        })
        .collect();
    Some(format!(
        "margins records={} active_mean={:.3} subduction_mean={:.3} andean_mean={:.3} \
         initiated_mean={:.3} boundary_mean={:.3} passive_mean={:.3} regime_active_mean={:.3} \
         passive_floor_age_p50_mean={:.1} at_boundary_mean={:.3} front_km_p50_mean={:.0} \
         front_continental_mean={:.3} front_per_coast_mean={:.3} fragment_crust_mean={:.3} \
         fragment_coast_mean={:.3} final_landmasses={} final_active={:.3} \
         final_passive={:.3} final_coast_km={:.0} reversals={} series(t:active/passive)={}",
        rec.len(),
        mean(&|m| m.active_share()),
        mean(&|m| m.subduction_share()),
        mean(&andean),
        mean(&initiated),
        mean(&|m| m.boundary_share()),
        mean(&|m| m.passive_share()),
        mean(&|m| m.regime_active_share()),
        floor.iter().sum::<f64>() / floor.len().max(1) as f64,
        at_boundary,
        front_km,
        front_cont,
        front_len,
        fragment_crust,
        fragment_coast,
        last.landmasses,
        last.active_share(),
        last.passive_share(),
        last.coast_km,
        last.reversals,
        series.join(",")
    ))
}

/// The surface's summary line (P1-04): `surface_final=<steps> land_frac=… sediment_km3=… …`,
/// when the world has a surface.
pub(crate) fn surface_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let st = world.surface.as_ref()?;
    let t = &st.totals;
    let n = st.log.cells().max(1) as f64;
    let land = st
        .elevation_m
        .iter()
        .filter(|&&z| f64::from(z) > st.sea_level_m)
        .count() as f64;
    let peak = st.elevation_m.iter().copied().fold(f32::MIN, f32::max);
    let dx = f64::from(st.grid.r) * 300.0;
    let sediment: f64 = (0..st.log.cells()).map(|c| st.log.total(c)).sum::<f64>() * dx * dx / 1e9;
    let layers: f64 = st.log.count.iter().map(|&c| f64::from(c)).sum::<f64>() / n;
    // Hypsometry of the land (P1-05: the climate reads it), m above the sea.
    let mut lz: Vec<f64> = st
        .elevation_m
        .iter()
        .map(|&z| f64::from(z) - st.sea_level_m)
        .filter(|&z| z > 0.0)
        .collect();
    lz.sort_by(f64::total_cmp);
    let q = |f: f64| {
        lz.get(((lz.len().max(1) - 1) as f64 * f).round() as usize)
            .copied()
            .unwrap_or(0.0)
    };
    let (z50, z90, z99) = (q(0.5), q(0.9), q(0.99));
    let hist = world.history.as_ref();
    let global = |k: &str| world.globals.get(k).copied().unwrap_or(0.0);
    let cap_share = global(deeptime_core::plates::CAP_BOUND_GLOBAL)
        / global(deeptime_core::plates::CONTINENTAL_GLOBAL).max(1.0);
    Some(format!(
        "surface_final={} land_frac={:.3} max_elevation_m={:.0} sea_level_m={:.1} sediment_km3={:.3e} \
         eroded_km3={:.3e} fluvial_km3={:.3e} lake_km3={:.3e} marine_km3={:.3e} carbonate_km3={:.3e} \
         evaporite_km3={:.3e} to_sea_km3={:.3e} mass_residual_km3={:.3e} mean_log_layers={:.2} \
         spl_iterations_per_step={:.2} cg_iterations_per_step={:.1} sinks={} erosion_pulses={} basins={} sea_events={} frames={} \
         history_mib={:.1} land_z_p50_m={:.0} land_z_p90_m={:.0} land_z_p99_m={:.0} plate_cap_bound_share={:.5} \
         foreign_crust_cell_steps={:.0} spread_cell_steps={:.0} spread_km3={:.3e} \
         flexure_solves={:.0} flexure_iterations_per_solve={:.2} flexure_unconverged={:.0} \
         guard_worst_m={:.0} guard_steps_over_1/2/3/6_reliefs={:.0}/{:.0}/{:.0}/{:.0} \
         step_rise/fall_polar_m={:.0}/{:.0} step_rise/fall_rest_m={:.0}/{:.0}",
        t.steps,
        land / n,
        peak,
        st.sea_level_m,
        sediment,
        t.eroded_km3,
        t.fluvial_km3,
        t.lake_km3,
        t.marine_km3,
        t.carbonate_km3,
        t.evaporite_km3,
        t.to_sea_km3,
        t.mass_residual_km3,
        layers,
        t.spl_iterations as f64 / (t.steps.max(1)) as f64,
        t.diffusion_iterations as f64 / (t.steps.max(1)) as f64,
        t.sinks,
        st.events.counts[0],
        st.events.counts[1],
        st.events.counts[2],
        hist.map_or(0, |h| h.frame_count()),
        hist.map_or(0.0, |h| h.bytes() as f64 / 1048576.0),
        z50,
        z90,
        z99,
        cap_share,
        global(deeptime_core::surface::FOREIGN_CRUST_GLOBAL),
        global(deeptime_core::surface::spread::CELLS_GLOBAL),
        global(deeptime_core::surface::spread::MOVED_GLOBAL),
        global(deeptime_core::surface::flexure::SOLVES_GLOBAL),
        global(deeptime_core::surface::flexure::ITERATIONS_GLOBAL)
            / global(deeptime_core::surface::flexure::SOLVES_GLOBAL).max(1.0),
        global(deeptime_core::surface::flexure::UNCONVERGED_GLOBAL),
        global(deeptime_core::surface::guard::WORST_GLOBAL),
        global(deeptime_core::surface::guard::NEAR_GLOBALS[0]),
        global(deeptime_core::surface::guard::NEAR_GLOBALS[1]),
        global(deeptime_core::surface::guard::NEAR_GLOBALS[2]),
        global(deeptime_core::surface::guard::NEAR_GLOBALS[3]),
        global(deeptime_core::surface::guard::STEP_GLOBALS[0]),
        global(deeptime_core::surface::guard::STEP_GLOBALS[1]),
        global(deeptime_core::surface::guard::STEP_GLOBALS[2]),
        global(deeptime_core::surface::guard::STEP_GLOBALS[3]),
    ))
}

/// The planet's hypsometry at the end of the surface (P1-17 calibration), true-area weighted
/// (cell areas; the surface line's `land_frac` counts cells, which is the in-game share): land share,
/// land elevation mean and quantiles, the share of land above 1, 2 and 3 km, the median sea
/// depth, the shelf share of the sea (0–200 m), the continental crust's share of the planet and
/// the share of it above the sea (the plate model's crust type), and the share of the planet per
/// 1-km band from −7 to +6 km (Earth's bimodal curve). Earth on GMT's 1° relief, the same way:
/// land 0.289, median 427 m, p90 2,022 m, above 2 km 0.102, sea median 4,062 m (STATUS.md, P1-17).
pub(crate) fn hypsometry_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    use deeptime_core::deposits::fields::cell_area_km2;
    let st = world.surface.as_ref()?;
    let g = &st.grid;
    let (w, h) = (g.w as usize, g.h as usize);
    // m_per_block_h only scales every weight alike; 1 is enough for shares.
    let mut land: Vec<(f64, f64)> = Vec::new();
    let mut sea: Vec<(f64, f64)> = Vec::new();
    let mut bands = [0.0f64; 13];
    let mut total = 0.0;
    // Continental crust (the plate model's crust type): its share and the share of it above sea.
    let crust = world
        .fields
        .get("plates/crust_type")
        .filter(|f| f.data.len() == w * h);
    let (mut cont, mut cont_dry) = (0.0, 0.0);
    for j in 0..h {
        let a = cell_area_km2(g, j as u32, 1.0);
        for i in 0..w {
            let z = f64::from(st.elevation_m[j * w + i]) - st.sea_level_m;
            total += a;
            if crust.is_some_and(|f| f.data[j * w + i] >= 0.5) {
                cont += a;
                if z > 0.0 {
                    cont_dry += a;
                }
            }
            if z > 0.0 {
                land.push((z, a));
            } else {
                sea.push((-z, a));
            }
            let b = ((z / 1000.0).floor() + 7.0).clamp(0.0, 12.0) as usize;
            bands[b] += a;
        }
    }
    let quant = |v: &mut Vec<(f64, f64)>, qs: &[f64]| -> Vec<f64> {
        v.sort_by(|x, y| x.0.total_cmp(&y.0));
        let sum: f64 = v.iter().map(|x| x.1).sum();
        let mut out = vec![0.0; qs.len()];
        let mut acc = 0.0;
        let mut k = 0;
        for &(z, a) in v.iter() {
            acc += a;
            while k < qs.len() && acc >= qs[k] * sum {
                out[k] = z;
                k += 1;
            }
        }
        out
    };
    let land_a: f64 = land.iter().map(|x| x.1).sum();
    let sea_a: f64 = sea.iter().map(|x| x.1).sum();
    let above = |km: f64| {
        land.iter()
            .filter(|x| x.0 > km * 1000.0)
            .map(|x| x.1)
            .sum::<f64>()
            / land_a.max(1e-12)
    };
    let mean = land.iter().map(|x| x.0 * x.1).sum::<f64>() / land_a.max(1e-12);
    let (a1, a2, a3) = (above(1.0), above(2.0), above(3.0));
    let shelf = sea
        .iter()
        .filter(|x| x.0 <= 200.0)
        .map(|x| x.1)
        .sum::<f64>()
        / sea_a.max(1e-12);
    let lq = quant(&mut land, &[0.1, 0.25, 0.5, 0.75, 0.9]);
    let sq = quant(&mut sea, &[0.5]);
    Some(format!(
        "hypsometry land_frac_area={:.3} land_mean_m={:.0} land_z_p10_m={:.0} land_z_p25_m={:.0} \
         land_z_p50_m={:.0} land_z_p75_m={:.0} land_z_p90_m={:.0} land_above_1km={:.3} \
         land_above_2km={:.3} land_above_3km={:.3} sea_depth_p50_m={:.0} shelf_frac_of_sea={:.3} \
         continental_frac={:.3} continental_dry={:.3} bands_km[-7..6]={}",
        land_a / total.max(1e-12),
        mean,
        lq[0],
        lq[1],
        lq[2],
        lq[3],
        lq[4],
        a1,
        a2,
        a3,
        sq[0],
        shelf,
        cont / total.max(1e-12),
        cont_dry / cont.max(1e-12),
        bands
            .iter()
            .map(|b| format!("{:.3}", b / total.max(1e-12)))
            .collect::<Vec<_>>()
            .join("/")
    ))
}

/// How much of the map and of the planet is ice (the projection's evidence,
/// `docs/design/equal-area-projection.md` §7): land ice (`climate/ice_thickness_m` ≥ `ICE_MIN_M`,
/// else `climate/glacier`), sea ice (`climate/sea_ice` ≥ 0.5) and both, each as the share of the
/// map's cells (what a picture of the map shows: the cells of rows on the planet) and as the
/// true-area share (cell areas); and the share poleward of 60° both ways (the sphere's 13.4 %).
pub(crate) fn ice_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    use deeptime_core::climate::field;
    use deeptime_core::deposits::fields::{cell_area_km2, lat_of_row};
    let st = world.surface.as_ref()?;
    let g = &st.grid;
    let (w, h) = (g.w as usize, g.h as usize);
    let on = |name: &str| world.fields.get(name).filter(|f| f.data.len() == w * h);
    let (ice, glacier, sea_ice) = (
        on(field::ICE_THICKNESS),
        on(field::GLACIER),
        on(field::SEA_ICE),
    );
    if ice.is_none() && glacier.is_none() && sea_ice.is_none() {
        return None;
    }
    let (mut cells, mut area) = (0.0f64, 0.0f64);
    let mut c = [0.0f64; 4]; // land ice, sea ice, either, poleward of 60° (cells)
    let mut a = [0.0f64; 4];
    for j in 0..h {
        let ar = cell_area_km2(g, j as u32, 1.0);
        if ar <= 0.0 {
            continue; // a row wholly past a pole: not on the map
        }
        let polar = lat_of_row(g, j as u32).abs() >= 60.0;
        for i in 0..w {
            let k = j * w + i;
            let land_ice = match ice {
                Some(f) => f.data[k] >= deeptime_core::climate::ICE_MIN_M,
                None => glacier.is_some_and(|f| f.data[k] >= 0.5),
            };
            let sea = sea_ice.is_some_and(|f| f.data[k] >= 0.5);
            cells += 1.0;
            area += ar;
            for (n, hit) in [land_ice, sea, land_ice || sea, polar]
                .into_iter()
                .enumerate()
            {
                if hit {
                    c[n] += 1.0;
                    a[n] += ar;
                }
            }
        }
    }
    let (cells, area) = (cells.max(1.0), area.max(1e-12));
    Some(format!(
        "ice map_land_ice={:.4} map_sea_ice={:.4} map_ice={:.4} area_land_ice={:.4} \
         area_sea_ice={:.4} area_ice={:.4} map_poleward_60={:.4} area_poleward_60={:.4}",
        c[0] / cells,
        c[1] / cells,
        c[2] / cells,
        a[0] / area,
        a[1] / area,
        a[2] / area,
        c[3] / cells,
        a[3] / area,
    ))
}

/// The planet's hypsographic curve at the end of the surface (P1-17 hypsometry recalibration,
/// 2026-10-01), true-area weighted (cell areas): the elevation above sea level, m, that 0, 1, …, 100 %
/// of the surface stands above (the classic curve: 0 % is the highest cell, 100 % the deepest),
/// for plotting against Earth's.
pub(crate) fn hypsometry_curve_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    use deeptime_core::deposits::fields::cell_area_km2;
    let st = world.surface.as_ref()?;
    let g = &st.grid;
    let (w, h) = (g.w as usize, g.h as usize);
    let mut cells: Vec<(f64, f64)> = Vec::with_capacity(w * h);
    for j in 0..h {
        let a = cell_area_km2(g, j as u32, 1.0);
        for i in 0..w {
            cells.push((f64::from(st.elevation_m[j * w + i]) - st.sea_level_m, a));
        }
    }
    cells.sort_by(|x, y| y.0.total_cmp(&x.0));
    let total: f64 = cells.iter().map(|x| x.1).sum();
    let mut out = Vec::with_capacity(101);
    let (mut acc, mut k) = (0.0, 0usize);
    for &(z, a) in &cells {
        acc += a;
        while k <= 100 && acc >= k as f64 / 100.0 * total {
            out.push(format!("{z:.0}"));
            k += 1;
        }
    }
    while out.len() < 101 {
        out.push(format!("{:.0}", cells.last()?.0));
    }
    Some(format!("hypsometry_curve pct=0..100 z_m={}", out.join("/")))
}

/// How the planet's surface evolved (P1-17 calibration), from the deposit engine's recorded frames
/// (10-Myr means) every 50 Myr, true-area weighted: `t:land/land_z_p50_m`, land being the frame's
/// mean elevation above the datum (the sea-level cycles move the shore by ±100 m).
pub(crate) fn evolution_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    use deeptime_core::deposits::fields::{FieldHistory, cell_area_km2};
    let h = world.history.as_ref()?;
    let g = h.grid.clone();
    let w = g.w as usize;
    let n = h.frame_count();
    let mut parts = Vec::new();
    for k in (0..n)
        .rev()
        .step_by(5)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let f = h.frame(k).ok()?;
        let z = f.fields.get("deeptime:elevation_m")?;
        let (mut tot, mut land) = (0.0, 0.0);
        let mut lz: Vec<(f32, f64)> = Vec::new();
        for (c, &zc) in z.iter().enumerate() {
            let a = cell_area_km2(&g, (c / w) as u32, 1.0);
            tot += a;
            if zc > 0.0 {
                land += a;
                lz.push((zc, a));
            }
        }
        lz.sort_by(|x, y| x.0.total_cmp(&y.0));
        let mut acc = 0.0;
        let mut med = 0.0;
        for &(zc, a) in &lz {
            acc += a;
            if acc >= 0.5 * land {
                med = zc;
                break;
            }
        }
        parts.push(format!(
            "{:.0}:{:.3}/{:.0}",
            f.t_years / 1.0e6,
            land / tot.max(1e-12),
            med
        ));
    }
    Some(format!(
        "evolution t_myr:land/land_z_p50_m {}",
        parts.join(" ")
    ))
}

/// The climate's summary line (P1-05): `climate_final=<snapshots> t_surface_c=… koppen_land=A…/B…
/// …`, from the last snapshot, when the run had a climate stage.
fn climate_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note: Value =
        serde_json::from_str(world.notes.get(deeptime_core::climate::STAGE_NAME)?).ok()?;
    let last = &note["last"];
    let f = |v: &Value| v.as_f64().unwrap_or(f64::NAN);
    let shares = last["koppen_land_share"]
        .as_object()
        .map(|o| {
            o.iter()
                .map(|(k, v)| format!("{k}{:.2}", f(v)))
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_default();
    // The sea ice of the last plate step (the clock's series: annual-mean share of the ocean).
    let series_last = |name: &str| -> f64 {
        let col = note["series"]["columns"]
            .as_array()
            .and_then(|c| c.iter().position(|x| x == name));
        let row = note["series"]["rows"].as_array().and_then(|r| r.last());
        match (col, row) {
            (Some(i), Some(r)) => f(&r[i]),
            _ => f64::NAN,
        }
    };
    Some(format!(
        "climate_final={} t_surface_c={:.1} t_land_c={:.1} precip_land_mm={:.0} koppen_land={} \
         glaciated_land_frac={:.3} ice_sheet_land_frac={:.3} mountain_ice_land_frac={:.3} \
         present_co2_factor={:.3} present_reached={} present_ice_frac={:.3} present_sheet_frac={:.3} \
         present_natural_ice_frac={:.3} present_natural_sheet_frac={:.3} glaciations={} \
         ebm_t_global_c={:.1} ebm_net_toa_w_m2={:.3} \
         itcz_deg={:.1}..{:.1} land_z_p50_m={:.0} land_z_p90_m={:.0} land_above_2km={:.3} \
         co2_ppm={:.0} sea_level_m={:.1} ice_volume_km3={:.3e} sea_ice_frac={:.3} snowballs={} \
         hothouses={}",
        note["snapshots"],
        f(&last["t_surface_c"]),
        f(&last["t_land_c"]),
        f(&last["precip_land_mm"]),
        shares,
        f(&last["glaciated_land_frac"]),
        f(&last["ice_sheet_land_frac"]),
        f(&last["mountain_ice_land_frac"]),
        f(&note["present"]["co2_factor"]),
        note["present"]["reached"],
        f(&note["present"]["land_ice_frac"]),
        f(&note["present"]["land_ice_sheet_frac"]),
        f(&note["present"]["natural_land_ice_frac"]),
        f(&note["present"]["natural_land_ice_sheet_frac"]),
        note["glaciations"],
        f(&last["ebm_t_global_c"]),
        f(&last["ebm_net_toa_w_m2"]),
        f(&last["itcz_range_deg"][0]),
        f(&last["itcz_range_deg"][1]),
        f(&last["land_z_p50_m"]),
        f(&last["land_z_p90_m"]),
        f(&last["land_above_2km"]),
        f(&last["co2_ppm"]),
        f(&last["sea_level_m"]),
        f(&last["ice_volume_km3"]),
        series_last("sea_ice_frac"),
        note["snowballs"],
        note["hothouses"],
    ))
}

/// The carbon cycle over the run (the 2026-10-18 CO₂ trace): one entry per 50 Myr and the last
/// plate step, `t_myr:co2_ppm/t_global_c/t_land_c/degassing/weathering/land/kinetic/supply` (the
/// fluxes in 10¹² mol C a year: the land's with and without the supply limit, and the supply)
/// `/kernel_rel/kernel_rel_sea_t/kernel_high/land_high/land_frac/runoff_m_yr/ridge/subduction`
/// (km² a year).
fn carbon_series_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note: Value =
        serde_json::from_str(world.notes.get(deeptime_core::climate::STAGE_NAME)?).ok()?;
    let cols = note["series"]["columns"].as_array()?;
    let rows = note["series"]["rows"].as_array()?;
    let at = |name: &str| cols.iter().position(|x| x == name);
    let names = [
        "co2_ppm",
        "t_global_c",
        "t_land_mean_c",
        "degassing_mol_yr",
        "weathering_mol_yr",
        "weathering_land_mol_yr",
        "weathering_kinetic_mol_yr",
        "weathering_supply_mol_yr",
        "kernel_rel",
        "kernel_rel_sea_t",
        "kernel_high_share",
        "land_high_share",
        "land_frac",
        "runoff_land_m_yr",
        "ridge_km2_yr",
        "subduction_km2_yr",
    ];
    let it = at("t_myr")?;
    let idx: Vec<Option<usize>> = names.iter().map(|n| at(n)).collect();
    let mut parts = Vec::new();
    let mut next = f64::NEG_INFINITY;
    for (k, r) in rows.iter().enumerate() {
        let r = r.as_array()?;
        let t = r[it].as_f64().unwrap_or(f64::NAN);
        if t + 1e-6 < next && k + 1 != rows.len() {
            continue;
        }
        next = (t / 50.0).floor() * 50.0 + 50.0;
        let v: Vec<String> = idx
            .iter()
            .zip(names)
            .map(|(i, n)| {
                let x = i
                    .and_then(|i| r.get(i))
                    .and_then(Value::as_f64)
                    .unwrap_or(f64::NAN);
                if n.ends_with("mol_yr") {
                    format!("{:.3}", x / 1.0e12)
                } else if n == "co2_ppm" {
                    format!("{x:.0}")
                } else if n.ends_with("_c") {
                    format!("{x:.1}")
                } else {
                    format!("{x:.3}")
                }
            })
            .collect();
        parts.push(format!("{t:.0}:{}", v.join("/")));
    }
    Some(format!("carbon_series {}", parts.join(" ")))
}

/// The surface stage's land erosion by elevation band and by local relief (the erosion
/// recalibration of 2026-10-18, `surface::erosion_bands`): `erosion_bands <class>:<area share of
/// the land>/<net>/<stream power>/<critical-slope cut>/<diffusion>/<glacial>` in m/Myr, the
/// classes `z<lower end, m>`, `flat`, `steep` and `rz<lower end of relief, m>`.
fn erosion_bands_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let st = world.surface.as_ref()?;
    let b = &st.totals.by_band;
    let land: f64 = b.bands.iter().map(|p| p.area_km2_myr).sum();
    if land <= 0.0 {
        return None;
    }
    let one = |name: String, p: &deeptime_core::surface::erosion_bands::ProcessSums| {
        let r = |v: f64| {
            if p.area_km2_myr > 0.0 {
                v / p.area_km2_myr * 1000.0
            } else {
                f64::NAN
            }
        };
        format!(
            "{name}:{:.4}/{:.1}/{:.1}/{:.1}/{:.1}/{:.1}",
            p.area_km2_myr / land,
            r(p.net_km3),
            r(p.spl_km3),
            r(p.threshold_km3),
            r(p.diffusion_km3),
            r(p.glacial_km3)
        )
    };
    let mut parts = Vec::new();
    for (i, p) in b.bands.iter().enumerate() {
        parts.push(one(format!("z{}", i * 1000), p));
    }
    parts.push(one("flat".into(), &b.high_flat));
    parts.push(one("steep".into(), &b.high_steep));
    for (i, p) in b.by_relief.iter().enumerate() {
        let lo = if i == 0 {
            0.0
        } else {
            deeptime_core::surface::erosion_bands::RELIEF_FROM_M[i - 1]
        };
        parts.push(one(format!("rz{lo}"), p));
    }
    for (i, p) in b.by_window_relief.iter().enumerate() {
        let lo = if i == 0 {
            0.0
        } else {
            deeptime_core::surface::erosion_bands::RELIEF_FROM_M[i - 1]
        };
        parts.push(one(format!("wz{lo}"), p));
    }
    let windows: f64 = b.window_rate.iter().sum();
    let shares: Vec<String> = b
        .window_rate
        .iter()
        .map(|a| format!("{:.4}", a / windows.max(1e-30)))
        .collect();
    parts.push(format!("wrate:{}", shares.join("/")));
    Some(format!("erosion_bands {}", parts.join(" ")))
}

/// The per-cell climate's line (P1-26): `climate_cell t_2d_c=… zonal_c=… flux_fit_rms_k=… …`
/// from the last snapshot's per-cell figures, the triggered snapshots and every snowball's
/// escape CO₂; `None` under the banded climate.
fn percell_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note: Value =
        serde_json::from_str(world.notes.get(deeptime_core::climate::STAGE_NAME)?).ok()?;
    let c = &note["last"]["per_cell"];
    if c.is_null() {
        return None;
    }
    let f = |v: &Value| v.as_f64().unwrap_or(f64::NAN);
    let fc = &c["flux_corr"];
    let esc: Vec<String> = note["snowball_escape_co2_ppm"]
        .as_array()
        .map(|a| a.iter().map(|x| format!("{:.0}", f(x))).collect())
        .unwrap_or_default();
    Some(format!(
        "climate_cell t_2d_c={:.2} net_toa_2d_w_m2={:.3} years_2d={} zonal_c={:.2} \
         flux_fit_rms_k={:.3} flux_fit_worst_k={:.2} flux_fit_first_rms_k={:.2} \
         flux_w_m2={:.1}..{:.1} flux_capped={} d_face_w_m2_k={:.2}..{:.2} fit_iterations={} \
         atm_peak_pw={:.2}@{:.0} ocean_peak_pw={:.2}@{:.0} psi_sv={:.1} sinking_lat={} \
         triggered_snapshots={} escape_co2_ppm=[{}]",
        f(&c["t_global_c"]),
        f(&c["net_toa_w_m2"]),
        c["years"],
        f(&c["zonal_t_global_c"]),
        f(&fc["rms_k"]),
        f(&fc["worst_k"]),
        f(&fc["first_rms_k"]),
        f(&fc["min_w_m2"]),
        f(&fc["max_w_m2"]),
        fc["capped"],
        f(&fc["d_min_w_m2_k"]),
        f(&fc["d_max_w_m2_k"]),
        fc["iterations"],
        f(&c["atm_heat_peak_pw"]),
        f(&c["atm_heat_peak_lat"]),
        f(&c["ocean_heat_peak_pw"]),
        f(&c["ocean_heat_peak_lat"]),
        f(&c["gyre_psi_max_sv"]),
        c["sinking"]["lat"]
            .as_f64()
            .map_or("none".to_string(), |x| format!("{x:.0}")),
        note["triggered_snapshots"],
        esc.join(","),
    ))
}

/// The structure stage's summary line (P1-07): `structure_final=<steps> plutons_emplaced=… …`,
/// when the pipeline ran it.
fn structure_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note = world.notes.get(deeptime_core::structure::STAGE_NAME)?;
    let v: Value = serde_json::from_str(note).ok()?;
    Some(deeptime_core::structure::summary_line(&v))
}

/// The volcano stage's summary line (P1-21): `volcanoes_final=<steps> strato_born=… …`, when the
/// pipeline ran it.
fn volcanoes_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note = world
        .notes
        .get(deeptime_core::surface::volcanoes::STAGE_NAME)?;
    let v: Value = serde_json::from_str(note).ok()?;
    Some(deeptime_core::surface::volcanoes::summary_line(&v))
}

/// The impact stage's summary line (P2-08): `impacts_final=<steps> impactors=… …`, when the
/// pipeline ran it.
fn impacts_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note = world
        .notes
        .get(deeptime_core::surface::impacts::STAGE_NAME)?;
    let v: Value = serde_json::from_str(note).ok()?;
    Some(deeptime_core::surface::impacts::summary_line(&v))
}

/// The glaciers' summary line: `glacial_final eroded_km3=… till_km3=… …`, when the surface had
/// glaciers to model (a climate stage).
fn glacial_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    deeptime_core::surface::glacial::summary_line(&world.notes)
}

/// The young landscape's summary line (P1-06): `young_final=… land_z_p50_m=a->b …`, when the
/// pipeline ran it.
fn young_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note = world.notes.get(deeptime_core::surface::young::STAGE_NAME)?;
    let v: Value = serde_json::from_str(note).ok()?;
    v.get("after")?;
    Some(deeptime_core::surface::young::summary_line(&v))
}

/// The amplifier's summary line (P1-06): `amplify_final=8192x4096 mode=… sinks=0 passes[…]`, when
/// the pipeline ran it.
fn amplify_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    let note = world
        .notes
        .get(deeptime_core::surface::amplify::STAGE_NAME)?;
    let v: Value = serde_json::from_str(note).ok()?;
    Some(deeptime_core::surface::amplify::summary_line(&v))
}

/// The deposit engine's recorded history (P1-04): per field of the last frame, the share of cells
/// where it is non-zero and its mean there.
fn history_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    use deeptime_core::deposits::fields::FieldHistory;
    let h = world.history.as_ref()?;
    let k = h.frame_count().checked_sub(1)?;
    let f = h.frame(k).ok()?;
    let mut parts = vec![format!("frames={} last_t={}", h.frame_count(), f.t_years)];
    for (name, v) in &f.fields {
        let nz: Vec<f32> = v
            .iter()
            .copied()
            .filter(|x| x.is_finite() && *x != 0.0)
            .collect();
        let mean = nz.iter().map(|&x| f64::from(x)).sum::<f64>() / nz.len().max(1) as f64;
        parts.push(format!(
            "{}={:.1}%/{:.3}",
            name.trim_start_matches("deeptime:"),
            100.0 * nz.len() as f64 / v.len().max(1) as f64,
            mean
        ));
    }
    Some(parts.join(" "))
}

/// Per-stage wall time, and the surface's and the climate's per-process split when they ran to
/// the end.
pub(crate) fn timings_line(
    timings: &[deeptime_core::session::StageTiming],
    world: &deeptime_core::world::WorldState,
) -> String {
    let mut parts: Vec<String> = timings
        .iter()
        .map(|t| format!("{}={:.1}s/{}steps", t.name, t.millis / 1000.0, t.steps))
        .collect();
    // The stages that stepped beside their lead (P1-19): that time is inside the lead's.
    let beside: Vec<String> = timings
        .iter()
        .filter(|t| t.beside_millis > 0.0)
        .map(|t| format!("{}={:.1}s", t.name, t.beside_millis / 1000.0))
        .collect();
    if !beside.is_empty() {
        parts.push(format!("beside[{}]", beside.join(",")));
    }
    for stage in [
        deeptime_core::surface::STAGE_NAME,
        deeptime_core::climate::STAGE_NAME,
        deeptime_core::structure::STAGE_NAME,
        deeptime_core::surface::volcanoes::STAGE_NAME,
        deeptime_core::surface::impacts::STAGE_NAME,
        deeptime_core::surface::young::STAGE_NAME,
        deeptime_core::deposits::STAGE_NAME,
    ] {
        if let Some(note) = world
            .notes
            .get(stage)
            .and_then(|n| serde_json::from_str::<Value>(n).ok())
            && let Some(ms) = note["millis"].as_object()
        {
            parts.push(format!(
                "{stage}[{}]",
                ms.iter()
                    .map(|(k, v)| format!("{k}={:.1}s", v.as_f64().unwrap_or(0.0) / 1000.0))
                    .collect::<Vec<_>>()
                    .join(",")
            ));
        }
    }
    // The plate step's parts (a process-wide gauge, not a world note: P1-17).
    let plates = deeptime_core::plates::timings();
    if !plates.is_empty() {
        parts.push(format!(
            "{}[{}]",
            deeptime_core::plates::STAGE_NAME,
            plates
                .iter()
                .map(|(k, ms)| format!("{k}={:.1}s", ms / 1000.0))
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    // The GPU kernels' parts and their CPU twins (a process-wide gauge too; `compute.rs`):
    // seconds, and plain counts for the names ending in `_n`.
    let compute = deeptime_core::compute::timings();
    if !compute.is_empty() {
        parts.push(format!(
            "compute[{}]",
            compute
                .iter()
                .map(|(k, v)| if k.ends_with("_n") {
                    format!("{k}={v:.0}")
                } else {
                    format!("{k}={:.1}s", v / 1000.0)
                })
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    parts.join(" ")
}

pub struct RunOptions {
    /// A built-in name, a file or a share code.
    pub preset: String,
    /// `timeline.start`, as `config/deeptime/server.json` sets it on a server.
    pub start: Option<String>,
    /// `timeline.length_myr`.
    pub length_myr: Option<f64>,
    pub content: Option<PathBuf>,
    /// Pack placement tables (`data/climate_mapping/packs/*.json`) added to the content bundle's
    /// `climate_placement_patches`, in order: what the game passes when their biome mod is
    /// installed.
    pub placement_patch: Vec<PathBuf>,
    pub out: PathBuf,
    pub viewer: Option<PathBuf>,
    pub checkpoints: bool,
    /// `timeline.stop_myr`.
    pub stop_myr: Option<f64>,
    /// Write epoch checkpoints into `<out>/epochs/`.
    pub epochs: bool,
    pub common: Common,
}

/// `content` (a JSON object, or empty) with the pack placement tables in `patches` appended to its
/// `climate_placement_patches` list.
fn with_placement_patches(content: String, patches: &[PathBuf]) -> CliResult<String> {
    if patches.is_empty() {
        return Ok(content);
    }
    let mut root: serde_json::Value = if content.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(&content)
            .map_err(|e| format!("the content bundle is not valid JSON: {e}"))?
    };
    let key = deeptime_core::bake::placement::PATCHES_KEY;
    let list = root
        .as_object_mut()
        .ok_or("the content bundle is not a JSON object")?
        .entry(key)
        .or_insert_with(|| serde_json::json!([]));
    let list = list
        .as_array_mut()
        .ok_or_else(|| format!("the content bundle's {key} is not a list"))?;
    for p in patches {
        let text = read_text(p, "placement patch")?;
        let v: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| format!("{} is not valid JSON: {e}", p.display()))?;
        list.push(v);
    }
    Ok(root.to_string())
}

/// `deeptime run`: a whole planet into `--out`.
pub fn run(o: &RunOptions) -> CliResult<()> {
    let started = Instant::now();
    let preset = deeptime_core::preset::with_timeline(
        &load_preset(&o.preset)?,
        o.start.as_deref(),
        o.length_myr,
    )?;
    let preset = match o.stop_myr {
        Some(s) => deeptime_core::preset::with_stop(&preset, Some(s))?,
        None => preset,
    };
    let preset = apply_overrides(&preset, &o.common.set)?;
    let content = match &o.content {
        Some(p) => read_text(p, "content bundle")?,
        None => String::new(),
    };
    let content = with_placement_patches(content, &o.placement_patch)?;
    let mut s = Session::create(
        &preset,
        &content,
        session_options(&o.out, &o.common),
        Arc::new(AllocGauge),
    )?;
    if o.checkpoints {
        s.set_checkpoint_dir(Some(o.out.join("checkpoints")));
    }
    s.set_epochs(o.epochs)?;
    // What the editor would show for this run, beside what the run then measures.
    if let Ok(e) = deeptime_core::session::estimate_for(&preset, &content, o.common.mem_cap)
        && let Ok(e) = serde_json::from_str::<Value>(&e)
    {
        let mib = |v: &Value, k: &str| v[k].as_f64().unwrap_or(0.0) / 1048576.0;
        let stages = e["stages"].as_array().map_or(String::new(), |st| {
            st.iter()
                .map(|x| {
                    format!(
                        "{}={:.0}",
                        x["name"].as_str().unwrap_or("?"),
                        mib(x, "bytes")
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        });
        let own = e["stages"].as_array().map_or(String::new(), |st| {
            st.iter()
                .map(|x| {
                    format!(
                        "{}={:.0}",
                        x["name"].as_str().unwrap_or("?"),
                        mib(x, "own_bytes")
                    )
                })
                .collect::<Vec<_>>()
                .join(" ")
        });
        println!(
            "deeptime run: estimate peak_mib={:.0} base_mib={:.0} history_mib={:.0} spill_mib={:.0} cap_mib={:.0} stages_mib[{stages}] own_mib[{own}]",
            mib(&e, "peak_bytes"),
            mib(&e, "base_bytes"),
            mib(&e, "history_bytes"),
            mib(&e, "spill_bytes"),
            mib(&e, "mem_cap"),
        );
    }
    s.set_history_budget(o.common.history_mem);
    o.common.apply_compute(&s);
    drive(&s, o.common.verbose, |_, st| st == AdvanceStatus::Done)?;
    s.finalize_write()?;
    forward_log(&s, o.common.verbose);
    report_run(&s, &o.out, &o.preset, o.viewer.as_deref(), started)
}

pub struct ResumeOptions {
    /// The run's folder (it holds `epochs/`).
    pub out: PathBuf,
    /// The new stop; `None` keeps the run's own.
    pub stop_myr: Option<f64>,
    /// Keep writing epoch checkpoints.
    pub epochs: bool,
    pub viewer: Option<PathBuf>,
    pub common: Common,
}

/// `deeptime resume`: the run in `--out` again to another stop, from its epoch checkpoints.
pub fn resume(o: &ResumeOptions) -> CliResult<()> {
    let started = Instant::now();
    let index = deeptime_core::epochs::EpochIndex::read(&o.out)?.ok_or_else(|| {
        format!(
            "{} has no epoch checkpoints (run it with --epochs)",
            o.out.display()
        )
    })?;
    let preset = deeptime_core::preset::with_stop(&index.preset.to_string(), None)?;
    let preset = match o.stop_myr {
        Some(s) => deeptime_core::preset::with_stop(&preset, Some(s))?,
        None => index
            .preset
            .pointer("/timeline/stop_myr")
            .and_then(serde_json::Value::as_f64)
            .map_or(Ok(preset.clone()), |s| {
                deeptime_core::preset::with_stop(&preset, Some(s))
            })?,
    };
    let preset = apply_overrides(&preset, &o.common.set)?;
    let mut s = Session::resume_epoch(
        &preset,
        &index.content,
        session_options(&o.out, &o.common),
        Arc::new(AllocGauge),
    )?;
    s.set_epochs(o.epochs)?;
    s.set_history_budget(o.common.history_mem);
    o.common.apply_compute(&s);
    if let Some(r) = s.resumed() {
        println!(
            "deeptime resume: from plate step {} (t = {:.1} Myr), {} of {} steps to run, {} keyframes kept",
            r.from_step,
            r.from_t_years / 1.0e6,
            r.steps.saturating_sub(r.from_step),
            r.steps,
            r.keyframes_kept
        );
    }
    drive(&s, o.common.verbose, |_, st| st == AdvanceStatus::Done)?;
    s.finalize_write()?;
    forward_log(&s, o.common.verbose);
    report_run(
        &s,
        &o.out,
        &o.out.display().to_string(),
        o.viewer.as_deref(),
        started,
    )
}

/// What `run` and `resume` print once the world data is written.
fn report_run(
    s: &Session,
    out: &Path,
    label: &str,
    viewer: Option<&Path>,
    started: Instant,
) -> CliResult<()> {
    let manifest = deeptime_format::Manifest::read(out)?;
    if let Some(c) = manifest.notes.get(deeptime_core::deposits::STAGE_NAME) {
        print_census(c);
    }
    let p = s.progress();
    println!(
        "deeptime run: {} -> {} ({} stages, {} events, {} keyframes, peak {:.1} MiB) in {:.2}s",
        s.preset().name(),
        out.display(),
        p.stage_count,
        p.events_total,
        p.keyframes_ready,
        alloc::peak() as f64 / 1048576.0,
        started.elapsed().as_secs_f64()
    );
    for (name, a) in alloc::trace_summary() {
        println!(
            "deeptime run: memtrace phase={name} count={} seconds={:.1} live_peak_mib={:.1} rss_peak_mib={:.1} rss_end_mib={:.1} reserved_mib={:.1}",
            a.count,
            a.seconds,
            a.live_peak as f64 / 1048576.0,
            a.rss_peak as f64 / 1048576.0,
            a.rss_end as f64 / 1048576.0,
            a.reserved as f64 / 1048576.0,
        );
    }
    // The world as it stands, not a copy of it (a 32k full world is gigabytes).
    let timings = s.stage_timings()?;
    let lines = s.with_world(|world| {
        let mut lines: Vec<String> = [
            plates_line(world),
            margins_line(world),
            deeptime_core::provinces::coherence_line(world),
            surface_line(world),
            glacial_line(world),
            erosion_bands_line(world),
            climate_line(world),
            carbon_series_line(world),
            percell_line(world),
            structure_line(world),
            volcanoes_line(world),
            impacts_line(world),
            young_line(world),
            amplify_line(world),
        ]
        .into_iter()
        .flatten()
        .collect();
        lines.push(format!("timings {}", timings_line(&timings, world)));
        lines
    })?;
    for line in lines {
        println!("deeptime run: {line}");
    }
    if let Some(line) = spill_line(s)? {
        println!("deeptime run: {line}");
    }
    if let Some(b) = manifest.notes.get(deeptime_core::bake::NOTES_KEY) {
        println!("deeptime run: {}", deeptime_core::bake::summary_line(b));
    }
    if let Some(dir) = viewer {
        let title = format!("Deep Time — run {} ({label})", s.preset().name());
        deeptime_viewer::write_world_bundle(dir, out, &title)?;
        println!(
            "deeptime run: viewer -> {}",
            dir.join("index.html").display()
        );
    }
    Ok(())
}

pub struct StageOptions {
    pub name: String,
    pub input: Option<PathBuf>,
    pub preset: Option<String>,
    /// `timeline.start` for `--preset`.
    pub start: Option<String>,
    /// `timeline.length_myr` for `--preset`.
    pub length_myr: Option<f64>,
    /// Stop after this many steps of the stage (a mid-stage checkpoint).
    pub max_steps: Option<u64>,
    pub out: Option<PathBuf>,
    pub viewer: Option<PathBuf>,
    /// Keyframe layers the viewer bundle keeps (empty: all).
    pub viewer_layers: Vec<String>,
    pub work: Option<PathBuf>,
    /// Bake the world data into the work folder when the stage finishes the run.
    pub bake: bool,
    pub common: Common,
}

/// A work folder of its own for one call that was given none: the process id and a counter, so two
/// calls in one process (the CLI tests run in parallel threads) never share a folder: with the
/// process id alone (`deeptime-stage-<pid>`) the call that finished first would delete the folder
/// under the other.
pub(crate) fn temp_work(what: &str) -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!("deeptime-{what}-{}-{n}", std::process::id()))
}

/// `deeptime stage <name>`: one stage from a checkpoint (or from a preset, running the stages
/// before it first), then a checkpoint after it.
pub fn stage(o: &StageOptions) -> CliResult<()> {
    let work = o.work.clone().unwrap_or_else(|| match &o.out {
        Some(out) => {
            let mut name = out.file_name().unwrap_or_default().to_os_string();
            name.push(".work");
            out.with_file_name(name)
        }
        None => temp_work("stage"),
    });
    let opts = session_options(&work, &o.common);
    let s = match (&o.input, &o.preset) {
        (Some(input), None) => {
            // An epoch checkpoint is read with the epochs its history needs (`dump::chain_of`).
            let mut ck = Checkpoint::read_chain(&crate::dump::chain_of(input)?)?;
            ck.preset_json = apply_overrides(&ck.preset_json, &o.common.set)?;
            // A stage coupled to others (plates + surface) runs with its group, which starts at
            // the group's lead: a checkpoint whose next stage is the lead serves any member.
            let group = deeptime_core::preset::Preset::from_json(&ck.preset_json)
                .ok()
                .and_then(|p| group_of(&p.stages(), &o.name));
            match ck.next_stage_name() {
                Some(next) if next == o.name => {}
                Some(_)
                    if group
                        .as_ref()
                        .is_some_and(|g| g.start == ck.next_stage as usize) => {}
                next => {
                    let pos = ck.stage_names.iter().position(|n| *n == o.name);
                    let why = match (pos, next) {
                        (None, _) => format!("the pipeline has no stage {:?}", o.name),
                        (Some(p), _) if p < ck.next_stage as usize => {
                            format!("stage {} already ran in this checkpoint", o.name)
                        }
                        (_, Some(next)) => format!("stage {next} runs first"),
                        (_, None) => "every stage already ran".to_string(),
                    };
                    return Err(format!(
                        "{}: cannot run stage {}: {why} (pipeline {:?}, next {:?})",
                        input.display(),
                        o.name,
                        ck.stage_names,
                        next
                    )
                    .into());
                }
            }
            let mut s = Session::resume(ck, opts, Arc::new(AllocGauge))?;
            s.set_history_budget(o.common.history_mem);
            o.common.apply_compute(&s);
            s
        }
        (None, Some(preset)) => {
            let json = load_preset_with(preset, o.start.as_deref(), o.length_myr)?;
            let json = apply_overrides(&json, &o.common.set)?;
            let mut s = Session::create(&json, "", opts, Arc::new(AllocGauge))?;
            s.set_history_budget(o.common.history_mem);
            o.common.apply_compute(&s);
            let stages = s.preset().stages();
            let target = group_of(&stages, &o.name).map(|g| g.start).ok_or_else(|| {
                format!(
                    "the pipeline {:?} has no stage {:?}",
                    s.stage_names(),
                    o.name
                )
            })? as u32;
            if target > 0 {
                drive(&s, o.common.verbose, |s, st| {
                    st == AdvanceStatus::Done || s.progress().stage_index >= target
                })?;
            }
            s
        }
        _ => return Err("give exactly one of --in <checkpoint> and --preset <preset>".into()),
    };
    let events_before = s.progress().events_total;
    let stepping = Instant::now();
    match o.max_steps {
        None => {
            drive(&s, o.common.verbose, |_, st| {
                matches!(st, AdvanceStatus::StageDone | AdvanceStatus::Done)
            })?;
        }
        Some(max) => {
            // One step per call: advance with a zero budget runs exactly one.
            for _ in 0..max {
                let status = s.advance(Duration::ZERO);
                forward_log(&s, o.common.verbose);
                if status? != AdvanceStatus::Running {
                    break;
                }
            }
        }
    }
    let seconds = stepping.elapsed().as_secs_f64();
    let ck = s.checkpoint()?;
    let timing = s
        .stage_timings()?
        .into_iter()
        .find(|t| t.name == o.name)
        .map_or(0, |t| t.steps);
    let p = s.progress();
    println!(
        "deeptime stage {}: steps={} t_years={} events={} (+{}) sites={} keyframes={} seconds={:.3}",
        o.name,
        timing,
        ck.world.t_years,
        p.events_total,
        p.events_total - events_before,
        ck.world.sites.len(),
        p.keyframes_ready,
        seconds
    );
    let census = census_of(&ck.world.notes);
    if let Some(c) = &census {
        print_census(c);
    }
    if let Some(line) = plates_line(&ck.world) {
        println!(
            "deeptime stage {}: {line} steps={timing} seconds={seconds:.3}",
            o.name
        );
    }
    if let Some(line) = margins_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = deeptime_core::provinces::coherence_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = surface_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = glacial_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = erosion_bands_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = hypsometry_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = hypsometry_curve_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = ice_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = evolution_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = climate_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = carbon_series_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = percell_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let (Some(path), Some(note)) = (
        o.common.climate_json.as_ref(),
        ck.world.notes.get(deeptime_core::climate::STAGE_NAME),
    ) {
        std::fs::write(path, note)?;
    }
    if let Some(line) = structure_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = volcanoes_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = impacts_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = young_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = amplify_line(&ck.world) {
        println!("deeptime stage {}: {line}", o.name);
    }
    if let Some(line) = history_line(&ck.world) {
        println!("deeptime stage {}: history {line}", o.name);
    }
    if let Some(line) = spill_line(&s)? {
        println!("deeptime stage {}: {line}", o.name);
    }
    println!(
        "deeptime stage {}: timings {}",
        o.name,
        timings_line(&s.stage_timings()?, &ck.world)
    );
    if o.bake {
        if s.state() == SessionState::Done {
            s.finalize_write()?;
            println!(
                "deeptime stage {}: baked the world data into {}",
                o.name,
                work.display()
            );
        } else {
            println!(
                "deeptime stage {}: --bake ignored, the run is not complete",
                o.name
            );
        }
    }
    if let Some(out) = &o.out {
        ck.write(out)?;
        println!(
            "deeptime stage {}: checkpoint -> {} (next: {})",
            o.name,
            out.display(),
            ck.next_stage_name().unwrap_or("none, the run is complete")
        );
    }
    if let Some(dir) = &o.viewer {
        let index = KeyframeIndex::read(&work)?;
        let mut keyframes = index
            .frames
            .iter()
            .map(|e| read_keyframe(&work, e))
            .collect::<Result<Vec<_>, _>>()?;
        if !o.viewer_layers.is_empty() {
            for k in &mut keyframes {
                k.layers.retain(|(name, _)| o.viewer_layers.contains(name));
            }
        } else if o.name == deeptime_core::surface::young::STAGE_NAME
            || o.name == deeptime_core::surface::amplify::STAGE_NAME
        {
            // P1-06: the young landscape's and the amplifier's own frames (the deep-time frames
            // belong to the surface bundle); the before/after views come from the L3 layer.
            keyframes.retain(|k| k.stage.starts_with("young") || k.stage.starts_with("amplify"));
        } else if o.name == deeptime_core::deposits::STAGE_NAME {
            // The deposit bundle keeps the map context (elevation, plates, crust age, regime,
            // drainage, the rock and the trap structure) and the deposit layers; the surface's
            // bulk layers stay in the surface bundle. Keeps a 500 Myr bundle under the review cap.
            const LEFT_TO_THE_SURFACE_BUNDLE: [&str; 6] = [
                "surface/erosion_m_myr",
                "surface/deposition_m_myr",
                "surface/sediment_m",
                "surface/regolith_m",
                "surface/class",
                "structure/log_grade",
            ];
            for k in &mut keyframes {
                k.layers
                    .retain(|(name, _)| !LEFT_TO_THE_SURFACE_BUNDLE.contains(&name.as_str()));
            }
        }
        let info = deeptime_viewer::BundleInfo {
            title: format!("Deep Time — stage {}", o.name),
            source: format!("stage:{}", o.name),
            circumference: ck.world.circumference,
            sea_level_y: 63,
            format_version: deeptime_format::FORMAT_VERSION.to_owned(),
            created_utc: rfc3339_utc(SystemTime::now()),
        };
        // P1-07: the baked horizon stacks (the inspector), the elevation and the rock at the
        // surface as map layers, and meridian cross-sections.
        // `--viewer-layers` without `structure` leaves them out (a climate-only page stays small);
        // the deposit stage's bundle leaves the strata inspector to the structure stage's own (its
        // layers alone are ~20 MB, and the deposit bundles stay under the review cap).
        let structure = if o.name == deeptime_core::deposits::STAGE_NAME
            || o.name == deeptime_core::surface::young::STAGE_NAME
            || o.name == deeptime_core::surface::amplify::STAGE_NAME
            || !(o.viewer_layers.is_empty()
                || o.viewer_layers.iter().any(|l| l.starts_with("structure")))
        {
            None
        } else {
            crate::structure_view::build(&ck.world, &ck.preset_json)?
        };
        // P1-06: the hill-shaded before/after views of the amplifier.
        let amplified = if o.name == deeptime_core::surface::young::STAGE_NAME
            || o.name == deeptime_core::surface::amplify::STAGE_NAME
        {
            crate::amplify_view::build(&ck.world, &ck.preset_json, &work)?
        } else {
            None
        };
        let layers: Vec<&deeptime_format::LoadedLayer> =
            structure.iter().flat_map(|v| v.layers.iter()).collect();
        deeptime_viewer::write_bundle_with(
            dir,
            &info,
            &layers,
            structure.as_ref().map(|v| deeptime_viewer::Strata {
                layer: &v.strata,
                classes: v.classes.clone(),
            }),
            &deeptime_viewer::Timeline {
                keyframes,
                events: ck.world.events.clone(),
                sites: ck.world.sites.clone(),
            },
        )?;
        let mut report = deeptime_viewer::deposit_report(census.as_ref(), &ck.world.bodies);
        if let Some(v) = &structure {
            let st = crate::structure_view::write(dir, v)?;
            report
                .get_or_insert_with(|| json!({}))
                .as_object_mut()
                .expect("an object")
                .insert("structure".into(), st);
            println!(
                "deeptime stage {}: {} cross-sections -> {}",
                o.name,
                v.sections.len(),
                dir.join("sections").display()
            );
        }
        if let Some(v) = &amplified {
            let am = crate::amplify_view::write(dir, v)?;
            report
                .get_or_insert_with(|| json!({}))
                .as_object_mut()
                .expect("an object")
                .insert("amplify".into(), am);
            println!(
                "deeptime stage {}: {} before/after views -> {}",
                o.name,
                v.regions.len(),
                dir.join("amplify").display()
            );
        }
        if let Some(report) = report {
            deeptime_viewer::write_report(dir, &report)?;
        }
        println!(
            "deeptime stage {}: viewer -> {}",
            o.name,
            dir.join("index.html").display()
        );
    }
    if o.work.is_none() && o.out.is_none() {
        let _ = std::fs::remove_dir_all(&work);
    }
    Ok(())
}

pub struct BenchOptions {
    pub preset: String,
    pub start: Option<String>,
    pub length_myr: Option<f64>,
    pub threads: Vec<usize>,
    pub out: PathBuf,
    pub viewer: Option<PathBuf>,
    pub work: Option<PathBuf>,
    pub mem_cap: u64,
    pub verbose: bool,
    pub compute: Option<ComputeMode>,
}

/// `deeptime bench`: the preset once per thread count, with per-stage wall time and peak native
/// memory, into `bench.json` and a timing page (the Phase 1 budget gate's skeleton, P1-17).
pub fn bench(o: &BenchOptions) -> CliResult<()> {
    let preset = load_preset_with(&o.preset, o.start.as_deref(), o.length_myr)?;
    // What the editor shows a player before the run (plan §5.7.5): the gate compares the
    // measured peak with it (P1-17).
    let estimate: Value = serde_json::from_str(&deeptime_core::session::estimate_for(
        &preset, "", o.mem_cap,
    )?)?;
    let estimate_seconds: f64 = estimate["stages"].as_array().map_or(0.0, |a| {
        a.iter().filter_map(|s| s["seconds"].as_f64()).sum()
    });
    println!(
        "deeptime bench: estimate peak_bytes={} seconds={estimate_seconds:.0} stages={}",
        estimate["peak_bytes"],
        estimate["stages"].as_array().map_or(String::new(), |a| a
            .iter()
            .map(|s| format!(
                "{}:{}B/{:.0}s",
                s["name"].as_str().unwrap_or("?"),
                s["bytes"],
                s["seconds"].as_f64().unwrap_or(0.0)
            ))
            .collect::<Vec<_>>()
            .join(","))
    );
    let temp = o.work.is_none();
    let work = o.work.clone().unwrap_or_else(|| temp_work("bench"));
    let mut runs = Vec::new();
    let mut name = String::new();
    for &threads in &o.threads {
        let out = work.join(format!("t{threads}"));
        let _ = std::fs::remove_dir_all(&out);
        alloc::reset_peak();
        deeptime_core::plates::reset_timings();
        deeptime_core::compute::reset_timings();
        let started = Instant::now();
        let s = Session::create(
            &preset,
            "",
            SessionOptions {
                out_dir: out,
                threads,
                mem_cap: o.mem_cap,
            },
            Arc::new(AllocGauge),
        )?;
        if let Some(m) = o.compute {
            s.set_compute(m);
        }
        name = s.preset().name().to_owned();
        // P1-19 measurement: the structure step after the plates' instead of beside them.
        if std::env::var_os("DEEPTIME_BENCH_NO_OVERLAP").is_some() {
            s.set_overlap(false);
        }
        drive(&s, o.verbose, |_, st| st == AdvanceStatus::Done)?;
        let finalize = Instant::now();
        s.finalize_write()?;
        let finalize_ms = finalize.elapsed().as_secs_f64() * 1000.0;
        forward_log(&s, o.verbose);
        let total_ms = started.elapsed().as_secs_f64() * 1000.0;
        let peak = alloc::peak();
        let mut stages: Vec<Value> = s
            .stage_timings()?
            .iter()
            .map(|t| {
                json!({
                    "name": t.name, "steps": t.steps, "millis": t.millis,
                    "beside_millis": t.beside_millis,
                })
            })
            .collect();
        stages.push(json!({ "name": "finalize", "steps": 1, "millis": finalize_ms }));
        let world = s.checkpoint()?.world;
        if let Some(line) = plates_line(&world) {
            println!("deeptime bench: threads={threads} {line}");
        }
        let [dense, sparse, movers, visited] = deeptime_core::surface::marine::failure_counts();
        println!(
            "deeptime bench: threads={threads} slope_failure full_sweeps={dense} neighbourhood_passes={sparse} \
             movers_per_pass={:.0} cells_per_pass={:.0}",
            movers as f64 / sparse.max(1) as f64,
            visited as f64 / sparse.max(1) as f64
        );
        // The P1-19 audit of the diffusion solve, when its environment variable is set.
        if let Some(line) = deeptime_core::surface::diffusion::audit::audit_line() {
            println!("deeptime bench: threads={threads} {line}");
        }
        if let Some(line) = surface_line(&world) {
            println!("deeptime bench: threads={threads} {line}");
        }
        if let Some(line) = climate_line(&world) {
            println!("deeptime bench: threads={threads} {line}");
        }
        if let Some(line) = young_line(&world) {
            println!("deeptime bench: threads={threads} {line}");
        }
        if let Some(line) = amplify_line(&world) {
            println!("deeptime bench: threads={threads} {line}");
        }
        println!(
            "deeptime bench: threads={threads} timings {}",
            timings_line(&s.stage_timings()?, &world)
        );
        println!(
            "deeptime bench: threads={threads} total_ms={total_ms:.1} peak_bytes={peak} stages={}",
            stages
                .iter()
                .map(|s| format!(
                    "{}:{:.1}",
                    s["name"].as_str().unwrap_or("?"),
                    s["millis"].as_f64().unwrap_or(0.0)
                ))
                .collect::<Vec<_>>()
                .join(",")
        );
        runs.push(json!({
            "threads": threads,
            "total_millis": total_ms,
            "peak_bytes": peak,
            "stages": stages,
        }));
    }
    // What the editor shows the player for this preset under the cap: the core's estimate plus what the process
    // holds beyond the stages (`SERVER_OVERHEAD_BYTES`), so the Phase 1 exit can hold the measured peak against it.
    let estimate = match deeptime_core::session::estimate_for(&preset, "", o.mem_cap) {
        Ok(e) => serde_json::from_str::<Value>(&e).ok().map(|mut e| {
            let peak = e["peak_bytes"].as_u64().unwrap_or(0);
            e["shown_peak_bytes"] = json!(peak + crate::watch::editor::SERVER_OVERHEAD_BYTES);
            e
        }),
        Err(err) => {
            eprintln!("deeptime bench: no memory estimate for this preset: {err}");
            None
        }
    };
    let bench = json!({
        "preset": name,
        "preset_file": o.preset,
        "estimate": estimate,
        "version": deeptime_core::VERSION,
        "host": format!(
            "{}-{} ({} processors)",
            std::env::consts::OS,
            std::env::consts::ARCH,
            std::thread::available_parallelism().map_or(1, |n| n.get())
        ),
        "created_utc": rfc3339_utc(SystemTime::now()),
        "estimate": estimate,
        "runs": runs,
    });
    let mut text = serde_json::to_vec_pretty(&bench)?;
    text.push(b'\n');
    if let Some(parent) = o.out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&o.out, text)?;
    println!(
        "deeptime bench: {} runs -> {}",
        o.threads.len(),
        o.out.display()
    );
    if let Some(dir) = &o.viewer {
        deeptime_viewer::write_bench_page(dir, &bench)?;
        println!(
            "deeptime bench: viewer -> {}",
            dir.join("index.html").display()
        );
    }
    if temp {
        let _ = std::fs::remove_dir_all(&work);
    }
    Ok(())
}
