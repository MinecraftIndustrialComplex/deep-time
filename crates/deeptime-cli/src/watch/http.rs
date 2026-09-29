// SPDX-License-Identifier: LGPL-3.0-or-later
//! The watch server: a small HTTP/1.1 server on std's `TcpListener` (GET and HEAD, keep-alive, a
//! thread per connection, at most [`MAX_CONNECTIONS`]) for the live page, its stream
//! (server-sent events) and the frame images. It only reads the session (`copy_keyframe`, which
//! never blocks on a step); nothing a page sends reaches the simulation.
//!
//! Who may connect: loopback and Tailscale addresses only ([`tailnet_or_loopback`]), for the
//! address it binds and for every peer. On the Mac it binds 127.0.0.1 and `tailscale serve --tcp`
//! forwards the tailnet to it (tools/viewer/mac-watch.sh): tailscaled is already allowed through
//! macOS's application firewall, a freshly built binary is not (binding 100.x directly would
//! raise the firewall's "accept incoming connections?" dialog on the Mac's screen).
//!
//! Routes: `/` the page, `/live.js`, `/ramps.json`, `/state.json` (everything so far, the
//! stream's `hello`), `/stream` (SSE: `hello`, then `events`, `frame`, `progress`, `status`,
//! `bye`; a comment line every 15 s), `/frames/<run>/<n>/<layer index>.png` (the frame's layer,
//! encoded on first request and cached in the work folder's `frames/`).

use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use deeptime_core::session::Session;

use super::live::Live;

/// Connections served at once; more are answered 503.
pub const MAX_CONNECTIONS: usize = 64;
const MAX_HEADER_BYTES: u64 = 16 * 1024;
/// A keep-alive connection with no request for this long is closed.
const IDLE: Duration = Duration::from_secs(60);
/// A stream with nothing to say sends a comment this often (and so notices a closed page).
const PING: Duration = Duration::from_secs(15);

const NO_CACHE: &str = "no-cache";
/// A frame's image never changes within a run, and its URL names the run.
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

/// What the routes read.
pub struct App {
    pub live: Arc<Live>,
    pub session: Arc<Session>,
    /// The PNG cache (`<work>/frames`).
    pub frames_dir: PathBuf,
    /// `live.html` with the title.
    pub page: String,
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
    keep_alive: bool,
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
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
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

/// One request's line and headers (a body, which GET does not have, is skipped). `None` at a
/// clean end of the connection.
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
    let mut content_length = 0u64;
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
            let v = v.trim().to_ascii_lowercase();
            match k.trim().to_ascii_lowercase().as_str() {
                "connection" if v.contains("close") => keep_alive = false,
                "connection" if v.contains("keep-alive") => keep_alive = true,
                "content-length" => content_length = v.parse().unwrap_or(u64::MAX),
                _ => {}
            }
        }
    }
    if content_length > 0 {
        if content_length > 1 << 20 {
            keep_alive = false;
        } else {
            io::copy(&mut (&mut *r).take(content_length), &mut io::sink())?;
        }
    }
    let path = target.split(['?', '#']).next().unwrap_or("/").to_owned();
    Ok(Some(Request {
        method,
        path,
        keep_alive,
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
                "Forbidden: the Deep Time watch page serves the tailnet only\n",
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
        if req.method != "GET" && !head {
            return respond(
                &mut out,
                &Response::text(405, "GET only: the page only reads the run\n"),
                false,
                false,
            );
        }
        if req.path == "/stream" && !head {
            return stream_events(out, app);
        }
        respond(&mut out, &route(app, &req.path), head, req.keep_alive)?;
        if !req.keep_alive {
            return Ok(());
        }
    }
}

fn route(app: &App, path: &str) -> Response {
    match path {
        "/" | "/index.html" => Response::ok(
            "text/html; charset=utf-8",
            NO_CACHE,
            app.page.as_bytes().to_vec(),
        ),
        "/live.js" => Response::ok(
            "text/javascript; charset=utf-8",
            NO_CACHE,
            deeptime_viewer::LIVE_JS.as_bytes().to_vec(),
        ),
        "/ramps.json" => Response::ok(
            "application/json",
            NO_CACHE,
            deeptime_viewer::RAMPS_JSON.as_bytes().to_vec(),
        ),
        "/state.json" => Response::ok(
            "application/json",
            "no-store",
            app.live.snapshot().0.into_bytes(),
        ),
        "/favicon.ico" => Response {
            status: 204,
            content_type: "image/x-icon",
            cache: IMMUTABLE,
            body: Vec::new(),
        },
        _ => match path.strip_prefix("/frames/") {
            Some(rest) => frame_png(app, rest),
            None => Response::text(404, "not found\n"),
        },
    }
}

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// `<run>/<n>/<layer index>.png`: from the cache, or copied out of the session and encoded.
fn frame_png(app: &App, rest: &str) -> Response {
    let mut it = rest.split('/');
    let (Some(run), Some(n), Some(file), None) = (it.next(), it.next(), it.next(), it.next())
    else {
        return Response::text(404, "not found\n");
    };
    let (Ok(n), Some(Ok(layer))) = (
        n.parse::<u32>(),
        file.strip_suffix(".png").map(str::parse::<u32>),
    ) else {
        return Response::text(404, "not found\n");
    };
    if run != app.live.run() {
        return Response::text(404, "that run is over: reload the page\n");
    }
    let Some(size) = app.live.frame_size(n as usize) else {
        return Response::text(404, "no such frame yet\n");
    };
    if layer as usize >= size.layers {
        return Response::text(404, "no such layer\n");
    }
    let path = app.frames_dir.join(format!("kf-{n}-{layer}.png"));
    if let Ok(bytes) = std::fs::read(&path) {
        return Response::ok("image/png", IMMUTABLE, bytes);
    }
    let mut rgba = vec![0u8; size.w as usize * size.h as usize * 4];
    if let Err(e) = app.session.copy_keyframe(n, layer, &mut rgba) {
        return Response::text(500, format!("frame {n} layer {layer}: {e}\n"));
    }
    match deeptime_viewer::encode_png(size.w, size.h, &rgba) {
        Ok(png) => {
            // Two requests for one image may both encode it; each renames its own file in.
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
fn stream_events(mut out: TcpStream, app: &App) -> io::Result<()> {
    out.write_all(
        b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-store\r\n\
          X-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\nretry: 2000\n\n",
    )?;
    let (hello, mut cur) = app.live.snapshot();
    send(&mut out, "hello", &hello)?;
    out.flush()?;
    loop {
        match app.live.next(&mut cur, PING) {
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
            "100.96.0.7",
            "100.127.255.254",
            "fd7a:115c:a1e0::7",
            "::ffff:100.96.0.7",
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
    }
}
