#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Load-based admission for heavy jobs on the test VM.

Ben, 2026-10-03 (decisions log): "No set slots, instead we should have a system of: Guess load, run
if load is less than what is available + 25%, (measure load while running to fine-tune future
guesses)." This replaces the M6-era caps (one Minecraft server at a time, two simulations per agent,
four on the box, eight cargo jobs).

Every heavy job (Minecraft servers, simulations, cargo and Gradle builds, renders) runs through it:

    admit.py run --kind <kind> [--server] [--label L] [--status-json F] -- <command...>

  1. The job declares a kind (`gametest:mic`, `preview:standalone`, `sim:earthlike_quick_16k:2t`,
     `cargo:test`, `gradle:build`, ...; `--kind auto` classifies the command line).
  2. Its load is estimated for that kind: CPU cores (mean over a run) and resident memory (peak),
     from the seed table below until the kind has measured runs, then from those runs (a weighted
     90th percentile of the recent ones; newer runs weigh more).
  3. It starts when its estimate fits what is free now. Memory (Ben, decisions log 2026-10-03: "Don't
     kill old things, just prevent new ones from launching if their usage isn't within it ... If we
     are at 3.5Gb and a run is predicted to use 200mb it is green to run. If an agent predicts it will
     use 0.4 [GB] then it will be red since that would use up 100% of the machine after margin is
     applied."): the job's estimate times 1.25 must be under free memory, where free memory is
     MemAvailable minus the part of each admitted job's estimate it has not reached yet (its
     reservation). The margin sits on the new job's prediction, not on the free memory, and there is no
     standing memory headroom. Running jobs are never stopped. CPU (Ben, "available + 25%"): the
     estimate fits in 1.25 x free cores, where free cores are the CPU count minus the larger of the
     1-minute load and the measured busy cores of the last 30 s, minus the reservations and a small
     headroom (RESERVE_CORES) for the play server, the workbench and ssh; CPU overcommit only slows
     jobs down. Otherwise it waits in order, and says why on stderr.
  4. While it runs, its processes are sampled every few seconds (CPU and RSS of its process tree,
     plus processes that carry its job id in their environment, plus the Gradle daemon its Gradle
     client is talking to); at the end its mean and peak CPU, mean and peak RSS and wall time go to
     the kind's history, which refines the next estimate.
  5. A server kind (`--server`, or a kind starting with gametest/planet/smoke/preview/client/server)
     also gets a slot: its own ports (game, RCON, query, voice chat and spares), an X display number
     and a private folder, in the environment as DEEPTIME_PORT_GAME, DEEPTIME_PORT_RCON,
     DEEPTIME_PORT_QUERY, DEEPTIME_PORT_VOICE, DEEPTIME_PORT_BASE (base .. base+9 are the slot's),
     DEEPTIME_DISPLAY, DEEPTIME_SERVER_DIR (removed when the job ends) and DEEPTIME_SERVER_SLOT.
     The play server (25765, voice 24464) and the workbench (8642) are outside the pool.

Nesting: the command sees DEEPTIME_ADMIT_JOB (and DEEPTIME_ADMIT_PID, its own pid). A non-server
`admit.py run` inside an admitted job runs at once, as part of the job. A server-kind one inside a
server job takes over the job's slot when nobody holds it (so `mac.sh` can admit a whole command and
the server launcher inside it uses that admission), and is admitted on its own otherwise.

Fairness: waiting jobs are considered oldest first; a smaller job may start ahead of a bigger one
that does not fit yet, until the bigger one has waited STARVE_MIN minutes: from then nothing behind
it starts until it fits.

The old server lock (deeptime-cache/server.lock, from mac.sh --server before this tool) counts as an
admitted server job while it exists, so agents still on the old scripts and this tool share the box.

Other commands:
    admit.py status [--json]     what is admitted, what waits and why, free capacity, kind estimates
    admit.py estimate <kind>     the estimate for a kind and where it came from
    admit.py classify -- <cmd>   the kind `--kind auto` would give a command line ("light": none)
    admit.py forget <kind>       drop a kind's measured history (back to its seed)

