// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench's routes: the page, its API, each run's stream (server-sent events) and frame
//! images. Requests only choose inputs (a new run's preset) or cancel a run; nothing a page sends
//! changes a running simulation.
//!
//! The server itself is the shared viewer core's (`mic_viewer::http`, vendor/mic-viewer; decisions
//! log 2026-10-21): HTTP/1.1 on std's `TcpListener` (GET, HEAD and JSON POSTs, keep-alive, a thread
//! per connection, at most 64), born here and moved there with the slippy map so mic-rtsim's
//! viewer shares it. [`App`] is its `Handler`.
//!
//! Who may connect: loopback only, and with `--allow-tailnet` Tailscale addresses too
//! (`mic_viewer::http::address_allowed`), for the address it binds and for every peer; every
//! request, GET included, must name a loopback host (`localhost`, `127.0.0.1`, `[::1]`, any port)
//! or with `--allow-tailnet` a tailnet one (an IP literal, a bare MagicDNS name, `*.ts.net`), so a
//! foreign site's name pointed at this address (DNS rebinding) is refused before any route. On the
//! test VM the service binds 127.0.0.1:8642 and the laptop reaches it through its ssh tunnel
//! (tools/remote/tunnels.sh, http://localhost:8642/). On the Mac it binds 127.0.0.1 and
//! `tailscale serve --tcp` forwards the tailnet to it, with `--allow-tailnet`
//! (tools/viewer/mac-watch.sh): tailscaled is already allowed through macOS's application
//! firewall, a freshly built binary is not (binding 100.x directly raises the firewall's "accept
//! incoming connections?" dialog on the Mac's screen and blocks until someone answers). A POST must
//! be `application/json` (a cross-site form cannot send that without a CORS preflight, which this
//! server never grants) and same-origin when the browser says where it came from.
//!
//! Routes:
//! - `/` the page, `/workbench.js`, `/deposits.js`, `/ramps.json`
//! - `GET /api/catalog` the editor's definitions; `POST /api/preset` a form state's effective
//!   preset, share code, problems and estimate; `POST /api/decode` a share code as a form state
//! - `GET /api/runs` the runs and the queue; `POST /api/runs` queue a run (`{"json", "label",
//!   "cap_gib"}`: the memory cap, 0 or absent = the workbench's `--mem-cap`; a run whose estimate
//!   is over its cap answers 400 with the reason, as `POST /api/preset` does with `can_run:
//!   false`); `POST /api/runs/<id>/cancel`
//! - `GET /api/runs/<id>/stream` (SSE: `hello`, then `events`, `frame`, `progress`, `status`,
//!   `bye`; a comment line every 15 s), `/api/runs/<id>/state.json` (the `hello`),
//!   `/api/runs/<id>/frames/<n>/<layer index>.png` (encoded on first request, cached in the run's
//!   `frames/`), `/api/runs/<id>/deposits` (the deposit sites the overlay draws, from the world in
//!   memory while the run goes on and from the finished run's `sites/` and `bodies/`; `deposits.rs`)
//! - `GET /api/runs/<id>/rock[/slice.png|/section]`, `/sections.js`: the rock at a depth below the
//!   surface and cross-sections, read from the run's bake ([`super::sections`])
//! - the final world map: `GET /api/runs/<id>/world.json`, `…/world/sample?x=&z=` and
//!   `…/tiles/<layer>/<z>/<x>/<y>.png` ([`super::tiles`]), `/worldmap.js` and the shared slippy
//!   map it drives, `/mic-viewer/map.js`
//! - stage 2 (P1-20): with `--preview`, `POST /api/preview`, `GET /api/preview/…`,
//!   `GET /api/inspect` and `GET /area3d/…` go to tools/workbench/preview.py ([`super::stage2`]).
//!   Without it: `POST /api/preview` (a box of a run's world →
//!   a 3D block preview job) and `GET /api/inspect?run=&x=&z=` (a column's rock layers, ore bodies
//!   and "why here"); both answer 501 with `{"available": false, "message"}` until it lands.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use mic_viewer::http::{Handler, IMMUTABLE, NO_CACHE, Reply, Request, Response, query_param};
use mic_viewer::sse::{PING, SseWriter};
use serde_json::{Value, json};

