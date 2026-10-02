#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Preview a box of a Deep Time world in 3D: the job runner and the workbench contract (P1-20
stage 2). Runs on the Mac, where the server sets live (tools/smoke/install.sh); the laptop drives
it through tools/workbench/preview-area.sh. README.md next to this file documents the contract.

A job generates one or more boxes of a world on a headless server and exports each with
`deeptime-inspect export-area`:

    install  the profile's server set, only if it is missing (or --install); then, always, the Deep Time
             jar: Gradle builds it (a one-second no-op when nothing changed) and the set's copy is
             replaced when its bytes differ, so a preview never boots older code than its checkout
    prepare  the job's own world folder <job>/world: synthetic world data, a world-data folder
             (a run's bake), or a preset the server generates from on its first start; datapacks
    boot     the profile's server with --universe <job> --world world (the smoke world is untouched)
    generate each box with Chunky (a box across the seam in two pieces, each in the canonical span)
    probe    /deeptime probe palette|claims|tw, sites near, why <site>, probe site (probes.jsonl)
    stop     save-all flush, stop
    export   area-<i>.dtva.gz per box (+ --verify), then the world is deleted (--keep-world keeps it)

Commands:
    preview.py run    [spec options] [--job ID] [--lock-held] [--install] [--keep-world]
    preview.py refresh-jar --profile P        build the Deep Time jar and replace the profile's copy if it
                                              differs (what every `run` does first); no server, no job
    preview.py submit [spec options]          start `run` detached; prints {"job", "dir", ...}
    preview.py status <job>                   the job's status.json
    preview.py list                           every job, newest first
    preview.py serve  [--host H] [--port P]   the contract over HTTP, plus the viewer (for testing;
                                              the workbench server calls api() itself)

Spec options: --profile, --size, --box x0,z0,x1,z1 (repeatable), --y lo,hi, and one world source:
--synth K [--strata S] | --preset P [--start S] [--length-myr N] | --world-data DIR |
--test-terrain K | --level-type T (not a Deep Time world: the pack's own generator, e.g.
minecraft:normal, for comparison renders of vanilla or Terralith terrain; --locate BIOME
(repeatable) then centres a --locate-side box on the nearest such biome, found with /locate after
the boot); plus --datapack DIR (repeatable), --seed N, --no-mic-rock-tags.

Locking: every server boot on the Mac holds Deep Time's single server lock (the one
tools/remote/mac.sh --server takes). `run` takes it itself, unless --lock-held says the caller
(mac.sh --server) already holds it.

