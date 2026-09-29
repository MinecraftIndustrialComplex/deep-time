// SPDX-License-Identifier: LGPL-3.0-or-later
//! `deeptime watch`: the browser workbench (decisions log 2026-09-28, "How should Ben watch a
//! simulation while it runs?" and "Should Deep Time get a browser workbench on the Mac…"; STATUS
//! P1-20, stage 1). One process serves a page on the tailnet with
//!
//! - the Create World editor's controls ([`editor`]: the game's presets, field catalogue, timeline
//!   choices and share codes) and a Run button;
//! - the runs ([`runs`]): a queue served one at a time, each kept on disk after it ends (the
//!   newest `--keep`) so it can be reopened, rewound and compared side by side;
//! - a live view of each run: map, globe, layers and events as each step lands, with pause, step,
//!   rewind, speed and "follow live".
//!
//! How a run is watched: its preset's keyframe cadence is set to one frame per step
//! ([`live_cadence`]; `--every-myr` chooses another), so the session renders every step through the
//! same keyframe path the in-game timelapse uses; the runner advances one step at a time and
//! publishes new frames (`keyframe_info`), events (`events_since`) and progress into the run's
//! [`live::Live`]; the server ([`http`]) streams them and encodes a frame's layer as PNG when a page
//! first asks. The layers are whatever the frames name, so a new stage's layers appear without a
//! change here.
//!
//! Watching changes nothing in a run: keyframes are drawn from the world between steps, and the
//! page only chooses a new run's inputs or cancels one. The test
//! `workbench_streams_every_step_and_matches_the_batch_run` holds it to that (the watched world
//! equals the batch run's, and its last frame is the batch run's last keyframe pixel for pixel).

mod editor;
mod http;
mod live;
mod runs;

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use deeptime_core::session::{Session, SessionState};
use serde_json::{Value, json};

use crate::run::{CliResult, Common, load_preset_with};

pub use http::tailnet_or_loopback;

/// The workbench's default port (tools/viewer/mac-watch.sh uses it too).
pub const DEFAULT_PORT: u16 = 8642;

/// Finished runs kept by default (about 1 GB each for a 500-Myr pipeline_quick_16k run).
pub const DEFAULT_KEEP: usize = 8;

/// "A frame after every step": a keyframe cadence of one year, shorter than any step.
const EVERY_STEP_MYR: f64 = 1.0e-6;

pub struct WatchOptions {
    /// A run to queue at start (a built-in name, a file or a share code); the page starts others.
    pub preset: Option<String>,
    pub start: Option<String>,
    pub length_myr: Option<f64>,
    /// Loopback or a Tailscale address (`tailnet_or_loopback`).
    pub bind: IpAddr,
    /// 0 = any free port.
    pub port: u16,
    /// Where runs are kept (a folder per run).
    pub runs: PathBuf,
    /// Finished runs kept; older ones are deleted.
    pub keep: usize,
    /// A frame every this many Myr instead of every step.
    pub every_myr: Option<f64>,
    /// Stop once nothing runs or waits (after `preset`'s run): scripts and tests.
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

/// A running workbench: its runs and its server.
pub struct Served {
    pub wb: Arc<runs::Workbench>,
    server: http::Server,
}

impl Served {
    pub fn addr(&self) -> SocketAddr {
        self.server.addr()
    }