State: deeptime-cache/admit/ (state.json: jobs and queue; history.json: per-kind runs; admit.log:
one line per admission and per finished job). Every change takes an exclusive flock on state.lock.
Settings: environment, or /etc/default/deeptime-admit (KEY=value lines). Standard library only.
"""

import fcntl
import json
import math
import os
import random
import re
import shutil
import signal
import socket
import string
import sys
import threading
import time

# ------------------------------------------------------------------------------------- settings


def _defaults_file(path="/etc/default/deeptime-admit"):
    out = {}
    try:
        for line in open(path):
            line = line.strip()
            if line and not line.startswith("#") and "=" in line:
                k, v = line.split("=", 1)
                out[k.strip()] = v.strip().strip('"')
    except OSError:
        pass
    return out


_FILE = _defaults_file()


def setting(name, default):
    v = os.environ.get("DEEPTIME_ADMIT_" + name, _FILE.get(name))
    if v is None:
        return default
    return type(default)(v) if not isinstance(default, bool) else v not in ("", "0", "false", "no")


CACHE = setting("CACHE", "/data/ban/deeptime-cache")
DIR = setting("DIR", os.path.join(CACHE, "admit"))
OVERCOMMIT = setting("OVERCOMMIT", 0.25)          # CPU: fits in 1.25 x free cores. Memory: estimate x 1.25 < free
MEM_CEILING = 0.9                                  # an estimate is capped so x1.25 fits in this share of the box
RESERVE_CORES = setting("RESERVE_CORES", 4.0)      # play server 1, a workbench run's burst 2, ssh/system 1
STARVE_MIN = setting("STARVE_MIN", 15.0)
POLL_S = setting("POLL_S", 3.0)
SAMPLE_S = setting("SAMPLE_S", 5.0)
RAMP_S = setting("RAMP_S", 180.0)                  # a CPU reservation lasts until reached, or this long
HISTORY_KEEP = setting("HISTORY_KEEP", 20)
QUICK_S = setting("QUICK_S", 60.0)                 # a kind whose runs are this short and small starts at once
MIN_RECORD_S = setting("MIN_RECORD_S", 15.0)       # shorter runs are not recorded (too few samples)
PORT_BASE = setting("PORT_BASE", 26000)
PORT_STRIDE = 10
SLOTS = setting("SLOTS", 40)
DISPLAY_BASE = setting("DISPLAY_BASE", 200)
LEGACY_LOCK = setting("LEGACY_LOCK", os.path.join(CACHE, "server.lock"))
LEGACY_STALE_S = 3 * 3600                          # mac.sh's old rule for a stale lock
RESERVED_PORTS = {25765, 24464, 8642}              # play server, its voice chat, the workbench
GIB = 1024 ** 3
HZ = os.sysconf("SC_CLK_TCK") if hasattr(os, "sysconf") else 100
PAGE = os.sysconf("SC_PAGE_SIZE") if hasattr(os, "sysconf") else 4096

SERVER_PREFIXES = ("gametest", "planet", "smoke", "preview", "client", "server")

# ------------------------------------------------------------------------------------ the seeds
# A kind's first guess, before it has measured runs: mean CPU cores over a run, peak resident GiB,
# peak cores, typical wall seconds. Measured on the 32-vCPU VM where noted; the rest are the
# JVM heaps the launchers set (-Xmx) plus their overhead, and the builds' usual parallelism.
SEEDS = {
    "gametest:standalone": (4.0, 8.0, 16.0, 600),   # cargo synth + 3 runs, Gradle, -Xmx6G server
    "gametest:mic": (4.0, 11.0, 16.0, 900),         # -Xmx8G, 160 mods
    "gametest:terrablender": (4.0, 8.0, 16.0, 600),
    "gametest:ev": (4.0, 8.0, 16.0, 600),
    "planet:standalone": (3.0, 8.0, 12.0, 600),     # -Xmx6G
    "planet:mic": (4.0, 11.0, 12.0, 900),
    "smoke:standalone": (3.0, 8.0, 12.0, 600),      # -Xmx6G
    "smoke:mic": (4.0, 10.0, 12.0, 900),
    "preview:standalone": (3.0, 8.0, 12.0, 300),    # -Xmx6G, Chunky over a box
    "preview:mic": (4.0, 10.0, 12.0, 600),
    "client:dh": (6.0, 15.0, 16.0, 1500),           # server -Xmx6G + client -Xmx6G + llvmpipe
    "server:legacy": (4.0, 10.0, 12.0, 900),        # a holder of the old server.lock
    "server": (4.0, 10.0, 12.0, 900),
    "cargo:test": (10.0, 12.0, 32.0, 600),
    "cargo:clippy": (8.0, 6.0, 32.0, 300),
    "cargo:build": (8.0, 6.0, 32.0, 300),
    "cargo:other": (4.0, 4.0, 16.0, 120),
    "gradle:build": (4.0, 8.0, 16.0, 400),          # daemon -Xmx2G, test JVMs, natives through cargo
    "install": (3.0, 4.0, 8.0, 300),                # tools/smoke/install.sh: packwiz, Gradle jar
    "render": (4.0, 6.0, 16.0, 300),
    "sim": (2.0, 5.0, 2.0, 900),                    # see seed(): threads and planet size
    "shell": (1.0, 1.0, 4.0, 120),                  # an unclassified command (shell:<program>)
}


def seed(kind):
    """(cpu, mem_gib, peak_cpu, wall_s) for a kind without measured runs."""
    if kind in SEEDS:
        return SEEDS[kind]
    if "+" in kind:   # a compound command (run one after another): its heaviest part
        parts = [seed(k) for k in kind.split("+")]
        return (max(p[0] for p in parts), max(p[1] for p in parts), max(p[2] for p in parts),
                sum(p[3] for p in parts))
    head = kind.split(":", 1)[0]
    if head == "sim":
        # sim:<preset>:<N>t -- N Rayon threads (0 or "all": every core); memory from the planet size.
        m = re.search(r":(\d+|all)t$", kind)
        n = 0 if not m or m.group(1) == "all" else int(m.group(1))
        cpu = 16.0 if n == 0 else 0.9 * n
        mem = 9.0 if "64k" in kind else 7.0 if "32k" in kind else 3.0 if ("quick" in kind or "4k" in kind) else 5.0
        return (cpu, mem, float(n or 32), 900)
    for prefix in (head + ":", head):
        best = [v for k, v in SEEDS.items() if k == prefix or k.startswith(prefix)]
        if best:
            return max(best, key=lambda v: (v[0], v[1]))
    return SEEDS["shell"]


def is_server_kind(kind):
    return kind.split(":", 1)[0].split("+", 1)[0] in SERVER_PREFIXES or \
        any(k.split(":", 1)[0] in SERVER_PREFIXES for k in kind.split("+"))


# ------------------------------------------------------------------------------- the estimator

def weighted_percentile(values, weights, q):
    """The q-quantile (0..1) of values with weights (step function, no interpolation)."""
    pairs = sorted(zip(values, weights))
    total = sum(w for _, w in pairs)
    if not pairs or total <= 0:
        return None
    acc = 0.0
    for v, w in pairs:
        acc += w
        if acc >= q * total - 1e-12:
            return v
    return pairs[-1][0]


DECAY = 0.85      # each older run weighs 0.85 of the next newer one
PCT = 0.9


def estimate(kind, history):
    """{"cpu", "mem_gib", "peak_cpu", "wall_s", "runs", "source"} for a kind.

    Measured runs (newest first, weight DECAY**i) give a weighted 90th percentile of the runs' mean
    cores and of their peak RSS. With fewer than three runs the seed stays in the mix with the weight
    of one run, so a single odd run cannot swing the estimate all the way."""
    s_cpu, s_mem, s_peak, s_wall = seed(kind)
    runs = sorted(history.get(kind, []), key=lambda r: r.get("end", 0), reverse=True)[:HISTORY_KEEP]
    if not runs:
        return {"cpu": s_cpu, "mem_gib": s_mem, "peak_cpu": s_peak, "wall_s": s_wall, "runs": 0, "source": "seed"}
    w = [DECAY ** i for i in range(len(runs))]
    cpu = [r["cpu_mean"] for r in runs]
    mem = [r["rss_peak_gib"] for r in runs]
    peak = [r.get("cpu_peak", r["cpu_mean"]) for r in runs]
    wall = [r.get("wall_s", s_wall) for r in runs]
    if len(runs) < 3:
        sw = DECAY ** len(runs)
        cpu, mem, peak, wall = cpu + [s_cpu], mem + [s_mem], peak + [s_peak], wall + [s_wall]
        w = w + [sw]
    return {"cpu": round(weighted_percentile(cpu, w, PCT), 2),
            "mem_gib": round(weighted_percentile(mem, w, PCT), 2),
            "peak_cpu": round(weighted_percentile(peak, w, PCT), 2),
            "wall_s": round(weighted_percentile(wall, w, 0.5)),
            "runs": len(runs),
            "source": f"measured ({len(runs)} run{'s' if len(runs) != 1 else ''}"
                      + (", seed mixed in" if len(runs) < 3 else "") + ")"}


# ---------------------------------------------------------------------------- the decision

def pending(job, now, extra_cpu=0.0, extra_rss=0.0):
    """The part of an admitted job's estimate it has not used yet: (cores, GiB).

    CPU: until the job has once reached 80 % of its estimate, or for RAMP_S seconds after it started,
    whichever is first (after that its real use is in the measured load). Memory: for the job's whole
    life, the estimate's peak minus what it holds now (memory that arrives late is the dangerous kind).
    A job running inside another job's reservation (a slot it took over) reserves nothing itself; its
    use counts as its host's (extra_cpu, extra_rss)."""
    if job.get("inherited"):
        return 0.0, 0.0
    est_cpu, est_mem = job["est"]["cpu"], job["est"]["mem_gib"]
    m = job.get("measured", {})
    age = now - job.get("admitted_at", now)
    cpu = 0.0
    if age < RAMP_S and max(m.get("cpu_max", 0.0), m.get("cpu", 0.0) + extra_cpu) < 0.8 * est_cpu:
        cpu = max(0.0, est_cpu - m.get("cpu", 0.0) - extra_cpu)
    mem = max(0.0, est_mem - m.get("rss_gib", 0.0) - extra_rss)
    return cpu, mem