Standard library only.
"""

import argparse
import datetime
import hashlib
import json
import os
import random
import re
import shutil
import signal
import socket
import string
import subprocess
import sys
import tempfile
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "tools", "smoke"))
import jarstate  # noqa: E402  (tools/smoke/jarstate.py: the jar comparison install.sh's record shares)

VIEWER = os.path.join(REPO, "tools", "review", "pages", "area3d")
PROFILES = ("standalone", "mic", "mic-ev", "terrablender", "ev")
MAX_SIDE = 256
DEEP_TIME_Y = (-512, 639)
VANILLA_Y = (-64, 319)
LEVEL_TYPES = {16384: "deeptime:planet", 32768: "deeptime:planet_32k"}
# The Mac's shared cache (tools/remote/mac.sh): the server lock and the dev-shell profiles.
MAC_CACHE = "/Volumes/Graphide/ban/deeptime-cache"

DONE = re.compile(r"Done \(")
RCON_UP = re.compile(r"RCON running on")
FATAL = re.compile(
    r"MixinApplyError|Mixin apply failed|InvalidInjectionException|Preparing crash report"
    r"|Failed to start the minecraft server|Mod [Ll]oading has failed"
    r"|\[deeptime\] The server stops before creating the world")
PROBE = re.compile(r"\[deeptime-probe\]\s*(\{.*\})")
GEN_START = re.compile(r"\[deeptime\] generating the world's history")
GEN_DONE = re.compile(r"\[deeptime\] world data written to")
CHUNKY_DONE = re.compile(r"Task finished for|Task complete")
CHUNKY_COUNT = re.compile(r"Processed:\s*([\d,]+)\s*chunks")


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")


def jobs_root():
    return os.environ.get("DEEPTIME_WORKBENCH_JOBS") or os.path.join(REPO, "smoke", "workbench", "jobs")


def cache_dir():
    c = os.environ.get("DEEPTIME_CACHE")
    if c:
        return c
    if os.path.isdir(MAC_CACHE):
        return MAC_CACHE
    return os.path.join(REPO, "smoke")


def new_job_id():
    stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
    return stamp + "-" + "".join(random.choice(string.ascii_lowercase + string.digits) for _ in range(4))


def fold(x, c):
    return (x + c // 2) % c - c // 2


# ---------------------------------------------------------------------------------------------
# The spec: what a job generates. Validated the same way for the CLI and the HTTP API.
# ---------------------------------------------------------------------------------------------

def allowed_roots():
    roots = [REPO, jobs_root(), os.path.dirname(os.path.dirname(REPO))]   # …/mic (pack, Docs)
    if runs_root():
        roots.append(runs_root())   # a run's bake is a world_data source
    extra = os.environ.get("DEEPTIME_WORKBENCH_ROOTS", "")
    roots += [r for r in extra.split(os.pathsep) if r]
    return [os.path.realpath(r) for r in roots]


def check_path(p, what):
    """A path the spec names must exist and lie under the repo, the jobs root or …/mic."""
    real = os.path.realpath(os.path.expanduser(p))
    if not any(real == r or real.startswith(r + os.sep) for r in allowed_roots()):
        raise ValueError(f"{what} {p} is outside the allowed roots ({', '.join(allowed_roots())})")
    if not os.path.exists(real):
        raise ValueError(f"{what} {p} does not exist")
    return real


def normalise_spec(spec, max_side=MAX_SIDE):
    """Checks a spec (a dict, as in README.md) and fills its defaults. Raises ValueError."""
    s = json.loads(json.dumps(spec))
    s.setdefault("profile", "standalone")
    if s["profile"] not in PROFILES:
        raise ValueError(f"profile must be one of {', '.join(PROFILES)}")
    s["size"] = int(s.get("size", 16384))
    if s["size"] not in LEVEL_TYPES:
        raise ValueError("size must be 16384 or 32768")
    boxes = s.pop("boxes", None) or ([s.pop("box")] if "box" in s else [])
    w0 = s.get("world", {})
    locate = [str(b) for b in (w0.get("locate") or [])]
    if locate and not w0.get("level_type"):
        raise ValueError("locate needs level_type (a biome is located in the pack's own generator)")
    if not boxes and not locate:
        raise ValueError("a box is required: [x0, z0, x1, z1] (x1, z1 exclusive)")
    norm = []
    for b in boxes:
        if isinstance(b, str):
            b = [int(v) for v in b.split(",")]
        if len(b) != 4 or not all(isinstance(v, int) for v in b):
            raise ValueError(f"box {b} must be four integers x0, z0, x1, z1")
        x0, z0, x1, z1 = b
        if x1 <= x0 or z1 <= z0:
            raise ValueError(f"box {b}: x1 > x0 and z1 > z0 are required")
        if x1 - x0 > max_side or z1 - z0 > max_side:
            raise ValueError(f"box {b} is larger than {max_side} x {max_side} blocks")
        c = s["size"]
        if abs(z0) > c // 4 or abs(z1) > c // 4:
            raise ValueError(f"box {b}: z must lie within ±C/4 = ±{c // 4}")
        norm.append([x0, z0, x1, z1])
    s["boxes"] = norm
    y = s.get("y") or list(VANILLA_Y if w0.get("level_type") else DEEP_TIME_Y)
    if isinstance(y, str):
        y = [int(v) for v in y.split(",")]
    if len(y) != 2 or y[1] < y[0] or y[0] < DEEP_TIME_Y[0] or y[1] > DEEP_TIME_Y[1]:
        raise ValueError(f"y {y} must be lo, hi within {DEEP_TIME_Y[0]}..{DEEP_TIME_Y[1]}")
    s["y"] = y
    w = s.setdefault("world", {})
    sources = [k for k in ("synth", "preset", "world_data", "test_terrain", "level_type") if w.get(k)]
    if len(sources) != 1:
        raise ValueError("world needs exactly one of synth, preset, world_data, test_terrain, level_type")
    if w.get("level_type"):
        if not re.fullmatch(r"[a-z0-9_.-]+:[a-z0-9_./-]+", w["level_type"]):
            raise ValueError(f"level_type {w['level_type']!r} is not a resource location")
        for b in locate:
            if not re.fullmatch(r"[a-z0-9_.-]+:[a-z0-9_./-]+(@-?\d+,-?\d+)?", b):
                raise ValueError(f"locate {b!r} is not a biome id (optionally @x,z: searched from there)")
        w["locate"] = locate
        side = int(w.get("locate_side", 256))
        if not 16 <= side <= max_side:
            raise ValueError(f"locate_side {side} must be in 16..{max_side}")
        w["locate_side"] = side
    if w.get("world_data"):
        w["world_data"] = check_path(w["world_data"], "world_data")
        if not os.path.isfile(os.path.join(w["world_data"], "manifest.json")):
            raise ValueError(f"world_data {w['world_data']} has no manifest.json")
    if w.get("preset") and (os.sep in w["preset"] or w["preset"].endswith(".json")):
        w["preset"] = check_path(w["preset"], "preset")
    if w.get("synth"):
        w.setdefault("strata", "placeholder")
    w["datapacks"] = [check_path(d, "datapack") for d in w.get("datapacks", [])]
    for d in w["datapacks"]:
        if not os.path.isfile(os.path.join(d, "pack.mcmeta")):
            raise ValueError(f"datapack {d} has no pack.mcmeta")
    if w.get("seed") is not None:
        w["seed"] = int(w["seed"])
    props = [str(p) for p in (w.get("jvm_props") or [])]
    for p in props:
        if not re.fullmatch(r"deeptime\.[A-Za-z0-9_.]+=[A-Za-z0-9_.:-]*", p):
            raise ValueError(f"jvm_prop {p!r} must be deeptime.<name>=<value>")
    w["jvm_props"] = props
    s["keep_world"] = bool(s.get("keep_world", False))
    s["max_side"] = int(max_side)
    return s


def spec_from_args(a):
    world = {}
    for k in ("synth", "strata", "preset", "start", "world_data", "test_terrain", "seed", "level_type",
              "locate", "locate_side", "jvm_props"):
        v = getattr(a, k, None)
        if v is not None:
            world[k] = v
    if a.length_myr is not None:
        world["length_myr"] = a.length_myr
    world["datapacks"] = a.datapack or []
    if a.no_mic_rock_tags:
        world["mic_rock_tags"] = False
    spec = {"profile": a.profile, "size": a.size, "boxes": a.box or [], "world": world,
            "keep_world": a.keep_world}
    if a.y:
        spec["y"] = a.y
    return spec


# ---------------------------------------------------------------------------------------------
# Job files.
# ---------------------------------------------------------------------------------------------

def job_dir(job):
    if not re.fullmatch(r"[A-Za-z0-9._-]+", job or ""):
        raise ValueError(f"bad job id {job!r}")
    return os.path.join(jobs_root(), job)


def write_json(path, data):
    tmp = path + ".tmp"
    with open(tmp, "w") as f:
        json.dump(data, f, indent=1)
    os.replace(tmp, path)


TERMINAL = ("done", "failed")


def pid_alive(pid):
    try:
        os.kill(int(pid), 0)
        return True
    except (OSError, ValueError, TypeError):
        return False


def status(job):
    """The job's status.json (README.md, "Status"), or {"job", "state": "unknown"}. A job whose
    runner died without finishing (a crash, a reboot) reads as failed."""
    p = os.path.join(job_dir(job), "status.json")
    try:
        st = json.load(open(p))
    except (OSError, ValueError):
        return {"job": job, "state": "unknown"}
    if st.get("state") not in TERMINAL and st.get("pid") and not pid_alive(st["pid"]):
        st["state"], st["error"] = "failed", "the runner stopped without finishing (see runner.log)"
        st["message"] = st["error"]
    return st


def list_jobs():
    root = jobs_root()
    if not os.path.isdir(root):
        return []
    out = [status(j) for j in sorted(os.listdir(root), reverse=True) if os.path.isdir(os.path.join(root, j))]
    return [s for s in out if s.get("state") != "unknown"]


class Job:
    def __init__(self, job, spec):
        self.id, self.spec, self.dir = job, spec, job_dir(job)
        os.makedirs(self.dir, exist_ok=True)
        self.st = {"job": job, "state": "queued", "spec": spec, "created": now(), "updated": now(),
                   "timings": {}, "exports": [], "message": "", "pid": os.getpid()}
        if spec.get("note"):
            self.st["note"] = spec["note"]
        if spec.get("note_kind"):
            self.st["note_kind"] = spec["note_kind"]
        self.log = open(os.path.join(self.dir, "runner.log"), "a", buffering=1)

    def set(self, state=None, message=None, **kw):
        if state:
            self.st["state"] = state
        if message is not None:
            self.st["message"] = message
        self.st.update(kw)
        self.st["updated"] = now()
        write_json(os.path.join(self.dir, "status.json"), self.st)
        self.say(f"[{self.st['state']}] {self.st['message']}")

    def say(self, line):
        print(line, flush=True)
        self.log.write(f"{now()} {line}\n")

    def timing(self, key, value):
        self.st["timings"][key] = value
        self.set()


# ---------------------------------------------------------------------------------------------
# The Mac's server lock (the same directory lock tools/remote/mac.sh --server takes).
# ---------------------------------------------------------------------------------------------

class ServerLock:
    STALE_S = 3 * 3600

    def __init__(self, job):
        self.path = os.path.join(cache_dir(), "server.lock")
        self.job, self.held = job, False

    def acquire(self):
        while True:
            try:
                os.mkdir(self.path)
                with open(os.path.join(self.path, "owner"), "w") as f:
                    f.write(f"{os.path.basename(REPO)} workbench {self.job.id}\n")
                self.held = True
                return
            except FileExistsError:
                try:
                    age = time.time() - os.stat(self.path).st_mtime
                except OSError:
                    continue
                if age > self.STALE_S:
                    self.job.say("breaking a stale server lock")
                    shutil.rmtree(self.path, ignore_errors=True)
                    continue
                try:
                    owner = open(os.path.join(self.path, "owner")).read().strip()
                except OSError:
                    owner = "?"
                self.job.set("waiting", f"waiting for the Mac's server lock (held by {owner})")
                time.sleep(30)

    def release(self):
        if self.held:
            shutil.rmtree(self.path, ignore_errors=True)
            self.held = False


# ---------------------------------------------------------------------------------------------
# Tools.
# ---------------------------------------------------------------------------------------------

def dev(cmd):
    """`cmd` inside the repo's dev shell: as is when already in one (mac.sh), else through the
    GC-rooted profile mac.sh keeps (never a fresh flake evaluation per call on the Mac)."""
    if os.environ.get("IN_NIX_SHELL"):
        return cmd
    key = hashlib.sha256(open(os.path.join(REPO, "flake.nix"), "rb").read()
                         + open(os.path.join(REPO, "flake.lock"), "rb").read()).hexdigest()[:16]
    profile = os.path.join(cache_dir(), f"devshell-{key}")
    if os.path.exists(profile):
        return ["nix", "develop", profile, "-c"] + cmd
    return ["nix", "develop", REPO, "-c"] + cmd


def tool_env():
    """The environment for Gradle, cargo and friends. tools/remote/mac.sh exports the Mac's
    shared caches for every command it runs, but the live workbench's `deeptime watch` (started by
    tools/viewer/mac-watch.sh) has none of that: its jobs would fill the small internal disk with a
    second ~/.gradle and ~/.cargo. So on the Mac, point them at the SSD's caches unless set."""
    env = dict(os.environ)
    if cache_dir() == MAC_CACHE:
        for var, sub in (("GRADLE_USER_HOME", "gradle"), ("CARGO_HOME", "cargo")):
            if not env.get(var) and os.path.isdir(os.path.join(MAC_CACHE, sub)):
                env[var] = os.path.join(MAC_CACHE, sub)
    return env