    /// Stops the runner (cancelling a running run) and the server.
    pub fn stop(mut self) {
        self.wb.close();
        self.server.stop();
    }
}

/// Opens the runs folder and starts serving.
pub fn serve(o: &WatchOptions) -> CliResult<Served> {
    if !tailnet_or_loopback(o.bind) {
        return Err(format!(
            "--bind {}: only loopback or a Tailscale address (100.64.0.0/10, fd7a:115c:a1e0::/48); \
             the page is never served on the LAN",
            o.bind
        )
        .into());
    }
    if let Some(m) = o.every_myr
        && !(m.is_finite() && m > 0.0)
    {
        return Err(format!("--every-myr {m}: must be > 0").into());
    }
    let wb = runs::Workbench::open(&o.runs, o.keep, o.common.clone(), o.every_myr)?;
    let app = Arc::new(http::App {
        wb: Arc::clone(&wb),
        page: deeptime_viewer::workbench_page("Deep Time workbench"),
    });
    let server = http::Server::start(SocketAddr::new(o.bind, o.port), app).map_err(|e| {
        wb.close();
        format!("cannot serve on {}:{}: {e}", o.bind, o.port)
    })?;
    Ok(Served { wb, server })
}

/// `deeptime watch`.
pub fn watch(o: &WatchOptions) -> CliResult<()> {
    let served = serve(o)?;
    println!(
        "deeptime watch: serving http://{}/ (runs in {}, the newest {} finished kept; pid {})",
        served.addr(),
        o.runs.display(),
        o.keep,
        std::process::id()
    );
    let mut first = None;
    if let Some(p) = &o.preset {
        let json = load_preset_with(p, o.start.as_deref(), o.length_myr)?;
        let run = served
            .wb
            .submit(&json, None, o.common.mem_cap)
            .map_err(|(_, m)| m)?;
        println!("deeptime watch: queued run {}", run.id);
        first = Some(run);
    }
    if !o.exit_when_done {
        served.server.join();
        return Ok(());
    }
    served.wb.wait_idle();
    let state = first
        .as_ref()
        .map(|r| r.live.summary().3)
        .unwrap_or(Value::Null);
    served.stop();
    match state["state"].as_str() {
        None | Some("done") => Ok(()),
        Some(s) => Err(format!("the run ended {s}: {}", state["message"]).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::{Ipv4Addr, TcpStream};
    use std::path::Path;
    use std::time::{Duration, Instant};

    use deeptime_core::checkpoint::Checkpoint;
    use deeptime_core::session::{AdvanceStatus, NoGauge, SessionOptions};
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

    fn options(runs: &Path, keep: usize, bind: IpAddr) -> WatchOptions {
        WatchOptions {
            preset: None,
            start: None,
            length_myr: None,
            bind,
            port: 0,
            runs: runs.to_path_buf(),
            keep,
            every_myr: None,
            exit_when_done: true,
            common: quiet(),
        }
    }

    const LOCAL: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

    /// The coupled plates + surface pipeline at test size: 20 Myr, 10 steps, keyframes every
    /// 10 Myr (the batch run's cadence).
    fn small_pipeline() -> String {
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
        // The streaming tests read the coupled group (plates, surface, structure) as the whole
        // pipeline; the deposit stage after it (P1-14) has its own tests.
        v.as_object_mut().unwrap().remove("deposits");
        v.to_string()
    }

    fn dummy(steps: u32, sleep_ms: u32) -> String {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../deeptime-core/tests/presets/dummy.json"
        ))
        .unwrap();
        let mut v: Value = serde_json::from_str(&text).unwrap();
        v["dummy"]["steps"] = json!(steps);
        v["dummy"]["sleep_ms"] = json!(sleep_ms);
        v.to_string()
    }

    /// One request on a fresh connection: (status, body).
    fn request(
        addr: SocketAddr,
        method: &str,
        path: &str,
        headers: &str,
        body: &str,
    ) -> (u16, Vec<u8>) {
        let mut s = TcpStream::connect(addr).unwrap();
        write!(
            s,
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n{headers}Content-Length: {}\r\n\r\n{body}",
            body.len()
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

    fn get(addr: SocketAddr, path: &str) -> (u16, Vec<u8>) {
        request(addr, "GET", path, "", "")
    }

    fn post(addr: SocketAddr, path: &str, body: &Value) -> (u16, Value) {
        let (status, b) = request(
            addr,
            "POST",
            path,
            "Content-Type: application/json\r\n",
            &body.to_string(),
        );
        (status, serde_json::from_slice(&b).unwrap_or(Value::Null))
    }

    fn get_json(addr: SocketAddr, path: &str) -> Value {
        let (status, b) = get(addr, path);
        assert_eq!(status, 200, "{path}");
        serde_json::from_slice(&b).unwrap()
    }

    fn run_state(addr: SocketAddr, id: &str) -> Value {
        get_json(addr, "/api/runs")["runs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["id"] == id)
            .cloned()
            .unwrap_or(Value::Null)
    }

    /// Reads a run's stream until its status is final; signals once `hello` arrived.
    fn read_stream(
        addr: SocketAddr,
        id: String,
        tx: std::sync::mpsc::Sender<()>,
    ) -> Vec<(String, Value)> {
        let mut s = TcpStream::connect(addr).unwrap();
        write!(
            s,
            "GET /api/runs/{id}/stream HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n"
        )
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
                let done = event == "status"
                    && ["done", "failed", "cancelled"].contains(&v["state"].as_str().unwrap_or(""));
                msgs.push((event.clone(), v));
                if done {
                    break;
                }
            }
        }
        msgs
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

    fn wait_for(what: &str, mut f: impl FnMut() -> bool) {
        let until = Instant::now() + Duration::from_secs(60);
        while !f() {
            assert!(Instant::now() < until, "timed out waiting for {what}");
            std::thread::sleep(Duration::from_millis(20));
        }
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
        for addr in ["0.0.0.0", "10.0.0.217", "::"] {
            let err = serve(&options(&dir, 2, addr.parse().unwrap()))
                .err()
                .unwrap()
                .to_string();
            assert!(err.contains("never served on the LAN"), "{addr}: {err}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A run started through the API streams every step as a frame while it runs and every event;
    /// its last frame (now read back from disk) is the batch run's last keyframe pixel for pixel,
    /// and its world equals the batch world (watching changes nothing).
    #[test]
    fn workbench_streams_every_step_and_matches_the_batch_run() {
        let dir = tmp("stream");
        let preset = small_pipeline();

        // The batch run, as `deeptime run` drives it (the preset's own 10-Myr keyframes).
        let batch_dir = dir.join("batch");
        let batch = Session::create(
            &preset,
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

        let served = serve(&options(&dir.join("runs"), 8, LOCAL)).unwrap();
        let addr = served.addr();
        served.wb.hold(true);
        let (status, created) = post(
            addr,
            "/api/runs",
            &json!({ "json": preset, "label": "small" }),
        );
        assert_eq!(status, 201, "{created}");
        let id = created["id"].as_str().unwrap().to_owned();
        let (tx, rx) = std::sync::mpsc::channel();
        let sid = id.clone();
        let reader = std::thread::spawn(move || read_stream(addr, sid, tx));
        // The page is connected before the first step.
        rx.recv_timeout(Duration::from_secs(30)).unwrap();
        served.wb.hold(false);
        let msgs = reader.join().unwrap();
        served.wb.wait_idle();

        let (hello_kind, hello) = &msgs[0];
        assert_eq!(hello_kind, "hello");
        assert_eq!(hello["frames"], json!([]));
        assert_eq!(hello["status"]["state"], "queued");
        // The coupled group grows as stages land (plates, climate, surface, structure, …): read it, don't pin it.
        let stages: Vec<&str> = hello["meta"]["stages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s.as_str().unwrap())
            .collect();
        assert_eq!(stages[0], "plates");
        assert!(stages.contains(&"surface"), "{stages:?}");
        let group = stages.join("+");
        assert_eq!(hello["meta"]["label"], "small");
        let frames: Vec<&Value> = msgs
            .iter()
            .filter(|(k, _)| k == "frame")
            .map(|(_, v)| v)
            .collect();
        assert_eq!(frames.len(), 10, "one frame per 2-Myr step");
        for (i, f) in frames.iter().enumerate() {
            assert_eq!(f["n"], i);
            assert_eq!(f["stage"], group.as_str());
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

        // The last frame, layer by layer, is the batch run's final keyframe (from disk now).
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
            let (status, png) = get(addr, &format!("/api/runs/{id}/frames/9/{i}.png"));
            assert_eq!(status, 200);
            assert_eq!(
                png,
                deeptime_viewer::encode_png(batch_last.w, batch_last.h, rgba).unwrap(),
                "layer {}",
                names[i]
            );
        }
        assert_eq!(get(addr, &format!("/api/runs/{id}/frames/0/0.png")).0, 200);
        assert!(dir.join(format!("runs/{id}/frames/kf-0-0.png")).is_file());
        assert_eq!(get(addr, &format!("/api/runs/{id}/frames/10/0.png")).0, 404);
        assert_eq!(get(addr, "/api/runs/nope/frames/0/0.png").0, 404);
        let state: Value =
            serde_json::from_slice(&get(addr, &format!("/api/runs/{id}/state.json")).1).unwrap();
        assert_eq!(state["frames"].as_array().unwrap().len(), 10);
        assert_eq!(state["status"]["state"], "done");
        let listed = run_state(addr, &id);
        assert_eq!(listed["state"], "done");
        assert_eq!(listed["frames"], 10);
        assert_eq!(listed["label"], "small");
        assert!(listed["share_code"].as_str().unwrap().starts_with("DT1."));

        // Watching changed nothing: the same world as the batch run.
        let watched = served
            .wb
            .run(&id)
            .unwrap()
            .final_world
            .lock()
            .unwrap()
            .take()
            .unwrap();
        assert!(
            world_bytes(watched) == world_bytes(batch_world),
            "the watched run's world differs from the batch run's"
        );
        assert!(dir.join(format!("runs/{id}/world/manifest.json")).is_file());
        served.stop();
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// The page, the editor API, the queue (one run at a time, a bounded wait, cancelling), the
    /// POST guard, the stage-2 stubs, and runs kept across a restart and pruned.
    #[test]
    fn workbench_queue_editor_and_kept_runs() {
        let dir = tmp("queue");
        let runs = dir.join("runs");
        let served = serve(&options(&runs, 8, LOCAL)).unwrap();
        let addr = served.addr();

        let (status, page) = get(addr, "/");
        assert_eq!(status, 200);
        assert!(String::from_utf8(page).unwrap().contains("workbench.js"));
        assert_eq!(get(addr, "/workbench.js").0, 200);
        assert_eq!(get(addr, "/ramps.json").0, 200);
        assert_eq!(get(addr, "/nothing").0, 404);
        let catalog = get_json(addr, "/api/catalog");
        assert_eq!(catalog["fields"][0]["id"], "seed");
        let (status, eff) = post(
            addr,
            "/api/preset",
            &json!({ "preset": "pipeline_quick_16k", "start": "random", "length_myr": 100, "fields": { "seed": 7 } }),
        );
        assert_eq!(status, 200);
        assert_eq!(eff["can_run"], true, "{eff}");
        let (status, dec) = post(addr, "/api/decode", &json!({ "code": eff["share_code"] }));
        assert_eq!(status, 200);
        assert_eq!(dec["imported"]["seed"], 7);
        assert_eq!(
            post(addr, "/api/decode", &json!({ "code": "DT1.x" })).0,
            400
        );

        // The POST guard.
        let body = json!({ "json": dummy(2, 0) }).to_string();
        assert_eq!(
            request(
                addr,
                "POST",
                "/api/runs",
                "Content-Type: text/plain\r\n",
                &body
            )
            .0,
            415
        );
        assert_eq!(
            request(
                addr,
                "POST",
                "/api/runs",
                "Content-Type: application/json\r\nOrigin: http://evil.example.com\r\n",
                &body
            )
            .0,
            403
        );
        let mut s = TcpStream::connect(addr).unwrap();
        write!(
            s,
            "POST /api/runs HTTP/1.1\r\nHost: evil.example.com:8642\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        let mut resp = String::new();
        s.read_to_string(&mut resp).unwrap();
        assert!(resp.starts_with("HTTP/1.1 403"), "{resp}");
        assert_eq!(post(addr, "/api/runs", &json!({ "json": "{}" })).0, 400);
        assert_eq!(
            post(
                addr,
                "/api/preview",
                &json!({ "run": "x", "box": [0, 0, 16, 16] })
            )
            .0,
            501
        );
        assert_eq!(get(addr, "/api/inspect?run=x&x=0&z=0").0, 501);

        // One at a time: with the queue held, three wait and a fourth is refused; one is
        // cancelled before it starts.
        served.wb.hold(true);
        let ids: Vec<String> = (0..3)
            .map(|k| {
                let (status, v) = post(
                    addr,
                    "/api/runs",
                    &json!({ "json": dummy(2 + k, 0), "label": format!("d{k}") }),
                );
                assert_eq!(status, 201, "{v}");
                v["id"].as_str().unwrap().to_owned()
            })
            .collect();
        let (status, refused) = post(addr, "/api/runs", &json!({ "json": dummy(2, 0) }));
        assert_eq!(status, 409);
        assert!(
            refused["error"].as_str().unwrap().contains("one at a time"),
            "{refused}"
        );
        let list = get_json(addr, "/api/runs");
        assert_eq!(list["queue"].as_array().unwrap().len(), 3);
        assert_eq!(run_state(addr, &ids[2])["queue_position"], 3);
        assert_eq!(
            post(addr, &format!("/api/runs/{}/cancel", ids[1]), &json!({})).0,
            200
        );
        assert_eq!(run_state(addr, &ids[1])["state"], "cancelled");
        served.wb.hold(false);
        served.wb.wait_idle();
        assert_eq!(run_state(addr, &ids[0])["state"], "done");
        assert_eq!(run_state(addr, &ids[2])["state"], "done");
        assert_eq!(
            post(addr, &format!("/api/runs/{}/cancel", ids[0]), &json!({})).0,
            409
        );

        // Cancelling the running run keeps what it made.
        let (_, slow) = post(
            addr,
            "/api/runs",
            &json!({ "json": dummy(40, 40), "label": "slow" }),
        );
        let slow = slow["id"].as_str().unwrap().to_owned();
        wait_for("the slow run's first frames", || {
            let r = run_state(addr, &slow);
            r["state"] == "running" && r["frames"].as_u64().unwrap_or(0) >= 2
        });
        assert_eq!(
            post(addr, &format!("/api/runs/{slow}/cancel"), &json!({})).0,
            200
        );
        served.wb.wait_idle();
        let r = run_state(addr, &slow);
        assert_eq!(r["state"], "cancelled");
        let kept_frames = r["frames"].as_u64().unwrap();
        assert!((2..80).contains(&kept_frames), "{r}");
        assert_eq!(
            get(addr, &format!("/api/runs/{slow}/frames/1/0.png")).0,
            200
        );
        served.stop();

        // A restart reads every run back, with its frames and events; images come from disk.
        let served = serve(&options(&runs, 8, LOCAL)).unwrap();
        let addr = served.addr();
        let list = get_json(addr, "/api/runs");
        assert_eq!(list["runs"].as_array().unwrap().len(), 4);
        assert_eq!(list["runs"][0]["id"], slow, "newest first");
        let r = run_state(addr, &slow);
        assert_eq!(
            (r["state"].clone(), r["frames"].as_u64().unwrap()),
            (json!("cancelled"), kept_frames)
        );
        let d0 = run_state(addr, &ids[0]);
        assert_eq!(d0["state"], "done");
        assert!(d0["events"].as_u64().unwrap() > 0);
        assert_eq!(
            get(addr, &format!("/api/runs/{}/frames/0/0.png", ids[2])).0,
            200
        );
        served.stop();
        // Keeping 2: the two oldest finished runs (and their folders) go.
        let served = serve(&options(&runs, 2, LOCAL)).unwrap();
        let list = get_json(served.addr(), "/api/runs");
        let left: Vec<&str> = list["runs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["id"].as_str().unwrap())
            .collect();
        assert_eq!(left, [slow.as_str(), ids[2].as_str()]);
        assert!(!runs.join(&ids[0]).exists() && !runs.join(&ids[1]).exists());
        served.stop();
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
