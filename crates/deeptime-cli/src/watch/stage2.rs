// SPDX-License-Identifier: LGPL-3.0-or-later
//! The workbench's stage 2 (P1-20): the 3D block preview of an area and click-to-inspect,
//! served by `tools/workbench/preview.py` (the job runner that generates a box on the Mac's
//! headless Minecraft server and exports it with `deeptime-inspect export-area`). This server only
//! forwards: each request runs `<python> preview.py http <METHOD> <path?query>` with the body on
//! stdin, which answers on stdout with one JSON line `{"status", "content_type", "length"}` and
//! then the body (tools/workbench/README.md, "The contract").
//!
//! Routes forwarded (`deeptime watch --preview <preview.py>`): `POST /api/preview`,
//! `GET /api/preview[/<job>[/<file>]]`, `GET /api/inspect`, `GET /area3d/…` (the 3D viewer).
//! Without `--preview` they keep answering 501, as stage 1 left them.

use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use serde_json::Value;

/// How to reach the preview runner.
#[derive(Clone, Debug)]
pub struct Preview {
    /// The Python interpreter (on the Mac `/usr/bin/python3` works: the runner is stdlib-only).
    pub python: PathBuf,
    /// `tools/workbench/preview.py` of the checkout whose smoke server sets it uses.
    pub script: PathBuf,
    /// The workbench's runs folder (`--runs`): the runner finds a run's bake at
    /// `<runs>/<id>/world` (`DEEPTIME_WORKBENCH_RUNS`). `watch::serve` fills it in.
    pub runs: Option<PathBuf>,
    /// The `deeptime` binary the runner asks for `inspect-column` (`DEEPTIME_WORKBENCH_DEEPTIME`):
    /// this very executable. `watch::serve` fills it in.
    pub deeptime: Option<PathBuf>,
}

/// One forwarded answer.
pub struct Answer {
    pub status: u16,
    pub content_type: &'static str,
    pub body: Vec<u8>,
}

/// Whether stage 2 answers this path.
pub fn handles(path: &str) -> bool {
    path == "/api/inspect"
        || path == "/api/preview"
        || path.starts_with("/api/preview/")
        || path.starts_with("/area3d/")
}

/// The response headers take a `'static` content type: the few the runner sends.
fn static_type(t: &str) -> &'static str {
    const KNOWN: [&str; 6] = [
        "application/json",
        "application/x-ndjson",
        "text/html; charset=utf-8",
        "text/javascript; charset=utf-8",
        "text/css; charset=utf-8",
        "text/plain",
    ];
    KNOWN
        .iter()
        .find(|k| **k == t)
        .copied()
        .unwrap_or("application/octet-stream")
}

impl Preview {
    /// Forwards one request. `target` is the path with its query (`/api/inspect?run=…`).
    pub fn call(&self, method: &str, target: &str, body: &[u8]) -> io::Result<Answer> {
        let mut cmd = Command::new(&self.python);
        cmd.arg(&self.script).args(["http", method, target]);
        if let Some(runs) = &self.runs {
            cmd.env("DEEPTIME_WORKBENCH_RUNS", runs);
        }
        if let Some(bin) = &self.deeptime {
            cmd.env("DEEPTIME_WORKBENCH_DEEPTIME", bin);
        }
        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        {
            let mut stdin = child.stdin.take().expect("piped");
            stdin.write_all(body)?;
        }
        // stderr is drained on its own thread, so a runner that logs a lot never blocks on it.
        let mut stderr = child.stderr.take().expect("piped");
        let err_reader = std::thread::spawn(move || {
            let mut err = String::new();
            let _ = stderr.read_to_string(&mut err);
            err
        });
        let mut out = Vec::new();
        child.stdout.take().expect("piped").read_to_end(&mut out)?;
        let status = child.wait()?;
        let err = err_reader.join().unwrap_or_default();
        let bad = |why: String| io::Error::other(format!("{}: {why}", self.script.display()));
        let nl = out.iter().position(|&b| b == b'\n').ok_or_else(|| {
            bad(format!(
                "no answer (exit {status}){}",
                if err.is_empty() {
                    String::new()
                } else {
                    format!(": {}", err.trim())
                }
            ))
        })?;
        let head: Value =
            serde_json::from_slice(&out[..nl]).map_err(|e| bad(format!("bad header: {e}")))?;
        let length = head["length"].as_u64().unwrap_or(0) as usize;
        let rest = &out[nl + 1..];
        if rest.len() != length {
            return Err(bad(format!(
                "{} body bytes, the header says {length}",
                rest.len()
            )));
        }
        Ok(Answer {
            status: head["status"]
                .as_u64()
                .and_then(|s| u16::try_from(s).ok())
                .unwrap_or(500),
            content_type: static_type(head["content_type"].as_str().unwrap_or("")),
            body: rest.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes() {
        for p in [
            "/api/inspect",
            "/api/preview",
            "/api/preview/j1",
            "/api/preview/j1/area-0.dtva.gz",
            "/area3d/",
            "/area3d/area3d.js",
        ] {
            assert!(handles(p), "{p}");
        }
        for p in [
            "/api/runs",
            "/",
            "/api/previews",
            "/area3dx",
            "/workbench.js",
        ] {
            assert!(!handles(p), "{p}");
        }
        assert_eq!(static_type("application/json"), "application/json");
        assert_eq!(static_type("image/png"), "application/octet-stream");
    }

    /// A stand-in runner speaking the protocol (`sh -c`, so no Python is needed in the tests).
    #[cfg(unix)]
    #[test]
    fn forwards_through_the_protocol() {
        let dir = std::env::temp_dir().join(format!("deeptime-stage2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let script = dir.join("fake.sh");
        // Echoes the method and path, and the body it got, as JSON.
        std::fs::write(
            &script,
            "body=$(cat); m=\"$2 $3 $body\"; printf '{\"status\": 202, \"content_type\": \"application/json\", \"length\": %d}\\n%s' ${#m} \"$m\"\n",
        )
        .unwrap();
        let p = Preview {
            python: "sh".into(),
            script: script.clone(),
            runs: None,
            deeptime: None,
        };
        let a = p.call("POST", "/api/preview?x=1", b"{}").unwrap();
        assert_eq!(a.status, 202);
        assert_eq!(a.content_type, "application/json");
        assert_eq!(
            String::from_utf8(a.body).unwrap(),
            "POST /api/preview?x=1 {}"
        );
        // The runner is told where the runs and the deeptime binary are.
        std::fs::write(
            &script,
            "m=\"$DEEPTIME_WORKBENCH_RUNS $DEEPTIME_WORKBENCH_DEEPTIME\"; printf '{\"status\": 200, \"content_type\": \"text/plain\", \"length\": %d}\\n%s' ${#m} \"$m\"\n",
        )
        .unwrap();
        let p2 = Preview {
            runs: Some("/runs".into()),
            deeptime: Some("/bin/deeptime".into()),
            ..p.clone()
        };
        let a = p2.call("GET", "/api/inspect", b"").unwrap();
        assert_eq!(String::from_utf8(a.body).unwrap(), "/runs /bin/deeptime");
        std::fs::write(&script, "echo nothing-useful >&2; exit 3\n").unwrap();
        let e = p.call("GET", "/api/inspect", b"").err().unwrap();
        assert!(e.to_string().contains("nothing-useful"), "{e}");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