def run_cmd(job, cmd, cwd=REPO, log_name=None, check=True):
    """Runs cmd; with log_name its output goes to that log in the job folder, else stdout is
    captured (stderr goes to runner.log)."""
    job.say("$ " + " ".join(cmd))
    env = tool_env()
    if log_name:
        with open(os.path.join(job.dir, log_name), "a") as out:
            r = subprocess.run(cmd, cwd=cwd, env=env, stdout=out, stderr=subprocess.STDOUT, text=True)
    else:
        r = subprocess.run(cmd, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        if r.stderr:
            job.log.write(r.stderr)
    if check and r.returncode != 0:
        tail = "" if log_name else ((r.stderr or "") + (r.stdout or ""))[-2000:]
        raise RuntimeError(f"{os.path.basename(cmd[0])} failed with status {r.returncode}"
                           + (f": {tail}" if tail else f" (see {log_name})"))
    return r


def load_env(path):
    env = {}
    for line in open(path):
        if "=" in line:
            k, v = line.strip().split("=", 1)
            env[k] = v
    return env


def port_free(port):
    with socket.socket() as s:
        return s.connect_ex(("127.0.0.1", port)) != 0


class Server:
    """The profile's server booted on the job's world (--universe <job> --world world)."""

    def __init__(self, job, smoke, install, tools, jvm_extra):
        self.job, self.smoke = job, smoke
        self.dir = os.path.join(smoke, "server")
        args_file = os.path.join(self.dir, "libraries", "net", "neoforged", "neoforge",
                                 install["neoforge"], "unix_args.txt")
        if not os.path.exists(args_file):
            raise RuntimeError(f"{args_file} is missing: run tools/smoke/install.sh {job.spec['profile']}")
        self.command = ([tools["JAVA"], "-Xmx6G", "-XX:+UseG1GC"] + jvm_extra
                        + [f"@{args_file}", "--nogui", "--universe", job.dir, "--world", "world"])
        self.log_path = os.path.join(job.dir, "server.log")
        self.lines, self.fatal, self.done_at, self.rcon_up, self.proc = [], None, None, False, None
        self.timed = []
        self.rcon_port, self.password = install["rcon_port"], open(os.path.join(smoke, "rcon.pw")).read().strip()
        self.mcrcon = tools["MCRCON"]

    def start(self):
        self.started = time.time()
        self._log = open(self.log_path, "a", buffering=1, errors="replace")
        self.proc = subprocess.Popen(self.command, cwd=self.dir, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                     text=True, errors="replace", bufsize=1, start_new_session=True)
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        for line in self.proc.stdout:
            self._log.write(line)
            self.lines.append(line)
            if "[deeptime]" in line:
                self.timed.append((time.time(), line.rstrip()))
            if self.done_at is None and DONE.search(line):
                self.done_at = time.time()
            if RCON_UP.search(line):
                self.rcon_up = True
            if self.fatal is None and FATAL.search(line):
                self.fatal = line.rstrip()

    def wait_ready(self, timeout=1800):
        deadline, last = time.time() + timeout, 0
        while time.time() < deadline:
            if self.done_at and self.rcon_up:
                return
            if self.fatal:
                raise RuntimeError(f"the server failed to start: {self.fatal[:300]}")
            if self.proc.poll() is not None:
                raise RuntimeError(f"the server exited with status {self.proc.returncode} while starting")
            if time.time() - last > 30:
                last = time.time()
                self.job.set(message=f"booting, {int(last - self.started)} s, {len(self.lines)} log lines")
            time.sleep(0.25)
        raise RuntimeError(f"timed out after {timeout} s waiting for the server")

    def rcon(self, command, timeout=300):
        r = subprocess.run([self.mcrcon, "-H", "127.0.0.1", "-P", str(self.rcon_port), "-p", self.password,
                            "-c", command], capture_output=True, text=True, errors="replace", timeout=timeout)
        return r.stdout.strip()

    def probe(self, command):
        """Runs a /deeptime command; its [deeptime-probe] JSON from the log (RCON cuts long replies)."""
        before = len(self.lines)
        reply = self.rcon(command)
        deadline = time.time() + 5
        while time.time() < deadline:
            for l in reversed(self.lines[before:]):
                m = PROBE.search(l)
                if m:
                    try:
                        return json.loads(m.group(1)), reply
                    except ValueError:
                        pass
            time.sleep(0.2)
        m = PROBE.search(reply)
        try:
            return (json.loads(m.group(1)) if m else None), reply
        except ValueError:
            return None, reply

    def stop(self):
        if self.proc is None or self.proc.poll() is not None:
            return
        try:
            self.rcon("stop", timeout=60)
        except Exception:
            pass
        try:
            self.proc.wait(timeout=180)
            return
        except subprocess.TimeoutExpired:
            pass
        os.killpg(os.getpgid(self.proc.pid), signal.SIGTERM)
        try:
            self.proc.wait(timeout=60)
        except subprocess.TimeoutExpired:
            os.killpg(os.getpgid(self.proc.pid), signal.SIGKILL)
            self.proc.wait(timeout=60)


# ---------------------------------------------------------------------------------------------
# Steps.
# ---------------------------------------------------------------------------------------------

def pieces(box, c):
    """A box's x range split at the seam into pieces inside the canonical span, as
    (x0, x1) folded (x1 exclusive), each with its z range."""
    x0, z0, x1, z1 = box
    x0f = fold(x0, c)
    out, x = [], x0f
    end = x0f + (x1 - x0)
    while x < end:
        stop = min(end, c // 2) if x < c // 2 else end
        a, b = fold(x, c), fold(x, c) + (stop - x)
        out.append((a, b, z0, z1))
        x = stop
    return out


def chunky_square(piece, c):
    """Chunky's rectangle for a piece: the chunk range plus one chunk around it, never past the
    seam on a side where the piece touches it (the other piece covers that side). Chunky covers
    the chunks (int) centre >> 4 ± ⌈radius / 16⌉."""
    x0, x1, z0, z1 = piece
    cx0, cx1, cz0, cz1 = (x0 >> 4) - 1, ((x1 - 1) >> 4) + 1, (z0 >> 4) - 1, ((z1 - 1) >> 4) + 1
    half_c = c // 32
    cx0, cx1 = max(cx0, -half_c), min(cx1, half_c - 1)
    ccx, ccz = (cx0 + cx1) // 2, (cz0 + cz1) // 2
    rx, rz = max(ccx - cx0, cx1 - ccx), max(ccz - cz0, cz1 - ccz)
    return ccx * 16 + 8, ccz * 16 + 8, rx * 16, rz * 16, (cx1 - cx0 + 1) * (cz1 - cz0 + 1)


def box_overlaps_bounds(box, bounds, c):
    x0, z0, x1, z1 = box
    bx0, _by0, bz0, bx1, _by1, bz1 = bounds
    if bz0 > z1 - 1 or bz1 < z0:
        return False
    span = bx1 - bx0
    return any((x - bx0) % c <= span for x in range(x0, x1))


def install_profile(job, spec):
    smoke = os.path.join(REPO, "smoke", spec["profile"])
    if os.path.exists(os.path.join(smoke, "install.json")) and not spec.get("_install"):
        return smoke
    job.set("installing", f"tools/smoke/install.sh {spec['profile']} (see install.log)")
    t = time.time()
    run_cmd(job, ["bash", os.path.join(REPO, "tools", "smoke", "install.sh"), spec["profile"],
                  "--size", str(spec["size"])], log_name="install.log")
    job.timing("install_s", round(time.time() - t, 1))
    return smoke


def build_deeptime_jar(job):
    """Gradle's `jar` task on this checkout: it tracks the Java sources, the resources and presets,
    and (hostNatives -> cargo) the Rust natives the jar packs. A no-op takes about a second."""
    try:
        run_cmd(job, dev(["./gradlew", "jar", "-q", "--console=plain"]), cwd=os.path.join(REPO, "neoforge"),
                log_name="build.log")
    except RuntimeError as e:
        tail = ""
        try:
            tail = open(os.path.join(job.dir, "build.log"), errors="replace").read()[-1500:]
        except OSError:
            pass
        raise RuntimeError(f"the Deep Time jar does not build, so the preview would boot older code: {e}\n{tail}")


def refresh_deeptime_jar(job, smoke, build=build_deeptime_jar):
    """Makes the profile's server set carry the jar this checkout builds now. The set's copy is
    taken when install.sh runs and the file name never changes, so it goes stale silently (a
    jar from before P1-15's synthetic deposit preset failed every preview with `unknown key`).
    Staleness is therefore decided by content: build (Gradle), compare sha256, replace when
    different (tools/smoke/jarstate.py). The result is in the job status as `jar` and in install.json."""
    t = time.time()
    job.set("preparing", "building the Deep Time jar (Gradle; the natives are packed into it)")
    build(job)
    built = jarstate.built_jar(REPO)
    res = jarstate.refresh(built, os.path.join(smoke, "server", "mods"))
    fields = jarstate.jar_fields(REPO, built)
    install_json = os.path.join(smoke, "install.json")
    with open(install_json) as f:
        install = json.load(f)
    if any(install.get(k) != v for k, v in fields.items() if k != "deeptime_jar_recorded"):
        jarstate.record(install_json, fields)   # a refresh, or an install.json from before the record
    seconds = round(time.time() - t, 1)
    src = f"source {fields['deeptime_src_rev']}, tree {fields['deeptime_src_hash']}"
    if res["changed"]:
        was = ", ".join(f"{w['name']} {w['sha256'][:12]}" for w in res["was"]) or "none installed"
        msg = f"refreshed the Deep Time jar in smoke/{os.path.basename(smoke)}: {was} -> {res['name']} {res['sha256'][:12]} ({src})"
    else:
        msg = f"the Deep Time jar is current: {res['name']} {res['sha256'][:12]} ({src})"
    job.st["jar"] = {"refreshed": res["changed"], "sha256": res["sha256"], "src_rev": fields["deeptime_src_rev"],
                     "src_hash": fields["deeptime_src_hash"], "was": [w["sha256"] for w in res["was"]],
                     "seconds": seconds}
    job.st["timings"]["jar_s"] = seconds
    job.set(message=msg)
    return res


def build_tools(job, smoke):
    """deeptime-inspect (release) and the profile's block colour table."""
    job.set("preparing", "building deeptime-inspect")
    run_cmd(job, dev(["cargo", "build", "--release", "-q", "-p", "deeptime-inspect"]), log_name="build.log")
    inspect = os.path.join(REPO, "target", "release", "deeptime-inspect")
    mods = os.path.join(smoke, "server", "mods")
    colors = os.path.join(smoke, "colors.json")
    stamp = hashlib.sha256("\n".join(sorted(os.listdir(mods))).encode()).hexdigest()[:16]
    stamp_file = colors + ".mods"
    fresh = os.path.exists(colors) and os.path.exists(stamp_file) and open(stamp_file).read().strip() == stamp
    if not fresh:
        # Minecraft's client resources jar: this checkout's Gradle build, else the main checkout's
        # (a worktree's Mac copy has no neoforge/build until it runs Gradle itself).
        vanilla = None
        for art in (os.path.join(REPO, "neoforge", "build", "moddev", "artifacts"),
                    os.path.join(os.path.dirname(REPO), "deep-time", "neoforge", "build", "moddev", "artifacts")):
            if os.path.isdir(art):
                vanilla = next((os.path.join(art, f) for f in sorted(os.listdir(art)) if "client-extra" in f), None)
            if vanilla:
                break
        if vanilla:
            job.set(message="building the block colour table from the mod jars")
            run_cmd(job, [inspect, "colors", "--jars", mods, "--vanilla", vanilla, "--out", colors], log_name="build.log")
            open(stamp_file, "w").write(stamp)
        else:
            job.say("no Minecraft client resources jar (run a Gradle build once): fallback colours")
            colors = None
    return inspect, colors


class Prepared:
    """Server-dir files a job changes for its boot, restored afterwards."""

    def __init__(self, job, smoke):
        self.job, self.server = job, os.path.join(smoke, "server")
        self.saved = []

    def save(self, path):
        backup = path + ".workbench-backup"
        if os.path.exists(path):
            shutil.copy2(path, backup)
            self.saved.append((path, backup))
        else:
            self.saved.append((path, None))

    def restore(self):
        for path, backup in reversed(self.saved):
            if backup:
                shutil.move(backup, path)
            elif os.path.exists(path):
                os.remove(path)
        self.saved = []


def prepare_world(job, spec, smoke, prep):
    """The job's world folder, the server.properties it boots with, and its JVM flags."""
    w, c = spec["world"], spec["size"]
    world = os.path.join(job.dir, "world")
    os.makedirs(world, exist_ok=True)
    jvm = []
    if w.get("synth"):
        job.set("preparing", f"synthetic world data {w['synth']} (strata {w['strata']}, C = {c})")
        run_cmd(job, dev(["cargo", "run", "--release", "-q", "-p", "deeptime-cli", "--", "synth", "--kind", w["synth"],
                          "--strata", w["strata"], "--size", str(c), "--out", os.path.join(world, "deeptime")]),
                log_name="build.log")
    elif w.get("world_data"):
        job.set("preparing", f"copying world data from {w['world_data']}")
        # The keyframes (a run's timelapse, tens of MB) are not part of what a world reads.
        shutil.copytree(w["world_data"], os.path.join(world, "deeptime"), dirs_exist_ok=True,
                        ignore=shutil.ignore_patterns("keyframes"))
    elif w.get("test_terrain"):
        jvm.append(f"-Ddeeptime.testTerrain={w['test_terrain']}")
    elif w.get("level_type"):
        job.set("preparing", f"a {w['level_type']} world (the pack's own generator, no Deep Time world data)")
    jvm += [f"-D{p}" for p in w.get("jvm_props", [])]
    # config/deeptime/server.json decides a first start (P1-13); only a preset job has one.
    cfg = os.path.join(prep.server, "config", "deeptime", "server.json")
    prep.save(cfg)
    if os.path.exists(cfg):
        os.remove(cfg)
    if w.get("preset"):
        os.makedirs(os.path.dirname(cfg), exist_ok=True)
        write_json(cfg, {"_about": "Written by deep-time/tools/workbench/preview.py for one job; restored after.",
                         "preset": w["preset"],
                         "timeline": {"start": w.get("start", "supercontinent"),
                                      "length_myr": float(w.get("length_myr", 1000))},
                         "memory_cap_gib": float(w.get("memory_cap_gib", 4)), "threads": 0})
    datapacks = list(w.get("datapacks", []))
    tags = os.path.join(os.path.dirname(os.path.dirname(REPO)), "Docs", "datapacks", "deeptime-mic-rock-tags")
    if spec["profile"] in ("mic", "mic-ev") and w.get("mic_rock_tags", True) and os.path.isfile(os.path.join(tags, "pack.mcmeta")) \
            and tags not in datapacks:
        datapacks.append(tags)   # until MIC ships them (Q16), as -PwithMic and the MIC smokes do
    for d in datapacks:
        dst = os.path.join(world, "datapacks", os.path.basename(d.rstrip(os.sep)))
        shutil.copytree(d, dst, dirs_exist_ok=True)
    spec["world"]["datapacks_used"] = datapacks
    props = os.path.join(prep.server, "server.properties")
    prep.save(props)
    text = open(props).read()
    level = w.get("level_type") or LEVEL_TYPES[c]
    text = re.sub(r"(?m)^level-type=.*$", "level-type=" + level.replace(":", "\\:"), text)
    if w.get("seed") is not None:
        text = re.sub(r"(?m)^level-seed=.*\n?", "", text) + f"level-seed={w['seed']}\n"
    open(props, "w").write(text)
    return world, jvm


def generate(job, server, spec):
    c = spec["size"]
    runs = []
    for bi, box in enumerate(spec["boxes"]):
        for p in pieces(box, c):
            cx, cz, rx, rz, n = chunky_square(p, c)
            job.set("generating", f"box {bi + 1}/{len(spec['boxes'])}: Chunky {n} chunks around x {p[0]}..{p[1]}, z {p[2]}..{p[3]}")
            before = sum(1 for l in server.lines if CHUNKY_DONE.search(l))
            t = time.time()
            for cmd in ("chunky shape rectangle", f"chunky center {cx} {cz}", f"chunky radius {rx} {rz}", "chunky start"):
                job.say(f"   {cmd}: {server.rcon(cmd)[:160]}")
            deadline = time.time() + 3600
            done = None
            while time.time() < deadline:
                time.sleep(1)
                hits = [l for l in server.lines if CHUNKY_DONE.search(l)]
                if len(hits) > before:
                    done = hits[-1]
                    break
                if server.proc.poll() is not None:
                    raise RuntimeError("the server stopped while generating")
            if done is None:
                raise RuntimeError("Chunky did not finish within an hour")
            m = CHUNKY_COUNT.search(done)
            runs.append({"box": bi, "piece": list(p), "chunky": {"center": [cx, cz], "radius": [rx, rz]},
                         "chunks_asked": n, "chunks": int(m.group(1).replace(",", "")) if m else None,
                         "seconds": round(time.time() - t, 1)})
            job.timing("chunky", runs)
    return runs


LOCATED = re.compile(r"is at \[(-?\d+), [^,]+, (-?\d+)\]")


def locate_boxes(job, server, spec):
    """Appends a locate_side box centred on the nearest of each located biome (level_type worlds)."""
    w = spec["world"]
    side = w.get("locate_side", 256)
    found = []
    for spec_ in w.get("locate", []):
        biome, _, at = spec_.partition("@")
        cmd = f"locate biome {biome}"
        if at:
            ax, az = at.split(",")
            cmd = f"execute positioned {ax} 64 {az} run {cmd}"
        job.set("locating", f"/{cmd}")
        reply = server.rcon(cmd, timeout=600)
        m = LOCATED.search(reply or "")
        job.say(f"   {cmd}: {(reply or '').strip()[:200]}")
        if not m:
            found.append({"biome": spec_, "reply": (reply or "").strip()[:300]})
            continue
        x, z = int(m.group(1)), int(m.group(2))
        box = [x - side // 2, z - side // 2, x - side // 2 + side, z - side // 2 + side]
        spec["boxes"].append(box)
        found.append({"biome": spec_, "at": [x, z], "box": box})
    job.set(located=found)
    job.timing("located", found)
    return found


def probe(job, server, spec):
    if spec["world"].get("level_type"):
        return None   # not a Deep Time world: nothing to probe
    job.set("probing", "Deep Time's probes (palette, claims, sites, why)")
    t = time.time()
    path = os.path.join(job.dir, "probes.jsonl")
    lines = []

    def rec(cmd):
        p, reply = server.probe(cmd)
        lines.append({"command": cmd, "probe": p, "reply": reply})
        return p

    rec("deeptime probe tw")
    rec("deeptime probe palette")
    rec("deeptime probe claims")
    rec("deeptime probe magma")
    rec("deeptime probe geothermal")
    c = spec["size"]
    why_done = set()
    for box in spec["boxes"]:
        x0, z0, x1, z1 = box
        cx, cz = fold((x0 + x1) // 2, c), (z0 + z1) // 2
        rec(f"deeptime probe column {cx} {cz}")
        radius = int(((x1 - x0) ** 2 + (z1 - z0) ** 2) ** 0.5 / 2) + 1024
        near = rec(f"deeptime sites near {cx} {cz} {radius}") or {}
        for s in near.get("sites", []):
            if s["id"] in why_done or not box_overlaps_bounds(box, s.get("bounds", [0] * 6), c):
                continue
            why_done.add(s["id"])
            rec(f"deeptime why {s['id']}")
            sx, sy, sz = s["center"]
            # `probe site` reports the block only from a loaded chunk; Chunky's are unloaded.
            server.rcon(f"forceload add {sx} {sz}")
            rec(f"deeptime probe site {sx} {sy} {sz}")
            server.rcon(f"forceload remove {sx} {sz}")
    with open(path, "w") as f:
        for l in lines:
            f.write(json.dumps(l) + "\n")
    job.timing("probes_s", round(time.time() - t, 1))
    return path


def export(job, spec, inspect, colors, probes):
    world = os.path.join(job.dir, "world")
    meta = os.path.join(job.dir, "meta.json")
    write_json(meta, {"job": job.id, "profile": spec["profile"], "size": spec["size"], "world": spec["world"],
                      "install": job.st.get("install"), "created": job.st["created"], "note": spec.get("note"),
                      "request": spec.get("request")})
    exports = []
    for bi, box in enumerate(spec["boxes"]):
        job.set("exporting", f"box {bi + 1}/{len(spec['boxes'])}: deeptime-inspect export-area")
        out = os.path.join(job.dir, f"area-{bi}.dtva.gz")
        cmd = [inspect, "export-area", "--world", world, "--box", ",".join(map(str, box)),
               "--y", f"{spec['y'][0]},{spec['y'][1]}", "--span", str(spec["size"]), "--out", out,
               "--meta", meta, "--verify"]
        if spec.get("max_side", MAX_SIDE) > MAX_SIDE:
            cmd += ["--max-side", str(spec["max_side"])]
        if probes:
            cmd += ["--probes", probes]
        if colors:
            cmd += ["--colors", colors]
        t = time.time()
        r = run_cmd(job, cmd)
        summary = json.loads(r.stdout)
        exports.append({"box": box, "file": os.path.basename(out), "url": f"{job.id}/{os.path.basename(out)}",
                        "bytes": summary["bytes"], "seconds": round(time.time() - t, 1), "summary": summary})
        job.set(exports=exports)
    return exports


def run_job(job, lock_held=False):
    spec = job.spec
    lock = ServerLock(job)
    prep = None
    server = None
    t_all = time.time()
    try:
        write_json(os.path.join(job.dir, "spec.json"), spec)
        smoke = install_profile(job, spec)
        refresh_deeptime_jar(job, smoke)
        install = json.load(open(os.path.join(smoke, "install.json")))
        job.st["install"] = {k: install.get(k) for k in ("profile", "neoforge", "pack", "pack_ref", "deeptime_jar",
                                                          "deeptime_jar_sha256", "deeptime_src_rev", "deeptime_src_hash")}
        tools = load_env(os.path.join(REPO, "smoke", "tools.env"))
        inspect, colors = build_tools(job, smoke)
        if not lock_held:
            lock.acquire()
        for port in (install["game_port"], install["rcon_port"]):
            if not port_free(port):
                raise RuntimeError(f"port {port} is in use: another server is running outside the lock")
        prep = Prepared(job, smoke)
        world, jvm = prepare_world(job, spec, smoke, prep)
        server = Server(job, smoke, install, tools, jvm)
        job.set("booting", f"{spec['profile']} server on {world}")
        server.start()
        server.wait_ready()
        job.timing("boot_s", round(server.done_at - server.started, 1))
        starts = [t for t, l in server.timed if GEN_START.search(l)]
        dones = [t for t, l in server.timed if GEN_DONE.search(l)]
        if starts and dones:
            job.timing("first_start_generation_s", round(dones[0] - starts[0], 1))
        if spec["world"].get("locate"):
            locate_boxes(job, server, spec)
        generate(job, server, spec)
        probes = probe(job, server, spec)
        job.set("stopping", "save-all flush, stop")
        t = time.time()
        server.rcon("save-all flush")
        time.sleep(2)
        server.stop()
        job.timing("stop_s", round(time.time() - t, 1))
        prep.restore()
        lock.release()
        export(job, spec, inspect, colors, probes)
        if not spec.get("keep_world"):
            shutil.rmtree(world, ignore_errors=True)
        job.timing("total_s", round(time.time() - t_all, 1))
        job.set("done", "exported" + ("; refreshed the Deep Time jar first" if job.st.get("jar", {}).get("refreshed") else ""),
                finished=now(), export=job.st["exports"][0]["file"] if job.st["exports"] else None)
        return 0
    except Exception as e:
        job.set("failed", str(e)[:2000], finished=now(), error=str(e)[:2000])
        return 1
    finally:
        if server:
            server.stop()
        if prep:
            prep.restore()
        lock.release()


# ---------------------------------------------------------------------------------------------
# The contract: submit / status / list, and the HTTP form of it.
# ---------------------------------------------------------------------------------------------

def submit(spec, job=None):
    """Starts a job in the background; returns its first status. Raises ValueError on a bad spec."""
    spec = normalise_spec(spec)
    active = [s for s in list_jobs() if s.get("state") not in TERMINAL]
    if len(active) >= MAX_ACTIVE:
        raise Busy(f"{len(active)} previews are already queued or running (one server boots at a time); "
                   f"wait for {active[-1]['job']} to finish")
    job = job or new_job_id()
    j = Job(job, spec)
    j.set("queued", "starting the runner")
    write_json(os.path.join(j.dir, "spec.json"), spec)
    log = open(os.path.join(j.dir, "runner.out"), "a")
    child = subprocess.Popen([sys.executable, os.path.abspath(__file__), "run", "--job", job, "--from-spec"],
                             cwd=REPO, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
                             start_new_session=True)
    j.set(pid=child.pid)   # the runner's, until it writes its own status
    return j.st


MAX_ACTIVE = 3


class Busy(Exception):
    pass


# ---------------------------------------------------------------------------------------------
# The workbench's stage-2 routes (crates/deeptime-cli/src/watch/stage2.rs forwards them here).
# A run that finished with P1-08's bake (world data 0.1 in <runs>/<id>/world) is previewed and
# inspected as its own terrain: /api/inspect answers from `deeptime inspect-column` on the bake and
# a preview boots the real mod on a world that uses the bake. A run without one (a placeholder, a
# run older than format 0.1, one still running) keeps the old behaviour: the synthetic deposit
# world at the same latitude and longitude, and every answer says so.
# ---------------------------------------------------------------------------------------------

RUNS_ENV = "DEEPTIME_WORKBENCH_RUNS"          # the workbench's runs folder (set by `deeptime watch`)
DEEPTIME_ENV = "DEEPTIME_WORKBENCH_DEEPTIME"  # the deeptime binary for inspect-column (idem)
RUN_ID = re.compile(r"[A-Za-z0-9._-]+")
PREVIEW_C = 16384
PREVIEW_PRESET = os.path.join(REPO, "crates", "deeptime-core", "tests", "presets", "deposits_world_16k.json")
PREVIEW_LENGTH_MYR = 500
INSPECT_SIDE = 32


def runs_root():
    r = os.environ.get(RUNS_ENV)
    return os.path.realpath(r) if r else None


def run_bake(run):
    """(bake, why_not): the run's P1-08 bake, or None and the reason there is none. The bake is a
    dict: dir (its world-data folder), c (circumference), manifest."""
    root = runs_root()
    if not root:
        return None, "this workbench does not tell the preview runner where its runs are"
    if not RUN_ID.fullmatch(run or ""):
        return None, f"{run!r} is not a run of this workbench"
    d = os.path.join(root, run, "world")
    try:
        with open(os.path.join(d, "manifest.json")) as f:
            m = json.load(f)
    except (OSError, ValueError):
        return None, "the run has not finished writing its world data (or is older than the bake)"
    try:
        major, minor = (int(v) for v in str(m.get("format_version", "0.0")).split(".")[:2])
    except ValueError:
        major, minor = 0, 0
    notes = m.get("notes") or {}
    layers = {l.get("name") for l in m.get("layers", [])}
    c = (m.get("scale") or {}).get("circumference_blocks")
    if (major, minor) < (0, 1) or not isinstance(notes.get("bake"), dict) or notes["bake"].get("baked") is False:
        return None, f"the run's world data is format {m.get('format_version')} with no bake"
    if (notes.get("run") or {}).get("placeholder"):
        return None, "the run is a placeholder preset: its terrain is a synthetic stand-in"
    if not {"elevation", "strata/horizons"} <= layers:
        return None, "the run's world data has no terrain or strata layer"
    if c not in LEVEL_TYPES:
        return None, f"the run's circumference {c} has no Deep Time level type"
    return {"dir": d, "c": c, "manifest": m}, ""


def run_terrain_note(run, bake):
    m = bake["manifest"]
    return (f"This is run {run}'s own terrain. It is the run's finished bake (world data {m.get('format_version')}, "
            f"{m.get('source', '')}, C = {bake['c']:,}), the very data a world made from this run gets, and the preview "
            f"is generated by the real mod on the Mac's headless {os.environ.get('DEEPTIME_WORKBENCH_PROFILE', 'mic')} "
            f"server. A bake is the planet at the end of the run, so it does not change with the frame you were looking at.")


def preview_world_note(run, why=""):
    return (f"Not run {run}'s terrain: {why or 'it has no bake'}. Only a run that finished with the P1-08 bake "
            f"(world data 0.1) is previewed as itself. This preview is Deep Time's synthetic deposit world "
            f"(deposits_world_16k, {PREVIEW_LENGTH_MYR} Myr, C = {PREVIEW_C:,}) generated by the real mod on the "
            f"Mac's headless {os.environ.get('DEEPTIME_WORKBENCH_PROFILE', 'mic')} server, at the latitude and "
            f"longitude you picked. Its sites cluster near x -5,600, z -1,000.")


def latlon_to_block(lat, lon, c=PREVIEW_C):
    """workbench.js blockXZ: x = longitude east of the seam scaled to C from −C/2; north is −z."""
    return round(lon / 360 * c - c / 2), round(-lat / 360 * c)


def box_on_world(req, c=PREVIEW_C):
    """The request's box on the planet of circumference c (from lat_lon when given), at most
    256 × 256 around its centre; returns (box, a note about any shrinking)."""
    ll = req.get("lat_lon") or {}
    if all(isinstance(ll.get(k), (int, float)) for k in ("north", "south", "west", "east")):
        x0, z0 = latlon_to_block(ll["north"], ll["west"], c)
        x1, z1 = latlon_to_block(ll["south"], ll["east"], c)
    else:
        b = req.get("box") or {}
        try:
            x0, z0, x1, z1 = (int(b[k]) for k in ("x0", "z0", "x1", "z1"))
        except (KeyError, TypeError, ValueError):
            raise ValueError("give box {x0, z0, x1, z1} or lat_lon {north, south, west, east}")
    x0, x1, z0, z1 = min(x0, x1), max(x0, x1), min(z0, z1), max(z0, z1)
    note = ""
    w, d = max(16, x1 - x0), max(16, z1 - z0)
    if w > MAX_SIDE or d > MAX_SIDE:
        note = f"The box you drew ({x1 - x0} × {z1 - z0} blocks) was cut to its central {min(w, MAX_SIDE)} × {min(d, MAX_SIDE)}. "
    cx, cz = (x0 + x1) // 2, (z0 + z1) // 2
    w, d = min(w, MAX_SIDE), min(d, MAX_SIDE)
    x0, z0 = cx - w // 2, max(-c // 4, min(c // 4 - d, cz - d // 2))
    return [fold(x0, c), z0, fold(x0, c) + w, z0 + d], note


box_on_preview_world = box_on_world   # the name P1-20 stage 2 used


def viewer_url(job, st, pick=None):
    f = st.get("export") or "area-0.dtva.gz"
    q = f"src=../api/preview/{job}/{f}"
    if pick:
        q += f"&pickcol={pick[0]},{pick[1]}"
    return f"area3d/?{q}"


def for_page(st):
    """A status as the workbench page's Stage2.render wants it: `status_url` while it runs, `url`
    (the viewer on the export) when done, `message`/`error` otherwise; `note` always."""
    out = dict(st)
    out.pop("spec", None)
    job = st.get("job")
    if st.get("state") == "done":
        out["url"] = viewer_url(job, st, (st.get("spec") or {}).get("pick"))
    elif st.get("state") == "failed":
        out["error"] = st.get("error") or st.get("message") or "failed"
    elif st.get("state") != "unknown":
        out["status_url"] = f"api/preview/{job}"
    return out


def workbench_preview(req):
    run = req.get("run", "?")
    request = {k: req.get(k) for k in ("run", "frame", "t_years", "box", "lat_lon")}
    bake, why = run_bake(run)
    profile = os.environ.get("DEEPTIME_WORKBENCH_PROFILE", "mic")
    if bake:
        box, shrink = box_on_world(req, bake["c"])
        spec = {"profile": profile, "size": bake["c"], "boxes": [box],
                "world": {"world_data": bake["dir"]},
                "note": shrink + run_terrain_note(run, bake), "note_kind": "run", "request": request}
    else:
        box, shrink = box_on_world(req)
        spec = {"profile": profile, "size": PREVIEW_C, "boxes": [box],
                "world": {"preset": PREVIEW_PRESET, "start": "supercontinent", "length_myr": PREVIEW_LENGTH_MYR},
                "note": shrink + preview_world_note(run, why), "note_kind": "test-world", "request": request}
    return 202, for_page(submit(spec))


def deeptime_bin():
    p = os.environ.get(DEEPTIME_ENV)
    if p and os.path.isfile(p):
        return p
    for rel in ("release", "debug"):
        p = os.path.join(REPO, "target", rel, "deeptime")
        if os.path.isfile(p):
            return p
    return shutil.which("deeptime")


def inspect_bake(run, bake, x, z):
    """One column of a run's bake through `deeptime inspect-column --why`: the evaluator behind the
    game's terrain and strata (parity-tested against the Java reader), in milliseconds."""
    exe = deeptime_bin()
    if not exe:
        raise ValueError("no deeptime binary to ask (set " + DEEPTIME_ENV + ")")
    t = time.time()
    p = subprocess.run([exe, "inspect-column", "--world-data", bake["dir"], "--why", str(x), str(z)],
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=120)
    if p.returncode != 0:
        raise ValueError("deeptime inspect-column: " + p.stderr.decode(errors="replace").strip()[-400:])
    col = json.loads(p.stdout)
    return {"source": "bake", "run": run, "column": {"x": x, "z": z}, "note_kind": "run",
            "note": run_terrain_note(run, bake) + " This answer is what Deep Time decides for the column, not what "
                    "other mods' features add to it; the 3D preview shows the placed blocks.",
            "terrain_height": round(col["surface"], 1), "top_y": col["top_y"], "circumference": bake["c"],
            "seconds": round(time.time() - t, 3), "inspect": col}


def workbench_inspect(q):
    """One column of a run. A run with a bake answers at once from it (inspect_bake). Otherwise,
    as before P1-08: from a finished test-world preview that holds the column (the viewer opens with
    the column's top block selected), else the preview being made of it, else a new 32 × 32
    preview around it."""
    run = q.get("run", "?")
    bake, why = run_bake(run)
    c = bake["c"] if bake else PREVIEW_C
    try:
        if bake and q.get("x") not in (None, "") and q.get("z") not in (None, ""):
            x, z = int(float(q["x"])), int(float(q["z"]))
        elif q.get("lat") not in (None, "") and q.get("lon") not in (None, ""):
            x, z = latlon_to_block(float(q["lat"]), float(q["lon"]), c)
        else:
            x, z = int(q["x"]), int(q["z"])
    except (KeyError, ValueError):
        raise ValueError("give x and z (or lat and lon)")
    x = fold(x, c)
    if bake:
        return 200, inspect_bake(run, bake, x, z)
    if abs(z) > PREVIEW_C // 4 - INSPECT_SIDE:
        raise ValueError(f"z {z} is past the preview planet's pole cap")
    for st in list_jobs():
        spec = st.get("spec") or {}
        if spec.get("size") != PREVIEW_C or not spec.get("boxes") or (spec.get("world") or {}).get("preset") != PREVIEW_PRESET:
            continue
        bx0, bz0, bx1, bz1 = spec["boxes"][0]
        inside = (x - bx0) % PREVIEW_C < bx1 - bx0 and bz0 <= z < bz1
        if not inside or st.get("state") == "failed":
            continue
        out = for_page(st)
        if st.get("state") == "done":
            out["url"] = viewer_url(st["job"], st, (x, z))
        out["column"] = {"x": x, "z": z}
        out["note"] = out.get("note") or preview_world_note(run, why)
        out["note_kind"] = "test-world"
        return 200, out
    h = INSPECT_SIDE // 2
    spec = {"profile": os.environ.get("DEEPTIME_WORKBENCH_PROFILE", "mic"), "size": PREVIEW_C,
            "boxes": [[x - h, z - h, x + h, z + h]], "pick": [x, z],
            "world": {"preset": PREVIEW_PRESET, "start": "supercontinent", "length_myr": PREVIEW_LENGTH_MYR},
            "note": ("There is no fast column answer for this run; this generates the 32 × 32 blocks around the "
                     "column of a test world (about a minute) and opens it with the column's top block selected. ")
            + preview_world_note(run, why),
            "note_kind": "test-world",
            "request": {"run": run, "frame": q.get("frame"), "x": q.get("x"), "z": q.get("z")}}
    out = for_page(submit(spec))
    out["column"] = {"x": x, "z": z}
    return 202, out


def static_file(name):
    f = os.path.realpath(os.path.join(VIEWER, name or "index.html"))
    if not (f.startswith(os.path.realpath(VIEWER) + os.sep) and os.path.isfile(f)):
        return None
    ctype = {"html": "text/html; charset=utf-8", "js": "text/javascript; charset=utf-8",
             "css": "text/css; charset=utf-8", "json": "application/json"}.get(f.rsplit(".", 1)[-1], "application/octet-stream")
    return 200, ctype, open(f, "rb").read()


def api(method, path, body=b""):
    """The HTTP contract (README.md) for a server to mount: returns (status, content type, bytes).

    POST /api/preview        body: a spec, or the workbench's {run, frame, t_years, box, lat_lon}
                                                   -> 202 {"job", "status_url", "note", ...}
    GET  /api/preview                              -> 200 [status, …] newest first
    GET  /api/preview/<job>                        -> 200 the status (+ `url` when done, `status_url` while running)
    GET  /api/preview/<job>/<file>                 -> the job's files (area-0.dtva.gz, probes.jsonl, logs)
    GET  /api/inspect?run=&frame=&x=&z=[&lat=&lon=] -> one column (see workbench_inspect)
    GET  /area3d/[<file>]                          -> the viewer
    """
    def js(code, obj):
        return code, "application/json", json.dumps(obj, indent=1).encode()

    from urllib.parse import parse_qsl
    route, _, query = path.partition("?")
    parts = [p for p in route.split("/") if p]
    if parts[:1] == ["area3d"] and method == "GET":
        return static_file("/".join(parts[1:])) or js(404, {"error": "not found"})
    try:
        if parts == ["api", "inspect"] and method == "GET":
            return js(*workbench_inspect(dict(parse_qsl(query))))
        if parts[:2] != ["api", "preview"]:
            return js(404, {"error": "not found"})
        if method == "POST" and len(parts) == 2:
            req = json.loads(body or b"{}")
            if "run" in req and "boxes" not in req:
                return js(*workbench_preview(req))
            st = submit(req)
            st["status_url"] = f"api/preview/{st['job']}"
            return js(202, st)
        if method == "GET" and len(parts) == 2:
            return js(200, list_jobs())
        if method == "GET" and len(parts) == 3:
            st = status(parts[2])
            return js(200 if st.get("state") != "unknown" else 404, for_page(st))
        if method == "GET" and len(parts) == 4:
            d = job_dir(parts[2])
            name = parts[3]
            if not re.fullmatch(r"[A-Za-z0-9._-]+", name):
                return js(404, {"error": "not found"})
            p = os.path.join(d, name)
            if not os.path.isfile(p):
                return js(404, {"error": "not found"})
            ctype = {"json": "application/json", "jsonl": "application/x-ndjson", "log": "text/plain",
                     "out": "text/plain"}.get(name.rsplit(".", 1)[-1], "application/octet-stream")
            return 200, ctype, open(p, "rb").read()
    except Busy as e:
        return js(503, {"error": str(e)})
    except ValueError as e:
        return js(400, {"error": str(e)})
    return js(405, {"error": "method not allowed"})


def http_once(method, path):
    """`preview.py http METHOD PATH`: one request through api() for a server that proxies (the
    workbench's Rust server): the body on stdin; on stdout one JSON line {"status", "content_type",
    "length"} and then exactly `length` bytes of body."""
    body = sys.stdin.buffer.read() if method == "POST" else b""
    out = sys.stdout.buffer
    sys.stdout = sys.stderr   # progress lines (Job.say) must not reach the protocol stream
    try:
        code, ctype, data = api(method, path, body)
    except Exception as e:   # never leave the proxy without an answer
        code, ctype, data = 500, "application/json", json.dumps({"error": f"preview.py: {e}"}).encode()
    out.write(json.dumps({"status": code, "content_type": ctype, "length": len(data)}).encode() + b"\n")
    out.write(data)
    out.flush()


def serve(host, port):
    """The contract over HTTP plus the viewer at /area3d/ (a test harness for the workbench)."""
    from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

    class H(BaseHTTPRequestHandler):
        def _send(self, code, ctype, data):
            self.send_response(code)
            self.send_header("Content-Type", ctype)
            self.send_header("Content-Length", str(len(data)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(data)

        def do_GET(self):
            if self.path.split("?")[0] in ("/", "/area3d"):
                self.send_response(302)
                self.send_header("Location", "/area3d/")
                self.end_headers()
                return
            self._send(*api("GET", self.path))

        def do_POST(self):
            n = int(self.headers.get("Content-Length") or 0)
            self._send(*api("POST", self.path, self.rfile.read(n)))

    print(f"serving the preview contract and the viewer on http://{host}:{port}/ (jobs in {jobs_root()})", flush=True)
    ThreadingHTTPServer((host, port), H).serve_forever()


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)

    def spec_args(p):
        p.add_argument("--profile", default="standalone", choices=PROFILES)
        p.add_argument("--size", type=int, default=16384)
        p.add_argument("--box", action="append", help="x0,z0,x1,z1 (x1, z1 exclusive); repeatable")
        p.add_argument("--y", help="lo,hi inclusive (default -512,639)")
        p.add_argument("--synth")
        p.add_argument("--strata")
        p.add_argument("--preset")
        p.add_argument("--start")
        p.add_argument("--length-myr", type=float)
        p.add_argument("--world-data")
        p.add_argument("--test-terrain")
        p.add_argument("--level-type", help="not a Deep Time world: the pack's generator, e.g. minecraft:normal")
        p.add_argument("--locate", action="append", help="a biome id to centre a box on (with --level-type)")
        p.add_argument("--locate-side", type=int)
        p.add_argument("--jvm-prop", dest="jvm_props", action="append",
                       help="deeptime.<name>=<value>, a system property for the server (e.g. deeptime.steepScale=0)")
        p.add_argument("--datapack", action="append")
        p.add_argument("--seed", type=int)
        p.add_argument("--no-mic-rock-tags", action="store_true")
        p.add_argument("--keep-world", action="store_true")
        p.add_argument("--max-side", type=int, default=MAX_SIDE)

    r = sub.add_parser("run")
    spec_args(r)
    r.add_argument("--job")
    r.add_argument("--from-spec", action="store_true", help="read <job>/spec.json (what submit wrote)")
    r.add_argument("--lock-held", action="store_true", help="the caller (mac.sh --server) holds the server lock")
    r.add_argument("--install", action="store_true", help="run tools/smoke/install.sh <profile> first")
    s = sub.add_parser("submit")
    spec_args(s)
    s.add_argument("--job")
    st = sub.add_parser("status")
    st.add_argument("job")
    sub.add_parser("list")
    rj = sub.add_parser("refresh-jar", help="bring a profile's Deep Time jar up to date (no server, no job)")
    rj.add_argument("--profile", default="mic", choices=PROFILES)
    sv = sub.add_parser("serve")
    sv.add_argument("--host", default="127.0.0.1")
    sv.add_argument("--port", type=int, default=8765)
    hp = sub.add_parser("http", help="one request through the contract (the workbench server's proxy)")
    hp.add_argument("method", choices=["GET", "POST"])
    hp.add_argument("path")
    a = ap.parse_args()
    if a.cmd == "http":
        return http_once(a.method, a.path)

    if a.cmd == "status":
        print(json.dumps(status(a.job), indent=1))
    elif a.cmd == "list":
        print(json.dumps(list_jobs(), indent=1))
    elif a.cmd == "refresh-jar":
        smoke = os.path.join(REPO, "smoke", a.profile)
        if not os.path.exists(os.path.join(smoke, "install.json")):
            sys.exit(f"preview.py: no {smoke}/install.json: run tools/smoke/install.sh {a.profile}")
        # Not a job: its folder is a throwaway, so the workbench's job list never shows it.
        os.environ["DEEPTIME_WORKBENCH_JOBS"] = tempfile.mkdtemp(prefix="deeptime-refresh-jar-")
        j = Job("refresh-jar", {})
        try:
            refresh_deeptime_jar(j, smoke)
        except RuntimeError as e:
            sys.exit(f"preview.py: {e}")
        print(json.dumps(j.st["jar"], indent=1))
    elif a.cmd == "serve":
        serve(a.host, a.port)
    elif a.cmd == "submit":
        try:
            print(json.dumps(submit(spec_from_args(a), a.job), indent=1))
        except ValueError as e:
            sys.exit(f"preview.py: {e}")
    elif a.cmd == "run":
        job = a.job or new_job_id()
        try:
            if a.from_spec:
                spec = normalise_spec(json.load(open(os.path.join(job_dir(job), "spec.json"))))
            else:
                spec = normalise_spec(spec_from_args(a), a.max_side)
        except ValueError as e:
            sys.exit(f"preview.py: {e}")
        spec["_install"] = a.install
        j = Job(job, spec)
        j.set("queued", "runner started")
        rc = run_job(j, lock_held=a.lock_held)
        print(json.dumps(j.st, indent=1))
        sys.exit(rc)


if __name__ == "__main__":
    main()