def pendings(jobs, now):
    """[(cores, GiB)] for each job, a slot's guest counted as part of its host."""
    guest_cpu, guest_rss = {}, {}
    for j in jobs:
        if j.get("inherited"):
            m = j.get("measured", {})
            guest_cpu[j["inherited"]] = guest_cpu.get(j["inherited"], 0.0) + m.get("cpu", 0.0)
            guest_rss[j["inherited"]] = guest_rss.get(j["inherited"], 0.0) + m.get("rss_gib", 0.0)
    return [pending(j, now, guest_cpu.get(j["id"], 0.0), guest_rss.get(j["id"], 0.0)) for j in jobs]


def quick(est):
    """A kind known to be short and small (two or more measured runs, a median under QUICK_S, at most
    1.5 cores and 2 GiB): a quick look such as a python3 one-liner. It starts at once, still measured;
    waiting would cost more than it saves."""
    return est.get("runs", 0) >= 2 and est.get("wall_s", 1e9) <= QUICK_S and est["cpu"] <= 1.5 and est["mem_gib"] <= 2.0


def plan(queue, jobs, cap, now, overcommit=None, reserve_cores=None, starve_min=None):
    """Which waiting jobs start now. queue: entries with "id", "enq", "est" (oldest first after
    sorting); jobs: admitted entries; cap: {"free_cpu", "mem_avail_gib", "cpu_total", "mem_total_gib"}.
    Returns ({id: True to start | reason string}, {"avail_cpu", "avail_mem"} before any start)."""
    overcommit = OVERCOMMIT if overcommit is None else overcommit
    reserve_cores = RESERVE_CORES if reserve_cores is None else reserve_cores
    starve_min = STARVE_MIN if starve_min is None else starve_min
    ps = pendings(jobs, now)
    pend_cpu = sum(p[0] for p in ps)
    pend_mem = sum(p[1] for p in ps)
    avail_cpu = cap["free_cpu"] - pend_cpu - reserve_cores
    avail_mem = cap["mem_avail_gib"] - pend_mem        # free now; no standing headroom
    info = {"avail_cpu": round(avail_cpu, 2), "avail_mem": round(avail_mem, 2),
            "pending_cpu": round(pend_cpu, 2), "pending_mem": round(pend_mem, 2)}
    # No estimate may exceed what the box could ever give, or it would wait forever.
    max_cpu = max(1.0, cap["cpu_total"] - reserve_cores)
    max_mem = max(1.0, cap["mem_total_gib"] * MEM_CEILING / (1 + overcommit))
    out = {}
    blocked_by = None
    for w in sorted(queue, key=lambda e: (e["enq"], e["id"])):
        if quick(w["est"]):
            out[w["id"]] = True
            continue
        if blocked_by:
            out[w["id"]] = (f"behind {blocked_by['kind']} ({blocked_by['label']}), which has waited "
                            f"{(now - blocked_by['enq']) / 60:.0f} min: nothing starts ahead of it until it fits")
            continue
        cpu = min(w["est"]["cpu"], max_cpu)
        mem = min(w["est"]["mem_gib"], max_mem)
        fits_cpu = cpu <= (1 + overcommit) * avail_cpu if avail_cpu > 0 else cpu <= 0
        fits_mem = mem <= 0 or mem * (1 + overcommit) < avail_mem
        if fits_cpu and fits_mem:
            out[w["id"]] = True
            avail_cpu -= cpu
            avail_mem -= mem        # the next job in this pass sees it as reserved
            continue
        short = []
        if not fits_cpu:
            short.append(f"{cpu:.1f} cores, {max(avail_cpu, 0):.1f} free (+{overcommit:.0%} = {max(avail_cpu, 0) * (1 + overcommit):.1f})")
        if not fits_mem:
            short.append(f"{mem:.2f} GiB (+{overcommit:.0%} margin = {mem * (1 + overcommit):.2f}), "
                         f"{max(avail_mem, 0):.2f} free: it must be under the free memory")
        out[w["id"]] = "needs " + "; ".join(short)
        if now - w["enq"] >= starve_min * 60:
            blocked_by = w
    return out, info


# ---------------------------------------------------------------------------- classification

_HEAVY = [
    # (pattern, kind or a function of the match and the command)
    (r"\bgametest\.sh\b", lambda c: "gametest:" + ("mic" if "-PwithMic" in c else "terrablender" if "-PwithTerraBlender" in c
                                                   else "ev" if "-PwithEv" in c else "standalone")),
    (r"\bgametest-planet\.sh\b|\brun_planet\.py\b", lambda c: "planet:" + ("mic" if "-PwithMic" in c else "standalone")),
    (r"\btools/smoke/run\.py\b", lambda c: "smoke:" + _opt(c, "--profile", "standalone")),
    (r"\bpreview\.py\s+run\b|\bpreview-area\.sh\b", lambda c: "preview:" + _opt(c, "--profile", "standalone")),
    (r"\bdh-client-check\.sh\b", lambda c: "client:" + (_arg_after(c, "dh-client-check.sh") or "dh")),
    (r"\bmac-screenshots\.sh\b|\bmac-client\.sh\b|\bev-assets", lambda c: "client:other"),
    (r"\bbluemap\b", lambda c: "render:bluemap"),
    (r"\b(?:deeptime|deeptime-cli)\b[^;&|]*?\s(?:run|bench|stage|sweep)\b", lambda c: _sim_kind(c)),
    (r"\bcargo\s+(?:\+\S+\s+)?(?:test|nextest)\b", lambda c: "cargo:test"),
    (r"\bcargo\s+(?:\+\S+\s+)?clippy\b", lambda c: "cargo:clippy"),
    (r"\bcargo\s+(?:\+\S+\s+)?(?:build|run|xwin|zigbuild)\b", lambda c: "cargo:build"),
    (r"\bgradlew?\b", lambda c: "gradle:build"),
    (r"\btools/smoke/install\.sh\b", lambda c: "install"),
    (r"\bcargo\b", lambda c: "cargo:other"),
]
_DETACHED = re.compile(r"\bsetsid\b|\bnohup\b|\bdetach\.sh\s+(?:start|wait|tail)\b|&\s*$|\bdeeptime-admit\b|\badmit\.py\b")
_LIGHT = re.compile(r"^\s*(?:true|false|:|exit|echo|printf|cat|tail|head|less|grep|rg|ls|command\s+ls|find|du|df|"
                    r"free|uptime|ps|pgrep|kill|pkill|stat|test|\[|wc|sort|uniq|diff|cmp|sha256sum|md5sum|date|"
                    r"sleep|mkdir|rm|cp|mv|ln|touch|chmod|readlink|realpath|basename|dirname|env|which|type|"
                    r"git|jq|sqlite3|tar|zcat|zgrep|gzip|journalctl|systemctl|ss|nproc|id|whoami|hostname|uname|"
                    r"cut|awk|sed|tr|column|nl|xxd|file|lsof|deeptime-admit|"
                    r"sudo\s+python3\s+/usr/local/lib/deeptime/idle-watcher\.py)\b")


def _opt(cmd, name, default):
    m = re.search(re.escape(name) + r"[ =](\S+)", cmd)
    return re.sub(r"[^A-Za-z0-9_.-]", "", m.group(1)) if m else default


