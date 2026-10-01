// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench's runs (P1-20): a queue served one run at a time (the Mac is shared: at most
//! [`QUEUE_MAX`] wait, more are refused), each run in its own folder under `--runs`, kept after it
//! ends so it can be reopened, rewound and compared; the newest `--keep` finished runs stay, older
//! ones are deleted. A run is started only by a request (the command line's `--preset`, or the
//! page's Run button) and afterwards only read or cancelled: the page never changes a running
//! simulation.
//!
//! A run's folder: `run.json` (the request; the meta, progress and final status once it ends),
//! `events.jsonl` (the events exactly as the session wrote them, appended as they come),
//! `world/` (the session's output: a keyframe per step under `world/keyframes/`, the world data once
//! done) and `frames/` (PNGs of frame layers, encoded when a page first asks). A folder whose
//! `run.json` has no final status was cut short by a stopped server: it comes back "interrupted",
//! with the frames and events it had.

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use deeptime_core::preset::Preset;
use deeptime_core::session::{AdvanceStatus, DEFAULT_MEM_CAP, Session, SessionOptions};
use deeptime_format::Ramps;
use deeptime_format::keyframes::{KeyframeIndex, read_keyframe};
use deeptime_format::manifest::rfc3339_utc;
use serde_json::{Value, json};

use super::editor::Catalog;
use super::live::{FrameSize, Live};
use super::{live_cadence, progress_json};
use crate::alloc::{self, AllocGauge};
use crate::run::{
    CliResult, Common, forward_log, plates_line, spill_line, surface_line, timings_line,
};

/// Runs that may wait behind the running one.
pub const QUEUE_MAX: usize = 3;

/// Progress is published at least this often while nothing else changes.
const PROGRESS_EVERY: Duration = Duration::from_millis(250);

const FINAL_STATES: [&str; 4] = ["done", "failed", "cancelled", "interrupted"];

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// One run of the workbench.
pub struct Run {
    pub id: String,
    pub dir: PathBuf,
    created_utc: String,
    label: String,
    /// The preset as the page chose it (canonical, with its timeline): its share code is the run's.
    preset_json: String,
    share_code: String,
    mem_cap: u64,
    pub live: Arc<Live>,
    /// While it runs: frames come from the session (its ring, else its files).
    session: Mutex<Option<Arc<Session>>>,
    cancel: AtomicBool,
    /// After it ran: its keyframe index, read once.
    index: Mutex<Option<Arc<KeyframeIndex>>>,
    events_file: Mutex<Option<BufWriter<File>>>,
    #[cfg(test)]
    pub final_world: Mutex<Option<deeptime_core::world::WorldState>>,
}

impl Run {
    fn world_dir(&self) -> PathBuf {
        self.dir.join("world")
    }

    pub fn frames_dir(&self) -> PathBuf {
        self.dir.join("frames")
    }

    fn state(&self) -> String {
        self.live.summary().3["state"]
            .as_str()
            .unwrap_or("")
            .to_owned()
    }

    pub fn is_finished(&self) -> bool {
        FINAL_STATES.contains(&self.state().as_str())
    }

    /// Layer `layer` of frame `n` as RGBA8: from the session while it runs, else from the frame's
    /// file.
    pub fn frame_rgba(&self, n: u32, layer: u32, size: FrameSize) -> Result<Vec<u8>, String> {
        if let Some(s) = lock(&self.session).clone() {
            let mut rgba = vec![0u8; size.w as usize * size.h as usize * 4];
            s.copy_keyframe(n, layer, &mut rgba)
                .map_err(|e| e.to_string())?;
            return Ok(rgba);
        }
        let index = {
            let mut g = lock(&self.index);
            if g.is_none() {
                *g = Some(Arc::new(
                    KeyframeIndex::read(&self.world_dir()).map_err(|e| e.to_string())?,
                ));
            }
            Arc::clone(g.as_ref().expect("read above"))
        };
        let entry = index
            .frames
            .get(n as usize)
            .ok_or_else(|| format!("no frame {n}"))?;
        let kf = read_keyframe(&self.world_dir(), entry).map_err(|e| e.to_string())?;
        kf.layers
            .into_iter()
            .nth(layer as usize)
            .map(|(_, rgba)| rgba)
            .ok_or_else(|| format!("frame {n} has no layer {layer}"))
    }

