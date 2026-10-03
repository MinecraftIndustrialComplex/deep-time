// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench server: a small HTTP/1.1 server on std's `TcpListener` (GET, HEAD and JSON
//! POSTs, keep-alive, a thread per connection, at most [`MAX_CONNECTIONS`]) for the page, its
//! API, each run's stream (server-sent events) and frame images. Requests only choose inputs
//! (a new run's preset) or cancel a run; nothing a page sends changes a running simulation.
//!
//! Who may connect: loopback and Tailscale addresses only ([`tailnet_or_loopback`]), for the
//! address it binds and for every peer. On the Mac it binds 127.0.0.1 and `tailscale serve --tcp`
//! forwards the tailnet to it (tools/viewer/mac-watch.sh): tailscaled is already allowed through
//! macOS's application firewall, a freshly built binary is not (binding 100.x directly raises the
//! firewall's "accept incoming connections?" dialog on the Mac's screen and blocks until someone
//! answers). A POST must be `application/json` (a cross-site form cannot send that without a CORS
//! preflight, which this server never grants), same-origin when the browser says where it came
//! from, and addressed to a tailnet or local name (no DNS rebinding).
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
//!   `…/tiles/<layer>/<z>/<x>/<y>.png` ([`super::tiles`]), `/worldmap.js`
//! - stage 2 (P1-20): with `--preview`, `POST /api/preview`, `GET /api/preview/…`,
//!   `GET /api/inspect` and `GET /area3d/…` go to tools/workbench/preview.py ([`super::stage2`]).
//!   Without it: `POST /api/preview` (a box of a run's world →
//!   a 3D block preview job) and `GET /api/inspect?run=&x=&z=` (a column's rock layers, ore bodies
//!   and "why here"); both answer 501 with `{"available": false, "message"}` until it lands.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use serde_json::{Value, json};

use super::live::Live;
use super::runs::Workbench;

/// Connections served at once; more are answered 503.
pub const MAX_CONNECTIONS: usize = 64;
const MAX_HEADER_BYTES: u64 = 16 * 1024;
const MAX_BODY_BYTES: u64 = 1 << 20;
/// A keep-alive connection with no request for this long is closed.
const IDLE: Duration = Duration::from_secs(60);
/// A stream with nothing to say sends a comment this often (and so notices a closed page).
const PING: Duration = Duration::from_secs(15);

const NO_CACHE: &str = "no-cache";
/// A frame's image never changes, and its URL names the run.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// Loopback, or Tailscale's ranges: 100.64.0.0/10 (the CGNAT block its IPv4 addresses come
/// from) and fd7a:115c:a1e0::/48. Everything else — the LAN, the unspecified address — is refused.
pub fn tailnet_or_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(a) => a.is_loopback() || (a.octets()[0] == 100 && a.octets()[1] & 0xc0 == 0x40),
        IpAddr::V6(a) => {
            a.is_loopback()
                || a.to_ipv4_mapped()
                    .is_some_and(|v4| tailnet_or_loopback(IpAddr::V4(v4)))
                || a.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0]
        }
    }
}

/// A `Host` a tailnet page may use: an IP literal, `localhost`, a bare machine name (MagicDNS)
/// or a `*.ts.net` name. Anything else would be a foreign site's name pointed at this address.
pub fn tailnet_host(host: &str) -> bool {
    let name = if let Some(rest) = host.strip_prefix('[') {
        rest.split(']').next().unwrap_or("")
    } else {
        host.rsplit_once(':')
            .filter(|(_, port)| port.bytes().all(|b| b.is_ascii_digit()))
            .map_or(host, |(h, _)| h)
    };
    let name = name.to_ascii_lowercase();
    !name.is_empty()
        && (name.parse::<IpAddr>().is_ok()
            || name == "localhost"
            || !name.contains('.')
            || name.ends_with(".ts.net"))
}

/// What the routes read.
pub struct App {
    pub wb: Arc<Workbench>,
    /// The page with its title.
    pub page: String,
    /// Stage 2's runner (`--preview`), which answers the preview, inspect and viewer routes.
    pub preview: Option<super::stage2::Preview>,
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
            content_type: a.content_type,
            cache: "no-store",
            body: a.body,
        },
        Err(e) => Response::error(502, format!("the preview runner failed: {e}")),
    })
}