def _arg_after(cmd, word):
    m = re.search(re.escape(word) + r"\s+([A-Za-z0-9_-]+)", cmd)
    return m.group(1) if m else None


def _sim_kind(cmd):
    m = re.search(r"--preset[ =](\S+)", cmd)
    preset = os.path.basename(m.group(1).strip("'\"")) if m else "default"
    preset = re.sub(r"[^A-Za-z0-9_.-]", "", re.sub(r"\.json$", "", preset))[:40] or "default"
    m = re.search(r"--threads[ =](\d+)", cmd) or re.search(r"RAYON_NUM_THREADS=(\d+)", cmd)
    n = int(m.group(1)) if m else int(os.environ.get("RAYON_NUM_THREADS", "0") or 0)
    return f"sim:{preset}:{n if n else 'all'}t"


def _program(parts):
    """What an unrecognised command runs, for its kind: the first script named (x.sh, x.py), else
    the first program word that is not cd, env or an assignment. Each learns its own load."""
    words = [w for p in parts for w in p.split()]
    for w in words:
        w = w.strip("'\"")
        if re.search(r"\.(sh|py)$", w):
            return re.sub(r"[^A-Za-z0-9_.-]", "", os.path.basename(w))[:40]
    for p in parts:
        for w in p.split():
            if w in ("cd", "env", "exec", "time", "nice", "sudo") or "=" in w:
                continue
            if re.match(r"^\s*cd\s", p):
                break
            return re.sub(r"[^A-Za-z0-9_.-]", "", os.path.basename(w))[:40] or "other"
    return "other"


def classify(cmd):
    """The kind for a shell command line, "light" when it needs no admission (a quick look, a
    detached start whose job admits itself, git), "shell:<script or program>" when nothing heavy is
    recognised."""
    if _DETACHED.search(cmd):
        return "light"
    # Quoted text (a grep pattern with `|` in it) is not shell structure.
    cmd = re.sub(r"'[^']*'|\"(?:[^\"\\]|\\.)*\"", lambda m: m.group(0)[0] + re.sub(r"[|;&\n]", " ", m.group(0)[1:-1]) + m.group(0)[0], cmd)
    kinds = []
    for part in re.split(r"&&|\|\||;|\n", cmd):
        for pat, fn in _HEAVY:
            if re.search(pat, part):
                k = fn(part)
                if k not in kinds:
                    kinds.append(k)
                break
    if not kinds:
        parts = [p for p in re.split(r"&&|\|\||;|\||\n", cmd) if p.strip()]
        if parts and all(_LIGHT.match(p) or re.match(r"^\s*cd\s", p) for p in parts):
            return "light"
        return "shell:" + _program(parts)
    servers = [k for k in kinds if is_server_kind(k)]
    # A server launcher dominates (it runs the builds it needs itself); otherwise the parts are
    # joined: the history of `cargo:clippy+cargo:test` is its own.
    if servers:
        return servers[0] if len(servers) == 1 else "+".join(servers)
    return "+".join(kinds[:3])


# ----------------------------------------------------------------------------- the machine

def read_proc_stat():
    """(busy jiffies, total jiffies) of all CPUs."""
    with open("/proc/stat") as f:
        parts = f.readline().split()[1:]
    vals = [int(x) for x in parts[:8]]
    idle = vals[3] + vals[4]          # idle + iowait
    return sum(vals) - idle, sum(vals)


def meminfo():
    out = {}
    with open("/proc/meminfo") as f:
        for line in f:
            k, v = line.split(":", 1)
            out[k] = int(v.split()[0]) * 1024
    return out


def loadavg1():
    with open("/proc/loadavg") as f:
        return float(f.read().split()[0])


def capacity(sys_samples, now):
    """Free cores and memory now. sys_samples: [[t, busy, total], ...] kept in the state (the caller
    appends the current one first)."""
    ncpu = os.cpu_count() or 1
    busy30 = None
    cur = sys_samples[-1] if sys_samples else None
    old = [s for s in sys_samples if now - s[0] >= 20]
    if cur and old:
        base = old[-1]
        dt = cur[2] - base[2]
        if dt > 0:
            busy30 = (cur[1] - base[1]) / dt * ncpu
    load = loadavg1()
    used = max(load, busy30 if busy30 is not None else 0.0)
    mi = meminfo()
    return {"cpu_total": float(ncpu), "load1": load, "busy30": None if busy30 is None else round(busy30, 2),
            "free_cpu": round(ncpu - used, 2), "mem_total_gib": round(mi["MemTotal"] / GIB, 2),
            "mem_avail_gib": round(mi.get("MemAvailable", mi.get("MemFree", 0)) / GIB, 2)}


def boot_time():
    with open("/proc/stat") as f:
        for line in f:
            if line.startswith("btime"):
                return int(line.split()[1])
    return 0


class P:
    __slots__ = ("pid", "ppid", "start", "ticks", "rss", "comm", "uid")


def proc_stat(pid):
    with open(f"/proc/{pid}/stat") as f:
        s = f.read()
    p = P()
    p.pid = pid
    p.comm = s[s.index("(") + 1:s.rindex(")")]
    r = s[s.rindex(")") + 2:].split()
    p.ppid = int(r[1])
    p.ticks = int(r[11]) + int(r[12]) + int(r[13]) + int(r[14])   # utime stime cutime cstime
    p.start = int(r[19])
    p.rss = int(r[21]) * PAGE
    return p


def scan():
    procs = {}
    for d in os.listdir("/proc"):
        if d.isdigit():
            try:
                procs[int(d)] = proc_stat(int(d))
            except (OSError, ValueError, IndexError):
                pass
    return procs


def children_map(procs):
    kids = {}
    for p in procs.values():
        kids.setdefault(p.ppid, []).append(p.pid)
    return kids


def subtree(root, kids):
    out, stack = set(), [root]
    while stack:
        q = stack.pop()
        if q in out:
            continue
        out.add(q)
        stack.extend(kids.get(q, ()))
    return out


def pid_start(pid):
    try:
        return proc_stat(pid).start
    except (OSError, ValueError, IndexError):
        return None


def alive(pid, start):
    return pid is not None and pid_start(pid) == start


def cmdline(pid):
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as f:
            return f.read().replace(b"\0", b" ").decode(errors="replace")
    except OSError:
        return ""


def environ_job(pid):
    try:
        with open(f"/proc/{pid}/environ", "rb") as f:
            for kv in f.read().split(b"\0"):
                if kv.startswith(b"DEEPTIME_ADMIT_JOB="):
                    return kv.split(b"=", 1)[1].decode(errors="replace")
    except OSError:
        pass
    return None


def _tcp_rows():
    rows = []
    for path in ("/proc/net/tcp", "/proc/net/tcp6"):
        try:
            with open(path) as f:
                next(f)
                for line in f:
                    c = line.split()
                    rows.append((int(c[1].rsplit(":", 1)[1], 16), int(c[2].rsplit(":", 1)[1], 16), c[3], c[9]))
        except (OSError, StopIteration):
            pass
    return rows   # (local port, remote port, state, inode)


def _socket_inodes(pid):
    out = set()
    try:
        for fd in os.listdir(f"/proc/{pid}/fd"):
            try:
                t = os.readlink(f"/proc/{pid}/fd/{fd}")
            except OSError:
                continue
            if t.startswith("socket:["):
                out.add(t[8:-1])
    except OSError:
        pass
    return out