    /// `run.json`: the request, and what the page shows of the run.
    fn persist(&self) {
        let (_, _, progress, status) = self.live.summary();
        let v = json!({
            "id": self.id, "created_utc": self.created_utc, "label": self.label,
            "preset_json": self.preset_json, "share_code": self.share_code, "mem_cap": self.mem_cap,
            "meta": self.live.meta(), "progress": progress, "status": status,
        });
        let _ = deeptime_format::fsutil::write_atomic(
            &self.dir.join("run.json"),
            format!("{v:#}\n").as_bytes(),
        );
    }

    /// The runs list's line.
    fn list_json(&self, queue_position: Option<usize>) -> Value {
        let (frames, events, progress, status) = self.live.summary();
        let meta = self.live.meta();
        // The planet seed, for the run card (the preset's top-level `seed`, where it has one).
        let seed = serde_json::from_str::<Value>(&self.preset_json)
            .ok()
            .and_then(|v| v.get("seed").cloned())
            .unwrap_or(Value::Null);
        json!({
            "id": self.id, "label": self.label, "created_utc": self.created_utc,
            "preset": meta["preset"], "start": meta["start"], "length_myr": meta["length_myr"],
            "stages": meta["stages"], "circumference": meta["circumference"], "seed": seed,
            "share_code": self.share_code, "mem_cap": self.mem_cap,
            "state": status["state"], "message": status["message"],
            "seconds": status["seconds"], "frames": frames, "events": events,
            "total_permille": progress["total_permille"], "t_years": progress["t_years"],
            "elapsed_ms": progress["elapsed_ms"], "eta_ms": progress["eta_ms"],
            "queue_position": queue_position,
        })
    }
}

/// The page header of a run of `preset` (known before it starts).
fn meta_of(
    id: &str,
    preset_json: &str,
    label: &str,
    created_utc: &str,
    every_myr: Option<f64>,
    threads: usize,
) -> Value {
    let v: Value = serde_json::from_str(preset_json).unwrap_or(Value::Null);
    let p = Preset::from_json(preset_json).ok();
    let stages: Vec<String> = p
        .as_ref()
        .map(|p| p.stages().iter().map(|s| s.name().to_owned()).collect())
        .unwrap_or_default();
    json!({
        "run": id,
        "title": format!("Deep Time — {label}"),
        "label": label,
        "preset": p.as_ref().map_or("?", |p| p.name()),
        "start": v["timeline"]["start"],
        "length_myr": v["timeline"]["length_myr"],
        "frame_every_myr": every_myr,
        "stages": stages,
        "circumference": p.as_ref().map(Preset::circumference),
        "t_start_years": p.as_ref().map(|p| p.initial_world().t_years),
        "threads": threads,
        "version": deeptime_core::VERSION,
        "created_utc": created_utc,
    })
}

/// A frame message from a keyframe's info (`keyframeInfo`, or the index entry), with each
/// layer's ramp.
fn frame_message(
    ramps: &Ramps,
    n: u32,
    t: f64,
    stage: &str,
    w: u32,
    h: u32,
    layers: &[String],
) -> (Value, FrameSize) {
    (
        json!({
            "n": n, "t": t, "stage": stage, "w": w, "h": h, "layers": layers,
            "ramps": layers.iter().map(|l| ramps.for_layer(l).0).collect::<Vec<_>>(),
        }),
        FrameSize {
            w,
            h,
            layers: layers.len(),
        },
    )
}

struct Sched {
    queue: VecDeque<Arc<Run>>,
    current: Option<Arc<Run>>,
    /// Tests hold the queue to connect before a run starts.
    held: bool,
    closing: bool,
}

/// The runs and the one thread that runs them.
pub struct Workbench {
    runs_dir: PathBuf,
    keep: usize,
    common: Common,
    every_myr: Option<f64>,
    ramps: Ramps,
    pub catalog: Catalog,
    /// Every run, oldest first.
    runs: Mutex<Vec<Arc<Run>>>,
    sched: Mutex<Sched>,
    wake: Condvar,
    runner: Mutex<Option<JoinHandle<()>>>,
}

/// Why a request was refused: an HTTP status and a message for the page.
pub type Refusal = (u16, String);

