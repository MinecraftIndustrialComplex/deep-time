#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The shared test-run pool (Ben, 2026-10-22: "If both deltas and continent spread need to do a world gen
to test they should just merge their test."). Runs on the test VM; the laptop's entry is pool.sh. See
README.md for the commands and the layout.

    pool.py run     --key K --arm NAME --preset P --length L --starts A,B --seeds 1,2 [--threads T]
                    [--set k=v]... [+ENV=v]... [--measures a,b] [--keep-ck] [--retry] [--rerun]
    pool.py measure --arm NAME [--measures a,b]      measures over the cached folders, no sim
    pool.py table   --base ARM --arm ARM [--arm ARM ...] [--measure v1census] [--json FILE]
    pool.py alias   NEW OLD --check-seed S [--preset P --length L --start A --threads T --set k=v]
    pool.py build   --key K                          the binary of a source key (run does it itself)
    pool.py arms | status | gc [--days N] [--apply] | keep KEY [reason]

Every simulation, plate check, field dump, build and measure is admitted on its own by the VM's
deeptime-admit; the orchestrator itself is light and is started through detach.sh by pool.sh.
Standard library only (measures run in their own Python, resolved with nix)."""
import argparse
import fcntl
import hashlib
import json
import os
import re
import shutil
import signal
import socket
import subprocess
import sys
import threading
import time
from contextlib import contextmanager
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import pooltable  # noqa: E402
import srckey  # noqa: E402

POOL = Path(os.environ.get("DEEPTIME_POOL", "/data/ban/deeptime-runs/pool"))
STAGE = "volcanoes"
MIN_FREE = 6 * 2 ** 30  # a run needs about 1.2 GiB while it runs; refuse to start under this much free
KEY_RE = re.compile(r"^[0-9a-f]{12}$")
PYENV_EXPR = ('(builtins.getFlake "nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages '
              '(p: [%s])')

_say_lock = threading.Lock()
_children = []
_children_lock = threading.Lock()


class Fail(Exception):
    pass


def say(msg):
    with _say_lock:
        print(msg, flush=True)


def now():
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def atomic_write(path, text):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f".{path.name}.{os.getpid()}.{threading.get_ident()}.tmp")
    tmp.write_text(text)
    os.replace(tmp, path)


def read_json(path, default=None):
    try:
        return json.loads(Path(path).read_text())
    except (OSError, ValueError):
        return default


@contextmanager
def flock(path, what=None, blocking=True):
    """An exclusive flock on a lock file. With `what`, says so when it has to wait for another holder. The
    OS drops the lock when the holder dies, so a crashed run never leaves one behind."""
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(path, os.O_RDWR | os.O_CREAT, 0o644)
    waited = 0.0
    try:
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            if not blocking:
                raise
            if what:
                say(f"{what}: another request holds it; waiting")
            t0 = time.time()
            fcntl.flock(fd, fcntl.LOCK_EX)
            waited = time.time() - t0
        os.ftruncate(fd, 0)
        os.write(fd, f"{socket.gethostname()} pid {os.getpid()} since {now()}\n".encode())
        yield waited
    finally:
        os.close(fd)


def lock_held(path):
    """True when someone holds the lock file (a nonblocking try)."""
    try:
        fd = os.open(path, os.O_RDWR)
    except OSError:
        return False
    try:
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        return False
    except BlockingIOError:
        return True
    finally:
        os.close(fd)


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def sha256_tree(folder):
    h = hashlib.sha256()
    for p in sorted(Path(folder).rglob("*")):
        if p.is_file():
            h.update(f"{p.relative_to(folder)} {sha256_file(p)}\n".encode())
    return h.hexdigest()


def tree_bytes(path):
    total = 0
    for dp, _dn, fn in os.walk(path):
        for f in fn:
            try:
                total += os.lstat(os.path.join(dp, f)).st_size
            except OSError:
                pass
    return total


def human(n):
    for unit in ("B", "KiB", "MiB", "GiB"):
        if n < 1024 or unit == "GiB":
            return f"{n:.1f} {unit}" if unit != "B" else f"{n} B"
        n /= 1024.0


# ----------------------------------------------------------------------------- registry and arms

def load_registry(extra=()):
    reg = read_json(HERE / "measures.json")
    reg.setdefault("python_envs", {"numpy": ["numpy", "scipy"]})
    files = list(extra) + [f for f in os.environ.get("DEEPTIME_POOL_MEASURES", "").split(":") if f]
    for f in files:
        more = read_json(f)
        if more is None:
            raise Fail(f"cannot read the measures registry {f}")
        reg["measures"].update(more.get("measures", {}))
        reg["dump_fields"] = list(dict.fromkeys(reg.get("dump_fields", []) + more.get("dump_fields", [])))
        reg["python_envs"].update(more.get("python_envs", {}))
    return reg


def dump_fields_of(reg, names):
    out = list(reg.get("dump_fields", []))
    for n in names:
        out += reg["measures"][n].get("fields", [])
    return list(dict.fromkeys(out))


def argkey_of(sets, envs, threads):
    blob = json.dumps({"sets": sorted(sets), "envs": sorted(envs), "threads": threads}, sort_keys=True)
    return hashlib.sha256(blob.encode()).hexdigest()[:8]


def arm_dir(key, preset, length, argkey):
    return POOL / key / f"{preset}-{length}" / argkey


def aliases():
    return read_json(POOL / "aliases.json", {})


def resolve_key(key, preset, length, argkey):
    """The key whose folder holds this arm's runs: the key itself, or the one it is a verified alias of."""
    a = aliases().get(key)
    if a and (a.get("all_arms") or [preset, str(length), argkey] in a.get("arms", [])):
        return a["of"]
    return key