def gradle_daemons(client_pids, procs):
    """The Gradle daemons these Gradle clients are connected to (a daemon runs one build at a time,
    so while a client of ours is connected, the daemon's work is ours)."""
    if not client_pids:
        return set()
    rows = _tcp_rows()
    by_inode = {r[3]: r for r in rows}
    want_ports = set()
    for c in client_pids:
        for ino in _socket_inodes(c):
            r = by_inode.get(ino)
            if r and r[2] == "01":
                want_ports.add(r[1])
    if not want_ports:
        return set()
    daemon_inodes = {r[3] for r in rows if r[2] == "01" and r[0] in want_ports}
    out = set()
    for p in procs.values():
        if p.comm != "java":
            continue
        if "GradleDaemon" not in cmdline(p.pid):
            continue
        if _socket_inodes(p.pid) & daemon_inodes:
            out.add(p.pid)
    return out


class Sampler:
    """Measures one job: its process tree, the processes carrying its job id, and the Gradle
    daemon its client uses, minus the trees of other admitted jobs (nested admissions)."""

    def __init__(self, job_id, root_pid):
        self.job_id, self.root = job_id, root_pid
        self.prev = {}            # pid -> (start, ticks)
        self.prev_t = None
        self.env_cache = {}       # (pid, start) -> job id or None
        self.btime = boot_time()
        self.samples = []         # (t, cores, rss bytes)
        self.cpu_ticks = 0
        self.t0 = time.time()

    def attributed(self, procs, exclude_roots=()):
        kids = children_map(procs)
        mine = subtree(self.root, kids) if self.root in procs else set()
        for p in procs.values():
            if p.pid in mine:
                continue
            key = (p.pid, p.start)
            if key not in self.env_cache:
                job = environ_job(p.pid)
                # Gradle and Kotlin daemons outlive the build that started them: attributed by socket below.
                if job is not None and p.comm == "java" and ("GradleDaemon" in cmdline(p.pid) or "KotlinCompileDaemon" in cmdline(p.pid)):
                    job = None
                self.env_cache[key] = job
            if self.env_cache[key] == self.job_id:
                mine |= subtree(p.pid, kids)
        clients = {q for q in mine if procs[q].comm == "java" and "GradleWrapperMain" in cmdline(q)} if mine else set()
        for d in gradle_daemons(clients, procs):
            mine |= subtree(d, kids)
        for r in exclude_roots:
            if r in procs and r != self.root:
                mine -= subtree(r, kids)
        return mine

    def sample(self, exclude_roots=()):
        now = time.time()
        procs = scan()
        mine = self.attributed(procs, exclude_roots)
        cur = {q: (procs[q].start, procs[q].ticks) for q in mine}
        rss = sum(procs[q].rss for q in mine)
        if self.prev_t is not None:
            dticks = 0
            for q, (st, ticks) in cur.items():
                was = self.prev.get(q)
                if was and was[0] == st:
                    dticks += max(0, ticks - was[1])
                elif self.btime + st / HZ >= self.prev_t - 0.5:
                    dticks += ticks          # born since the last sample: all of it is this interval's
            # A process that ended was reaped by its parent, whose cutime/cstime now carry the child's
            # whole life, of which the part up to the last sample was counted already: take it off.
            # (A reaper outside the job -- an orphan's -- makes this undercount a little instead.)
            for q, (st, ticks) in self.prev.items():
                if cur.get(q, (None,))[0] != st:
                    dticks -= ticks
            dticks = max(0, dticks)
            dt = now - self.prev_t
            cores = min(dticks / HZ / dt, float(os.cpu_count() or 1)) if dt > 0 else 0.0
            self.cpu_ticks += dticks
            self.samples.append((now, cores, rss))
        else:
            self.samples.append((now, 0.0, rss))
        self.prev, self.prev_t = cur, now
        return self.samples[-1]

    def current(self, window=30.0):
        """(mean cores over the last window, rss GiB now)."""
        if not self.samples:
            return 0.0, 0.0
        t = self.samples[-1][0]
        recent = [s[1] for s in self.samples[1:] if t - s[0] <= window] or [0.0]
        return sum(recent) / len(recent), self.samples[-1][2] / GIB

    def summary(self):
        wall = time.time() - self.t0
        cores = [s[1] for s in self.samples[1:]]
        rss = [s[2] for s in self.samples]
        ordered = sorted(cores)
        return {"wall_s": round(wall, 1),
                "cpu_mean": round(self.cpu_ticks / HZ / wall, 2) if wall > 0 else 0.0,
                "cpu_p90": round(ordered[int(0.9 * (len(ordered) - 1))], 2) if ordered else 0.0,
                "cpu_peak": round(max(cores), 2) if cores else 0.0,
                "rss_mean_gib": round(sum(rss) / len(rss) / GIB, 2) if rss else 0.0,
                "rss_peak_gib": round(max(rss) / GIB, 2) if rss else 0.0,
                "samples": len(self.samples)}


# ------------------------------------------------------------------------------- the state

class Store:
    """state.json (jobs, queue, sys samples) and history.json under one flock."""

    def __init__(self, d=None, servers=None):
        self.dir = d or DIR
        # Each server job's private folder: <cache>/servers/<job id>, removed when the job ends.
        self.servers = servers or (os.path.join(CACHE, "servers") if d is None else os.path.join(self.dir, "servers"))
        os.makedirs(self.dir, exist_ok=True)
        os.makedirs(self.servers, exist_ok=True)
        self.lock_path = os.path.join(self.dir, "state.lock")
        self.state_path = os.path.join(self.dir, "state.json")
        self.hist_path = os.path.join(self.dir, "history.json")
        self.log_path = os.path.join(self.dir, "admit.log")
        self.fd = None

    def __enter__(self):
        self.fd = os.open(self.lock_path, os.O_RDWR | os.O_CREAT, 0o664)
        fcntl.flock(self.fd, fcntl.LOCK_EX)
        self.state = self._read(self.state_path, {"jobs": {}, "queue": {}, "sys": []})
        self._hist = None
        return self

    @property
    def history(self):
        if self._hist is None:
            self._hist = self._read(self.hist_path, {})
        return self._hist

    def __exit__(self, *exc):
        try:
            if exc[0] is None:
                self._write(self.state_path, self.state)
                if self._hist is not None:
                    self._write(self.hist_path, self._hist)
        finally:
            fcntl.flock(self.fd, fcntl.LOCK_UN)
            os.close(self.fd)
            self.fd = None

    @staticmethod
    def _read(path, default):
        try:
            with open(path) as f:
                return json.load(f)
        except (OSError, ValueError):
            return default

    @staticmethod
    def _write(path, data):
        tmp = f"{path}.{os.getpid()}.tmp"
        with open(tmp, "w") as f:
            json.dump(data, f, indent=1, sort_keys=True)
        os.chmod(tmp, 0o664)
        os.replace(tmp, path)

    def log(self, msg):
        try:
            if os.path.exists(self.log_path) and os.path.getsize(self.log_path) > 5 * 1024 * 1024:
                os.replace(self.log_path, self.log_path + ".1")
            with open(self.log_path, "a") as f:
                f.write(time.strftime("%Y-%m-%dT%H:%M:%SZ ", time.gmtime()) + msg + "\n")
        except OSError:
            pass

    def servers_dir(self):
        return self.servers


