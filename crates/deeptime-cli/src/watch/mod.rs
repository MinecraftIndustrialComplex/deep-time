// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime watch` (decisions log 2026-09-28, "How should Ben watch a simulation while it
//! runs?"): runs a session like `deeptime run` and serves a live page of it while it runs — map,
//! globe, layers and events as each step lands, with pause, step, rewind, speed and "follow
//! live". Runs are started by this command, never from the browser.
//!
//! How it works: the preset's keyframe cadence is set to one frame per step (`--every-myr`
//! chooses another), so the session renders every step through the same keyframe path the
//! in-game timelapse uses; the driving thread advances one step at a time and publishes new
//! frames (`keyframe_info`), events (`events_since`) and progress into [`live::Live`]; the server
//! ([`http`]) streams them to every page and encodes a frame's layer as PNG when a page first asks
//! for it (`copy_keyframe`: the ring, else the frame's file on disk, so every step stays
//! reachable while the run goes on). The layers are whatever the frames name, so a new stage's
//! layers appear without a change here.
//!
//! Watching changes nothing in the run: keyframes are drawn from the world between steps. The
//! test `watch_streams_every_step_and_matches_the_batch_run` holds it to that (the watched world
//! equals the batch run's, and its last frame is the batch run's last keyframe pixel for pixel).

mod http;
mod live;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use deeptime_core::session::{AdvanceStatus, Session, SessionState};
use deeptime_format::Ramps;
use deeptime_format::manifest::rfc3339_utc;
use serde_json::{Value, json};

use crate::alloc::{self, AllocGauge};
use crate::run::{
    CliResult, Common, forward_log, load_preset_with, plates_line, read_text, session_options,
    surface_line, timings_line,
};

pub use http::tailnet_or_loopback;
use live::{FrameSize, Live};

/// The watch server's default port (tools/viewer/mac-watch.sh uses it too).
pub const DEFAULT_PORT: u16 = 8642;

/// "A frame after every step": a keyframe cadence of one year, shorter than any step.
const EVERY_STEP_MYR: f64 = 1.0e-6;

/// Progress is published at least this often while nothing else changes.
const PROGRESS_EVERY: Duration = Duration::from_millis(250);

pub struct WatchOptions {
    /// A built-in name, a file or a share code.
    pub preset: String,
    pub start: Option<String>,
    pub length_myr: Option<f64>,
    pub content: Option<PathBuf>,
    /// Loopback or a Tailscale address (`tailnet_or_loopback`).
    pub bind: IpAddr,
    /// 0 = any free port (tests).
    pub port: u16,
    /// World data in `<work>/world`, frame images in `<work>/frames` (default: a temporary
    /// folder).
    pub work: Option<PathBuf>,
    /// A frame every this many Myr instead of every step.
    pub every_myr: Option<f64>,
    /// Also write the static viewer bundle here when the run is done.
    pub viewer: Option<PathBuf>,
    /// Stop serving once the run is done (default: serve until killed).
    pub exit_when_done: bool,
    pub common: Common,
}

/// The preset with the live cadence: a keyframe after every step, or every `every_myr`.
/// Keyframes only read the world, so this changes nothing but the timelapse's density (a dummy
/// preset with a timeline keeps its eight intervals: the timeline sets them).
pub fn live_cadence(json: &str, every_myr: Option<f64>) -> CliResult<String> {
    let every_myr = every_myr.unwrap_or(EVERY_STEP_MYR);
    let mut v: Value = serde_json::from_str(json)?;
    let obj = v.as_object_mut().ok_or("the preset is not a JSON object")?;
    if obj.contains_key("planet") {
        let kf = obj.entry("keyframes").or_insert_with(|| json!({}));
        let kf = kf
            .as_object_mut()
            .ok_or("$.keyframes: expected a JSON object")?;
        kf.insert("every_myr".into(), json!(every_myr));
    } else if let Some(w) = obj
        .get_mut("dummy")
        .and_then(|d| d.get_mut("world"))
        .and_then(Value::as_object_mut)
    {
        w.insert("keyframe_every_years".into(), json!(every_myr * 1e6));
    } else if let Some(d) = obj
        .get_mut("deposits_synthetic")
        .and_then(Value::as_object_mut)
    {
        d.insert("keyframe_every_years".into(), json!(every_myr * 1e6));
    }
    Ok(v.to_string())
}