pub struct Server {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    accept: Option<JoinHandle<()>>,
}

impl Server {
    /// Binds `bind` and serves `app` on a background thread.
    pub fn start(bind: SocketAddr, app: Arc<App>) -> io::Result<Server> {
        if !tailnet_or_loopback(bind.ip()) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                format!("{} is neither loopback nor a Tailscale address", bind.ip()),
            ));
        }
        let listener = TcpListener::bind(bind)?;
        let addr = listener.local_addr()?;
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = Arc::clone(&stop);
        let accept = std::thread::Builder::new()
            .name("deeptime-watch-accept".into())
            .spawn(move || accept_loop(listener, app, stopping))?;
        Ok(Server {
            addr,
            stop,
            accept: Some(accept),
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Stops accepting (connections already open finish on their own).
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
        // Wake the blocking accept.
        let _ = TcpStream::connect_timeout(&self.addr, Duration::from_secs(1));
        if let Some(h) = self.accept.take() {
            let _ = h.join();
        }
    }

    /// Serves until the process ends.
    pub fn join(mut self) {
        if let Some(h) = self.accept.take() {
            let _ = h.join();
        }
    }
}

fn accept_loop(listener: TcpListener, app: Arc<App>, stop: Arc<AtomicBool>) {
    let active = Arc::new(AtomicUsize::new(0));
    for conn in listener.incoming() {
        if stop.load(Ordering::Acquire) {
            break;
        }
        let Ok(mut stream) = conn else { continue };
        if active.fetch_add(1, Ordering::AcqRel) >= MAX_CONNECTIONS {
            active.fetch_sub(1, Ordering::AcqRel);
            let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
            let _ = respond(
                &mut stream,
                &Response::text(503, "too many connections\n"),
                false,
                false,
            );
            continue;
        }
        let (app, done, stop) = (Arc::clone(&app), Arc::clone(&active), Arc::clone(&stop));
        let spawned = std::thread::Builder::new()
            .name("deeptime-watch-conn".into())
            .spawn(move || {
                let _ = handle(stream, &app, &stop);
                done.fetch_sub(1, Ordering::AcqRel);
            });
        if spawned.is_err() {
            active.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

struct Request {
    method: String,
    path: String,
    query: String,
    keep_alive: bool,
    host: String,
    origin: Option<String>,
    content_type: String,
    body: Vec<u8>,
}

struct Response {
    status: u16,
    content_type: &'static str,
    cache: &'static str,
    body: Vec<u8>,
}

impl Response {
    fn ok(content_type: &'static str, cache: &'static str, body: Vec<u8>) -> Response {
        Response {
            status: 200,
            content_type,
            cache,
            body,
        }
    }

    fn text(status: u16, body: impl Into<String>) -> Response {
        Response {
            status,
            content_type: "text/plain; charset=utf-8",
            cache: "no-store",
            body: body.into().into_bytes(),
        }
    }

    fn json(status: u16, v: &Value) -> Response {
        Response {
            status,
            content_type: "application/json",
            cache: "no-store",
            body: v.to_string().into_bytes(),
        }
    }

    fn error(status: u16, message: impl Into<String>) -> Response {
        Response::json(status, &json!({ "error": message.into() }))
    }
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "Internal Server Error",
    }
}

fn respond(out: &mut TcpStream, r: &Response, head: bool, keep_alive: bool) -> io::Result<()> {
    let mut msg = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: {}\r\n\
         X-Content-Type-Options: nosniff\r\nConnection: {}\r\n\r\n",
        r.status,
        reason(r.status),
        r.content_type,
        r.body.len(),
        r.cache,
        if keep_alive { "keep-alive" } else { "close" }
    )
    .into_bytes();
    if !head {
        msg.extend_from_slice(&r.body);
    }
    out.write_all(&msg)?;
    out.flush()
}

/// One request (its body read when it has one of at most 1 MiB). `None` at a clean end of the
/// connection.
fn read_request(r: &mut BufReader<TcpStream>) -> io::Result<Option<Request>> {
    let mut limited = (&mut *r).take(MAX_HEADER_BYTES);
    let mut line = String::new();
    loop {
        line.clear();
        if limited.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if !line.trim().is_empty() {
            break;
        }
    }
    let mut parts = line.split_whitespace();
    let (Some(method), Some(target), Some(version)) = (parts.next(), parts.next(), parts.next())
    else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bad request line",
        ));
    };
    let (method, target) = (method.to_owned(), target.to_owned());
    let mut keep_alive = version == "HTTP/1.1";
    let (mut content_length, mut host, mut origin, mut content_type) =
        (0u64, String::new(), None, String::new());
    loop {
        line.clear();
        if limited.read_line(&mut line)? == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "headers cut short or too long",
            ));
        }
        let l = line.trim_end_matches(['\r', '\n']);
        if l.is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            let v = v.trim();
            let lower = v.to_ascii_lowercase();
            match k.trim().to_ascii_lowercase().as_str() {
                "connection" if lower.contains("close") => keep_alive = false,
                "connection" if lower.contains("keep-alive") => keep_alive = true,
                "content-length" => content_length = v.parse().unwrap_or(u64::MAX),
                "host" => host = v.to_owned(),
                "origin" => origin = Some(v.to_owned()),
                "content-type" => content_type = lower,
                _ => {}
            }
        }
    }
    if content_length > MAX_BODY_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the request body is over 1 MiB",
        ));
    }
    let mut body = Vec::new();
    if content_length > 0 {
        (&mut *r).take(content_length).read_to_end(&mut body)?;
    }
    let (path, query) = match target.split('#').next().unwrap_or("/").split_once('?') {
        Some((p, q)) => (p.to_owned(), q.to_owned()),
        None => (
            target.split('#').next().unwrap_or("/").to_owned(),
            String::new(),
        ),
    };
    Ok(Some(Request {
        method,
        path,
        query,
        keep_alive,
        host,
        origin,
        content_type,
        body,
    }))
}