def cleanup(st, store=None):
    """Drops entries whose process is gone; returns the dropped jobs."""
    gone = []
    for table in ("jobs", "queue"):
        for jid, e in list(st[table].items()):
            if not alive(e.get("pid"), e.get("start")):
                gone.append((table, st[table].pop(jid)))
    for jid, e in st["jobs"].items():
        c = e.get("claim")
        if c and not alive(c.get("pid"), c.get("start")):
            e["claim"] = None
    if store:
        for table, e in gone:
            store.log(f"dropped {table[:-1]} {e['id']} {e['kind']} ({e.get('label')}): its process {e.get('pid')} is gone")
        # Folders of jobs that are gone (their admit process was killed), once nothing uses them.
        known = {e.get("server_dir") for e in st["jobs"].values()}
        try:
            names = os.listdir(store.servers_dir())
        except OSError:
            names = []
        old = [os.path.join(store.servers_dir(), n) for n in names
               if os.path.join(store.servers_dir(), n) not in known
               and time.time() - os.stat(os.path.join(store.servers_dir(), n)).st_mtime > 600]
        if old:
            used = dirs_in_use(old)
            for d in old:
                if d not in used:
                    shutil.rmtree(d, ignore_errors=True)
                    store.log(f"removed the leftover server folder {d}")
    return gone


def dirs_in_use(dirs):
    """The folders some process has its working directory in."""
    out = set()
    for d in os.listdir("/proc"):
        if not d.isdigit():
            continue
        try:
            cwd = os.readlink(f"/proc/{d}/cwd")
        except OSError:
            continue
        for x in dirs:
            if cwd == x or cwd.startswith(x + os.sep):
                out.add(x)
    return out


def legacy_job(now):
    """The old server lock (mac.sh --server, preview.py, detached-server.sh before this tool) as an
    admitted server job, or None."""
    try:
        st = os.stat(LEGACY_LOCK)
    except OSError:
        return None
    if now - st.st_mtime > LEGACY_STALE_S:
        return None
    try:
        with open(os.path.join(LEGACY_LOCK, "owner")) as f:
            owner = f.read().strip()
    except OSError:
        owner = "?"
    # Its processes are not ours to measure: its estimate is reserved for RAMP_S after it was taken,
    # its memory for ten minutes; after that its use is in the measured load.
    age = now - st.st_mtime
    est = estimate("server:legacy", {})
    return {"id": "legacy-server-lock", "kind": "server:legacy", "label": owner, "admitted_at": st.st_mtime,
            "est": est, "legacy": True,
            "measured": {"cpu": 0.0, "cpu_max": 0.0 if age < RAMP_S else est["cpu"],
                         "rss_gib": 0.0 if age < 600 else est["mem_gib"]}}


def sys_sample(st, now):
    busy, total = read_proc_stat()
    s = [x for x in st.get("sys", []) if now - x[0] <= 90]
    s.append([now, busy, total])
    st["sys"] = s[-40:]
    return st["sys"]


def port_free(port, udp=False):
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM if udp else socket.SOCK_STREAM)
    try:
        if not udp:
            s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
        s.bind(("0.0.0.0", port))
        return True
    except OSError:
        return False
    finally:
        s.close()


def allocate_slot(st, check=port_free):
    used = {e.get("slot") for e in st["jobs"].values() if e.get("slot") is not None}
    for n in range(SLOTS):
        if n in used:
            continue
        base = PORT_BASE + n * PORT_STRIDE
        ports = range(base, base + PORT_STRIDE)
        if any(p in RESERVED_PORTS for p in ports):
            continue
        if all(check(p) for p in ports[:3]) and check(base + 3, True) and check(base + 3):
            return n
    return None


def slot_env(n, server_dir):
    base = PORT_BASE + n * PORT_STRIDE
    return {"DEEPTIME_SERVER_SLOT": str(n), "DEEPTIME_PORT_BASE": str(base), "DEEPTIME_PORT_GAME": str(base),
            "DEEPTIME_PORT_RCON": str(base + 1), "DEEPTIME_PORT_QUERY": str(base + 2),
            "DEEPTIME_PORT_VOICE": str(base + 3), "DEEPTIME_DISPLAY": f":{DISPLAY_BASE + n}",
            "DEEPTIME_SERVER_DIR": server_dir}


def new_id():
    return time.strftime("%Y%m%d-%H%M%S-") + "".join(random.choice(string.ascii_lowercase + string.digits) for _ in range(4))


# ------------------------------------------------------------------------------------- run

