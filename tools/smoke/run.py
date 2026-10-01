#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Boot a Deep Time server set headlessly, drive it over RCON, and write a report (plan §9.5).

    tools/smoke/run.py --profile <standalone|mic|terrablender>
        [--fresh]                 delete the world first
        [--level-type <preset>]   override server.properties' level-type (boot-only runs,
                                  e.g. minecraft:normal before Deep Time's preset exists)
        [--chunky <radius>]       pregenerate a square of this radius (Chunky)
        [--chunky-center <x,z>|seam]  its centre (default 0,0); `seam` is x = C/2, z = 0, so the
                                  square straddles the seam and the region check below proves
                                  nothing was written outside the canonical span (P0-06)
        [--chunky-at <x,z,r>]     one more Chunky square (repeatable; Q10 renders two sites)
        [--chunky-seam]           also pregenerate at least 128 columns either side of the
                                  seam (x = C/2), z in [-64, 80), inside the canonical span
                                  (deeptime-inspect seam, plan §9.7)
        [--dh]                    Distant Horizons far terrain (P3-04; profile dh, dh-next or mic): the
                                  override registers (or Toroidal World refuses it, recorded), DH's own
                                  `/dh pregen` is fed from the world data on the server alone, and what
                                  DH stores agrees with the chunks generated afterwards (dh_checks)
        [--restart]               stop, start again and confirm the world data was not regenerated
        [--scenario <name>]       a first-start stop scenario instead of a normal boot (P1-13):
                                  first_start_no_config, first_start_circumference_mismatch or
                                  first_start_native_failure (see SCENARIOS); implies a fresh world
        [--timeout <s>]           how long to wait for "Done (" (default 1500)

Adapted from mic-climate's smoke/run.py (MIT, same author). Standard library only; the JDK and
mcrcon come from smoke/tools.env, which install.sh resolves out of nixpkgs once.