fn state_name(s: SessionState) -> &'static str {
    match s {
        SessionState::Running => "running",
        SessionState::Done => "done",
        SessionState::Cancelled => "cancelled",
        SessionState::Failed => "failed",
        SessionState::Finalized => "finalized",
    }
}

fn progress_json(s: &Session) -> Value {
    let p = s.progress();
    json!({
        "state": state_name(p.state),
        "stage_index": p.stage_index,
        "stage_count": p.stage_count,
        "stage": s.stage_names().get(p.stage_index as usize),
        "stage_permille": p.stage_permille,
        "total_permille": p.total_permille,
        "t_years": -(p.sim_time as f64),
        "elapsed_ms": p.elapsed_ms,
        "eta_ms": p.eta_ms,
        "bytes_in_use": p.bytes_in_use,
        "bytes_peak": p.bytes_peak,
        "keyframes": p.keyframes_ready,
        "events": p.events_total,
    })
}

/// Cursors of what the driving thread has published.
#[derive(Default)]
struct Published {
    frames: u32,
    events: u64,
    progress_at: Option<Instant>,
}

/// A running watch: the session, what the page sees of it, and the server.
pub struct Watch {
    session: Arc<Session>,
    live: Arc<Live>,
    server: http::Server,
    ramps: Ramps,
    work: PathBuf,
    viewer: Option<PathBuf>,
    title: String,
    verbose: bool,
}