fn handle(stream: TcpStream, app: &App, stop: &AtomicBool) -> io::Result<()> {
    let peer = stream.peer_addr()?;
    stream.set_nodelay(true)?;
    stream.set_write_timeout(Some(Duration::from_secs(30)))?;
    let mut out = stream;
    if !tailnet_or_loopback(peer.ip()) {
        return respond(
            &mut out,
            &Response::text(
                403,
                "Forbidden: the Deep Time workbench serves the tailnet only\n",
            ),
            false,
            false,
        );
    }
    out.set_read_timeout(Some(IDLE))?;
    let mut reader = BufReader::new(out.try_clone()?);
    loop {
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        let req = match read_request(&mut reader) {
            Ok(Some(req)) => req,
            Ok(None) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::InvalidData => {
                return respond(
                    &mut out,
                    &Response::text(400, format!("{e}\n")),
                    false,
                    false,
                );
            }
            Err(e) => return Err(e),
        };
        let head = req.method == "HEAD";
        let resp = match req.method.as_str() {
            "GET" | "HEAD" => {
                if !head
                    && let Some(id) = req
                        .path
                        .strip_prefix("/api/runs/")
                        .and_then(|r| r.strip_suffix("/stream"))
                {
                    return match app.wb.run(id) {
                        Some(run) => stream_events(out, &run.live),
                        None => respond(
                            &mut out,
                            &Response::error(404, format!("no run {id}")),
                            false,
                            false,
                        ),
                    };
                }
                get(app, &req)
            }
            "POST" => post(app, &req),
            _ => Response::error(405, "GET, HEAD or POST"),
        };
        respond(&mut out, &resp, head, req.keep_alive)?;
        if !req.keep_alive {
            return Ok(());
        }
    }
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<&'a str> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v)
}

fn get(app: &App, req: &Request) -> Response {
    if let Some(r) = stage2(app, req) {
        return r;
    }
    // Rock at depth and sections, read from a run's bake (sections/).
    if let Some(r) = super::sections::route(&app.wb, &req.path, &req.query) {
        return Response {
            status: r.status,
            content_type: r.content_type,
            cache: r.cache,
            body: r.body,
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
        "/favicon.ico" => Response {
            status: 204,
            content_type: "image/x-icon",
            cache: IMMUTABLE,
            body: Vec::new(),
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
                return Response {
                    status: a.status,
                    content_type: a.content_type,
                    cache: a.cache,
                    body: a.body,
                };
            }
            match tail.strip_prefix("frames/") {
                Some(f) => frame_png(&run, f),
                None => Response::text(404, "not found\n"),
            }
        }
    }
}