impl Workbench {
    /// Opens `runs_dir` (reading back the runs kept there, pruning to `keep`) and starts the
    /// runner.
    pub fn open(
        runs_dir: &Path,
        keep: usize,
        common: Common,
        every_myr: Option<f64>,
    ) -> CliResult<Arc<Workbench>> {
        std::fs::create_dir_all(runs_dir)?;
        let ramps = Ramps::builtin();
        let catalog = Catalog::load()?.with_default_cap(common.mem_cap);
        let mut runs = Vec::new();
        let mut dirs: Vec<PathBuf> = std::fs::read_dir(runs_dir)?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("run.json").is_file())
            .collect();
        dirs.sort();
        for dir in dirs {
            match restore(&dir, &ramps) {
                Ok(run) => runs.push(Arc::new(run)),
                Err(e) => eprintln!("deeptime watch: skipping {}: {e}", dir.display()),
            }
        }
        let wb = Arc::new(Workbench {
            runs_dir: runs_dir.to_path_buf(),
            keep: keep.max(1),
            common,
            every_myr,
            ramps,
            catalog,
            runs: Mutex::new(runs),
            sched: Mutex::new(Sched {
                queue: VecDeque::new(),
                current: None,
                held: false,
                closing: false,
            }),
            wake: Condvar::new(),
            runner: Mutex::new(None),
        });
        wb.prune();
        let me = Arc::clone(&wb);
        let handle = std::thread::Builder::new()
            .name("deeptime-watch-runner".into())
            .spawn(move || me.runner())?;
        *lock(&wb.runner) = Some(handle);
        Ok(wb)
    }

    pub fn run(&self, id: &str) -> Option<Arc<Run>> {
        lock(&self.runs).iter().find(|r| r.id == id).cloned()
    }

    /// Queues a run of `preset_json` (a complete preset, as the editor builds it) with a memory cap
    /// of `mem_cap` bytes (0 = the workbench's default). A run whose estimated peak is over its cap
    /// is refused here, whatever the page said: it would fail on the cap partway through.
    pub fn submit(
        &self,
        preset_json: &str,
        label: Option<&str>,
        mem_cap: u64,
    ) -> Result<Arc<Run>, Refusal> {
        let canon = self.catalog.runnable(preset_json).map_err(|e| (400, e))?;
        let mem_cap = if mem_cap == 0 {
            self.catalog.default_cap()
        } else {
            mem_cap
        };
        self.catalog
            .within_cap(&canon, mem_cap)
            .map_err(|e| (400, e))?;
        let mut sched = lock(&self.sched);
        if sched.closing {
            return Err((503, "the workbench is stopping".into()));
        }
        if sched.queue.len() >= QUEUE_MAX {
            return Err((
                409,
                format!(
                    "{QUEUE_MAX} runs are already waiting behind the running one (the Mac runs one \
                     at a time); cancel one or wait"
                ),
            ));
        }
        let run = self
            .create(&canon, label, mem_cap)
            .map_err(|e| (500, e.to_string()))?;
        lock(&self.runs).push(Arc::clone(&run));
        sched.queue.push_back(Arc::clone(&run));
        drop(sched);
        self.wake.notify_all();
        Ok(run)
    }

    fn create(&self, preset_json: &str, label: Option<&str>, mem_cap: u64) -> CliResult<Arc<Run>> {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_millis());
        let mut id = format!("{millis:011x}");
        let mut bump = millis;
        while self.runs_dir.join(&id).exists() {
            bump += 1;
            id = format!("{bump:011x}");
        }
        let v: Value = serde_json::from_str(preset_json)?;
        let label = label.map(str::trim).filter(|l| !l.is_empty()).map_or_else(
            || {
                let mut l = v["name"].as_str().unwrap_or("preset").to_owned();
                if let Some(s) = v["timeline"]["start"].as_str() {
                    l.push_str(&format!(" · {s}"));
                }
                if let Some(m) = v["timeline"]["length_myr"].as_f64() {
                    l.push_str(&format!(" · {m} Myr"));
                }
                l
            },
            str::to_owned,
        );
        let created_utc = rfc3339_utc(SystemTime::now());
        let meta = meta_of(
            &id,
            preset_json,
            &label,
            &created_utc,
            self.every_myr,
            self.common.threads,
        );
        let dir = self.runs_dir.join(&id);
        std::fs::create_dir_all(&dir)?;
        let live = Arc::new(Live::new(id.clone(), meta));
        live.set_status(json!({ "state": "queued" }));
        let run = Arc::new(Run {
            id,
            dir,
            created_utc,
            label,
            preset_json: preset_json.to_owned(),
            share_code: deeptime_core::share_code::encode(preset_json)?,
            mem_cap,
            live,
            session: Mutex::new(None),
            cancel: AtomicBool::new(false),
            index: Mutex::new(None),
            events_file: Mutex::new(None),
            #[cfg(test)]
            final_world: Mutex::new(None),
        });
        run.persist();
        Ok(run)
    }

    /// Cancels a queued run (it never starts) or the running one (it stops after its current
    /// step and keeps its frames).
    pub fn cancel(&self, id: &str) -> Result<(), Refusal> {
        let run = self.run(id).ok_or((404, format!("no run {id}")))?;
        let mut sched = lock(&self.sched);
        if let Some(pos) = sched.queue.iter().position(|r| r.id == id) {
            sched.queue.remove(pos);
            drop(sched);
            run.live.set_status(
                json!({ "state": "cancelled", "message": "cancelled before it started" }),
            );
            run.persist();
            self.wake.notify_all();
            return Ok(());
        }
        if sched.current.as_ref().is_some_and(|r| r.id == id) {
            run.cancel.store(true, Ordering::Release);
            if let Some(s) = lock(&run.session).as_ref() {
                s.cancel();
            }
            return Ok(());
        }
        Err((409, format!("run {id} is not running or waiting")))
    }

    /// `GET /api/runs`: every run (newest first), the running one and the queue.
    pub fn list_json(&self) -> Value {
        let sched = lock(&self.sched);
        let position = |id: &str| sched.queue.iter().position(|r| r.id == id).map(|p| p + 1);
        let runs: Vec<Value> = lock(&self.runs)
            .iter()
            .rev()
            .map(|r| r.list_json(position(&r.id)))
            .collect();
        json!({
            "runs": runs,
            "running": sched.current.as_ref().map(|r| r.id.clone()),
            "queue": sched.queue.iter().map(|r| r.id.clone()).collect::<Vec<_>>(),
            "queue_max": QUEUE_MAX,
            "keep": self.keep,
        })
    }

    /// Waits until nothing runs or waits.
    pub fn wait_idle(&self) {
        let mut sched = lock(&self.sched);
        while !sched.closing && (sched.current.is_some() || !sched.queue.is_empty()) {
            sched = self.wake.wait(sched).unwrap_or_else(|e| e.into_inner());
        }
    }

    #[cfg(test)]
    pub fn hold(&self, held: bool) {
        lock(&self.sched).held = held;
        self.wake.notify_all();
    }

    /// Stops the runner (a running run is cancelled) and ends every page's streams.
    pub fn close(&self) {
        let current = {
            let mut sched = lock(&self.sched);
            sched.closing = true;
            sched.current.clone()
        };
        if let Some(run) = current {
            run.cancel.store(true, Ordering::Release);
            if let Some(s) = lock(&run.session).as_ref() {
                s.cancel();
            }
        }
        self.wake.notify_all();
        if let Some(h) = lock(&self.runner).take() {
            let _ = h.join();
        }
        for r in lock(&self.runs).iter() {
            r.live.close();
        }
    }

    fn runner(&self) {
        loop {
            let run = {
                let mut sched = lock(&self.sched);
                loop {
                    if sched.closing {
                        return;
                    }
                    if !sched.held
                        && let Some(r) = sched.queue.pop_front()
                    {
                        sched.current = Some(Arc::clone(&r));
                        break r;
                    }
                    sched = self.wake.wait(sched).unwrap_or_else(|e| e.into_inner());
                }
            };
            self.execute(&run);
            lock(&self.sched).current = None;
            self.prune();
            self.wake.notify_all();
        }
    }

    /// Runs `run` to its end and records how it ended.
    fn execute(&self, run: &Arc<Run>) {
        alloc::reset_peak();
        let started = Instant::now();
        run.live.set_status(json!({ "state": "running" }));
        run.persist();
        let result = self.drive(run);
        // The world is freed now; the frames come from their files from here on.
        *lock(&run.session) = None;
        if let Some(mut f) = lock(&run.events_file).take() {
            let _ = f.flush();
        }
        let seconds = started.elapsed().as_secs_f64();
        let status = match result {
            Ok(lines) => json!({ "state": "done", "lines": lines, "seconds": seconds,
                                 "world": run.world_dir().display().to_string() }),
            Err(_) if run.cancel.load(Ordering::Acquire) => {
                json!({ "state": "cancelled", "message": "cancelled from the page", "seconds": seconds })
            }
            Err(e) => json!({ "state": "failed", "message": e.to_string(), "seconds": seconds }),
        };
        match status["state"].as_str() {
            Some("done") => {
                for l in status["lines"].as_array().into_iter().flatten() {
                    println!("deeptime watch: {} {}", run.id, l.as_str().unwrap_or(""));
                }
            }
            s => eprintln!(
                "deeptime watch: {} {}: {}",
                run.id,
                s.unwrap_or("?"),
                status["message"]
            ),
        }
        run.live.set_status(status);
        run.persist();
    }

    fn drive(&self, run: &Arc<Run>) -> CliResult<Vec<String>> {
        let started = Instant::now();
        let preset = live_cadence(&run.preset_json, self.every_myr)?;
        let s = Arc::new(Session::create(
            &preset,
            "",
            SessionOptions {
                out_dir: run.world_dir(),
                threads: self.common.threads,
                mem_cap: run.mem_cap,
            },
            Arc::new(AllocGauge),
        )?);
        *lock(&run.session) = Some(Arc::clone(&s));
        *lock(&run.events_file) = Some(BufWriter::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(run.dir.join("events.jsonl"))?,
        ));
        let mut published = Published::default();
        // The frames the last advance draws are held back until the bake has written the water:
        // `finalize_write` draws the last frame again from the bake's rivers and lakes
        // (`water_view`), and a page that had fetched its image would keep the first drawing
        // (a frame's image is cached for good).
        let mut held_from = u32::MAX;
        loop {
            if run.cancel.load(Ordering::Acquire) {
                s.cancel();
            }
            let before = s.keyframe_count();
            // A zero budget runs exactly one step.
            let status = s.advance(Duration::ZERO);
            forward_log(&s, self.common.verbose);
            if matches!(status, Ok(AdvanceStatus::Done)) {
                held_from = before;
            }
            self.publish(run, &s, &mut published, false, held_from)?;
            match status? {
                AdvanceStatus::Done => break,
                AdvanceStatus::Cancelled => return Err("cancelled".into()),
                AdvanceStatus::Running | AdvanceStatus::StageDone => {}
            }
        }
        self.publish(run, &s, &mut published, true, held_from)?;
        run.live.set_status(json!({ "state": "finalizing" }));
        s.finalize_write()?;
        self.publish(run, &s, &mut published, true, u32::MAX)?;
        forward_log(&s, self.common.verbose);
        let p = s.progress();
        let mut lines = vec![format!(
            "{} ({} stages, {} events, {} frames, peak {:.1} MiB) in {:.2}s",
            s.preset().name(),
            p.stage_count,
            p.events_total,
            p.keyframes_ready,
            alloc::peak() as f64 / 1048576.0,
            started.elapsed().as_secs_f64()
        )];
        // The world as it stands, not a copy (a 32k full world is gigabytes; P1-17).
        let timings = s.stage_timings()?;
        s.with_world(|world| {
            lines.extend(plates_line(world));
            lines.extend(surface_line(world));
            lines.push(format!("timings {}", timings_line(&timings, world)));
            #[cfg(test)]
            {
                *lock(&run.final_world) = Some(world.clone());
            }
        })?;
        lines.extend(spill_line(&s)?);
        run.live
            .publish(Vec::new(), Vec::new(), Some(progress_json(&s)));
        Ok(lines)
    }

    /// New frames and events since the last call, and the progress (at most every
    /// [`PROGRESS_EVERY`] unless something else changed or `force`). Frames from number
    /// `held_from` on are not announced yet (`u32::MAX`: none are held).
    fn publish(
        &self,
        run: &Run,
        s: &Session,
        p: &mut Published,
        force: bool,
        held_from: u32,
    ) -> CliResult<()> {
        let mut frames = Vec::new();
        let count = s.keyframe_count().min(held_from);
        while p.frames < count {
            let info: Value = serde_json::from_str(&s.keyframe_info(p.frames)?)?;
            let layers: Vec<String> = info["layers"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|v| v.as_str().unwrap_or("").to_owned())
                        .collect()
                })
                .unwrap_or_default();
            frames.push(frame_message(
                &self.ramps,
                p.frames,
                info["t"].as_f64().unwrap_or(0.0),
                info["stage"].as_str().unwrap_or(""),
                info["w"].as_u64().unwrap_or(0) as u32,
                info["h"].as_u64().unwrap_or(0) as u32,
                &layers,
            ));
            p.frames += 1;
        }
        // One event per call keeps each event's JSON text exactly as the session wrote it.
        let mut events = Vec::new();
        loop {
            let one = s.events_since(p.events, 1);
            let text = one
                .strip_prefix('[')
                .and_then(|t| t.strip_suffix(']'))
                .ok_or_else(|| format!("events_since returned {one}"))?;
            if text.is_empty() {
                break;
            }
            events.push(text.to_owned());
            p.events += 1;
        }
        if !events.is_empty()
            && let Some(f) = lock(&run.events_file).as_mut()
        {
            for e in &events {
                writeln!(f, "{e}")?;
            }
            f.flush()?;
        }
        let due = force
            || !frames.is_empty()
            || !events.is_empty()
            || p.progress_at.is_none_or(|t| t.elapsed() >= PROGRESS_EVERY);
        let progress = due.then(|| {
            p.progress_at = Some(Instant::now());
            progress_json(s)
        });
        run.live.publish(frames, events, progress);
        Ok(())
    }

    /// Deletes the oldest finished runs beyond `keep` (never a waiting or running one).
    fn prune(&self) {
        let mut runs = lock(&self.runs);
        let finished: Vec<Arc<Run>> = runs.iter().filter(|r| r.is_finished()).cloned().collect();
        if finished.len() <= self.keep {
            return;
        }
        for r in &finished[..finished.len() - self.keep] {
            r.live.close();
            if let Err(e) = std::fs::remove_dir_all(&r.dir) {
                eprintln!("deeptime watch: could not prune {}: {e}", r.dir.display());
            }
            runs.retain(|x| x.id != r.id);
        }
    }
}