impl Watch {
    /// Creates the session and starts serving (the run starts with [`Watch::run`]).
    pub fn start(o: &WatchOptions) -> CliResult<Watch> {
        if !tailnet_or_loopback(o.bind) {
            return Err(format!(
                "--bind {}: only loopback or a Tailscale address (100.64.0.0/10, \
                 fd7a:115c:a1e0::/48); the page is never served on the LAN",
                o.bind
            )
            .into());
        }
        if let Some(m) = o.every_myr
            && !(m.is_finite() && m > 0.0)
        {
            return Err(format!("--every-myr {m}: must be > 0").into());
        }
        let preset = live_cadence(
            &load_preset_with(&o.preset, o.start.as_deref(), o.length_myr)?,
            o.every_myr,
        )?;
        let content = match &o.content {
            Some(p) => read_text(p, "content bundle")?,
            None => String::new(),
        };
        let work = o.work.clone().unwrap_or_else(|| {
            std::env::temp_dir().join(format!("deeptime-watch-{}", std::process::id()))
        });
        let world = work.join("world");
        let frames = work.join("frames");
        // A new run: no image of an earlier run in this folder may be served for this one.
        if let Err(e) = std::fs::remove_dir_all(&frames)
            && e.kind() != std::io::ErrorKind::NotFound
        {
            return Err(e.into());
        }
        std::fs::create_dir_all(&frames)?;
        let session = Arc::new(Session::create(
            &preset,
            &content,
            session_options(&world, &o.common),
            Arc::new(AllocGauge),
        )?);
        let p = session.preset();
        let mut title = format!("Deep Time — live: {}", p.name());
        if let Some(s) = &o.start {
            title.push_str(&format!(", {s}"));
        }
        if let Some(l) = o.length_myr {
            title.push_str(&format!(", {l} Myr"));
        }
        let run = format!(
            "{:x}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_millis())
        );
        let meta = json!({
            "title": title,
            "preset": p.name(),
            "preset_arg": o.preset,
            "start": o.start,
            "length_myr": o.length_myr,
            "frame_every_myr": o.every_myr,
            "stages": session.stage_names(),
            "circumference": p.circumference(),
            "t_start_years": p.initial_world().t_years,
            "threads": session.threads(),
            "version": deeptime_core::VERSION,
            "started_utc": rfc3339_utc(SystemTime::now()),
        });
        let live = Arc::new(Live::new(run, meta));
        let app = Arc::new(http::App {
            live: Arc::clone(&live),
            session: Arc::clone(&session),
            frames_dir: frames,
            page: deeptime_viewer::live_page(&title),
        });
        let server = http::Server::start(SocketAddr::new(o.bind, o.port), app)
            .map_err(|e| format!("cannot serve on {}:{}: {e}", o.bind, o.port))?;
        Ok(Watch {
            session,
            live,
            server,
            ramps: Ramps::builtin(),
            work,
            viewer: o.viewer.clone(),
            title,
            verbose: o.common.verbose,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.server.addr()
    }

    /// Runs the session to the end, publishing after every step, then writes the world data (and
    /// the static viewer if asked). A failure is shown on the page too.
    pub fn run(&self) -> CliResult<()> {
        let r = self.drive();
        if let Err(e) = &r {
            self.live
                .set_status(json!({ "state": "failed", "message": e.to_string() }));
        }
        r
    }

    fn drive(&self) -> CliResult<()> {
        let s = &*self.session;
        let started = Instant::now();
        let mut published = Published::default();
        loop {
            // A zero budget runs exactly one step.
            let status = s.advance(Duration::ZERO);
            forward_log(s, self.verbose);
            self.publish(&mut published, false)?;
            match status? {
                AdvanceStatus::Done => break,
                AdvanceStatus::Cancelled => return Err("the session was cancelled".into()),
                AdvanceStatus::Running | AdvanceStatus::StageDone => {}
            }
        }
        self.publish(&mut published, true)?;
        self.live.set_status(json!({ "state": "finalizing" }));
        let world_dir = s.out_dir().to_path_buf();
        s.finalize_write()?;
        forward_log(s, self.verbose);
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
        let world = s.checkpoint()?.world;
        lines.extend(plates_line(&world));
        lines.extend(surface_line(&world));
        lines.push(format!(
            "timings {}",
            timings_line(&s.stage_timings()?, &world)
        ));
        drop(world);
        if let Some(dir) = &self.viewer {
            deeptime_viewer::write_world_bundle(dir, &world_dir, &self.title)?;
            lines.push(format!("viewer -> {}", dir.join("index.html").display()));
        }
        for l in &lines {
            println!("deeptime watch: {l}");
        }
        self.live
            .publish(Vec::new(), Vec::new(), Some(progress_json(s)));
        self.live.set_status(json!({
            "state": "done",
            "lines": lines,
            "world": world_dir.display().to_string(),
            "seconds": started.elapsed().as_secs_f64(),
        }));
        Ok(())
    }

    /// New frames and events since the last call, and the progress (at most every
    /// [`PROGRESS_EVERY`] unless something else changed or `force`).
    fn publish(&self, p: &mut Published, force: bool) -> CliResult<()> {
        let s = &*self.session;
        let mut frames = Vec::new();
        let count = s.keyframe_count();
        while p.frames < count {
            let mut info: Value = serde_json::from_str(&s.keyframe_info(p.frames)?)?;
            let names: Vec<String> = info["layers"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .map(|v| v.as_str().unwrap_or("").to_owned())
                        .collect()
                })
                .unwrap_or_default();
            let size = FrameSize {
                w: info["w"].as_u64().unwrap_or(0) as u32,
                h: info["h"].as_u64().unwrap_or(0) as u32,
                layers: names.len(),
            };
            info["ramps"] = json!(
                names
                    .iter()
                    .map(|n| self.ramps.for_layer(n).0)
                    .collect::<Vec<_>>()
            );
            frames.push((info, size));
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
        let due = force
            || !frames.is_empty()
            || !events.is_empty()
            || p.progress_at.is_none_or(|t| t.elapsed() >= PROGRESS_EVERY);
        let progress = due.then(|| {
            p.progress_at = Some(Instant::now());
            progress_json(s)
        });
        self.live.publish(frames, events, progress);
        Ok(())
    }

    /// Ends the streams and stops serving.
    pub fn stop(mut self) {
        self.live.close();
        self.server.stop();
    }

    /// Serves until the process is killed.
    pub fn serve(self) {
        self.server.join();
    }
}

/// `deeptime watch`.
pub fn watch(o: &WatchOptions) -> CliResult<()> {
    let w = Watch::start(o)?;
    println!(
        "deeptime watch: serving http://{}/ (run {}; world data and frame images in {})",
        w.addr(),
        w.live.run(),
        w.work.display()
    );
    let r = w.run();
    match &r {
        Ok(()) => println!(
            "deeptime watch: run finished; {} (pid {})",
            if o.exit_when_done {
                "stopping".to_owned()
            } else {
                "still serving until stopped".to_owned()
            },
            std::process::id()
        ),
        Err(e) => eprintln!("deeptime watch: the run failed: {e}; the page keeps what it made"),
    }
    if o.exit_when_done {
        w.stop();
    } else {
        w.serve();
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::{Ipv4Addr, TcpStream};
    use std::path::Path;

    use deeptime_core::checkpoint::Checkpoint;
    use deeptime_core::session::{NoGauge, SessionOptions};
    use deeptime_core::world::WorldState;
    use deeptime_format::keyframes::{KeyframeIndex, read_keyframe};

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("deeptime-watch-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn quiet() -> Common {
        Common {
            threads: 2,
            mem_cap: 0,
            verbose: false,
        }
    }

    fn options(preset: &Path, work: &Path, bind: IpAddr) -> WatchOptions {
        WatchOptions {
            preset: preset.to_string_lossy().into_owned(),
            start: None,
            length_myr: None,
            content: None,
            bind,
            port: 0,
            work: Some(work.to_path_buf()),
            every_myr: None,
            viewer: None,
            exit_when_done: true,
            common: quiet(),
        }
    }

    /// The coupled plates + surface pipeline at test size: 20 Myr, 10 steps, keyframes every
    /// 10 Myr (the batch run's cadence).
    fn small_pipeline(dir: &Path) -> PathBuf {
        let json = load_preset_with(
            "pipeline_quick_16k",
            Some("scattered_continents"),
            Some(20.0),
        )
        .unwrap();
        let mut v: Value = serde_json::from_str(&json).unwrap();
        v["resolution"]["l0_samples"] = json!(3000);
        v["resolution"]["l1_r"] = json!(64);
        v["keyframes"]["size"] = json!([64, 32]);
        v["keyframes"]["every_myr"] = json!(10.0);
        let path = dir.join("small.json");
        std::fs::write(&path, v.to_string()).unwrap();
        path
    }

    /// One GET on a fresh connection: (status, body).
    fn get(addr: SocketAddr, path: &str) -> (u16, Vec<u8>) {
        let mut s = TcpStream::connect(addr).unwrap();
        write!(
            s,
            "GET {path} HTTP/1.1\r\nHost: t\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut all = Vec::new();
        s.read_to_end(&mut all).unwrap();
        let split = all.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let head = String::from_utf8_lossy(&all[..split]).into_owned();
        let status = head[9..12].parse().unwrap();
        let body = all[split + 4..].to_vec();
        if let Some(l) = head
            .lines()
            .find_map(|l| l.strip_prefix("Content-Length: "))
        {
            assert_eq!(l.parse::<usize>().unwrap(), body.len());
        }
        (status, body)
    }

    /// The world without its wall-clock timings (the surface's note keeps `millis`).
    fn world_bytes(mut w: WorldState) -> Vec<u8> {
        for text in w.notes.values_mut() {
            if let Ok(Value::Object(mut o)) = serde_json::from_str::<Value>(text) {
                o.remove("millis");
                *text = Value::Object(o).to_string();
            }
        }
        Checkpoint {
            preset_json: String::new(),
            content_json: String::new(),
            stage_names: Vec::new(),
            next_stage: 0,
            stage_state: Vec::new(),
            world: w,
        }
        .to_bytes()
        .unwrap()
    }

    #[test]
    fn live_cadence_asks_for_every_step() {
        let planet =
            live_cadence(r#"{"planet": {}, "keyframes": {"size": [8, 4]}}"#, None).unwrap();
        let v: Value = serde_json::from_str(&planet).unwrap();
        assert_eq!(v["keyframes"]["every_myr"], json!(EVERY_STEP_MYR));
        assert_eq!(v["keyframes"]["size"], json!([8, 4]));
        let v: Value =
            serde_json::from_str(&live_cadence(r#"{"planet": {}}"#, Some(5.0)).unwrap()).unwrap();
        assert_eq!(v["keyframes"]["every_myr"], json!(5.0));
        let v: Value = serde_json::from_str(
            &live_cadence(
                r#"{"dummy": {"world": {"keyframe_every_years": 5e7}}}"#,
                None,
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(v["dummy"]["world"]["keyframe_every_years"], json!(1.0));
        assert!(live_cadence("[1]", None).is_err());
    }

    #[test]
    fn never_binds_the_lan() {
        let dir = tmp("bind");
        let preset = small_pipeline(&dir);
        for addr in ["0.0.0.0", "10.0.0.217", "::"] {
            let err = Watch::start(&options(&preset, &dir.join("w"), addr.parse().unwrap()))
                .err()
                .unwrap()
                .to_string();
            assert!(err.contains("never served on the LAN"), "{addr}: {err}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The live stream in miniature: every step arrives as a frame while the run goes on, every
    /// event arrives, the last frame is the batch run's last keyframe pixel for pixel, and the
    /// watched world equals the batch world (watching changes nothing).
    #[test]
    fn watch_streams_every_step_and_matches_the_batch_run() {
        let dir = tmp("stream");
        let preset = small_pipeline(&dir);

        // The batch run, as `deeptime run` drives it (the preset's own 10-Myr keyframes).
        let batch_dir = dir.join("batch");
        let batch = Session::create(
            &std::fs::read_to_string(&preset).unwrap(),
            "",
            SessionOptions {
                out_dir: batch_dir.clone(),
                threads: 2,
                mem_cap: 0,
            },
            Arc::new(NoGauge),
        )
        .unwrap();
        while batch.advance(Duration::from_secs(3600)).unwrap() != AdvanceStatus::Done {}
        batch.finalize_write().unwrap();
        let batch_world = batch.checkpoint().unwrap().world;
        let index = KeyframeIndex::read(&batch_dir).unwrap();
        assert!(
            index.frames.len() < 10,
            "the batch run keeps its 10-Myr cadence"
        );
        let batch_last = read_keyframe(&batch_dir, index.frames.last().unwrap()).unwrap();

        let w = Watch::start(&options(
            &preset,
            &dir.join("watch"),
            IpAddr::V4(Ipv4Addr::LOCALHOST),
        ))
        .unwrap();
        let addr = w.addr();
        let (tx, rx) = std::sync::mpsc::channel();
        let reader = std::thread::spawn(move || {
            let mut s = TcpStream::connect(addr).unwrap();
            s.write_all(b"GET /stream HTTP/1.1\r\nHost: t\r\n\r\n")
                .unwrap();
            let mut r = BufReader::new(s);
            let mut line = String::new();
            r.read_line(&mut line).unwrap();
            assert!(line.starts_with("HTTP/1.1 200"), "{line}");
            let mut headers = String::new();
            loop {
                line.clear();
                r.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                headers.push_str(&line);
            }
            assert!(headers.contains("Content-Type: text/event-stream"));
            let (mut msgs, mut event) = (Vec::new(), String::new());
            loop {
                line.clear();
                if r.read_line(&mut line).unwrap() == 0 {
                    break;
                }
                let l = line.trim_end();
                if let Some(e) = l.strip_prefix("event: ") {
                    event = e.to_owned();
                } else if let Some(d) = l.strip_prefix("data: ") {
                    let v: Value = serde_json::from_str(d).unwrap();
                    if event == "hello" {
                        tx.send(()).unwrap();
                    }
                    let done = event == "status" && v["state"] == "done";
                    msgs.push((event.clone(), v));
                    if done {
                        break;
                    }
                }
            }
            msgs
        });
        // The page is connected before the first step.
        rx.recv_timeout(Duration::from_secs(30)).unwrap();
        w.run().unwrap();
        let msgs = reader.join().unwrap();

        let (hello_kind, hello) = &msgs[0];
        assert_eq!(hello_kind, "hello");
        assert_eq!(hello["frames"], json!([]));
        assert_eq!(hello["meta"]["stages"], json!(["plates", "surface"]));
        let run = hello["run"].as_str().unwrap().to_owned();
        let frames: Vec<&Value> = msgs
            .iter()
            .filter(|(k, _)| k == "frame")
            .map(|(_, v)| v)
            .collect();
        assert_eq!(frames.len(), 10, "one frame per 2-Myr step");
        for (i, f) in frames.iter().enumerate() {
            assert_eq!(f["n"], i);
            assert_eq!(f["stage"], "plates+surface");
            assert_eq!(
                f["layers"].as_array().unwrap().len(),
                f["ramps"].as_array().unwrap().len()
            );
            assert!(
                f["layers"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|l| l == "elevation")
            );
        }
        let ts: Vec<f64> = frames.iter().map(|f| f["t"].as_f64().unwrap()).collect();
        assert!(ts.windows(2).all(|p| p[1] > p[0]), "{ts:?}");
        assert_eq!(*ts.last().unwrap(), 0.0);
        let streamed: Vec<Value> = msgs
            .iter()
            .filter(|(k, _)| k == "events")
            .flat_map(|(_, v)| v["events"].as_array().unwrap().clone())
            .collect();
        // Both sides read through the same parser from the same text.
        let expected: Vec<Value> = batch_world
            .events
            .iter()
            .map(|e| serde_json::from_str(&e.to_json().to_string()).unwrap())
            .collect();
        assert!(!expected.is_empty());
        assert_eq!(streamed, expected, "every event, in emission order");
        assert_eq!(msgs.last().unwrap().1["state"], "done");
        assert!(msgs.iter().any(|(k, _)| k == "progress"));

        // The last frame, layer by layer, is the batch run's final keyframe.
        let last = frames.last().unwrap();
        assert_eq!(last["t"].as_f64().unwrap(), batch_last.t);
        let names: Vec<&str> = last["layers"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(names, batch_last.layer_names());
        for (i, (_, rgba)) in batch_last.layers.iter().enumerate() {
            let (status, png) = get(addr, &format!("/frames/{run}/9/{i}.png"));
            assert_eq!(status, 200);
            assert_eq!(
                png,
                deeptime_viewer::encode_png(batch_last.w, batch_last.h, rgba).unwrap(),
                "layer {}",
                names[i]
            );
        }
        // Rewinding: the first step's frame is still there, and now cached.
        assert_eq!(get(addr, &format!("/frames/{run}/0/0.png")).0, 200);
        assert!(dir.join("watch/frames/kf-0-0.png").is_file());
        assert_eq!(get(addr, &format!("/frames/{run}/10/0.png")).0, 404);
        assert_eq!(get(addr, "/frames/other/0/0.png").0, 404);
        let (status, page) = get(addr, "/");
        assert_eq!(status, 200);
        assert!(String::from_utf8(page).unwrap().contains("live.js"));
        assert_eq!(get(addr, "/live.js").0, 200);
        assert_eq!(get(addr, "/ramps.json").0, 200);
        assert_eq!(get(addr, "/nothing").0, 404);
        let (status, state) = get(addr, "/state.json");
        assert_eq!(status, 200);
        let state: Value = serde_json::from_slice(&state).unwrap();
        assert_eq!(state["frames"].as_array().unwrap().len(), 10);
        assert_eq!(state["status"]["state"], "done");
        // Keep-alive: two requests on one connection.
        let mut s = TcpStream::connect(addr).unwrap();
        s.write_all(b"GET /ramps.json HTTP/1.1\r\nHost: t\r\n\r\nHEAD / HTTP/1.1\r\nHost: t\r\nConnection: close\r\n\r\n")
            .unwrap();
        let mut both = String::new();
        s.read_to_string(&mut both).unwrap();
        assert_eq!(both.matches("HTTP/1.1 200 OK").count(), 2);

        // Watching changed nothing: the same world as the batch run.
        let watched = w.session.checkpoint().unwrap().world;
        assert!(
            world_bytes(watched) == world_bytes(batch_world),
            "the watched run's world differs from the batch run's"
        );
        assert!(dir.join("watch/world/manifest.json").is_file());
        w.stop();
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