def run(kind, cmd, server=False, label=None, status_json=None, cpu=None, mem=None, store=None, quiet=False):
    """Admit, run cmd (argv list), measure, record. Returns the exit status."""
    store = store or Store()
    label = label or os.path.basename(os.getcwd())
    text = " ".join(cmd)
    if kind == "auto":
        # `zsh -c '<line>'`, `nix develop <profile> -c zsh -c '<line>'`: classify the innermost line.
        cs = [i for i, w in enumerate(cmd[:-1]) if w == "-c"]
        kind = classify(cmd[cs[-1] + 1] if cs else text)
    if server and not is_server_kind(kind):
        # A server job of a script the classifier does not know: its kind is the script's name.
        words = re.findall(r"[^\s'\";&|()]+", text)
        script = next((os.path.basename(w) for w in reversed(words) if re.search(r"\.(sh|py)$", w)), "other")
        kind = "server:" + re.sub(r"\.(sh|py)$", "", script)
    server = server or is_server_kind(kind)
    say = (lambda m: None) if quiet else (lambda m: print(f"admit: {m}", file=sys.stderr, flush=True))
    env = dict(os.environ)
    if kind == "light":
        return _exec_child(cmd, env, None, None, store, say)

    parent = env.get("DEEPTIME_ADMIT_JOB")
    me, my_start = os.getpid(), pid_start(os.getpid())
    job_id = new_id()
    claimed = None
    with store as s:
        cleanup(s.state, s)
        pj = s.state["jobs"].get(parent) if parent else None
        if pj and not server:
            # Part of an admitted job: run at once, measured with it.
            pass
        elif pj and server and pj.get("slot") is not None and not pj.get("claim"):
            pj["claim"] = {"pid": me, "start": my_start, "job": job_id}
            claimed = pj
            sd = os.path.join(s.servers_dir(), job_id)
            os.makedirs(sd, exist_ok=True)
            s.state["jobs"][job_id] = {"id": job_id, "kind": kind, "label": label, "pid": me, "start": my_start,
                                       "admitted_at": time.time(), "enq": time.time(), "inherited": parent,
                                       "est": estimate(kind, s.history), "slot": pj["slot"], "server_dir": sd,
                                       "owns_dir": True, "cmd": text[:200], "measured": {}}
            s.log(f"admitted {job_id} {kind} ({label}) in the slot of {parent} ({pj['kind']}); waited 0 s")
        else:
            pj = None
        if not pj and not claimed:
            est = estimate(kind, s.history)
            if cpu is not None:
                est["cpu"], est["source"] = float(cpu), "given"
            if mem is not None:
                est["mem_gib"], est["source"] = float(mem), "given"
            s.state["queue"][job_id] = {"id": job_id, "kind": kind, "label": label, "pid": me, "start": my_start,
                                        "enq": time.time(), "est": est, "server": server, "cmd": text[:200]}
    if parent and pj and not claimed and not server:
        return _exec_child(cmd, env, None, None, store, say)

    if not claimed:
        _install_wait_signals(store, job_id)
        waited_from = time.time()
        last_reason, last_said = None, 0.0
        while True:
            with store as s:
                now = time.time()
                cleanup(s.state, s)
                samples = sys_sample(s.state, now)
                cap = capacity(samples, now)
                jobs = list(s.state["jobs"].values())
                leg = legacy_job(now)
                if leg:
                    jobs.append(leg)
                decision, info = plan(list(s.state["queue"].values()), jobs, cap, now)
                d = decision.get(job_id, "not queued")
                if d is True:
                    q = s.state["queue"].pop(job_id)
                    slot = sd = None
                    if server:
                        slot = allocate_slot(s.state)
                        if slot is None:
                            s.state["queue"][job_id] = q
                            d = f"no free server slot (all {SLOTS} in use or their ports taken)"
                    if d is True:
                        if server:
                            sd = os.path.join(s.servers_dir(), job_id)
                            os.makedirs(sd, exist_ok=True)
                        q.update({"admitted_at": now, "slot": slot, "server_dir": sd, "owns_dir": bool(sd),
                                  "measured": {}, "claim": None, "waited_s": round(now - waited_from, 1),
                                  "admitted_with": {"free_cpu": cap["free_cpu"], "load1": cap["load1"],
                                                    "busy30": cap["busy30"], "mem_avail_gib": cap["mem_avail_gib"],
                                                    **info}})
                        s.state["jobs"][job_id] = q
                        s.log(f"admitted {job_id} {kind} ({label}) after {now - waited_from:.0f} s: estimate "
                              f"{q['est']['cpu']:.1f} cores {q['est']['mem_gib']:.1f} GiB ({q['est']['source']}); free "
                              f"{cap['free_cpu']:.1f} cores (load {cap['load1']:.1f}, busy {cap['busy30']}) "
                              f"{cap['mem_avail_gib']:.1f} GiB, reserved by running jobs {info['pending_cpu']:.1f} cores "
                              f"{info['pending_mem']:.1f} GiB" + (f"; slot {slot}" if server else ""))
                        break
            if d is not True and (d != last_reason or time.time() - last_said >= 60):
                say(f"{kind} waiting ({time.time() - waited_from:.0f} s): {d}")
                if status_json:
                    _merge_json(status_json, {"state": "waiting", "message": f"waiting for room on the test VM: {d}"})
                last_reason, last_said = d, time.time()
            time.sleep(POLL_S)
        job = q
        say(f"{kind} admitted after {job['waited_s']:.0f} s (estimate {job['est']['cpu']:.1f} cores, "
            f"{job['est']['mem_gib']:.1f} GiB, {job['est']['source']})"
            + (f"; ports {PORT_BASE + job['slot'] * PORT_STRIDE}..{PORT_BASE + job['slot'] * PORT_STRIDE + 9}" if server else ""))
        if status_json:
            _merge_json(status_json, {"state": "starting", "message": "admitted on the test VM"})
    else:
        with store as s:
            job = s.state["jobs"][job_id]

    env["DEEPTIME_ADMIT_JOB"] = job_id
    env["DEEPTIME_ADMIT_KIND"] = kind
    if job.get("slot") is not None:
        env.update(slot_env(job["slot"], job["server_dir"]))
    return _exec_child(cmd, env, job_id, job, store, say)


def _merge_json(path, fields):
    try:
        with open(path) as f:
            d = json.load(f)
    except (OSError, ValueError):
        d = {}
    d.update(fields)
    tmp = path + ".admit.tmp"
    try:
        with open(tmp, "w") as f:
            json.dump(d, f, indent=1)
        os.replace(tmp, path)
    except OSError:
        pass


def _install_wait_signals(store, job_id):
    def bail(sig, _frame):
        with store as s:
            s.state["queue"].pop(job_id, None)
        sys.exit(128 + sig)
    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sig, bail)


def _exec_child(cmd, env, job_id, job, store, say):
    """Runs cmd as a child (it sees its own pid as DEEPTIME_ADMIT_PID), samples it while it runs when
    it is a job of its own, records the result. Returns its exit status."""
    r, w = os.pipe()
    pid = os.fork()
    if pid == 0:
        os.close(r)
        env = dict(env)
        env["DEEPTIME_ADMIT_PID"] = str(os.getpid())
        for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
            signal.signal(sig, signal.SIG_DFL)
        try:
            os.execvpe(cmd[0], cmd, env)
        except OSError as e:
            os.write(w, f"{cmd[0]}: {e}".encode())
            os._exit(127)
    os.close(w)
    err = os.read(r, 4096)
    os.close(r)
    if err:
        say(err.decode(errors="replace"))

    def forward(sig, _frame):
        try:
            os.kill(pid, sig)
        except OSError:
            pass
    for sig in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(sig, forward)

    sampler, stop = None, threading.Event()
    if job_id:
        sampler = Sampler(job_id, pid)

        def loop():
            while not stop.wait(SAMPLE_S):
                try:
                    with open(store.state_path) as f:
                        st = json.load(f)
                    others = [e["pid"] for k, e in st.get("jobs", {}).items() if k != job_id]
                    sampler.sample(exclude_roots=others)
                    cpu, rss = sampler.current()
                    with store as s:
                        e = s.state["jobs"].get(job_id)
                        if e is not None:
                            m = e.setdefault("measured", {})
                            m.update({"cpu": round(cpu, 2), "rss_gib": round(rss, 2),
                                      "cpu_max": round(max(m.get("cpu_max", 0.0), cpu), 2),
                                      "rss_max_gib": round(max(m.get("rss_max_gib", 0.0), rss), 2), "t": time.time()})
                except Exception as ex:  # measuring must never kill the job
                    say(f"sampling failed: {ex}")
        try:
            sampler.sample()
        except Exception:
            pass
        threading.Thread(target=loop, daemon=True).start()
    while True:
        try:
            _, status = os.waitpid(pid, 0)
            break
        except InterruptedError:
            continue
        except ChildProcessError:
            status = 0
            break
    rc = os.waitstatus_to_exitcode(status)
    rc = 128 - rc if rc < 0 else rc
    if job_id:
        stop.set()
        try:
            sampler.sample(exclude_roots=[])
        except Exception:
            pass
        _finish(store, job_id, job, sampler.summary(), rc, say)
    return rc