/// Cursors of what the driving thread has published.
#[derive(Default)]
struct Published {
    frames: u32,
    events: u64,
    progress_at: Option<Instant>,
}

/// A run read back from its folder.
fn restore(dir: &Path, ramps: &Ramps) -> CliResult<Run> {
    let v: Value = serde_json::from_slice(&std::fs::read(dir.join("run.json"))?)?;
    let id = v["id"].as_str().ok_or("run.json has no id")?.to_owned();
    let mut status = v["status"].clone();
    if !status["state"]
        .as_str()
        .is_some_and(|s| FINAL_STATES.contains(&s))
    {
        status =
            json!({ "state": "interrupted", "message": "the workbench stopped during this run" });
    }
    let world = dir.join("world");
    let frames = match KeyframeIndex::read(&world) {
        Ok(index) => index
            .frames
            .iter()
            .map(|e| frame_message(ramps, e.n, e.t, &e.stage, e.w, e.h, &e.layers))
            .collect(),
        Err(_) => Vec::new(),
    };
    let events = match File::open(dir.join("events.jsonl")) {
        Ok(f) => BufReader::new(f)
            .lines()
            .map_while(Result::ok)
            .filter(|l| !l.trim().is_empty())
            .collect(),
        Err(_) => Vec::new(),
    };
    let preset_json = v["preset_json"].as_str().unwrap_or("").to_owned();
    let label = v["label"].as_str().unwrap_or(&id).to_owned();
    let created_utc = v["created_utc"].as_str().unwrap_or("").to_owned();
    let meta = if v["meta"].is_object() {
        v["meta"].clone()
    } else {
        meta_of(&id, &preset_json, &label, &created_utc, None, 0)
    };
    let live = Arc::new(Live::restored(
        id.clone(),
        meta,
        frames,
        events,
        v["progress"].clone(),
        status,
    ));
    let run = Run {
        id,
        dir: dir.to_path_buf(),
        created_utc,
        label,
        share_code: v["share_code"].as_str().unwrap_or("").to_owned(),
        preset_json,
        // 0 (a run of before the workbench had a default) ran with the core's default cap.
        mem_cap: v["mem_cap"]
            .as_u64()
            .filter(|&c| c != 0)
            .unwrap_or(DEFAULT_MEM_CAP),
        live,
        session: Mutex::new(None),
        cancel: AtomicBool::new(false),
        index: Mutex::new(None),
        events_file: Mutex::new(None),
        #[cfg(test)]
        final_world: Mutex::new(None),
    };
    if v["status"] != run.live.summary().3 {
        run.persist();
    }
    Ok(run)
}