use super::runs::{Run, Workbench};

/// What the routes read.
pub struct App {
    pub wb: Arc<Workbench>,
    /// The page with its title.
    pub page: String,
    /// Stage 2's runner (`--preview`), which answers the preview, inspect and viewer routes.
    pub preview: Option<super::stage2::Preview>,
    /// `--allow-tailnet`: tailnet names pass the POST guard as they pass the server's `Host` check.
    pub allow_tailnet: bool,
}

/// A stage-2 route forwarded to the preview runner (stage2.rs), when one is configured.
fn stage2(app: &App, req: &Request) -> Option<Response> {
    let p = app.preview.as_ref()?;
    if !super::stage2::handles(&req.path) {
        return None;
    }
    let target = if req.query.is_empty() {
        req.path.clone()
    } else {
        format!("{}?{}", req.path, req.query)
    };
    let method = if req.method == "POST" { "POST" } else { "GET" };
    Some(match p.call(method, &target, &req.body) {
        Ok(a) => Response {
            status: a.status,
            content_type: a.content_type.into(),
            cache: "no-store",
            body: a.body,
            csp: None,
        },
        Err(e) => Response::error(502, format!("the preview runner failed: {e}")),
    })
}

impl Handler for App {
    fn handle(&self, req: &Request) -> Reply {
        match req.method.as_str() {
            "GET" => {
                if let Some(id) = req
                    .path
                    .strip_prefix("/api/runs/")
                    .and_then(|r| r.strip_suffix("/stream"))
                {
                    return match self.wb.run(id) {
                        Some(run) => stream_events(run),
                        None => Response::error(404, format!("no run {id}")).into(),
                    };
                }
                get(self, req).into()
            }
            "HEAD" => get(self, req).into(),
            "POST" => post(self, req).into(),
            _ => Response::error(405, "GET, HEAD or POST").into(),
        }
    }
}

fn get(app: &App, req: &Request) -> Response {
    if let Some(r) = stage2(app, req) {
        return r;
    }
    // Rock at depth and sections, read from a run's bake (sections/).
    if let Some(r) = super::sections::route(&app.wb, &req.path, &req.query) {
        return Response {
            status: r.status,
            content_type: r.content_type.into(),
            cache: r.cache,
            body: r.body,
            csp: None,
        };
    }
    let path = req.path.as_str();
    match path {
        "/" | "/index.html" => Response::ok(
            "text/html; charset=utf-8",
            NO_CACHE,
            app.page.as_bytes().to_vec(),
        ),
        "/workbench.js" => Response::ok(
            "text/javascript; charset=utf-8",
            NO_CACHE,
            deeptime_viewer::WORKBENCH_JS.as_bytes().to_vec(),
        ),
        "/projection.js" => Response::ok(
            "text/javascript; charset=utf-8",
            NO_CACHE,
            deeptime_viewer::PROJECTION_JS.as_bytes().to_vec(),
        ),
        "/deposits.js" => Response::ok(
            "text/javascript; charset=utf-8",
            NO_CACHE,
            deeptime_viewer::DEPOSITS_JS.as_bytes().to_vec(),
        ),
        "/worldmap.js" => Response::ok(
            "text/javascript; charset=utf-8",
            NO_CACHE,
            deeptime_viewer::WORLDMAP_JS.as_bytes().to_vec(),
        ),
        "/ramps.json" => Response::ok(
            "application/json",
            NO_CACHE,
            deeptime_viewer::RAMPS_JSON.as_bytes().to_vec(),
        ),
        "/mic-viewer/map.js" => Response::ok(
            "text/javascript; charset=utf-8",
            NO_CACHE,
            mic_viewer::assets::MAP_JS.as_bytes().to_vec(),
        ),
        "/favicon.ico" => Response {
            status: 204,
            content_type: "image/x-icon".into(),
            cache: IMMUTABLE,
            body: Vec::new(),
            csp: None,
        },
        "/api/catalog" => Response::json(200, &app.wb.catalog.to_json()),
        "/api/runs" => Response::json(200, &app.wb.list_json()),
        "/api/inspect" => Response::json(
            501,
            &json!({
                "available": false,
                "run": query_param(&req.query, "run"),
                "message": "Click-to-inspect (a column's rock layers, ore bodies and \"why here\", \
                            through the same column evaluator as deeptime inspect-column) is the \
                            workbench's stage 2; it is not available yet.",
            }),
        ),
        _ => {
            let Some(rest) = path.strip_prefix("/api/runs/") else {
                return Response::text(404, "not found\n");
            };
            let (id, tail) = rest.split_once('/').unwrap_or((rest, ""));
            let Some(run) = app.wb.run(id) else {
                return Response::error(404, format!("no run {id} (it may have been pruned)"));
            };
            if tail == "state.json" {
                return Response::ok(
                    "application/json",
                    "no-store",
                    run.live.snapshot().0.into_bytes(),
                );
            }
            if tail == "deposits" {
                return Response::ok(
                    "application/json",
                    "no-store",
                    run.deposits_answer().to_vec(),
                );
            }
            if let Some(a) = super::tiles::route(&run.dir, tail, &req.query) {
                return a;
            }
            match tail.strip_prefix("frames/") {
                Some(f) => frame_png(&run, f),
                None => Response::text(404, "not found\n"),
            }
        }
    }
}