/// The POST guard: JSON only, same origin, a tailnet or local name.
fn post_refusal(req: &Request) -> Option<Response> {
    if !req.content_type.starts_with("application/json") {
        return Some(Response::error(415, "POST application/json"));
    }
    if !tailnet_host(&req.host) {
        return Some(Response::error(
            403,
            format!("not a tailnet address: {}", req.host),
        ));
    }
    if let Some(o) = &req.origin {
        let from = o.split_once("://").map_or(o.as_str(), |(_, rest)| rest);
        if from != req.host {
            return Some(Response::error(
                403,
                format!("cross-origin request from {o}"),
            ));
        }
    }
    None
}

fn post(app: &App, req: &Request) -> Response {
    if let Some(refused) = post_refusal(req) {
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
        _ => match path
            .strip_prefix("/api/runs/")
            .and_then(|r| r.strip_suffix("/cancel"))
        {
            Some(id) => match app.wb.cancel(id) {
                Ok(()) => Response::json(200, &json!({ "ok": true })),
                Err((status, msg)) => Response::error(status, msg),
            },
            None => Response::error(404, "not found"),
        },
    }
}

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// `<n>/<layer index>.png` of a run: from its cache, or drawn from the frame and encoded.
fn frame_png(run: &super::runs::Run, rest: &str) -> Response {
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

fn send(out: &mut TcpStream, event: &str, data: &str) -> io::Result<()> {
    out.write_all(format!("event: {event}\ndata: {data}\n\n").as_bytes())
}

/// Server-sent events until the page goes away or the server closes (the body ends with the
/// connection).
fn stream_events(mut out: TcpStream, live: &Live) -> io::Result<()> {
    out.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\n\
          X-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\nretry: 2000\n\n",
    )?;
    let (hello, mut cur) = live.snapshot();
    send(&mut out, "hello", &hello)?;
    out.flush()?;
    loop {
        match live.next(&mut cur, PING) {
            None => {
                send(&mut out, "bye", "{}")?;
                return out.flush();
            }
            Some(msgs) if msgs.is_empty() => out.write_all(b": ping\n\n")?,
            Some(msgs) => {
                let mut buf = Vec::new();
                for (event, data) in msgs {
                    buf.extend_from_slice(format!("event: {event}\ndata: {data}\n\n").as_bytes());
                }
                out.write_all(&buf)?;
            }
        }
        out.flush()?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_loopback_and_the_tailnet() {
        for ok in [
            "127.0.0.1",
            "127.3.2.1",
            "::1",
            "100.64.0.1",
            "100.100.232.45",
            "100.127.255.254",
            "fd7a:115c:a1e0::1a36:e82e",
            "::ffff:100.100.232.45",
        ] {
            assert!(tailnet_or_loopback(ok.parse().unwrap()), "{ok}");
        }
        for no in [
            "0.0.0.0",
            "::",
            "10.0.0.217",
            "192.168.1.20",
            "100.63.255.255",
            "100.128.0.1",
            "8.8.8.8",
            "fd7a:115c:a1e1::1",
            "fe80::1",
            "::ffff:10.0.0.1",
        ] {
            assert!(!tailnet_or_loopback(no.parse().unwrap()), "{no}");
        }
        for ok in [
            "m6-mac-mini:8642",
            "m6-mac-mini.tail028f45.ts.net:8642",
            "100.100.232.45:8642",
            "127.0.0.1:8642",
            "localhost",
            "[fd7a:115c:a1e0::1]:8642",
        ] {
            assert!(tailnet_host(ok), "{ok}");
        }
        for no in [
            "evil.example.com",
            "evil.example.com:8642",
            "",
            "m6.local.example:1",
        ] {
            assert!(!tailnet_host(no), "{no}");
        }
    }
}