def _finish(store, job_id, job, summ, rc, say):
    with store as s:
        e = s.state["jobs"].pop(job_id, None) or job
        if e.get("inherited"):
            p = s.state["jobs"].get(e["inherited"])
            if p and p.get("claim") and p["claim"].get("job") == job_id:
                p["claim"] = None
                p["claimed_s"] = p.get("claimed_s", 0.0) + summ["wall_s"]
        record = {"end": time.time(), "rc": rc, "label": e.get("label"), **summ}
        # Short runs say little about load, except for unrecognised commands, which learn from
        # them that they are quick (see quick()).
        recorded = (summ["wall_s"] >= MIN_RECORD_S and summ["samples"] >= 3) or e["kind"].startswith("shell:")
        # A job whose time went mostly to a server launcher that took over its slot measured that
        # launcher (recorded under the launcher's kind): its own record would only be the wrapper.
        if e.get("claimed_s", 0.0) > 0.5 * summ["wall_s"]:
            recorded = False
        if recorded:
            runs = s.history.setdefault(e["kind"], [])
            runs.append(record)
            del runs[:-HISTORY_KEEP]
        s.log(f"finished {job_id} {e['kind']} ({e.get('label')}) rc {rc}: {summ['wall_s']:.0f} s, cpu mean "
              f"{summ['cpu_mean']:.2f} p90 {summ['cpu_p90']:.2f} peak {summ['cpu_peak']:.2f} cores, rss mean "
              f"{summ['rss_mean_gib']:.2f} peak {summ['rss_peak_gib']:.2f} GiB; estimate was "
              f"{e['est']['cpu']:.1f} cores {e['est']['mem_gib']:.1f} GiB" + ("" if recorded else " (not recorded)"))
        if e.get("server_dir") and e.get("owns_dir") and not os.environ.get("DEEPTIME_KEEP_SERVER_DIR"):
            shutil.rmtree(e["server_dir"], ignore_errors=True)
    say(f"{e['kind']} done in {summ['wall_s']:.0f} s: {summ['cpu_mean']:.1f} cores mean ({summ['cpu_peak']:.1f} peak), "
        f"{summ['rss_peak_gib']:.1f} GiB peak (estimate {e['est']['cpu']:.1f} cores, {e['est']['mem_gib']:.1f} GiB)")


# ------------------------------------------------------------------------------------ status

def status(as_json=False, store=None):
    store = store or Store()
    with store as s:
        now = time.time()
        cleanup(s.state, s)
        samples = sys_sample(s.state, now)
        cap = capacity(samples, now)
        jobs = list(s.state["jobs"].values())
        leg = legacy_job(now)
        if leg:
            jobs.append(leg)
        decision, info = plan(list(s.state["queue"].values()), jobs, cap, now)
        kinds = sorted(set(s.history) | {e["kind"] for e in jobs} | {e["kind"] for e in s.state["queue"].values()})
        ests = {k: estimate(k, s.history) for k in kinds}
        queue = sorted(s.state["queue"].values(), key=lambda e: e["enq"])
    if as_json:
        print(json.dumps({"capacity": cap, "plan": info, "jobs": jobs, "queue": queue,
                          "waiting_reason": {k: v for k, v in decision.items() if v is not True},
                          "estimates": ests}, indent=1, sort_keys=True))
        return 0
    print(f"capacity: {cap['cpu_total']:.0f} cores, load {cap['load1']:.1f}, busy (30 s) {cap['busy30']}, free "
          f"{cap['free_cpu']:.1f} cores; {cap['mem_avail_gib']:.1f} of {cap['mem_total_gib']:.0f} GiB available")
    print(f"reserved by running jobs (not yet used): {info['pending_cpu']:.1f} cores, {info['pending_mem']:.1f} GiB")
    print(f"rule, memory: a new job starts when its estimate x {1 + OVERCOMMIT:g} is under the free memory "
          f"({info['avail_mem']:.1f} GiB now = available minus reserved, no standing headroom): "
          f"largest estimate that starts now {max(info['avail_mem'], 0) / (1 + OVERCOMMIT):.1f} GiB; running jobs are never stopped")
    print(f"rule, CPU: a new job starts when its estimate fits in {1 + OVERCOMMIT:g} x the free cores "
          f"({info['avail_cpu']:.1f} now = free minus reserved minus {RESERVE_CORES:g} kept for the play server and ssh)")
    print(f"\nadmitted ({len(jobs)}):")
    for e in sorted(jobs, key=lambda e: e.get("admitted_at", 0)):
        m = e.get("measured", {})
        slot = e.get("slot")
        print(f"  {e['id']:<24} {e['kind']:<30} {str(e.get('label'))[:28]:<28} {(now - e.get('admitted_at', now)) / 60:5.1f} min"
              f"  est {e['est']['cpu']:4.1f}c {e['est']['mem_gib']:4.1f}G  now {m.get('cpu', 0):4.1f}c {m.get('rss_gib', 0):4.1f}G"
              + (f"  ports {PORT_BASE + slot * PORT_STRIDE}+" if slot is not None else "")
              + (f"  (in {e['inherited']}'s slot)" if e.get("inherited") else ""))
    print(f"\nwaiting ({len(queue)}):")
    for e in queue:
        print(f"  {e['id']:<24} {e['kind']:<30} {str(e.get('label'))[:28]:<28} {(now - e['enq']) / 60:5.1f} min"
              f"  est {e['est']['cpu']:4.1f}c {e['est']['mem_gib']:4.1f}G  {decision.get(e['id']) if decision.get(e['id']) is not True else 'starting'}")
    print("\nestimates (cores = mean over a run, GiB = peak resident):")
    for k in kinds:
        v = ests[k]
        print(f"  {k:<40} {v['cpu']:5.1f} cores {v['mem_gib']:5.1f} GiB  peak {v['peak_cpu']:5.1f} cores  ~{v['wall_s'] / 60:4.0f} min  {v['source']}")
    return 0


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    if not argv or argv[0] in ("-h", "--help"):
        print(__doc__)
        return 0
    verb, rest = argv[0], argv[1:]
    if verb == "run":
        kind, server, label, sj, cpu, mem, quiet = "auto", False, None, None, None, None, False
        while rest and rest[0] != "--":
            a = rest.pop(0)
            if a == "--kind":
                kind = rest.pop(0)
            elif a == "--server":
                server = True
            elif a == "--label":
                label = rest.pop(0)
            elif a == "--status-json":
                sj = rest.pop(0)
            elif a == "--cpu":
                cpu = float(rest.pop(0))
            elif a == "--mem-gib":
                mem = float(rest.pop(0))
            elif a == "--quiet":
                quiet = True
            else:
                print(f"admit: unknown option {a}", file=sys.stderr)
                return 2
        if not rest or rest[0] != "--" or len(rest) < 2:
            print("usage: admit.py run [--kind K|auto] [--server] [--label L] [--status-json F] -- <command...>", file=sys.stderr)
            return 2
        # Without the dev shell's library path the system tools this may call work; the command
        # gets it back.
        return run(kind, rest[1:], server=server, label=label, status_json=sj, cpu=cpu, mem=mem, quiet=quiet)
    if verb == "status":
        return status("--json" in rest)
    if verb == "estimate" and rest:
        with Store() as s:
            print(json.dumps(estimate(rest[0], s.history), indent=1))
        return 0
    if verb == "classify":
        if rest and rest[0] == "--":
            rest = rest[1:]
        print(classify(" ".join(rest)))
        return 0
    if verb == "forget" and rest:
        with Store() as s:
            n = len(s.history.pop(rest[0], []))
            s.log(f"forgot {n} runs of {rest[0]}")
        print(f"forgot {n} runs of {rest[0]}")
        return 0
    print(f"admit: unknown command {verb} (run, status, estimate, classify, forget)", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main())
