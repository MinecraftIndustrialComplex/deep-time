// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime run`, `deeptime stage` and `deeptime bench` (plan §3.3, §9.3): the CLI drives the same
//! `Session` the JNI wrapper drives, so `run` writes exactly the folder the game would.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

use deeptime_core::checkpoint::Checkpoint;
use deeptime_core::session::{AdvanceStatus, Session, SessionOptions};
use deeptime_format::keyframes::{KeyframeIndex, read_keyframe};
use deeptime_format::manifest::rfc3339_utc;
use serde_json::{Value, json};

use crate::alloc::{self, AllocGauge};

pub type CliResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Budget of one `advance` call; the CLI only needs it to print progress between calls.
const ADVANCE_BUDGET: Duration = Duration::from_millis(250);

/// Options shared by the subcommands that run a session.
pub struct Common {
    pub threads: usize,
    pub mem_cap: u64,
    pub verbose: bool,
}

fn read_text(path: &Path, what: &str) -> CliResult<String> {
    std::fs::read_to_string(path).map_err(|e| format!("{what} {}: {e}", path.display()).into())
}

fn session_options(out: &Path, c: &Common) -> SessionOptions {
    SessionOptions {
        out_dir: out.to_path_buf(),
        threads: c.threads,
        mem_cap: c.mem_cap,
    }
}