def touch_used(key):
    d = POOL / key
    if d.is_dir():
        try:
            (d / "used").write_text(now() + "\n")
        except OSError:
            pass


def arm_path(name):
    if not re.match(r"^[A-Za-z0-9_.+-]+$", name):
        raise Fail(f"arm name {name!r}: use letters, digits and _ . + - only")
    return POOL / "arms" / f"{name}.json"


def load_arm(name):
    arm = read_json(arm_path(name))
    if arm is None:
        raise Fail(f"no arm named {name} (pool.sh arms lists them; pool.sh run --arm {name} ... makes one)")
    return arm


# ----------------------------------------------------------------------------- admitted steps

def admit_cmd():
    p = shutil.which("deeptime-admit")
    return [p] if p else ["/usr/bin/python3", "/usr/local/lib/deeptime/admit.py"]


def _kill_children(*_):
    with _children_lock:
        procs = list(_children)
    for p in procs:
        try:
            p.terminate()
        except OSError:
            pass
    os._exit(143)


def step(kind, label, cmd, cwd=None, env_extra=None, stdout=None, capture_stdout=False):
    """One heavy step, admitted on its own (the orchestrator's own job is light, so none is a part of
    another). stdout goes to the open file `stdout`, or is returned; stderr joins stdout for a file and is
    returned apart otherwise. Returns (exit status, stdout text or None, stderr text or '')."""
    env = {k: v for k, v in os.environ.items() if k not in ("DEEPTIME_ADMIT_JOB", "DEEPTIME_ADMIT_PID")}
    env.update(env_extra or {})
    full = admit_cmd() + ["run", "--kind", kind, "--label", label, "--"] + [str(c) for c in cmd]
    if capture_stdout:
        p = subprocess.Popen(full, cwd=cwd, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    else:
        p = subprocess.Popen(full, cwd=cwd, env=env, stdout=stdout, stderr=subprocess.STDOUT)
    with _children_lock:
        _children.append(p)
    try:
        out, err = p.communicate()
    finally:
        with _children_lock:
            _children.remove(p)
    return p.returncode, out, err or ""


# ----------------------------------------------------------------------------- the binary

def ensure_binary(key, checkout, note=None, branch=None, commit=None):
    kd = POOL / key
    binp = kd / "deeptime.bin"
    if binp.exists():
        touch_used(key)
        return binp
    if checkout is None:
        raise Fail(f"no binary for source key {key} in the pool and no checkout to build it from (build it from a checkout "
                   f"at that key with pool.sh run, or with combine.sh)")
    got = srckey.compute(checkout, None)
    if got != key:
        raise Fail(f"the checkout {checkout} has source key {got}, not {key}: sync it again or commit the change")
    kd.mkdir(parents=True, exist_ok=True)
    with flock(kd / ".install.lock", what=f"building {key}") as waited:
        if binp.exists():
            say(f"binary {key}: built meanwhile by another request (waited {waited:.0f} s); reused")
            touch_used(key)
            return binp
        say(f"binary {key}: building from {checkout} (cargo build --release -p deeptime-cli)")
        t0 = time.time()
        rc, _o, _e = step("cargo:build", f"pool-build-{key[:6]}", ["cargo", "build", "--release", "-q", "-p", "deeptime-cli"],
                          cwd=str(checkout), stdout=sys.stdout)
        built = Path(checkout) / "target" / "release" / "deeptime"
        if rc != 0 or not built.exists():
            raise Fail(f"cargo build failed for {key} (exit {rc})")
        part = kd / ".deeptime.bin.part"
        shutil.copyfile(built, part)
        os.chmod(part, 0o755)
        os.replace(part, binp)
        atomic_write(kd / "meta.json", json.dumps({
            "srckey": key, "built": now(), "build_seconds": round(time.time() - t0), "checkout": str(checkout),
            "branch": branch, "commit": commit, "note": note, "binary_sha256": sha256_file(binp)}, indent=1) + "\n")
        say(f"binary {key}: built in {time.time() - t0:.0f} s")
    touch_used(key)
    return binp


# ----------------------------------------------------------------------------- measures

def pyenv(reg, name):
    """The python of a nix environment with the named packages (GC-rooted under the pool), built once."""
    pkgs = reg["python_envs"].get(name)
    if pkgs is None:
        raise Fail(f"the registry names no python environment {name!r}")
    link = POOL / f".pyenv-{name}"
    py = link / "bin" / "python3"
    if py.exists():
        return str(py)
    with flock(POOL / f".pyenv-{name}.lock", what=f"building the python environment {name}"):
        if not py.exists():
            expr = PYENV_EXPR % " ".join(f"p.{p}" for p in pkgs)
            r = subprocess.run(["nix", "build", "--impure", "--expr", expr, "--out-link", str(link)], capture_output=True, text=True)
            if r.returncode != 0 or not py.exists():
                raise Fail(f"nix build of the python environment {name} failed: {r.stderr.strip()[-400:]}")
    return str(py)


def measure_needs_ok(rd, spec):
    problems = []
    need = spec.get("needs", [])
    if "fields" in need:
        idx = read_json(Path(rd) / "fields" / "index.json")
        if idx is None:
            problems.append("no fields/ dump")
        else:
            missing = [f for f in spec.get("fields", []) if f not in idx]
            if missing:
                problems.append("the dump lacks " + ", ".join(missing) + " (the checkpoint is gone; rerun the run with --rerun)")
    for n, fname in (("plate", "plate.json"), ("rec", "rec.json")):
        if n in need and not (Path(rd) / fname).exists():
            problems.append(f"no {fname}")
    return problems


def run_measure(reg, name, rd, label):
    """Compute one measure over a run folder (final or staging) and write measures/<name>.json."""
    spec = reg["measures"][name]
    problems = measure_needs_ok(rd, spec)
    if problems:
        raise Fail(f"measure {name}: " + "; ".join(problems))
    py = pyenv(reg, spec["python"]) if spec.get("python") else sys.executable
    cmd = [c.replace("{run}", str(rd)).replace("{tools}", str(HERE)).replace("{py}", py) for c in spec["command"]]
    rc, out, err = step(f"shell:pool-measure-{name}", f"{label}-{name}", ["nice", "-n", "10"] + cmd, capture_stdout=True)
    if rc != 0:
        raise Fail(f"measure {name} exited {rc}: {(err or out or '').strip()[-600:]}")
    try:
        values = json.loads(out.strip().splitlines()[-1])
    except (ValueError, IndexError):
        raise Fail(f"measure {name} printed no JSON object: {(out or '')[-300:]}")
    atomic_write(Path(rd) / "measures" / f"{name}.json",
                 json.dumps({"name": name, "version": spec["version"], "at": now(), "values": values}, indent=1, sort_keys=True) + "\n")
    (Path(rd) / "measures" / f"{name}.error").unlink(missing_ok=True)
    return values


def ensure_measures(reg, rd, names, label):
    """Compute the measures of `names` the folder lacks or holds at an older version. Returns the names
    computed; a failing measure is reported and leaves a .error file, never fails the run."""
    done = []
    for name in names:
        spec = reg["measures"][name]
        have = read_json(Path(rd) / "measures" / f"{name}.json")
        if have and have.get("version") == spec["version"]:
            continue
        try:
            run_measure(reg, name, rd, label)
            done.append(name)
        except Fail as e:
            atomic_write(Path(rd) / "measures" / f"{name}.error", f"{now()} {e}\n")
            say(f"{label}: {e}")
    return done


def refresh_meta_measures(rd, reg):
    meta = read_json(Path(rd) / "meta.json")
    if meta is None:
        return
    meta["measures"] = {}
    for p in sorted((Path(rd) / "measures").glob("*.json")):
        m = read_json(p)
        if m:
            meta["measures"][m["name"]] = m["version"]
    atomic_write(Path(rd) / "meta.json", json.dumps(meta, indent=1, sort_keys=True) + "\n")


# ----------------------------------------------------------------------------- one run

def ensure_run(ctx, start, seed):
    """Make sure the run (start, seed) of the arm exists, with its measures. Returns (status, run folder).
    Per-run flock: a second request waits for the first and reuses it."""
    key = ctx["key"] if ctx.get("no_alias") else resolve_key(ctx["key"], ctx["preset"], ctx["length"], ctx["argkey"])
    arm = arm_dir(key, ctx["preset"], ctx["length"], ctx["argkey"])
    rid = f"{start}-{seed}"
    rd = arm / rid
    label = f"{key[:6]}/{ctx['preset']}-{ctx['length']}/{ctx['argkey']}/{rid}"
    failed = arm / f"{rid}.FAILED"
    t0 = time.time()
    if ctx.get("no_sim") and not (arm / rid / "meta.json").exists():
        return "missing", f"{label}: not in the pool (no simulation requested)"
    arm.mkdir(parents=True, exist_ok=True)
    if not (arm / "args.json").exists():
        atomic_write(arm / "args.json", json.dumps({
            "srckey": key, "preset": ctx["preset"], "length_myr": ctx["length"], "threads": ctx["threads"],
            "sets": sorted(ctx["sets"]), "envs": sorted(ctx["envs"]), "argkey": ctx["argkey"], "stage": STAGE}, indent=1) + "\n")
    touch_used(key)
    with flock(arm / f"{rid}.lock", what=f"run {label}") as waited:
        if ctx.get("rerun") and rd.exists():
            shutil.rmtree(rd)
        if ctx.get("retry") or ctx.get("rerun"):
            failed.unlink(missing_ok=True)
        status = None
        if (rd / "meta.json").exists():
            status = "reused" if waited < 1 else f"reused after waiting {waited:.0f} s for another request"
        elif failed.exists():
            return "FAILED", f"{label}: a previous attempt failed ({failed}); --retry runs it again:\n{failed.read_text()[-600:]}"
        elif ctx.get("no_sim"):
            return "missing", f"{label}: not in the pool (no simulation requested)"
        else:
            status = simulate(ctx, arm, rid, start, seed, label)
            if isinstance(status, tuple):
                return status
            if waited >= 1:
                status = f"{status} (after waiting {waited:.0f} s on a lock)"
        if ctx.get("keep_ck") and not (rd / "final.ck").exists():
            say(f"{label}: no checkpoint kept (it was deleted after the measures); --rerun to run it again")
        done = ensure_measures(ctx["reg"], rd, ctx["measures"], label)
        if done:
            refresh_meta_measures(rd, ctx["reg"])
            status += f"; measured {','.join(done)}"
    return status, rd


def step_file(kind, label, cmd, path, **kw):
    with open(path, "w") as f:
        return step(kind, label, cmd, stdout=f, **kw)


def simulate(ctx, arm, rid, start, seed, label):
    free = shutil.disk_usage(POOL).free
    if free < MIN_FREE:
        raise Fail(f"{label}: only {human(free)} free under {POOL}; run `pool.sh gc --apply` or free the disk")
    work = arm / f".work-{rid}"
    if work.exists():
        shutil.rmtree(work)
    (work / "measures").mkdir(parents=True)
    ck = work / "final.ck"
    binp = ctx["bin"]
    extra_env = {}
    for e in ctx["envs"]:
        k, _, v = e.partition("=")
        extra_env[k] = v.replace("@RUN@", str(work))
    t0 = time.time()
    say(f"{label}: running ({STAGE}, {ctx['length']} Myr, {ctx['threads']} threads)")
    cmd = ["nice", "-n", "10", str(binp), "stage", STAGE, "--preset", ctx["preset"], "--start", start,
           "--length-myr", str(ctx["length"]), "--threads", str(ctx["threads"]), "--set", f"seed={seed}",
           "--work", str(work / "sim.work"), "--out", str(ck)]
    for s in ctx["sets"]:
        cmd += ["--set", s]
    try:
        with open(work / "summary.txt", "w") as out:
            rc, _o, _e = step(f"sim:{ctx['preset']}:{ctx['threads']}t", f"pool-{label.replace('/', '-')}", cmd, env_extra=extra_env, stdout=out)
        sim_seconds = round(time.time() - t0)
        if rc != 0 or not ck.exists():
            raise Fail(f"the simulation exited {rc}")
        shutil.rmtree(work / "sim.work", ignore_errors=True)
        rc, _o, _e = step_file("shell:pool-plate-check", f"pool-{label.replace('/', '-')}-pc",
                               [str(binp), "plate-check", "--run", str(ck), "--json", str(work / "plate.json"),
                                "--record", str(work / "rec.json")], work / "plate-table.txt")
        if not (work / "plate.json").exists() or not (work / "rec.json").exists():
            raise Fail(f"plate-check wrote no result (exit {rc})")
        fields = dump_fields_of(ctx["reg"], ctx["measures"])
        rc, _o, _e = step_file("shell:pool-dump-fields", f"pool-{label.replace('/', '-')}-df",
                               [str(binp), "dump-fields", "--in", str(ck), "--out", str(work / "fields"), "--fields", ",".join(fields)],
                               work / "dump-fields.txt")
        if rc != 0 or not (work / "fields" / "index.json").exists():
            raise Fail(f"dump-fields failed (exit {rc}): {(work / 'dump-fields.txt').read_text()[-300:]}")
        meta = {
            "srckey": ctx["key"], "preset": ctx["preset"], "length_myr": ctx["length"], "start": start, "seed": seed,
            "threads": ctx["threads"], "sets": sorted(ctx["sets"]), "envs": sorted(ctx["envs"]), "argkey": ctx["argkey"],
            "stage": STAGE, "ran": now(), "sim_seconds": sim_seconds, "host": socket.gethostname(), "first_arm": ctx.get("arm"),
            "ck_sha256": sha256_file(ck), "ck_bytes": ck.stat().st_size, "fields_sha256": sha256_tree(work / "fields"),
            "dumped_fields": fields, "kept_ck": bool(ctx.get("keep_ck")), "measures": {}}
        # The measures run on the staging folder, so a finished folder always has them.
        ensure_measures(ctx["reg"], work, ctx["measures"], label)
        for p in sorted((work / "measures").glob("*.json")):
            m = read_json(p)
            if m:
                meta["measures"][m["name"]] = m["version"]
        if not ctx.get("keep_ck"):
            ck.unlink()
        atomic_write(work / "meta.json", json.dumps(meta, indent=1, sort_keys=True) + "\n")
        os.replace(work, arm / rid)
        say(f"{label}: ran in {sim_seconds} s, total {time.time() - t0:.0f} s")
        return "ran"
    except (Fail, OSError) as e:
        tail = ""
        try:
            tail = (work / "summary.txt").read_text()[-1500:]
        except OSError:
            pass
        atomic_write(arm / f"{rid}.FAILED", f"{now()} {e}\n{tail}\n")
        shutil.rmtree(work, ignore_errors=True)
        say(f"{label}: FAILED: {e} (marker {arm / (rid + '.FAILED')})")
        return ("FAILED", f"{label}: {e}")


def build_ctx(a, key):
    sets = list(a.set or [])
    if any(s.startswith("seed=") for s in sets):
        raise Fail("seed is the pool's own axis: use --seeds, not --set seed=")
    envs = list(a.envs)
    reg = load_registry(a.registry or [])
    names = [m for m in (a.measures.split(",") if a.measures else reg.get("default_measures", ["v1census"])) if m]
    for n in names:
        if n not in reg["measures"]:
            raise Fail(f"unknown measure {n} (registry has: {', '.join(reg['measures'])})")
    return {"key": key, "preset": a.preset, "length": a.length, "threads": a.threads, "sets": sets, "envs": envs,
            "argkey": argkey_of(sets, envs, a.threads), "measures": names, "reg": reg, "keep_ck": a.keep_ck,
            "retry": a.retry, "rerun": a.rerun, "arm": getattr(a, "arm", None)}


def run_many(ctx, runs):
    """All the runs at once (the admission tool queues them), 1 s apart. Returns {rid: (status, detail)}."""
    results, threads = {}, []

    def work(start, seed):
        rid = f"{start}-{seed}"
        try:
            results[rid] = ensure_run(ctx, start, seed)
        except Fail as e:
            results[rid] = ("FAILED", str(e))

    for start, seed in runs:
        t = threading.Thread(target=work, args=(start, seed), daemon=True)
        t.start()
        threads.append(t)
        time.sleep(1)
    for t in threads:
        t.join()
    return results


def report(results):
    bad = 0
    say("")
    for rid in sorted(results):
        status, detail = results[rid]
        if status in ("FAILED", "missing"):
            bad += 1
            say(f"RESULT {rid}: {status}: {detail}")
        else:
            say(f"RESULT {rid}: {status}")
    return 1 if bad else 0


def parse_runs(starts, seeds):
    starts = [s for s in starts.split(",") if s]
    seeds = [int(s) for s in seeds.split(",") if s]
    return [(st, sd) for st in starts for sd in seeds]


# ----------------------------------------------------------------------------- verbs

def cmd_run(a):
    ctx = build_ctx(a, a.key)
    runs = parse_runs(a.starts, a.seeds)
    binp = ensure_binary(a.key, Path(a.checkout) if a.checkout else None, a.note, a.branch, a.commit)
    ctx["bin"] = binp
    save_arm(a.arm, ctx, runs, a)
    say(f"arm {a.arm}: key {a.key}, {ctx['preset']}, {ctx['length']} Myr, argkey {ctx['argkey']}, threads {ctx['threads']}, "
        f"sets {ctx['sets'] or '-'}, envs {ctx['envs'] or '-'}, {len(runs)} runs")
    return report(run_many(ctx, runs))


def save_arm(name, ctx, runs, a):
    path = arm_path(name)
    with flock(POOL / "arms" / ".lock"):
        old = read_json(path)
        merged = [list(r) for r in runs]
        if old and (old["key"], old["preset"], old["length"], old["argkey"]) == (ctx["key"], ctx["preset"], ctx["length"], ctx["argkey"]):
            merged = sorted({tuple(r) for r in old["runs"]} | {tuple(r) for r in runs})
        elif old:
            say(f"arm {name}: replaces the arm of that name (it was key {old['key']}, {old['preset']}-{old['length']}, argkey {old['argkey']})")
        atomic_write(path, json.dumps({
            "name": name, "key": ctx["key"], "preset": ctx["preset"], "length": ctx["length"], "argkey": ctx["argkey"],
            "threads": ctx["threads"], "sets": sorted(ctx["sets"]), "envs": sorted(ctx["envs"]), "runs": [list(r) for r in merged],
            "branch": a.branch, "commit": a.commit, "note": a.note, "updated": now()}, indent=1) + "\n")


def cmd_build(a):
    ensure_binary(a.key, Path(a.checkout) if a.checkout else None, a.note, a.branch, a.commit)
    say(f"BUILD_OK {a.key} {POOL / a.key / 'deeptime.bin'}")
    return 0


def cmd_measure(a):
    reg = load_registry(a.registry or [])
    names = [m for m in (a.measures.split(",") if a.measures else reg.get("default_measures", ["v1census"])) if m]
    arms = [load_arm(n) for n in a.arm] if a.arm else [read_json(p) for p in sorted((POOL / "arms").glob("*.json"))]
    bad = 0
    for arm in arms:
        ctx = {"key": arm["key"], "preset": arm["preset"], "length": arm["length"], "argkey": arm["argkey"], "threads": arm["threads"],
               "sets": arm["sets"], "envs": arm["envs"], "measures": names, "reg": reg, "no_sim": True, "arm": arm["name"]}
        say(f"arm {arm['name']}: measures {','.join(names)} over {len(arm['runs'])} cached runs")
        bad += report(run_many(ctx, [tuple(r) for r in arm["runs"]]))
    return 1 if bad else 0


def arm_rows(arm, measure, version):
    """[(rid, values or None)] of an arm's runs for one measure."""
    key = resolve_key(arm["key"], arm["preset"], arm["length"], arm["argkey"])
    touch_used(key)
    rows = []
    for start, seed in arm["runs"]:
        rid = f"{start}-{seed}"
        m = read_json(arm_dir(key, arm["preset"], arm["length"], arm["argkey"]) / rid / "measures" / f"{measure}.json")
        rows.append((rid, m["values"] if m and m.get("version") == version else None))
    return rows


def cmd_table(a):
    reg = load_registry(a.registry or [])
    name = a.measure
    if name not in reg["measures"]:
        raise Fail(f"unknown measure {name} (registry has: {', '.join(reg['measures'])})")
    spec = reg["measures"][name]
    sel = []
    for n in [a.base] + a.arm:
        arm = load_arm(n)
        rows = arm_rows(arm, name, spec["version"])
        have = [v for _, v in rows if v is not None]
        miss = [rid for rid, v in rows if v is None]
        if miss:
            print(f"warning: arm {n}: {len(miss)} of {len(rows)} runs lack measure {name} ({', '.join(miss[:6])}{' ...' if len(miss) > 6 else ''});"
                  f" pool.sh run or pool.sh measure makes them", file=sys.stderr)
        sel.append((n, have))
    if any(not runs for _, runs in sel):
        raise Fail("an arm has no measured runs; nothing to tabulate")
    text, summary = pooltable.render(spec, sel[0], sel[1:], name)
    print(text, end="")
    if a.json:
        atomic_write(a.json, json.dumps(summary, indent=1, sort_keys=True, default=str) + "\n")
    return 0


def cmd_arms(a):
    rows = [read_json(p) for p in sorted((POOL / "arms").glob("*.json"))]
    if not rows:
        print("no arms")
    for arm in rows:
        key = resolve_key(arm["key"], arm["preset"], arm["length"], arm["argkey"])
        base = arm_dir(key, arm["preset"], arm["length"], arm["argkey"])
        have = sum(1 for s, sd in arm["runs"] if (base / f"{s}-{sd}" / "meta.json").exists())
        alias = f" (alias of {key})" if key != arm["key"] else ""
        print(f"{arm['name']}: key {arm['key']}{alias} {arm['preset']}-{arm['length']} argkey {arm['argkey']} threads {arm['threads']} "
              f"sets {arm['sets'] or '-'} envs {arm['envs'] or '-'}; {have}/{len(arm['runs'])} runs cached; {arm.get('branch') or '-'}@{arm.get('commit') or '-'}"
              f" {arm.get('updated', '')}")
    return 0


def key_dirs():
    return sorted(p for p in POOL.iterdir() if p.is_dir() and KEY_RE.match(p.name)) if POOL.is_dir() else []


def last_used(kd):
    times = []
    for f in ("used", "meta.json"):
        try:
            times.append((kd / f).stat().st_mtime)
        except OSError:
            pass
    return max(times) if times else kd.stat().st_mtime


def keep_list():
    keep = {}
    try:
        for line in (POOL / "KEEP").read_text().splitlines():
            line = line.strip()
            if line and not line.startswith("#"):
                k, _, why = line.partition(" ")
                keep[k] = why.strip()
    except OSError:
        pass
    return keep


def key_busy(kd):
    return any(lock_held(p) for p in kd.rglob("*.lock"))


def cmd_status(a):
    free = shutil.disk_usage(POOL).free
    print(f"pool {POOL}: {human(free)} free")
    keep = keep_list()
    al = aliases()
    for kd in key_dirs():
        meta = read_json(kd / "meta.json", {})
        age = (time.time() - last_used(kd)) / 86400
        runs = len(list(kd.glob("*/*/*/meta.json")))
        busy = " BUSY" if key_busy(kd) else ""
        print(f"{kd.name}: {human(tree_bytes(kd))}, {runs} runs, unused {age:.1f} d, {meta.get('branch') or '-'}@{meta.get('commit') or '-'}"
              f"{' ' + meta['note'] if meta.get('note') else ''}{' KEEP' if kd.name in keep else ''}{busy}")
    for new, e in al.items():
        print(f"alias {new} -> {e['of']} ({'all arms' if e.get('all_arms') else str(len(e.get('arms', []))) + ' arm(s)'}, checked {e.get('checked')})")
    return 0


def cmd_keep(a):
    if not KEY_RE.match(a.key):
        raise Fail(f"{a.key!r} is not a source key")
    with flock(POOL / ".keep.lock"):
        lines = [ln for ln in (POOL / "KEEP").read_text().splitlines() if ln.split(" ")[0] != a.key] if (POOL / "KEEP").exists() else []
        if not a.remove:
            lines.append(f"{a.key} {' '.join(a.reason)}".strip())
        atomic_write(POOL / "KEEP", "\n".join(lines) + ("\n" if lines else ""))
    print(f"{'unkept' if a.remove else 'kept'} {a.key}")
    return 0


def cmd_gc(a):
    keep = keep_list()
    al = aliases()
    pinned = set(keep)
    for new, e in al.items():
        if new in keep:
            pinned.add(e["of"])
    verb = "deleted" if a.apply else "would delete"
    cutoff = a.days * 86400
    dead = []
    for kd in key_dirs():
        age = time.time() - last_used(kd)
        size = human(tree_bytes(kd))
        if kd.name in keep:
            print(f"keep {kd.name}: in KEEP ({keep[kd.name] or 'no reason given'}), {size}")
        elif kd.name in pinned:
            print(f"keep {kd.name}: the alias target of a kept key, {size}")
        elif key_busy(kd):
            print(f"keep {kd.name}: a run or build holds a lock now, {size}")
        elif age < cutoff:
            print(f"keep {kd.name}: used {age / 86400:.1f} d ago (under {a.days} d), {size}")
        else:
            print(f"{verb} {kd.name} ({size}, unused {age / 86400:.1f} d)")
            dead.append(kd)
            if a.apply:
                shutil.rmtree(kd)
    # Staging folders of a request that died (no lock holder) and failure markers of deleted keys.
    for kd in key_dirs():
        if kd in dead:
            continue
        for w in kd.glob("*/*/.work-*"):
            lock = w.parent / (w.name[len(".work-"):] + ".lock")
            if time.time() - w.stat().st_mtime > 86400 and not lock_held(lock):
                print(f"{verb} {w.relative_to(POOL)} ({human(tree_bytes(w))}, a request that died)")
                if a.apply:
                    shutil.rmtree(w, ignore_errors=True)
    if a.apply and dead:
        gone = {kd.name for kd in dead}
        with flock(POOL / ".aliases.lock"):
            cur = aliases()
            new = {k: v for k, v in cur.items() if k not in gone and v["of"] not in gone}
            if new != cur:
                atomic_write(POOL / "aliases.json", json.dumps(new, indent=1, sort_keys=True) + "\n")
        for p in sorted((POOL / "arms").glob("*.json")):
            arm = read_json(p, {})
            if arm.get("key") in gone:
                p.unlink()
                print(f"deleted arm {arm['name']} (its key is gone)")
    print(f"{POOL}: {human(shutil.disk_usage(POOL).free)} free" + ("" if a.apply else "   (dry run; --apply deletes)"))
    return 0


def cmd_alias(a):
    new, old = a.new, a.old
    for k in (new, old):
        if not KEY_RE.match(k):
            raise Fail(f"{k!r} is not a source key")
    if new == old:
        raise Fail("the two keys are the same")
    old_target = aliases().get(old, {}).get("of", old)
    sets = list(a.set or [])
    envs = list(a.envs)
    reg = load_registry(a.registry or [])
    ctx_base = {"preset": a.preset, "length": a.length, "threads": a.threads, "sets": sets, "envs": envs,
                "argkey": argkey_of(sets, envs, a.threads), "measures": [], "reg": reg, "keep_ck": False,
                "retry": a.retry, "rerun": False}
    ctx_new = dict(ctx_base, key=new, arm=f"alias-check-{new}", no_alias=True)
    ctx_old = dict(ctx_base, key=old_target, arm=f"alias-check-{new}", no_alias=True)
    ctx_new["bin"] = ensure_binary(new, Path(a.checkout) if a.checkout else None, a.note, a.branch, a.commit)
    if (POOL / old_target / "deeptime.bin").exists():
        ctx_old["bin"] = POOL / old_target / "deeptime.bin"
    else:
        ctx_old["no_sim"] = True
    # The check must compare two real runs: NEW under its own key, never through the alias being checked.
    start, seed = a.start, a.check_seed
    say(f"alias check: {new} against {old_target}: {a.preset} {a.length} Myr, start {start}, seed {seed}, argkey {ctx_base['argkey']}")
    res = {}

    def one(tag, ctx):
        res[tag] = ensure_run(ctx, start, seed)

    th = [threading.Thread(target=one, args=("new", ctx_new)), threading.Thread(target=one, args=("old", ctx_old))]
    for t in th:
        t.start()
        time.sleep(1)
    for t in th:
        t.join()
    for tag in ("new", "old"):
        if res[tag][0] in ("FAILED", "missing"):
            raise Fail(f"the {tag} run ({'key ' + (new if tag == 'new' else old_target)}) is {res[tag][0]}: {res[tag][1]}")
    mn = read_json(Path(res["new"][1]) / "meta.json")
    mo = read_json(Path(res["old"][1]) / "meta.json")
    same_ck = mn["ck_sha256"] == mo["ck_sha256"]
    same_fields = mn["fields_sha256"] == mo["fields_sha256"]
    same_rec = sha256_file(Path(res["new"][1]) / "rec.json") == sha256_file(Path(res["old"][1]) / "rec.json")
    say(f"checkpoint sha256: new {mn['ck_sha256'][:16]} old {mo['ck_sha256'][:16]}: {'IDENTICAL' if same_ck else 'differ'}")
    say(f"dumped fields: {'identical' if same_fields else 'differ'}; dynamics record: {'identical' if same_rec else 'differ'}")
    if not same_ck:
        say(f"NO ALIAS: the two keys' final states differ bit for bit ({new} is a real change from {old}, or the run is not deterministic)")
        return 1
    with flock(POOL / ".aliases.lock"):
        cur = aliases()
        entry = cur.get(new, {"of": old_target, "arms": [], "all_arms": False})
        entry["of"] = old_target
        arm_id = [a.preset, str(a.length), ctx_base["argkey"]]
        if arm_id not in entry["arms"]:
            entry["arms"].append(arm_id)
        entry["all_arms"] = entry.get("all_arms", False) or a.all_arms
        entry["checked"] = f"{start}-{seed} {a.preset}-{a.length} {now()}"
        cur[new] = entry
        atomic_write(POOL / "aliases.json", json.dumps(cur, indent=1, sort_keys=True) + "\n")
    say(f"ALIAS RECORDED: {new} -> {old_target} for {a.preset}-{a.length}/{ctx_base['argkey']}"
        f"{' and every other arm (--all-arms)' if a.all_arms else ' only; other arms run under their own key unless --all-arms'}")
    return 0


# ----------------------------------------------------------------------------- command line

def split_env_tokens(argv):
    """+NAME=value tokens are environment variables for the runs (as in force-balance-ensemble.sh)."""
    envs, rest = [], []
    for t in argv:
        if re.match(r"^\+[A-Za-z_][A-Za-z0-9_]*=", t):
            envs.append(t[1:])
        else:
            rest.append(t)
    return envs, rest


def main():
    envs, argv = split_env_tokens(sys.argv[1:])
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="verb", required=True)

    def common(p, runs=True):
        p.add_argument("--registry", action="append", help="an extra measures registry (adds or replaces measures by name)")
        p.add_argument("--branch")
        p.add_argument("--commit")
        p.add_argument("--note")
        p.add_argument("--checkout", help="the synced checkout to build from (default: this one)")
        if runs:
            p.add_argument("--preset", default="earthlike_quick_16k")
            p.add_argument("--length", type=int, default=1000, help="Myr")
            p.add_argument("--threads", type=int, default=2)
            p.add_argument("--set", action="append", help="a --set k=v for the CLI (not seed)")
            p.add_argument("--measures", help="comma-separated measure names (default v1census)")
            p.add_argument("--keep-ck", action="store_true", help="keep the final checkpoint (about 0.8 GB)")
            p.add_argument("--retry", action="store_true", help="run again a run that left a FAILED marker")

    r = sub.add_parser("run")
    common(r)
    r.add_argument("--key", required=True)
    r.add_argument("--arm", required=True)
    r.add_argument("--starts", default="supercontinent,scattered_continents")
    r.add_argument("--seeds", default="1,2,3")
    r.add_argument("--rerun", action="store_true", help="delete the cached runs asked for and run them again")
    r.set_defaults(fn=cmd_run)

    b = sub.add_parser("build")
    common(b, runs=False)
    b.add_argument("--key", required=True)
    b.set_defaults(fn=cmd_build)

    m = sub.add_parser("measure")
    m.add_argument("--registry", action="append")
    m.add_argument("--arm", action="append")
    m.add_argument("--measures")
    m.set_defaults(fn=cmd_measure)

    t = sub.add_parser("table")
    t.add_argument("--registry", action="append")
    t.add_argument("--base", required=True)
    t.add_argument("--arm", action="append", required=True)
    t.add_argument("--measure", default="v1census")
    t.add_argument("--json")
    t.set_defaults(fn=cmd_table)

    al = sub.add_parser("alias")
    common(al)
    al.add_argument("new")
    al.add_argument("old")
    al.add_argument("--check-seed", type=int, required=True)
    al.add_argument("--start", default="supercontinent")
    al.add_argument("--all-arms", action="store_true", help="the alias covers every arm, not only the one checked")
    al.set_defaults(fn=cmd_alias, length=100)

    sub.add_parser("arms").set_defaults(fn=cmd_arms)
    sub.add_parser("status").set_defaults(fn=cmd_status)
    g = sub.add_parser("gc")
    g.add_argument("--days", type=float, default=3.0)
    g.add_argument("--apply", action="store_true")
    g.set_defaults(fn=cmd_gc)
    k = sub.add_parser("keep")
    k.add_argument("key")
    k.add_argument("reason", nargs="*")
    k.add_argument("--remove", action="store_true")
    k.set_defaults(fn=cmd_keep)

    a = ap.parse_args(argv)
    a.envs = envs
    if getattr(a, "checkout", None) is None and a.verb in ("run", "build", "alias"):
        a.checkout = str(HERE.parents[2])
    signal.signal(signal.SIGTERM, _kill_children)
    signal.signal(signal.SIGINT, _kill_children)
    try:
        POOL.mkdir(parents=True, exist_ok=True)
        sys.exit(a.fn(a) or 0)
    except (Fail, srckey.KeyError_) as e:
        say(f"pool: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