fn post(app: &App, req: &Request) -> Response {
    // JSON only, same origin (the server has checked the Host already).
    if let Some(refused) = mic_viewer::http::post_refusal(req, app.allow_tailnet) {
        return refused;
    }
    if let Some(r) = stage2(app, req) {
        return r;
    }
    let body: Value = match serde_json::from_slice(&req.body) {
        Ok(v) => v,
        Err(e) => return Response::error(400, format!("not JSON: {e}")),
    };
    let path = req.path.as_str();
    match path {
        "/api/preset" => {
            let r = app.wb.catalog.effective(&body);
            Response::json(if r.get("error").is_some() { 400 } else { 200 }, &r)
        }
        "/api/decode" => {
            let r = app.wb.catalog.decode(body["code"].as_str().unwrap_or(""));
            Response::json(if r.get("error").is_some() { 400 } else { 200 }, &r)
        }
        "/api/runs" => {
            let Some(json) = body["json"].as_str() else {
                return Response::error(400, "give the preset as \"json\"");
            };
            // The editor's memory cap (0 or absent = the workbench's default); a run over its cap
            // is refused inside `submit`, the same rule as the form's answer.
            let cap = match app.wb.catalog.cap_bytes(&body["cap_gib"]) {
                Ok(cap) => cap,
                Err(msg) => return Response::error(400, msg),
            };
            match app.wb.submit(json, body["label"].as_str(), cap) {
                Ok(run) => Response::json(201, &json!({ "id": run.id })),
                Err((status, msg)) => Response::error(status, msg),
            }
        }
        "/api/preview" => Response::json(
            501,
            &json!({
                "available": false,
                "request": body,
                "message": "The 3D block preview of an area is the workbench's stage 2, built \
                            beside this page; it is not available yet.",
            }),
        ),
        _ => {
            let rest = path.strip_prefix("/api/runs/");
            if let Some(id) = rest.and_then(|r| r.strip_suffix("/cancel")) {
                return match app.wb.cancel(id) {
                    Ok(()) => Response::json(200, &json!({ "ok": true })),
                    Err((status, msg)) => Response::error(status, msg),
                };
            }
            // Pick an epoch or continue (docs/design/playability-steering.md): a new run from the
            // finished run's epoch checkpoints, to `stop_myr` after its start.
            if let Some(id) = rest.and_then(|r| r.strip_suffix("/resume")) {
                let Some(stop) = body["stop_myr"].as_f64() else {
                    return Response::error(400, "give stop_myr (Myr after the start)");
                };
                return match app.wb.resume_run(id, stop) {
                    Ok(run) => Response::json(200, &json!({ "ok": true, "id": run.id })),
                    Err((status, msg)) => Response::error(status, msg),
                };
            }
            Response::error(404, "not found")
        }
    }
}

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// `<n>/<layer index>.png` of a run: from its cache, or drawn from the frame and encoded.
fn frame_png(run: &Run, rest: &str) -> Response {
    let (Some((n, file)), true) = (rest.split_once('/'), !rest.contains("..")) else {
        return Response::text(404, "not found\n");
    };
    let (Ok(n), Some(Ok(layer))) = (
        n.parse::<u32>(),
        file.strip_suffix(".png").map(str::parse::<u32>),
    ) else {
        return Response::text(404, "not found\n");
    };
    let Some(size) = run.live.frame_size(n as usize) else {
        return Response::text(404, "no such frame yet\n");
    };
    if layer as usize >= size.layers {
        return Response::text(404, "no such layer\n");
    }
    let dir = run.frames_dir();
    let path = dir.join(format!("kf-{n}-{layer}.png"));
    if let Ok(bytes) = std::fs::read(&path) {
        return Response::ok("image/png", IMMUTABLE, bytes);
    }
    let rgba = match run.frame_rgba(n, layer, size) {
        Ok(rgba) => rgba,
        Err(e) => return Response::text(500, format!("frame {n} layer {layer}: {e}\n")),
    };
    match deeptime_viewer::encode_png(size.w, size.h, &rgba) {
        Ok(png) => {
            // Two requests for one image may both encode it; each renames its own file in.
            let _ = std::fs::create_dir_all(&dir);
            let tmp = path.with_extension(format!(
                "png.{}.tmp",
                TMP_SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            if std::fs::write(&tmp, &png).is_ok() && std::fs::rename(&tmp, &path).is_err() {
                let _ = std::fs::remove_file(&tmp);
            }
            Response::ok("image/png", IMMUTABLE, png)
        }
        Err(e) => Response::text(500, format!("frame {n} layer {layer}: {e}\n")),
    }
}

/// Server-sent events until the page goes away or the run's stream ends (`bye`).
fn stream_events(run: Arc<Run>) -> Reply {
    Reply::Stream(Box::new(move |w: &mut SseWriter<'_>| {
        let (hello, mut cur) = run.live.snapshot();
        w.send("hello", &hello)?;
        loop {
            match run.live.next(&mut cur, PING) {
                None => return w.send("bye", "{}"),
                Some(msgs) if msgs.is_empty() => w.ping()?,
                Some(msgs) => w.send_all(msgs.iter().map(|(e, d)| (*e, d.as_str())))?,
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    /// The workbench's `Host` policy: loopback names always, tailnet names only with
    /// `--allow-tailnet`, foreign names never (the server applies it to every request).
    #[test]
    fn loopback_names_and_the_tailnet_only_when_allowed() {
        use mic_viewer::http::host_allowed;
        for ok in [
            "localhost",
            "localhost:8642",
            "127.0.0.1:8642",
            "[::1]:8642",
        ] {
            assert!(host_allowed(ok, false), "{ok}");
        }
        for tail in [
            "mac-mini:8642",
            "viewer.tail0000.ts.net:8642",
            "100.96.0.7:8642",
            "[fd7a:115c:a1e0::1]:8642",
        ] {
            assert!(!host_allowed(tail, false), "{tail} without --allow-tailnet");
            assert!(host_allowed(tail, true), "{tail} with --allow-tailnet");
        }
        for no in [
            "evil.example.com",
            "evil.example.com:8642",
            "",
            "m6.local.example:1",
        ] {
            assert!(!host_allowed(no, true), "{no}");
        }
    }
}