What it proves, in order:
 1. The set boots: no mixin of Deep Time or Toroidal World fails (defaultRequire 1 kills
    the launch otherwise). Anything that looks like a mixin failure or a crash ends the run at
    once instead of after the timeout.
 2. Every Deep Time mixin was applied, by name (TW's are counted): the server runs with
    -Dmixin.debug.verbose=true and each "Mixing X from Y into Z" line is matched against the
    config's declared list. A declared mixin that never applied is listed.
 3. The Deep Time probes answer (from P0-05 on) and nothing is written outside the canonical span.
 4. First-start generation (P1-13): when the world had no data, the [deeptime] lines from
    "generating" to "world data written" are at most 5 s apart (plus log latency) and come before
    "Done ("; the report records the generation time. --restart then expects "world data present".

The exit code is 0 only if every check passed. smoke/<profile>/report.md is written either way.
"""

import argparse
import io
import json
import os
import re
import signal
import subprocess
import sys
import threading
import time
import zipfile
from datetime import datetime, timezone

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import jarstate  # noqa: E402  (what install.sh recorded about the Deep Time jar)

FATAL = re.compile(
    r"InjectionError|MixinTransformerError|MixinApplyError|Mixin apply failed"
    r"|InvalidInjectionException|Critical injection failure|Preparing crash report"
    r"|Failed to start the minecraft server|Mod [Ll]oading has failed|Cannot find class"
)
DONE = re.compile(r"Done \(")
# RCON comes up after "Done (", sometimes tens of seconds later: wait for it before any command.
RCON_UP = re.compile(r"RCON running on")
MIXING = re.compile(r"Mixing (\S+) from (\S+\.mixins\.json) into (\S+)")
PROBE = re.compile(r"\[deeptime-probe\]\s*(\{.*\})")
# First-start generation (P1-13, FirstStartGenerator's lines).
GEN_START = re.compile(r"\[deeptime\] generating the world's history")
GEN_PROGRESS = re.compile(r"\[deeptime\] .* elapsed")
GEN_DONE = re.compile(r"\[deeptime\] world data written to")
DATA_PRESENT = re.compile(r"\[deeptime\] world data present at")
STOPPING = re.compile(r"\[deeptime\] The server stops before creating the world")
# The largest gap allowed between two generation lines: the 5 s interval plus log latency.
MAX_PROGRESS_GAP_S = 5.75
# Configs whose every declared mixin must have applied by "Done (": Deep Time's own, whose
# targets are all loaded during boot (worldgen, commands, servers). Toroidal World's configs are
# recorded only: many of its mixins target classes an idle server never loads (villager AI,
# dragon phases, raids), and Mixin applies a mixin when its target class loads.
REQUIRED_CONFIGS = ["deeptime.mixins.json"]
RECORDED_CONFIGS = ["toroidal_world.mixins.json", "toroidal_world.shape.mixins.json"]


def load_env(path):
    env = {}
    for line in open(path):
        if "=" in line:
            k, v = line.strip().split("=", 1)
            env[k] = v
    return env


class Server:
    def __init__(self, server_dir, java, neoforge_version, log_path):
        args_file = os.path.join(server_dir, "libraries", "net", "neoforged", "neoforge",
                                 neoforge_version, "unix_args.txt")
        if not os.path.exists(args_file):
            sys.exit(f"run.py: {args_file} is missing -- run tools/smoke/install.sh")
        self.dir = server_dir
        self.command = [java, "@user_jvm_args.txt", "-Dmixin.debug.verbose=true",
                        f"@{args_file}", "--nogui"]
        self.log_path = log_path
        self.lines, self.fatal, self.done_at, self.started, self._proc = [], None, None, None, None
        self.timed = []

    def start(self):
        self.lines, self.fatal, self.done_at, self.rcon_up = [], None, None, False
        self.timed = []  # (arrival time, line) of every Deep Time line, for the progress cadence
        self.started = time.time()
        self._log = open(self.log_path, "a", buffering=1, errors="replace")
        self._log.write(f"\n===== start {datetime.now(timezone.utc).isoformat()} =====\n")
        self._proc = subprocess.Popen(self.command, cwd=self.dir, stdout=subprocess.PIPE,
                                      stderr=subprocess.STDOUT, text=True, errors="replace",
                                      bufsize=1, start_new_session=True)
        threading.Thread(target=self._read, daemon=True).start()
        if not os.path.exists(f"/proc/{self._proc.pid}/status"):
            threading.Thread(target=self._sample_rss, daemon=True).start()

    def _read(self):
        for line in self._proc.stdout:
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

    def rss_now_kb(self):
        """The JVM's resident set size now (ps works on Linux and macOS), or None."""
        if self._proc is None or self._proc.poll() is not None:
            return None
        r = subprocess.run(["ps", "-o", "rss=", "-p", str(self._proc.pid)], capture_output=True, text=True)
        return int(r.stdout.strip()) if r.stdout.strip().isdigit() else None

    def done_seconds(self):
        """The server's own "Done (12.345s)!" figure (world loading, after mod loading), or None."""
        for line in self.lines:
            m = re.search(r"Done \((\d+(?:\.\d+)?)s\)", line)
            if m:
                return float(m.group(1))
        return None

    def peak_rss_kb(self):
        """VmHWM on Linux; elsewhere (macOS) the highest RSS the sampler thread saw."""
        try:
            for line in open(f"/proc/{self._proc.pid}/status"):
                if line.startswith("VmHWM:"):
                    return int(line.split()[1])
        except (OSError, AttributeError):
            pass
        return getattr(self, "_peak_sampled", None)

    def _sample_rss(self):
        # macOS has no /proc: sample the JVM's RSS every 2 s while it runs.
        self._peak_sampled = 0
        while self._proc.poll() is None:
            r = subprocess.run(["ps", "-o", "rss=", "-p", str(self._proc.pid)], capture_output=True, text=True)
            if r.stdout.strip().isdigit():
                self._peak_sampled = max(self._peak_sampled, int(r.stdout.strip()))
            time.sleep(2)

    def wait_for_done(self, timeout):
        deadline, last = time.time() + timeout, 0.0
        while time.time() < deadline:
            if self.done_at and self.rcon_up:
                return True
            if self.fatal:
                return False
            if self._proc.poll() is not None:
                self.fatal = f"the server exited with status {self._proc.returncode} before it finished starting"
                return False
            if time.time() - last > 30:
                last = time.time()
                print(f"   ... {int(last - self.started)}s, {len(self.lines)} log lines", flush=True)
            time.sleep(0.25)
        self.fatal = f'timed out after {timeout}s waiting for "Done ("'
        return False

    def wait_for_exit(self, timeout):
        """For a run that must stop by itself: the exit status, or None on timeout (then killed)."""
        try:
            return self._proc.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(os.getpgid(self._proc.pid), signal.SIGKILL)
            self._proc.wait(timeout=60)
            return None
        finally:
            time.sleep(0.5)  # let the reader thread drain the pipe

    def stop(self, rcon):
        if self._proc is None or self._proc.poll() is not None:
            return
        try:
            rcon("stop", check=False)
        except Exception:
            pass
        try:
            self._proc.wait(timeout=180)
            return
        except subprocess.TimeoutExpired:
            pass
        os.killpg(os.getpgid(self._proc.pid), signal.SIGTERM)
        try:
            self._proc.wait(timeout=60)
        except subprocess.TimeoutExpired:
            os.killpg(os.getpgid(self._proc.pid), signal.SIGKILL)

    def applied(self, config):
        found = {}
        for line in self.lines:
            m = MIXING.search(line)
            if m and m.group(2) == config:
                found[m.group(1)] = m.group(3)
        return found

    def grep(self, pattern, limit=10):
        rx = re.compile(pattern)
        return [l.rstrip() for l in self.lines if rx.search(l)][:limit]


def declared_mixins(mods_dir, config):
    """The `mixins` (+ `server`) arrays of a config, searched through jars and their jar-in-jars."""
    def search(zf, name):
        names = zf.namelist()
        if config in names:
            data = json.loads(zf.read(config))
            return name, list(data.get("mixins", [])) + list(data.get("server", []))
        for inner in names:
            if inner.startswith("META-INF/jarjar/") and inner.endswith(".jar"):
                with zipfile.ZipFile(io.BytesIO(zf.read(inner))) as z2:
                    hit = search(z2, f"{name}!{os.path.basename(inner)}")
                    if hit:
                        return hit
        return None
    for name in sorted(os.listdir(mods_dir)):
        if name.endswith(".jar"):
            try:
                with zipfile.ZipFile(os.path.join(mods_dir, name)) as zf:
                    hit = search(zf, name)
                    if hit:
                        return hit
            except (zipfile.BadZipFile, ValueError):
                continue
    return None, []


class Rcon:
    def __init__(self, mcrcon, port, password):
        self.mcrcon, self.port, self.password = mcrcon, str(port), password

    def __call__(self, command, check=True):
        r = subprocess.run([self.mcrcon, "-H", "127.0.0.1", "-P", self.port, "-p", self.password,
                            "-c", command], capture_output=True, text=True, errors="replace",
                           timeout=300)
        if check and r.returncode != 0:
            raise RuntimeError(f"mcrcon failed for {command!r}: {r.stderr.strip() or r.stdout.strip()}")
        return r.stdout.strip()


class Steps:
    def __init__(self):
        self.rows = []
        self.compat = []   # P1-16: one row per `probe coe` / `probe cdg` (compat_row)

    def check(self, name, expected, actual, ok, note=""):
        self.rows.append((name, expected, str(actual), bool(ok), note))
        print(f"   [{'PASS' if ok else 'FAIL'}] {name}" + ("" if ok else f": expected {expected}; got {actual}"), flush=True)
        return ok

    def record(self, name, value, note=""):
        self.rows.append((name, "(recorded)", str(value), True, note))
        print(f"   [ -- ] {name}: {value}", flush=True)

    @property
    def failed(self):
        return [r for r in self.rows if not r[3]]


def compat_row(p):
    """One row of the report's COE/CDG table from a `probe coe` or `probe cdg` JSON line (P1-16)."""
    chunk = ",".join(str(v) for v in p.get("chunk", []))
    short = lambda s: (s or "none").split("/")[-1].split(":")[-1]
    if p["probe"] == "coe":
        sv, coe = p.get("site_vein"), p.get("coe") or {}
        site = f"{sv['site']} {short(sv['type'])}" if sv else "none"
        decided = f"{short(sv['vein'])} q {sv['quantile']:.3f} ({sv['ore_blocks_in_chunk']} ore blocks)" if sv else "no site vein"
        mod = (f"{short(coe.get('vein'))} mul {coe.get('random_mul', 0):.3f} amount {coe.get('amount')}"
               + (" (infinite)" if coe.get("infinite") else "")) if coe else "COE not installed"
    else:
        trap, cdg = p.get("trap"), p.get("cdg") or {}
        site = f"{trap['site']} {short(trap['type'])}" if trap else "none"
        decided = (f"{p.get('deep_time_amount')} mB (charge {trap['charge_mb']} × {trap['cdg_scale']} × share {trap['share']:.3f})"
                   if trap else f"{p.get('deep_time_amount')} mB")
        mod = f"base {cdg.get('base')} amount {cdg.get('amount')} mB" if cdg else "CDG not installed"
    return (p["probe"].upper(), chunk, site, decided, mod, p.get("agrees"))


def probe_json(server, rcon, command, wait=1.0):
    """Runs a /deeptime probe over RCON and returns its JSON (from the server log: RCON cuts long replies)."""
    before = len(server.lines)
    out = rcon(command, check=False)
    time.sleep(wait)
    for line in reversed(server.lines[before:]):
        m = PROBE.search(line)
        if m:
            try:
                return json.loads(m.group(1)), out
            except ValueError:
                pass
    m = PROBE.search(out)
    try:
        return (json.loads(m.group(1)) if m else None), out
    except ValueError:
        return None, out


def dh_checks(steps, server, rcon, smoke, radius_chunks=32):
    """Distant Horizons far terrain (P3-04, plan §10.5), on a set that has Distant Horizons.

    1. `/deeptime probe dh`: the API bound, and the override for the overworld either registered or
       was refused by Toroidal World (TW 0.19.3 refuses foreign generators on wrapped levels once
       its Distant Horizons mixins apply, which they do from DH 3.3.1). A refusal is recorded, with
       TW's message, and ends the DH checks: Distant Horizons then generates from real chunks.
    2. `/dh pregen start minecraft:overworld 0 0 <radius>`: Distant Horizons' own server-side
       pregeneration asks the registered generator for the sections around the origin and stores
       them in its database; no player and no client is involved.
    3. `/deeptime probe dh check`: what Distant Horizons then holds is far terrain layer for layer,
       and far terrain agrees with the chunks generated afterwards at the same columns.
    4. `/deeptime probe far bench`: the time a section takes without Distant Horizons' storing.
    """
    st, out = probe_json(server, rcon, "deeptime probe dh")
    if not steps.check("dh: /deeptime probe dh answers", "[deeptime-probe] JSON", "ok" if st else out[:200], st is not None):
        return
    steps.check("dh: Distant Horizons is installed and far terrain is on", "installed and enabled",
                f"installed={st.get('installed')} enabled={st.get('enabled')}", st.get("installed") and st.get("enabled"))
    if not steps.check("dh: the API was bound", "bound", st.get("version", st.get("api_error")), st.get("api_bound") is True,
                       note=st.get("api_naming", "")):
        return
    steps.record("dh: Distant Horizons version and API naming", f"{st.get('version')}, {st.get('api_naming')}")
    level = next((l for l in st.get("levels", []) if l.get("level") == "minecraft:overworld"), None)
    if not steps.check("dh: the overworld was offered to Distant Horizons", "a levels entry", json.dumps(level)[:200], level is not None):
        return
    refused_by_tw = not level["registered"] and "Toroidal World" in (level.get("message") or "")
    steps.check("dh: the override registered, or Toroidal World refused it", "registered | refused by Toroidal World",
                "registered" if level["registered"] else f"refused: {level.get('message')}", level["registered"] or refused_by_tw)
    steps.record("dh: far terrain override", "registered: Distant Horizons' LODs come from the world data" if level["registered"]
                 else "REFUSED by Toroidal World: Distant Horizons generates its LODs from real chunks")
    if not level["registered"]:
        return

    # Beside the spawn (land, by the spawn rule) but clear of the spawn chunks, which exist already:
    # Distant Horizons holds a generated chunk's own columns, not the generator's.
    spawn = st.get("spawn") or [0, 0]
    cx, cz = (spawn[0] + 768) // 64 * 64, spawn[1] // 64 * 64
    steps.record("dh: area checked", f"centre {cx}, {cz} (spawn {spawn[0]}, {spawn[1]}), pregen radius {radius_chunks} chunks")
    out = rcon(f"dh pregen start minecraft:overworld {cx} {cz} {radius_chunks}", check=False)
    print(f"   dh pregen start: {out[:160]}", flush=True)
    t0 = time.time()
    deadline = t0 + 900
    seen = 0
    while time.time() < deadline:
        time.sleep(5)
        status = rcon("dh pregen status", check=False)
        if "not running" in status.lower():
            break
        seen += 1
        if seen % 6 == 0:
            print(f"   dh pregen status: {status[:160]}", flush=True)
    pregen_s = time.time() - t0
    finished = time.time() < deadline
    steps.check("dh: Distant Horizons' pregeneration finished", "finished", f"{pregen_s:.0f} s" if finished else "timeout", finished)
    time.sleep(5)   # let DH flush the last sections to its store

    st, out = probe_json(server, rcon, "deeptime probe dh")
    level = next((l for l in (st or {}).get("levels", []) if l.get("level") == "minecraft:overworld"), {}) if st else {}
    sections = level.get("sections", 0)
    steps.check("dh: Distant Horizons asked the override for sections", "> 0", sections, sections > 0)
    steps.check("dh: no section failed", "0", f"{level.get('failures')}" + (f" ({level.get('first_failure')})" if level.get("first_failure") else ""),
                level.get("failures") == 0)
    steps.record("dh: ms per section inside Distant Horizons (mean)", level.get("mean_ms_per_section"),
                 note=f"{sections} sections in {pregen_s:.0f} s wall; {level.get('by_detail')}; "
                      f"{level.get('points_per_column')} data points a column")

    side = 24
    chk, out = probe_json(server, rcon, f"deeptime probe dh check {cx - 192} {cz - 192} {side}", wait=3.0)
    if not steps.check("dh: /deeptime probe dh check answers", "[deeptime-probe] JSON", "ok" if chk else out[:200], chk is not None):
        return
    with open(os.path.join(smoke, "dh-check.json"), "w") as f:
        json.dump({"status": st, "check": chk}, f, indent=1)
    stored, vs = chk.get("stored", {}), chk.get("vs_real", {})
    asked, held, equal = stored.get("asked", 0), stored.get("held", 0), stored.get("equal_to_far_terrain", 0)
    steps.check("dh: Distant Horizons holds the columns it was given", f">= 90 % of {asked}", held, asked > 0 and held >= 0.9 * asked)
    steps.check("dh: what it holds is far terrain, layer for layer", f"{held} of {held}", f"{equal} of {held}",
                held > 0 and equal == held, note=stored.get("differences", ""))
    n = vs.get("columns", 0)
    share = lambda k: f"{vs.get(k, 0)} of {n} ({100.0 * vs.get(k, 0) / n:.1f} %)" if n else "no columns"
    steps.check("dh: ground within 1 block of the generated chunk", ">= 90 %", share("height_within_1"),
                n > 0 and vs.get("height_within_1", 0) >= 0.9 * n, note=f"max {vs.get('height_max_abs')}: {vs.get('height_worst')}")
    steps.check("dh: water or dry as in the generated chunk", ">= 95 %", share("water_agree"), n > 0 and vs.get("water_agree", 0) >= 0.95 * n)
    steps.check("dh: biome as in the generated chunk", ">= 95 %", share("biome_same"), n > 0 and vs.get("biome_same", 0) >= 0.95 * n,
                note="; ".join(vs.get("biome_differences", [])))
    steps.record("dh: ground exact", share("height_exact"))
    steps.record("dh: ground block as in the generated chunk", share("ground_block_same"),
                 note="; ".join(vs.get("ground_block_differences", []))[:400])
    steps.record("dh: snow layer as in the generated chunk", share("snow_layer_agree"))

    for detail in (0, 3, 6):
        b, out = probe_json(server, rcon, f"deeptime probe far bench {cx} {cz} {detail} 64", wait=2.0)
        if b and "ms_per_section" in b:
            steps.record(f"far terrain: ms per section at detail {detail} (64 sections, one thread, no Distant Horizons)",
                         b["ms_per_section"], note=f"{b.get('layers_per_column')} layers a column")


def port_free(port):
    import socket
    with socket.socket() as s:
        return s.connect_ex(("127.0.0.1", port)) != 0


def region_chunks(path):
    """Chunks stored in a region file: non-zero entries of its 4 KiB location table."""
    if os.path.getsize(path) < 4096:
        return 0
    import struct
    header = open(path, "rb").read(4096)
    return sum(1 for i in range(1024) if struct.unpack_from(">I", header, 4 * i)[0] != 0)


def region_span_check(world, circumference):
    """Every region file that stores a chunk must satisfy -C/1024 <= rx < C/1024 (plan §9.5).

    Checked in region/, entities/ and poi/. Vanilla creates an empty .mca whenever it opens a
    region file for a read (structure checks, "is this chunk generated" lookups), and on a wrapped
    world some of those reads use raw coordinates past the edge (P0-06: eight 0-byte files at
    rx = 32 and -33 after a Chunky square across the 32k seam). Such a file stores nothing, so it
    is recorded, not counted. Returns (files with chunks, of them outside the span, empty files
    outside the span).
    """
    half = circumference // 1024
    outside, empty_outside, total = [], [], 0
    for sub in ("region", "entities", "poi"):
        d = os.path.join(world, sub)
        if not os.path.isdir(d):
            continue
        for f in os.listdir(d):
            m = re.match(r"r\.(-?\d+)\.(-?\d+)\.mca$", f)
            if not m:
                continue
            inside = -half <= int(m.group(1)) < half
            if region_chunks(os.path.join(d, f)) == 0:
                if not inside:
                    empty_outside.append(f"{sub}/{f}")
                continue
            total += 1
            if not inside:
                outside.append(f"{sub}/{f}")
    return total, outside, empty_outside


def chunky_center(spec, circumference):
    """`x,z`, or `seam` for the seam at x = C/2 (the square then straddles it)."""
    if spec == "seam":
        return circumference // 2, 0
    x, z = spec.split(",")
    return int(x), int(z)


def check_generation(steps, server, install, label=""):
    """The first-start generation of this boot, if any (P1-13). Returns the generation time or None."""
    starts = [t for t, l in server.timed if GEN_START.search(l)]
    if not starts:
        return None
    t0 = starts[0]
    done = [t for t, l in server.timed if GEN_DONE.search(l)]
    steps.check(f"{label}first start generated the world data", "[deeptime] world data written",
                "written" if done else "no 'world data written' line", bool(done))
    if not done:
        return None
    t1 = done[0]
    marks = [t for t, l in server.timed if t0 <= t <= t1 and (GEN_START.search(l) or GEN_PROGRESS.search(l) or GEN_DONE.search(l))]
    gaps = [b - a for a, b in zip(marks, marks[1:])]
    progress = [l for t, l in server.timed if t0 <= t <= t1 and GEN_PROGRESS.search(l)]
    # A generation shorter than the gap needs no progress line (P1-15's synthetic deposit world
    # generates in about 2 s): the start and done lines are then the only marks.
    steps.check(f"{label}[deeptime] progress lines at most 5 s apart", f"max gap <= {MAX_PROGRESS_GAP_S} s",
                f"{len(progress)} progress lines, max gap {max(gaps, default=0):.2f} s",
                (bool(progress) or t1 - t0 <= MAX_PROGRESS_GAP_S) and max(gaps, default=0) <= MAX_PROGRESS_GAP_S)
    steps.check(f"{label}generation finished before Done (", "before", "before" if server.done_at and t1 <= server.done_at else "after",
                bool(server.done_at) and t1 <= server.done_at)
    steps.record(f"{label}generation time (s)", f"{t1 - t0:.1f}")
    for l in progress[:1] + progress[-1:]:
        steps.record(f"{label}progress line", l.split("[deeptime] ", 1)[-1])
    return t1 - t0


# A scenario boots a world without data and expects Deep Time to stop the server before
# loadLevel() (plan §6.6, §7.13): exit status 1, no Done, no region files, no world
# data, no crash report. Each prepares the server and restores it afterwards.
SCENARIOS = {
    "first_start_no_config": "config/deeptime/server.json missing -> template written, clean stop",
    "first_start_circumference_mismatch": "a preset for the other circumference -> clean stop naming both",
    "first_start_native_failure": "-Ddeeptime.natives.forceFail=unsupported -> guidance, clean stop",
}


def run_scenario(name, a, smoke, server_dir, world, install, tools):
    import shutil
    config = os.path.join(server_dir, "config", "deeptime", "server.json")
    jvm_args = os.path.join(server_dir, "user_jvm_args.txt")
    backups = []

    def backup(path):
        saved = path + ".smoke-backup"
        if os.path.exists(path):
            shutil.copy2(path, saved)
        backups.append((path, saved if os.path.exists(saved) else None))

    size = install["size"]
    other = 32768 if size == 16384 else 16384
    if name == "first_start_no_config":
        backup(config)
        if os.path.exists(config):
            os.remove(config)
        # Keep the template the mod writes (the restore puts the smoke config back over it).
        watch_template(server_dir, os.path.join(smoke, "logs", "server.json.template"))
    elif name == "first_start_circumference_mismatch":
        backup(config)
        os.makedirs(os.path.dirname(config), exist_ok=True)
        wrong = "earthlike_quick_32k" if size == 16384 else "earthlike_quick_16k"
        with open(config, "w") as f:
            json.dump({"preset": wrong, "timeline": {"start": "random", "length_myr": 500}}, f, indent=2)
    elif name == "first_start_native_failure":
        backup(jvm_args)
        with open(jvm_args, "a") as f:
            f.write("-Ddeeptime.natives.forceFail=unsupported\n")
        if not os.path.exists(config):
            sys.exit("run.py: first_start_native_failure needs a server.json (install.sh --preset ...)")
    else:
        sys.exit(f"run.py: unknown scenario {name}; one of {', '.join(SCENARIOS)}")

    crash_dir = os.path.join(server_dir, "crash-reports")
    crashes_before = set(os.listdir(crash_dir)) if os.path.isdir(crash_dir) else set()
    log_path = os.path.join(smoke, "logs", f"{name}.log")
    open(log_path, "w").close()
    server = Server(server_dir, tools["JAVA"], install["neoforge"], log_path)
    steps = Steps()
    started = datetime.now(timezone.utc)
    print(f"== scenario {name}: {SCENARIOS[name]}", flush=True)
    try:
        server.start()
        status = server.wait_for_exit(a.timeout)
    finally:
        for path, saved in backups:
            if saved:
                shutil.move(saved, path)
            elif os.path.exists(path):
                os.remove(path)
    text = "".join(server.lines)
    crashes = (set(os.listdir(crash_dir)) if os.path.isdir(crash_dir) else set()) - crashes_before
    steps.check("the server exits by itself with status 1", "1", status, status == 1)
    steps.check("no Done (", "absent", "present" if server.done_at else "absent", server.done_at is None)
    steps.check("Deep Time explains the stop", "[deeptime] The server stops before creating the world",
                next((l for l in server.lines if STOPPING.search(l)), "absent").strip()[:200], bool(STOPPING.search(text)))
    steps.check("stopped before loadLevel()", "no 'Preparing level'", "present" if "Preparing level" in text else "absent",
                "Preparing level" not in text)
    half = [p for p in ("region", "entities", "poi", "data", "dimensions", "DIM-1", "DIM1", "deeptime", "deeptime.generating")
            if os.path.exists(os.path.join(world, p))]
    steps.check("no half-made world saved", "no region/, entities/, data/, dimensions/, deeptime/", ", ".join(half) or "none",
                not half)
    # Vanilla's Main writes level.dat (the world's settings) before the server thread starts, so it
    # exists before any mod event; nothing may write it again at the stop.
    level_dat = os.path.join(world, "level.dat")
    stop_at = next((t for t, l in server.timed if STOPPING.search(l)), None)
    mtime = os.path.getmtime(level_dat) if os.path.exists(level_dat) else None
    steps.check("level.dat is only vanilla Main's (not rewritten at the stop)", "written before the stop line",
                "absent" if mtime is None else (f"{stop_at - mtime:.1f} s before the stop line" if stop_at else "no stop line"),
                mtime is None or (stop_at is not None and mtime < stop_at))
    noise = [l.strip()[:160] for l in server.lines if re.search(
        r"Encountered an unexpected exception|Exception stopping the server|This crash report has been saved|"
        r"Failed to start the minecraft server", l)]
    steps.check("no crash report or shutdown exception", "none", "; ".join(noise[:3]) or "none", not noise and not crashes,
                note=", ".join(sorted(crashes)))

    if name == "first_start_no_config":
        ok, why = False, "missing"
        template = os.path.join(server_dir, "config", "deeptime", "server.json")
        # The backup was restored over it; the log names what was written, and the scenario
        # parses the template the mod wrote, which run.py keeps next to the log.
        written = next((l for l in server.lines if "Wrote a template to" in l), None)
        steps.check("the template is announced", "Wrote a template to …/config/deeptime/server.json",
                    (written or "absent").strip()[:200], written is not None)
        kept = os.path.join(smoke, "logs", "server.json.template")
        if os.path.exists(kept):
            try:
                t = json.load(open(kept))
                ok = t.get("preset") == "" and "timeline" in t and "_preset" in t
                why = "valid JSON, preset empty, explanations in _ keys" if ok else f"unexpected: {sorted(t)}"
            except ValueError as e:
                why = f"not JSON: {e}"
        steps.check("the template is valid JSON", "valid JSON", why, ok)
    elif name == "first_start_circumference_mismatch":
        line = next((l for l in server.lines if "circumference mismatch" in l), "")
        both = f"C = {other}" in line and f"C = {size}" in line
        steps.check("the message names both circumferences", f"C = {other} … C = {size}", line.strip()[:220] or "absent", both)
    elif name == "first_start_native_failure":
        guidance = "Deep Time natives failed to load: UNSUPPORTED_PLATFORM" in text
        steps.check("the natives' guidance block", "UNSUPPORTED_PLATFORM + what to do", "present" if guidance else "absent",
                    guidance and "what to do:" in text)
        cli = next((l for l in server.lines if "deeptime run --preset" in l), "")
        steps.check("the CLI route names the command", "deeptime run --preset …", cli.strip()[:220] or "absent", bool(cli))

    passed = not steps.failed
    print(f"{name}: {'OK' if passed else 'FAILED'}", flush=True)
    with open(os.path.join(smoke, f"report-{name}.md"), "w") as r:
        r.write(f"# Deep Time smoke scenario — {name}\n\n")
        r.write(f"**{name}: {'OK' if passed else 'FAILED'}** · {started.strftime('%Y-%m-%d %H:%M UTC')} · "
                f"{SCENARIOS[name]}\n\n| Check | Expected | Actual | Result | Note |\n|---|---|---|---|---|\n")
        for n, exp, act, ok, note in steps.rows:
            act = act.replace("|", "\\|")
            r.write(f"| {n} | {exp} | {act} | {'PASS' if ok else 'FAIL'} | {note} |\n")
        r.write("\n## Deep Time lines\n\n```\n" + "".join(l for l in server.lines if "deeptime" in l.lower())[-8000:] + "```\n")
    sys.exit(0 if passed else 1)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--profile", required=True)
    ap.add_argument("--fresh", action="store_true")
    ap.add_argument("--level-type")
    ap.add_argument("--chunky", type=int, default=0)
    ap.add_argument("--chunky-center", default="0,0", metavar="X,Z|seam",
                    help="centre of the Chunky square; `seam` is x = C/2, z = 0 (P0-06)")
    ap.add_argument("--chunky-seam", action="store_true")
    ap.add_argument("--chunky-at", action="append", default=[], metavar="X,Z,R",
                    help="one more Chunky square of radius R around X,Z (repeatable)")
    ap.add_argument("--restart", action="store_true")
    ap.add_argument("--scenario", choices=sorted(SCENARIOS))
    ap.add_argument("--timeout", type=int, default=1500)
    ap.add_argument("--dh", action="store_true",
                    help="Distant Horizons far terrain (P3-04; a set with Distant Horizons, e.g. dh or mic): the override "
                         "registers, DH's server-side pregeneration is fed from the world data, and what DH stores agrees "
                         "with the generated chunks (dh_checks)")
    ap.add_argument("--rcon", action="append", default=[], metavar="COMMAND",
                    help="an RCON command run after the chunky tasks (repeatable, e.g. 'deeptime sites near 0 0 9000'; "
                         "'sleep <seconds>' pauses); "
                         "its [deeptime-probe] line (from the server log when RCON cuts it) is recorded in the report "
                         "and appended to smoke/<profile>/rcon.jsonl (P1-15)")
    a = ap.parse_args()

    smoke = os.path.join(REPO, "smoke", a.profile)
    server_dir = os.path.join(smoke, "server")
    if not os.path.exists(os.path.join(smoke, "install.json")):
        sys.exit(f"run.py: no {smoke}/install.json -- run tools/smoke/install.sh {a.profile}")
    install = json.load(open(os.path.join(smoke, "install.json")))
    tools = load_env(os.path.join(REPO, "smoke", "tools.env"))
    password = open(os.path.join(smoke, "rcon.pw")).read().strip()
    world = os.path.join(server_dir, "planet")

    props_path = os.path.join(server_dir, "server.properties")
    props = open(props_path).read()
    level_type = a.level_type or install["level_type"]
    props = re.sub(r"(?m)^level-type=.*$", "level-type=" + level_type.replace(":", "\\:"), props)
    open(props_path, "w").write(props)
    is_planet = level_type.startswith("deeptime:")

    if a.fresh or a.scenario:
        import shutil
        # Keep world data written by --synth; drop only what the game generated -- which includes
        # the world data when it came from first-start generation (install.sh --preset, P1-13).
        generated = ["deeptime", "deeptime.generating"] if not install.get("synth") else []
        for sub in ["region", "entities", "poi", "DIM-1", "DIM1", "dimensions", "data", "level.dat", "level.dat_old",
                    "session.lock", "serverconfig"] + generated:
            p = os.path.join(world, sub)
            if os.path.isdir(p):
                shutil.rmtree(p)
            elif os.path.exists(p):
                os.remove(p)

    for port in (install["game_port"], install["rcon_port"]):
        if not port_free(port):
            try:
                owners = subprocess.run(["ss", "-ltnp", f"sport = :{port}"], capture_output=True, text=True).stdout
            except FileNotFoundError:   # macOS
                owners = subprocess.run(["lsof", "-nP", f"-iTCP:{port}", "-sTCP:LISTEN"], capture_output=True, text=True).stdout
            sys.exit(f"run.py: port {port} is in use -- another server (a peer session?) holds it:\n{owners}")

    os.makedirs(os.path.join(smoke, "logs"), exist_ok=True)
    if a.scenario:
        run_scenario(a.scenario, a, smoke, server_dir, world, install, tools)
    log_path = os.path.join(smoke, "logs", "server.log")
    open(log_path, "w").close()
    rcon = Rcon(tools["MCRCON"], install["rcon_port"], password)
    server = Server(server_dir, tools["JAVA"], install["neoforge"], log_path)
    steps = Steps()
    started = datetime.now(timezone.utc)

    print(f"== booting {a.profile} (level-type {level_type})", flush=True)
    server.start()
    ok = server.wait_for_done(a.timeout)
    boot_s = (server.done_at or time.time()) - server.started
    # Boot costs (plan Q10 compares sets): the JVM start to "Done (", the server's own figure, and
    # the resident set size once RCON is up (before any chunk generation).
    rss_done = server.rss_now_kb() if ok else None
    steps.record("boot: JVM start to Done ( (s)", f"{boot_s:.1f}")
    steps.record("boot: the server's Done (…s) figure", server.done_seconds())
    steps.record("RSS at Done (MB)", (rss_done or 0) // 1024)
    steps.check("server reaches Done (", "Done (", "Done" if ok else server.fatal, ok)
    gen_s = check_generation(steps, server, install)
    try:
        if ok:
            mods_dir = os.path.join(server_dir, "mods")
            # Deep Time's compat mixins (mixin.compat.<modid>) are skipped, with a log line, when
            # their mod is absent (DeepTimeMixinPlugin); they are not required then.
            skipped = {m.group(1).split(".mixin.", 1)[-1] for l in server.lines
                       if (m := re.search(r"Skipping mixin (\S+): \S+ is not installed", l))}
            for config in REQUIRED_CONFIGS:
                jar, declared = declared_mixins(mods_dir, config)
                declared = [d for d in declared if d not in skipped]
                applied = server.applied(config)
                short = {k.rsplit(".", 1)[-1] for k in applied} | set(applied)
                missing = [m for m in declared if m not in short and m.rsplit(".", 1)[-1] not in short]
                steps.check(f"{config}: every declared mixin applied", f"{len(declared)} of {len(declared)}",
                            f"{len(declared) - len(missing)} of {len(declared)}" + (f" (missing: {', '.join(missing)})" if missing else ""),
                            jar is not None and not missing, note=jar or "config not found")
            for config in RECORDED_CONFIGS:
                jar, declared = declared_mixins(mods_dir, config)
                steps.record(f"{config}: mixins applied by Done (", f"{len(server.applied(config))} of {len(declared)} declared (the rest target classes not loaded yet)")
            other = sorted({m.group(2) for l in server.lines if (m := MIXING.search(l)) and m.group(2).startswith("toroidal_world.compat.")})
            steps.record("Toroidal World compat configs that applied", ", ".join(other) or "none")

            if is_planet:
                out = rcon("deeptime probe tw", check=False)
                m = PROBE.search(out) or next((PROBE.search(l) for l in reversed(server.lines) if PROBE.search(l)), None)
                steps.check("/deeptime probe tw answers", "[deeptime-probe] JSON", out[:200] if not m else m.group(1), m is not None)
                if m:
                    tw = json.loads(m.group(1))
                    for k in ("stamped", "loopsX", "loopsZ", "widthChunks", "climateCompression"):
                        steps.record(f"tw.{k}", json.dumps(tw.get(k)).strip('"'))
                    print("   probe tw: " + " ".join(f"{k}={json.dumps(tw.get(k)).strip(chr(34))}" for k in
                                                     ("stamped", "loopsX", "loopsZ", "widthChunks", "climateCompression")), flush=True)
                out = rcon("deeptime probe column 0 0", check=False)
                m = PROBE.search(out)
                steps.check("/deeptime probe column 0 0 answers", "[deeptime-probe] JSON", out[:200], m is not None)
                # Every rock class -> its resolved blocks (P1-12), for `deeptime-inspect rock --palette`.
                out = rcon("deeptime probe palette", check=False)
                time.sleep(1)   # RCON may cut a long reply; the server log has the whole line
                pal = None
                for cand in [PROBE.search(out)] + [PROBE.search(l) for l in reversed(server.lines) if '"probe":"palette"' in l]:
                    try:
                        pal = json.loads(cand.group(1)) if cand else None
                    except ValueError:
                        pal = None
                    if pal:
                        break
                steps.check("/deeptime probe palette answers", "[deeptime-probe] JSON", "ok" if pal else out[:120], pal is not None)
                if pal:
                    with open(os.path.join(smoke, "palette.json"), "w") as f:
                        json.dump(pal, f, indent=1)
                    steps.check("every rock class resolves", f"{pal.get('count')} of {pal.get('count')}",
                                f"{pal.get('resolved')} of {pal.get('count')}", pal.get("resolved") == pal.get("count"),
                                note=f"sources {pal.get('sources')}; written to smoke/{a.profile}/palette.json")

            if is_planet:
                out = rcon("locate structure #minecraft:village", check=False)
                steps.check("locate structure #minecraft:village", "The nearest …", out[:160],
                            "nearest" in out.lower())
                # Deep Time's density functions under C2ME's compiler, and DH's world hooks.
                errs = server.grep(r"(?i)\[(c2me|distanthorizons|DistantHorizons)[^\]]*\].*(ERROR|Exception)|/ERROR\].*(c2me|distanthorizons)", 5)
                steps.check("no C2ME or Distant Horizons errors in the boot log", "none", "; ".join(errs)[:300] or "none", not errs)

            # Before anything generates chunks: a generated chunk replaces Distant Horizons' data for its columns.
            if a.dh and is_planet:
                dh_checks(steps, server, rcon, smoke)

            tasks = []
            if a.chunky:
                cx, cz = chunky_center(a.chunky_center, install["size"])
                steps.record("chunky square", f"centre {cx},{cz} radius {a.chunky} (x {cx - a.chunky}..{cx + a.chunky})")
                tasks.append(("chunky pregeneration finished", cx, cz, a.chunky))
            for spec in a.chunky_at:
                x, z, r = (int(v) for v in spec.split(","))
                steps.record("chunky square", f"centre {x},{z} radius {r}")
                tasks.append((f"chunky {x},{z} r{r} finished", x, z, r))
            if a.chunky_seam and is_planet:
                # Chunky's square is the chunks (int) centre >> 4 ± ceil(radius / 16): with
                # centre ±(C/2 - 72) and radius 64 that is x in [C/2 - 144, C/2) and
                # [-C/2, -C/2 + 144), z in [-64, 80): no chunk outside the canonical span
                # (the region check below).
                half = install["size"] // 2
                tasks.append(("chunky seam east finished", half - 72, 0, 64))
                tasks.append(("chunky seam west finished", -(half - 72), 0, 64))
            finished = re.compile(r"Task finished for|Task complete")
            for name, cx, cz, radius in tasks:
                before = len(server.grep(finished.pattern, 10**9))
                for cmd in (f"chunky center {cx} {cz}", "chunky shape square", f"chunky radius {radius}", "chunky start"):
                    print(f"   {cmd}: {rcon(cmd, check=False)[:160]}", flush=True)
                deadline = time.time() + 1800
                while time.time() < deadline:
                    time.sleep(10)
                    if len(server.grep(finished.pattern, 10**9)) > before:
                        break
                lines = server.grep(finished.pattern, 10**9)
                done = len(lines) > before
                steps.check(name, "finished", lines[-1][-160:] if done else "timeout", done)

            if a.rcon:
                open(os.path.join(smoke, "rcon.jsonl"), "w").close()
            for cmd in a.rcon:
                # `sleep <seconds>` pauses between commands (P1-25: let a simulation run between probes).
                if cmd.startswith("sleep "):
                    time.sleep(float(cmd.split()[1]))
                    continue
                before = len(server.lines)
                out = rcon(cmd, check=False)
                time.sleep(1)   # RCON may cut a long reply; the server log has the whole probe line
                line = next((m.group(1) for l in reversed(server.lines[before:]) if (m := PROBE.search(l))), None)
                if line is None:
                    m = PROBE.search(out)
                    line = m.group(1) if m else None
                steps.check(f"rcon: {cmd}", "[deeptime-probe] JSON" if cmd.startswith("deeptime") else "answers",
                            (line or out)[:300], line is not None or not cmd.startswith("deeptime"))
                with open(os.path.join(smoke, "rcon.jsonl"), "a") as f:
                    f.write(json.dumps({"command": cmd, "probe": json.loads(line) if line else None, "reply": out}) + "\n")
                print(f"   rcon {cmd}: {(line or out)[:200]}", flush=True)
                # P1-16: `probe coe` / `probe cdg` compare Deep Time's decision with what COE / CDG
                # hold for the chunk; the report gets a row per probe and a check that they agree.
                probe = json.loads(line) if line else None
                if probe and probe.get("probe") in ("coe", "cdg"):
                    steps.compat.append(compat_row(probe))
                    if "agrees" in probe:
                        steps.check(f"{cmd}: Deep Time and {probe['probe'].upper()} agree", "true",
                                    str(probe["agrees"]).lower(), probe["agrees"] is True)

            rcon("save-all flush")
            time.sleep(3)
            if is_planet:
                total, outside, empty = region_span_check(world, install["size"])
                print(f"   regions outside span: {len(outside)} (of {total} region files holding chunks; span "
                      f"{-install['size'] // 1024} <= rx < {install['size'] // 1024}; {len(empty)} empty files outside)",
                      flush=True)
                steps.check("regions outside span", "0", f"{len(outside)} (of {total} region/entities/poi files holding chunks"
                            + (f": {', '.join(sorted(outside)[:8])}" if outside else "") + ")", not outside,
                            note=f"-C/1024 <= rx < C/1024, C = {install['size']}")
                steps.record("empty region files outside the span (0 chunks, opened by lookups)",
                             f"{len(empty)}" + (f": {', '.join(sorted(empty)[:8])}" if empty else ""))
                if a.chunky_seam or (a.chunky and a.chunky_center == "seam"):
                    # Both sides of the seam hold chunks: the last region column (rx = C/1024 - 1)
                    # and the first (rx = -C/1024).
                    half = install["size"] // 1024
                    rdir = os.path.join(world, "region")
                    side = lambda rx: sum(region_chunks(os.path.join(rdir, f)) for f in os.listdir(rdir)
                                          if re.match(rf"r\.{rx}\.-?\d+\.mca$", f))
                    east, west = side(half - 1), side(-half)
                    steps.check("chunks stored on both sides of the seam", "east > 0 and west > 0",
                                f"east (rx={half - 1}) {east}, west (rx={-half}) {west}", east > 0 and west > 0)

            steps.record("peak RSS (MB)", (server.peak_rss_kb() or 0) // 1024)
            server.stop(rcon)

            if a.restart:
                manifest = os.path.join(world, "deeptime", "manifest.json")
                before = os.path.getmtime(manifest) if os.path.exists(manifest) else None
                server.start()
                ok2 = server.wait_for_done(a.timeout)
                steps.check("restart reaches Done (", "Done (", "Done" if ok2 else server.fatal, ok2)
                if ok2:
                    steps.record("restart boot: JVM start to Done ( (s)", f"{server.done_at - server.started:.1f}")
                    steps.record("restart boot: the server's Done (…s) figure", server.done_seconds())
                    steps.record("restart RSS at Done (MB)", (server.rss_now_kb() or 0) // 1024)
                after = os.path.getmtime(manifest) if os.path.exists(manifest) else None
                steps.check("world data not regenerated on restart", before, after, before == after)
                if install.get("preset") and not install.get("synth"):
                    present = next((l for l in server.lines if DATA_PRESENT.search(l)), None)
                    steps.check("restart logs 'world data present'", "[deeptime] world data present at …",
                                (present or "absent").strip()[-160:], present is not None)
                    steps.check("restart does not generate", "no 'generating' line",
                                "generated" if any(GEN_START.search(l) for l in server.lines) else "none",
                                not any(GEN_START.search(l) for l in server.lines))
                server.stop(rcon)
    finally:
        server.stop(rcon)

    passed = not steps.failed
    write_report(smoke, a, install, steps, level_type, boot_s, started, passed, server, gen_s)
    # Keep one report per circumference too (P0-06 compares 16k and 32k side by side).
    import shutil
    shutil.copyfile(os.path.join(smoke, "report.md"), os.path.join(smoke, f"report-C{install['size']}.md"))
    print(f"\n{'PASS' if passed else 'FAIL'} -- report: {os.path.join(smoke, 'report.md')} "
          f"(also report-C{install['size']}.md)", flush=True)
    sys.exit(0 if passed else 1)


def deeptime_build(install):
    """The jar's sha256 and source revision from install.json (install.sh, or a workbench preview that
    refreshed the jar), and whether this tree's sources still match what it was built from."""
    if not install.get("deeptime_jar_sha256"):
        return "not recorded (installed before install.json kept the jar's source): re-run tools/smoke/install.sh"
    ok, why = jarstate.status(REPO, install)
    return (f"sha256 {install['deeptime_jar_sha256'][:12]}, source {install.get('deeptime_src_rev', '?')}, "
            f"tree {install.get('deeptime_src_hash', '?')}" + ("" if ok else f" -- STALE: {why}"))


def write_report(smoke, a, install, steps, level_type, boot_s, started, passed, server, gen_s=None):
    excluded = open(os.path.join(smoke, "excluded.txt")).read().split() if os.path.exists(os.path.join(smoke, "excluded.txt")) else []
    mods = sorted(f for f in os.listdir(os.path.join(smoke, "server", "mods")) if f.endswith(".jar"))
    with open(os.path.join(smoke, "report.md"), "w") as r:
        r.write(f"# Deep Time smoke report — {a.profile}\n\n")
        r.write(f"**Result: {'PASS' if passed else 'FAIL'}** · {started.strftime('%Y-%m-%d %H:%M UTC')}\n\n")
        r.write("| Field | Value |\n|---|---|\n")
        for k, v in [("Profile", a.profile), ("Level type", level_type), ("Circumference", install["size"]),
                     ("NeoForge", install["neoforge"]), ("MIC pack", install["pack"] + (f" (working tree {install['pack_dirty_hash']})" if install.get("pack_dirty_hash") else "")),
                     ("Deep Time jar", install["deeptime_jar"]), ("Deep Time build", deeptime_build(install)),
                     ("Mod jars", len(mods)),
                     ("Boot time", f"{boot_s:.0f} s"),
                     ("First-start preset", (install.get("preset") or "none") + (
                         f" (timeline {install.get('timeline_start')} / {install.get('timeline_length_myr')} Myr)"
                         if install.get("preset") else "")),
                     ("Generation time", f"{gen_s:.1f} s" if gen_s is not None else "no generation this boot"),
                     ("Excluded jars", ", ".join(excluded) or "none")]:
            r.write(f"| {k} | {v} |\n")
        r.write("\n## Checks\n\n| Check | Expected | Actual | Result | Note |\n|---|---|---|---|---|\n")
        for name, exp, act, ok, note in steps.rows:
            act = act.replace("|", "\\|")
            r.write(f"| {name} | {exp} | {act} | {'PASS' if ok else 'FAIL'} | {note} |\n")
        if steps.compat:
            r.write("\n## COE and CDG (plan §7.14)\n\nDeep Time's decision for the chunk and what the mod holds "
                    "there (`/deeptime probe coe|cdg <chunkX> <chunkZ>`).\n\n"
                    "| Mod | Chunk | Site | Deep Time | The mod | Agree |\n|---|---|---|---|---|---|\n")
            for mod, chunk, site, decided, held, agrees in steps.compat:
                r.write(f"| {mod} | {chunk} | {site} | {decided} | {held} | {'' if agrees is None else ('yes' if agrees else 'NO')} |\n")
        if server.fatal:
            r.write(f"\n## First fatal line\n\n```\n{server.fatal}\n```\n")
        r.write("\n## Mod jars\n\n" + "\n".join(f"- {m}" for m in mods) + "\n")


def watch_template(server_dir, keep):
    """Copies config/deeptime/server.json to `keep` as soon as the mod writes it (first_start_no_config)."""
    path = os.path.join(server_dir, "config", "deeptime", "server.json")
    if os.path.exists(keep):
        os.remove(keep)

    def loop():
        deadline = time.time() + 3600
        while time.time() < deadline:
            if os.path.exists(path):
                time.sleep(0.2)
                try:
                    with open(path) as src, open(keep, "w") as dst:
                        dst.write(src.read())
                    return
                except OSError:
                    pass
            time.sleep(0.1)

    threading.Thread(target=loop, daemon=True).start()


if __name__ == "__main__":
    main()