/// Prints the session's log lines to stderr (DEBUG only when verbose).
fn forward_log(s: &Session, verbose: bool) {
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
        "deposits: province={} ({} provinces) count_mult={} — expected(§8.6) = Σ_tracts ρ_p50/1e5 · A_km2 · mean_F · count_mult",
        census["province"].as_str().unwrap_or("?"),
        census["provinces"],
        census["count_mult"],
    );
    for t in census["types"].as_array().into_iter().flatten() {
        let f = |k: &str| t[k].as_f64().unwrap_or(0.0);
        println!(
            "deposits {:<30} sites={:>3} expected={:>7.2} with_provinces={:>7.2} drawn={:>3} tracts={:>3} permissive_km2={:>9.0} mean_F={:.3} buried={} truncated={} eroded={} too_deep={} children={} guaranteed={} dropped={}",
            t["id"].as_str().unwrap_or("?"),
            f("sites"),
            f("expected_formula"),
            f("expected"),
            f("drawn"),
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
        );
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
fn plates_line(world: &deeptime_core::world::WorldState) -> Option<String> {
    world
        .plates
        .as_ref()
        .map(|st| deeptime_core::plates::stats::summary(st).line())
}

pub struct RunOptions {
    /// A built-in name, a file or a share code.
    pub preset: String,
    /// `timeline.start`, as `config/deeptime/server.json` sets it on a server.
    pub start: Option<String>,
    /// `timeline.length_myr`.
    pub length_myr: Option<f64>,
    pub content: Option<PathBuf>,
    pub out: PathBuf,
    pub viewer: Option<PathBuf>,
    pub checkpoints: bool,
    pub common: Common,
}

/// `deeptime run`: a whole planet into `--out`.
pub fn run(o: &RunOptions) -> CliResult<()> {
    let started = Instant::now();
    let preset = deeptime_core::preset::with_timeline(
        &load_preset(&o.preset)?,
        o.start.as_deref(),
        o.length_myr,
    )?;
    let content = match &o.content {
        Some(p) => read_text(p, "content bundle")?,
        None => String::new(),
    };
    let mut s = Session::create(
        &preset,
        &content,
        session_options(&o.out, &o.common),
        Arc::new(AllocGauge),
    )?;
    if o.checkpoints {
        s.set_checkpoint_dir(Some(o.out.join("checkpoints")));
    }
    drive(&s, o.common.verbose, |_, st| st == AdvanceStatus::Done)?;
    s.finalize_write()?;
    forward_log(&s, o.common.verbose);
    if let Some(c) = deeptime_format::Manifest::read(&o.out)?
        .notes
        .get(deeptime_core::deposits::STAGE_NAME)
    {
        print_census(c);
    }
    let p = s.progress();
    println!(
        "deeptime run: {} -> {} ({} stages, {} events, {} keyframes, peak {:.1} MiB) in {:.2}s",
        s.preset().name(),
        o.out.display(),
        p.stage_count,
        p.events_total,
        p.keyframes_ready,
        alloc::peak() as f64 / 1048576.0,
        started.elapsed().as_secs_f64()
    );
    if let Some(line) = plates_line(&s.checkpoint()?.world) {
        println!("deeptime run: {line}");
    }
    if let Some(dir) = &o.viewer {
        let title = format!("Deep Time — run {} ({})", s.preset().name(), o.preset);
        deeptime_viewer::write_world_bundle(dir, &o.out, &title)?;
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
    pub work: Option<PathBuf>,
    pub common: Common,
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
        None => std::env::temp_dir().join(format!("deeptime-stage-{}", std::process::id())),
    });
    let opts = session_options(&work, &o.common);
    let s = match (&o.input, &o.preset) {
        (Some(input), None) => {
            let ck = Checkpoint::read(input)?;
            match ck.next_stage_name() {
                Some(next) if next == o.name => {}
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
            Session::resume(ck, opts, Arc::new(AllocGauge))?
        }
        (None, Some(preset)) => {
            let json = load_preset_with(preset, o.start.as_deref(), o.length_myr)?;
            let s = Session::create(&json, "", opts, Arc::new(AllocGauge))?;
            let target = s
                .stage_names()
                .iter()
                .position(|n| *n == o.name)
                .ok_or_else(|| {
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
        let keyframes = index
            .frames
            .iter()
            .map(|e| read_keyframe(&work, e))
            .collect::<Result<Vec<_>, _>>()?;
        let info = deeptime_viewer::BundleInfo {
            title: format!("Deep Time — stage {}", o.name),
            source: format!("stage:{}", o.name),
            circumference: ck.world.circumference,
            sea_level_y: 63,
            format_version: deeptime_format::FORMAT_VERSION.to_owned(),
            created_utc: rfc3339_utc(SystemTime::now()),
        };
        deeptime_viewer::write_bundle_with(
            dir,
            &info,
            &[],
            None,
            &deeptime_viewer::Timeline {
                keyframes,
                events: ck.world.events.clone(),
                sites: ck.world.sites.clone(),
            },
        )?;
        if let Some(report) = deeptime_viewer::deposit_report(census.as_ref(), &ck.world.bodies) {
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
}

/// `deeptime bench`: the preset once per thread count, with per-stage wall time and peak native
/// memory, into `bench.json` and a timing page (the Phase 1 budget gate's skeleton, P1-17).
pub fn bench(o: &BenchOptions) -> CliResult<()> {
    let preset = load_preset_with(&o.preset, o.start.as_deref(), o.length_myr)?;
    let temp = o.work.is_none();
    let work = o.work.clone().unwrap_or_else(|| {
        std::env::temp_dir().join(format!("deeptime-bench-{}", std::process::id()))
    });
    let mut runs = Vec::new();
    let mut name = String::new();
    for &threads in &o.threads {
        let out = work.join(format!("t{threads}"));
        let _ = std::fs::remove_dir_all(&out);
        alloc::reset_peak();
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
        name = s.preset().name().to_owned();
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
            .map(|t| json!({ "name": t.name, "steps": t.steps, "millis": t.millis }))
            .collect();
        stages.push(json!({ "name": "finalize", "steps": 1, "millis": finalize_ms }));
        if let Some(line) = plates_line(&s.checkpoint()?.world) {
            println!("deeptime bench: threads={threads} {line}");
        }
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
    let bench = json!({
        "preset": name,
        "preset_file": o.preset,
        "version": deeptime_core::VERSION,
        "host": format!(
            "{}-{} ({} processors)",
            std::env::consts::OS,
            std::env::consts::ARCH,
            std::thread::available_parallelism().map_or(1, |n| n.get())
        ),
        "created_utc": rfc3339_utc(SystemTime::now()),
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
