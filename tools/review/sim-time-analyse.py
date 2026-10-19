#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Turns the sim-time runs (tools/review/sim-time-run.py) into the one JSON file the sim-time page
draws (review/pages/sim-time/, 2026-10-18). Standard library only; run it on the test VM, where
`perf` and the runs are.

    tools/review/sim-time-analyse.py --full <run dir> [--plain <run dir>] [--scale N=<run dir>]...
        [--build <label>=<run dir>]... [--cmp <label>=<file>]... [--perf-script <file>] --out <json>

`--full` is the profiled full run: trace.tsv (the span log of a trace build), threads.tsv,
summary.json, log.txt and perf.data (`perf record -k CLOCK_MONOTONIC -g`, frame pointers). The
spans form two lanes: the main lane, which is the session's own sequence (the critical path), and
the beside lane, the structure step that runs next to the next plate step inside `rayon::join`.
Every perf sample is classed as work, rayon wait (a worker spinning or stealing: no frame of ours
between the sample and rayon's wait loop) or kernel, and charged to the deepest span active at its
time (a sample with a `structure::` frame goes to the beside lane while a beside span runs). The
icicle re-roots each sample's stack under its span path with rayon's plumbing taken out.

`--scale` runs (the same preset, shorter, at 1, 2, 4, 8 and 16 threads) give every span node a
fit T(n) = S + P / n over n = 1, 2, 4, 8 (16 is shown against the fit's prediction). `--build`
runs are bench runs of other builds of the same commit (their stage times, from log.txt).
"""
import argparse
import bisect
import json
import math
import os
import re
import subprocess
import sys
from collections import Counter, defaultdict

PERF_HZ_DEFAULT = 199
BIN_S = 2.0
EPS_NS = 2_000  # a child may end this much after its parent (two clock reads)

# --------------------------------------------------------------------------------------------
# spans


def read_spans(path):
    out = []
    for line in open(path):
        f = line.rstrip("\n").split("\t")
        if len(f) < 7:
            continue
        out.append({"cat": f[0], "what": f[1], "a": int(f[2]), "b": int(f[3]), "cpu": int(f[4]),
                    "tid": int(f[5]), "loc": f[6]})
    return out


def name_bake_passes(spans):
    """The bake's passes all log as `bake:pass`; their call sites name them, in file order."""
    names = ["terrain", "strata", "climate", "water", "surface", "meta", "total"]
    lines = sorted({int(s["loc"].rsplit(":", 1)[1]) for s in spans if s["cat"] == "bake"})
    by_line = {ln: names[i] if i < len(names) else f"pass{i}" for i, ln in enumerate(lines)}
    keep = []
    for s in spans:
        if s["cat"] == "bake":
            s["what"] = by_line[int(s["loc"].rsplit(":", 1)[1])]
            if s["what"] == "total":
                s["cat"] = "bake_total"
        keep.append(s)
    return keep


class Tree:
    """The span tree of one trace: nodes keyed by path (a tuple of `cat:what`), each span placed
    under the innermost span of its lane that contains it."""

    def __init__(self, spans, extra_main=()):
        spans = name_bake_passes(spans)
        spans = [s for s in spans if s["cat"] != "bake_total" and s["b"] >= s["a"]]
        spans += list(extra_main)
        beside = sorted((s for s in spans if s["cat"] == "beside"), key=lambda s: s["a"])
        bstarts = [s["a"] for s in beside]

        def in_beside(s):
            i = bisect.bisect_right(bstarts, s["a"]) - 1
            return i >= 0 and s["b"] <= beside[i]["b"] + EPS_NS

        for s in spans:
            s["lane"] = "beside" if s["cat"] == "beside" or (s["cat"] == "structure" and in_beside(s)) else "main"
        self.nodes = {}
        self.spans = spans
        joins = sorted((s for s in spans if s["cat"] == "session" and s["what"] == "join"), key=lambda s: s["a"])
        jstarts = [s["a"] for s in joins]
        for lane in ("main", "beside"):
            stack = []
            for s in sorted((x for x in spans if x["lane"] == lane), key=lambda x: (x["a"], -x["b"])):
                while stack and stack[-1]["b"] <= s["a"]:
                    stack.pop()
                while stack and s["b"] > stack[-1]["b"] + EPS_NS:
                    stack.pop()  # a partial overlap: not a child of this one
                parent = stack[-1] if stack else None
                if parent is None and lane == "beside":
                    i = bisect.bisect_right(jstarts, s["a"]) - 1
                    parent = joins[i] if i >= 0 and s["b"] <= joins[i]["b"] + 50_000_000 else None
                s["parent"] = parent
                s["path"] = (parent["path"] if parent else ()) + (f"{s['cat']}:{s['what']}",)
                s["kids"] = []
                if parent is not None:
                    parent["kids"].append(s)
                stack.append(s)
        for s in spans:
            n = self.nodes.setdefault(s["path"], {"path": s["path"], "lane": s["lane"], "count": 0,
                                                  "wall_ns": 0, "self_ns": 0, "cpu_ns": 0})
            n["count"] += 1
            d = s["b"] - s["a"]
            n["wall_ns"] += d
            kids = sum(k["b"] - k["a"] for k in s["kids"] if k["lane"] == s["lane"])
            n["self_ns"] += max(0, d - kids)

    def segments(self, lane):
        """Disjoint (start, end, path) pieces: at each moment the deepest span of the lane."""
        out = []

        def emit(s):
            t = s["a"]
            for k in sorted((k for k in s["kids"] if k["lane"] == lane), key=lambda k: k["a"]):
                if k["a"] > t:
                    out.append((t, k["a"], s["path"]))
                emit(k)
                t = max(t, k["b"])
            if s["b"] > t:
                out.append((t, s["b"], s["path"]))

        roots = [s for s in self.spans if s["lane"] == lane and (s["parent"] is None or s["parent"]["lane"] != lane)]
        for r in sorted(roots, key=lambda s: s["a"]):
            emit(r)
        out.sort()
        return out


def cpu_curve(spans):
    pts = sorted((s["b"], s["cpu"]) for s in spans)
    xs, ys = [], []
    for x, y in pts:
        if ys and y < ys[-1]:
            y = ys[-1]
        xs.append(x)
        ys.append(y)
    return xs, ys


def interp(xs, ys, x):
    i = bisect.bisect_left(xs, x)
    if i <= 0:
        return ys[0] if ys else 0
    if i >= len(xs):
        return ys[-1]
    x0, x1, y0, y1 = xs[i - 1], xs[i], ys[i - 1], ys[i]
    return y0 + (y1 - y0) * (x - x0) / max(1, x1 - x0)


def tree_for(run_dir):
    spans = read_spans(os.path.join(run_dir, "trace.tsv"))
    t = Tree(spans)
    xs, ys = cpu_curve(spans)
    for s in t.spans:
        cpu = max(0.0, s["cpu"] - interp(xs, ys, s["a"])) if s.get("cpu") else 0.0
        t.nodes[s["path"]]["cpu_ns"] += cpu
    return t


# --------------------------------------------------------------------------------------------
# symbols


def match_angle(s, i):
    """Index of the '>' that closes the '<' at s[i] (skipping '->')."""
    depth = 0
    j = i
    while j < len(s):
        c = s[j]
        if c == "<":
            depth += 1
        elif c == ">" and not (j > 0 and s[j - 1] == "-"):
            depth -= 1
            if depth == 0:
                return j
        j += 1
    return len(s) - 1


def split_as(inner):
    depth = 0
    for j in range(len(inner) - 3):
        c = inner[j]
        if c == "<":
            depth += 1
        elif c == ">" and inner[j - 1:j] != "-":
            depth -= 1
        elif depth == 0 and inner.startswith(" as ", j):
            return inner[:j], inner[j + 4:]
    return inner, None


def drop_generics(s):
    out, depth, j = [], 0, 0
    while j < len(s):
        c = s[j]
        if c == "<":
            depth += 1
        elif c == ">" and not (j > 0 and s[j - 1] == "-"):
            depth = max(0, depth - 1)
        elif depth == 0:
            out.append(c)
        j += 1
    return "".join(out)


CLOSURE = re.compile(r"\{closure#\d+\}")
HASH = re.compile(r"::h[0-9a-f]{16}$")


def full_name(sym):
    s = HASH.sub("", sym.strip())
    if s.startswith("<"):
        j = match_angle(s, 0)
        inner, rest = s[1:j], s[j + 1:]
        typ, trait = split_as(inner)
        typ = typ.lstrip("&").removeprefix("mut ").removeprefix("dyn ")
        base = full_name(typ)
        if trait is not None and trait.startswith("core::ops::function::"):
            rest = ""
        s = base + drop_generics(rest)
    else:
        s = drop_generics(s)
    return CLOSURE.sub("{closure}", s)


IMPL = re.compile(r"::\{impl#\d+\}")


def short_name(full):
    parts = IMPL.sub("", full).split("::")
    if parts and parts[0] in ("deeptime_core", "deeptime_cli", "deeptime_format"):
        parts = parts[1:]
    keep = 4 if parts and parts[-1] == "{closure}" else 3
    return "::".join(parts[-keep:])


INLINED = re.compile(r"(deeptime_\w+(?:::[A-Za-z0-9_{}#]+)*?)::\{closure_env#\d+\}")


PLUMBING = ("rayon::", "rayon_core::", "core::ops::function::", "std::sys::", "std::thread::", "std::panicking",
            "std::panic::", "core::panic", "start_thread", "__GI___clone3", "clone3", "core::iter::",
            "std::rt::", "core::result::", "core::option::", "alloc::boxed::", "deeptime_core::session::",
            "deeptime_cli::", "main", "__libc_start", "_start", "crossbeam_", "[unknown]",
            "deeptime_core::stage::Stage::step")
WAIT = ("wait_until_cold", "wait_until_out_of_work", "rayon_core::sleep", "find_work", "WorkerThread::steal",
        "WorkerThread::take_local", "yield_now", "no_work_found", "LockLatch::wait", "crossbeam_deque",
        "rayon_core::latch::", "sched_yield", "WorkerThread::wait_until")
CATS_USER = (
    ("compression (lz4/zstd)", ("lz4_flex", "lz4::", "zstd", "miniz", "flate2", "LZ4", "lz4")),
    ("allocator", ("deeptime_core::alloc", "__rust_alloc", "__rust_dealloc", "__rust_realloc", "__rdl_",
                   "malloc", "_int_free", "_int_malloc", "cfree", "realloc", "calloc", "posix_memalign",
                   "__libc_free", "__libc_malloc", "mmap64", "munmap", "__mmap", "madvise")),
    ("memcpy / memset", ("memcpy", "memmove", "memset", "__memcpy", "__memmove", "__memset")),
    ("hashing (crc32/sha)", ("crc32fast", "sha2::", "Sha256", "crc32")),
    ("serde_json", ("serde_json", "serde::")),
    ("sorting", ("core::slice::sort", "slice::sort")),
)
CATS_KERNEL = (
    ("page faults", ("page_fault", "clear_page", "do_anonymous_page", "handle_mm_fault", "do_huge_pmd",
                     "folio_zero", "__do_fault", "filemap_map_pages")),
    ("munmap / madvise", ("munmap", "zap_pte", "unmap_", "madvise", "free_pages", "release_pages",
                          "tlb_finish")),
    ("file I/O", ("ksys_write", "vfs_write", "vfs_read", "ksys_read", "fsync", "ext4", "xfs", "iomap",
                  "generic_perform_write", "copy_page_from_iter", "filemap_read", "do_sys_open",
                  "do_sys_openat", "__x64_sys_write", "__x64_sys_read", "__x64_sys_openat", "__x64_sys_close",
                  "__x64_sys_rename", "__x64_sys_unlink")),
    ("mmap", ("vm_mmap", "do_mmap", "__x64_sys_mmap", "mmap_region")),
    ("futex / scheduling", ("futex", "schedule", "sched_", "__x64_sys_sched_yield", "do_sched_yield")),
)


class Symbols:
    def __init__(self):
        self.cache = {}
        self.names = []
        self.ids = {}

    def get(self, sym):
        """(name id, plumbing, wait, ours, user category, kernel category, full name, raw names a
        structure:: or plates:: closure)."""
        v = self.cache.get(sym)
        if v is None:
            full = full_name(sym)
            plumbing = full.startswith(PLUMBING) or full == ""
            wait = any(w in full for w in WAIT)
            ours = full.startswith("deeptime_")
            if plumbing and full.startswith("rayon::") and not wait:
                m = INLINED.search(sym)
                if m:  # a closure of ours inlined into rayon's iterator plumbing
                    full = CLOSURE.sub("{closure}", m.group(1)) + "::{closure}"
                    plumbing, ours = False, True
            shrt = short_name(full)
            ucat = next((c for c, pats in CATS_USER if any(p in full for p in pats)), None)
            kcat = next((c for c, pats in CATS_KERNEL if any(p in full for p in pats)), None)
            owner = "structure" if "deeptime_core::structure" in sym else (
                "plates" if "deeptime_core::plates" in sym else None)
            sid = self.ids.get(shrt)
            if sid is None:
                sid = self.ids[shrt] = len(self.names)
                self.names.append(shrt)
            v = self.cache[sym] = (sid, plumbing, wait, ours, ucat, kcat, full, owner, "::" in full)
        return v


def kernel_addr(a):
    return len(a) >= 16 and a[0] in "f"


# --------------------------------------------------------------------------------------------
# perf


def perf_samples(perf_data, script=None):
    """Yields (comm, tid, t_ns, frames) with frames leaf first as (is_kernel, symbol)."""
    if script:
        src = open(script, errors="replace")
        proc = None
    else:
        proc = subprocess.Popen(["perf", "script", "--no-inline", "-F", "comm,tid,time,ip,sym", "-i", perf_data],
                                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, errors="replace",
                                bufsize=1 << 20)
        src = proc.stdout
    head, frames = None, []
    for line in src:
        if line.startswith("\t"):
            s = line.strip()
            sp = s.find(" ")
            if sp < 0:
                frames.append((kernel_addr(s), "[unknown]"))
            else:
                frames.append((kernel_addr(s[:sp]), s[sp + 1:]))
        elif line.strip() == "":
            if head is not None:
                yield head + (frames,)
            head, frames = None, []
        else:
            f = line.split()
            try:
                t = round(float(f[-1].rstrip(":")) * 1e9)
                tid = int(f[-2])
            except (ValueError, IndexError):
                head = None
                continue
            head = (" ".join(f[:-2]), tid, t)
            frames = []
    if head is not None:
        yield head + (frames,)
    if proc:
        proc.wait()


# --------------------------------------------------------------------------------------------
# log parsing


def parse_log(path):
    out = {}
    try:
        txt = open(path, errors="replace").read()
    except OSError:
        return out
    m = re.search(r"threads=\d+ total_ms=([\d.]+) peak_bytes=(\d+) stages=(\S+)", txt)
    if m:
        out["total_s"] = float(m.group(1)) / 1000
        out["peak_bytes"] = int(m.group(2))
        out["stages_s"] = {k: float(v) / 1000 for k, v in (kv.split(":") for kv in m.group(3).split(","))}
    m = re.search(r"threads=\d+ timings (.*)", txt)
    if m:
        groups = {}
        for g, body in re.findall(r"(\w+)\[([^\]]*)\]", m.group(1)):
            d = {}
            for kv in body.split(","):
                if "=" in kv:
                    k, v = kv.split("=", 1)
                    try:
                        d[k] = float(v.rstrip("s"))
                    except ValueError:
                        pass
            groups[g] = d
        out["timings"] = groups
        out["stage_steps"] = {k: int(n) for k, n in re.findall(r"(\w+)=[\d.]+s/(\d+)steps", m.group(1))}
    m = re.search(r"amplify_final=.*?passes\[([^\]]*)\] sample=([\d.]+)s detail=([\d.]+)s sink=([\d.]+)s total=([\d.]+)s", txt)
    if m:
        d = {k: float(v.rstrip("s")) for k, v in (kv.split("=") for kv in m.group(1).split(","))}
        d.update(sample=float(m.group(2)), detail=float(m.group(3)), sink=float(m.group(4)), total=float(m.group(5)))
        cm = re.search(r"amplify_final=.*?character\[[^\]]*,([\d.]+)s\]", txt)
        if cm:
            d["character"] = float(cm.group(1))
        out["amplify"] = d
    m = re.search(r"projection .*?millis\[([^\]]*)\]", txt)
    if m:
        out["projection_s"] = {k: float(v) / 1000 for k, v in (kv.split("=") for kv in m.group(1).split())}
    m = re.search(r"INFO bake layers=.*?millis\[([^\]]*)\]", txt)
    if m:
        out["bake_s"] = {k: float(v.rstrip("s")) for k, v in (kv.split("=") for kv in m.group(1).split())}
    m = re.search(r"drawn again with the bake's water .*? in (\d+) ms", txt)
    if m:
        out["keyframe_redraw_s"] = int(m.group(1)) / 1000
    out["history"] = re.findall(r"history after (\w+): (.*)", txt)[-6:]
    return out


def summary(run_dir):
    try:
        s = json.load(open(os.path.join(run_dir, "summary.json")))
    except OSError:
        s = {}
    s["log"] = parse_log(os.path.join(run_dir, "log.txt"))
    return s


def thread_series(run_dir, t0_ns, t1_ns):
    """Per-thread busy cores per BIN_S bin from threads.tsv (jiffies sampled every 0.5 s)."""
    hz = os.sysconf("SC_CLK_TCK")
    last = {}
    names = {}
    nb = int(math.ceil((t1_ns - t0_ns) / 1e9 / BIN_S)) + 1
    busy = defaultdict(lambda: [0.0] * nb)
    try:
        f = open(os.path.join(run_dir, "threads.tsv"))
    except OSError:
        return {}, {}, nb
    next(f)
    total_j = {}
    for line in f:
        ns, tid, name, j = line.rstrip("\n").split("\t")
        ns, tid, j = int(ns), int(tid), int(j)
        names[tid] = name
        if tid in last:
            pns, pj = last[tid]
            dj = j - pj
            if dj > 0:
                b = int((ns - t0_ns) / 1e9 / BIN_S)
                if 0 <= b < nb:
                    busy[tid][b] += dj / hz / BIN_S
        last[tid] = (ns, j)
    for tid, (ns, j) in last.items():
        total_j[tid] = j
    return {t: v for t, v in busy.items()}, {t: (names[t], total_j[t] / hz) for t in names}, nb


# --------------------------------------------------------------------------------------------
# the full run


def label_of(path):
    return " / ".join(p.split(":", 1)[1] if p.startswith(("stage:", "cli:")) else p for p in path)


def analyse_full(run_dir, perf_script, prune):
    summ = summary(run_dir)
    spans = read_spans(os.path.join(run_dir, "trace.tsv"))
    clis = {s["what"]: s for s in spans if s["cat"] == "cli"}
    t_create = clis["create"]["a"]
    t_end = clis["end"]["b"] if "end" in clis else max(s["b"] for s in spans)
    t_fin = clis["finalize"]["b"]
    # the run before the first span and after the last: start-up, the bench's report, tear-down
    p_start = summ.get("start_mono_ns", t_create)
    wall_end = p_start + int(summ.get("wall_s", 0) * 1e9)
    extra = [
        {"cat": "process", "what": "start-up", "a": p_start, "b": t_create, "cpu": 0, "tid": 0, "loc": ""},
        {"cat": "cli", "what": "report", "a": t_fin, "b": t_end, "cpu": 0, "tid": 0, "loc": ""},
        {"cat": "process", "what": "tear-down", "a": t_end, "b": max(t_end, wall_end), "cpu": 0, "tid": 0, "loc": ""},
    ]
    tree = Tree(spans, extra)
    xs, ys = cpu_curve([s for s in spans if s["cat"] != "process"])
    for s in tree.spans:
        if s["cat"] in ("process",) or (s["cat"] == "cli" and s["what"] == "report"):
            continue
        tree.nodes[s["path"]]["cpu_ns"] += max(0.0, s["cpu"] - interp(xs, ys, s["a"]))
    main = tree.segments("main")
    beside = tree.segments("beside")
    m_starts = [x[0] for x in main]
    b_starts = [x[0] for x in beside]

    def at(segs, starts, t):
        i = bisect.bisect_right(starts, t) - 1
        if i >= 0 and segs[i][0] <= t < segs[i][1]:
            return segs[i][2]
        return None

    t0 = p_start
    t1 = max(t_end, wall_end)
    nb = int(math.ceil((t1 - t0) / 1e9 / BIN_S)) + 1
    perf = {"samples": 0, "hz": summ.get("perf_hz") or PERF_HZ_DEFAULT}
    node_cls = defaultdict(Counter)
    node_cat = defaultdict(Counter)
    node_fn = defaultdict(Counter)
    bins = {c: [0] * nb for c in ("work", "spin", "kernel")}
    tbins = defaultdict(lambda: {c: [0] * nb for c in ("work", "spin", "kernel")})
    folded = Counter()
    fn_self, fn_total = Counter(), Counter()
    kernel_leaf, bare_leaf = Counter(), Counter()
    cats = Counter()
    ambiguous = 0
    syms = Symbols()
    tid_names = {}
    pid_tids = set()
    _, tnames, _ = thread_series(run_dir, t0, t1)
    pid_tids = set(tnames)
    spin_id = syms.get("[rayon wait]")[0]
    kcat_ids = {}
    if os.path.exists(os.path.join(run_dir, "perf.data")) or perf_script:
        for comm, tid, t, frames in perf_samples(os.path.join(run_dir, "perf.data"), perf_script):
            if pid_tids and tid not in pid_tids:
                continue
            if not frames:
                continue
            perf["samples"] += 1
            tid_names.setdefault(tid, comm)
            kern = frames[0][0]
            kcat = None
            user = []
            cls = None
            ucat = None
            owners = set()
            seen_ours = False
            for is_k, sym in frames:
                if is_k:
                    if kcat is None:
                        kcat = syms.get(sym)[5]
                    continue
                v = syms.get(sym)
                if cls is None and v[8]:  # the leaf-most Rust frame decides: rayon's wait loop or work
                    cls = "spin" if v[2] else "work"
                if not seen_ours:
                    if ucat is None and v[4]:
                        ucat = v[4]
                    seen_ours = v[3]
                if v[7]:
                    owners.add(v[7])
                user.append(v)
            if cls is None:
                cls = "work"
            if cls == "work" and kern:
                cls = "kernel"
            has_structure = "structure" in owners
            path = None
            if has_structure:
                path = at(beside, b_starts, t)
                if path is None:
                    path = at(main, m_starts, t)
            else:
                path = at(main, m_starts, t)
                if path is not None and any(p.startswith("session:join") for p in path) and \
                        at(beside, b_starts, t) is not None and not any(
                            u[7] == "plates" for u in user):
                    ambiguous += 1
            if path is None:
                path = ("outside spans",)
            node_cls[path][cls] += 1
            b = int((t - t0) / 1e9 / BIN_S)
            if 0 <= b < nb:
                bins[cls][b] += 1
                tbins[tid][cls][b] += 1
            if cls == "spin":
                node_cat[path]["rayon wait"] += 1
                cats["rayon wait"] += 1
                folded[(path, (spin_id,))] += 1
                fn_self["[rayon wait]"] += 1
                continue
            if kern:
                cat = kcat or "other kernel"
                kernel_leaf[f"{cat} | {frames[0][1]}"] += 1
            else:
                cat = ucat or "compute"
            node_cat[path][cat] += 1
            cats[cat] += 1
            stack = [v[0] for v in reversed(user) if not v[1]]
            if kern:
                kid = kcat_ids.get(cat)
                if kid is None:
                    kid = kcat_ids[cat] = syms.get(f"[kernel: {cat}]")[0]
                stack.append(kid)
            if not stack:
                stack = [syms.get("[no frame of ours]")[0]]
                bare_leaf[" < ".join(v[6] for v in user[:4])[:300]] += 1
            # collapse direct recursion
            dedup = [stack[0]]
            for x in stack[1:]:
                if x != dedup[-1]:
                    dedup.append(x)
            stack = tuple(dedup[:48])
            folded[(path, stack)] += 1
            leaf = syms.names[stack[-1]]
            fn_self[leaf] += 1
            node_fn[path][leaf] += 1
            for x in set(stack):
                fn_total[syms.names[x]] += 1
    hz = perf["hz"]
    perf["ambiguous_join_samples"] = ambiguous
    # the icicle: span path then frames
    total = sum(folded.values())
    root = {"n": "run", "v": 0, "c": {}}
    for (path, stack), v in folded.items():
        node = root
        node["v"] += v
        for p in path:
            node = node["c"].setdefault("S" + p, {"n": p, "v": 0, "c": {}, "s": 1})
            node["v"] += v
        for x in stack:
            nm = syms.names[x]
            node = node["c"].setdefault(nm, {"n": nm, "v": 0, "c": {}})
            node["v"] += v
    cut = max(1, int(total * prune))

    def shrink(n):
        kids = [shrink(k) for k in n["c"].values() if k["v"] >= cut]
        kids.sort(key=lambda k: -k["v"])
        out = {"n": n["n"], "v": n["v"]}
        if n.get("s"):
            out["s"] = 1
        if kids:
            out["c"] = kids
        return out

    icicle = shrink(root)
    # nodes
    nodes = []
    for path, n in tree.nodes.items():
        c = node_cls.get(path, Counter())
        nodes.append({
            "path": list(path), "label": label_of(path), "lane": n["lane"], "count": n["count"],
            "wall_s": n["wall_ns"] / 1e9, "self_s": n["self_ns"] / 1e9, "cpu_trace_s": n["cpu_ns"] / 1e9,
            "self_work_s": c["work"] / hz, "self_spin_s": c["spin"] / hz, "self_kernel_s": c["kernel"] / hz,
            "cats": {k: v / hz for k, v in node_cat.get(path, Counter()).most_common(8)},
            "fns": [[k, v / hz] for k, v in node_fn.get(path, Counter()).most_common(10)],
        })
    if ("outside spans",) in node_cls:
        c = node_cls[("outside spans",)]
        nodes.append({"path": ["outside spans"], "label": "outside spans", "lane": "main", "count": 0,
                      "wall_s": 0, "self_s": 0, "cpu_trace_s": 0, "self_work_s": c["work"] / hz,
                      "self_spin_s": c["spin"] / hz, "self_kernel_s": c["kernel"] / hz,
                      "cats": {k: v / hz for k, v in node_cat[("outside spans",)].most_common(8)},
                      "fns": [[k, v / hz] for k, v in node_fn[("outside spans",)].most_common(10)]})
    # inclusive perf per node (self plus every descendant path)
    incl = defaultdict(Counter)
    for path, c in node_cls.items():
        for i in range(1, len(path) + 1):
            incl[path[:i]].update(c)
    for n in nodes:
        c = incl.get(tuple(n["path"]), Counter())
        n["work_s"] = c["work"] / hz
        n["spin_s"] = c["spin"] / hz
        n["kernel_s"] = c["kernel"] / hz
    # phases (from the finish spans)
    drive = clis["drive"]
    phases = [{"label": "create", "a": (t_create - t0) / 1e9, "b": (clis["create"]["b"] - t0) / 1e9}]
    prev = drive["a"]
    for s in sorted((s for s in spans if s["cat"] == "finish"), key=lambda s: s["a"]):
        lab = "coupled group" if "+" in s["what"] else s["what"]
        phases.append({"label": lab, "a": (prev - t0) / 1e9, "b": (s["b"] - t0) / 1e9})
        prev = s["b"]
    phases.append({"label": "finalize", "a": (clis["finalize"]["a"] - t0) / 1e9, "b": (t_fin - t0) / 1e9})
    # per plate step rows of the coupled group
    top = sorted((s for s in tree.spans if s["parent"] is not None and s["parent"]["path"] == drive_path(tree)
                  and s["lane"] == "main"), key=lambda s: s["a"])
    group_end = next((s["b"] for s in spans if s["cat"] == "finish" and "+" in s["what"]), drive["b"])
    cols = ["plates", "climate", "surface", "volcanoes", "impacts", "structure", "wait for structure beside",
            "keyframes and events", "session other", "structure beside"]
    rows = []
    cur = None
    for s in top:
        if s["a"] > group_end:
            break
        key = s["path"][-1]
        is_plates = key == "stage:plates" or (key == "session:join" and any(k["path"][-1] == "stage:plates" for k in s["kids"]))
        if is_plates:
            cur = {c: 0.0 for c in cols}
            cur["t"] = (s["a"] - t0) / 1e9
            rows.append(cur)
        if cur is None:
            continue
        d = (s["b"] - s["a"]) / 1e9
        if key == "session:join":
            lead = sum((k["b"] - k["a"]) for k in s["kids"] if k["lane"] == "main") / 1e9
            bes = sum((k["b"] - k["a"]) for k in s["kids"] if k["lane"] == "beside") / 1e9
            cur["plates"] += lead
            cur["wait for structure beside"] += max(0.0, d - lead)
            cur["structure beside"] += bes
        elif key.startswith("stage:"):
            nm = key.split(":", 1)[1]
            cur[nm if nm in cur else "session other"] += d
        elif key == "session:after_step":
            cur["keyframes and events"] += d
        else:
            cur["session other"] += d
    steps = {"cols": ["t"] + cols, "rows": [[round(r["t"], 2)] + [round(r[c], 4) for c in cols] for r in rows]}
    # timeline
    busy_t, tinfo, _ = thread_series(run_dir, t0, t1)
    busy = [0.0] * nb
    for v in busy_t.values():
        for i, x in enumerate(v):
            busy[i] += x
    order = sorted(busy_t, key=lambda t: (not tinfo[t][0].startswith("deeptime-rayon"), tinfo[t][0], t))
    threads = []
    for tid in order:
        nm = tinfo[tid][0]
        if tinfo[tid][1] < 1.0:
            continue
        tb = tbins.get(tid)
        threads.append({"tid": tid, "name": nm, "cpu_s": tinfo[tid][1],
                        "busy": [round(x, 3) for x in busy_t[tid]],
                        "work": [round(x / hz / BIN_S, 3) for x in tb["work"]] if tb else [],
                        "spin": [round(x / hz / BIN_S, 3) for x in tb["spin"]] if tb else [],
                        "kernel": [round(x / hz / BIN_S, 3) for x in tb["kernel"]] if tb else []})
    timeline = {"bin_s": BIN_S, "busy": [round(x, 3) for x in busy],
                "work": [round(x / hz / BIN_S, 3) for x in bins["work"]],
                "spin": [round(x / hz / BIN_S, 3) for x in bins["spin"]],
                "kernel": [round(x / hz / BIN_S, 3) for x in bins["kernel"]], "threads": threads}
    cpu_trace_end = interp(xs, ys, t_end) / 1e9
    reconcile = {
        "wall_runner_s": summ.get("wall_s"), "total_bench_s": summ.get("total_s"),
        "user_s": summ.get("user_s"), "system_s": summ.get("system_s"),
        "threads_tsv_cpu_s": sum(v[1] for v in tinfo.values()),
        "trace_cpu_at_end_s": cpu_trace_end,
        "perf_cpu_s": perf["samples"] / hz,
        "perf_work_s": sum(bins["work"]) / hz, "perf_spin_s": sum(bins["spin"]) / hz,
        "perf_kernel_s": sum(bins["kernel"]) / hz,
        "trace_span_wall": {"create": (clis["create"]["b"] - clis["create"]["a"]) / 1e9,
                            "drive": (drive["b"] - drive["a"]) / 1e9,
                            "finalize": (t_fin - clis["finalize"]["a"]) / 1e9,
                            "report": (t_end - t_fin) / 1e9,
                            "start-up": (t_create - p_start) / 1e9,
                            "tear-down": max(0, wall_end - t_end) / 1e9},
    }
    fns = {"self": [[k, v / hz] for k, v in fn_self.most_common(80)],
           "total": [[k, v / hz] for k, v in fn_total.most_common(120)]}
    return {"summary": summ, "nodes": nodes, "phases": phases, "steps": steps, "timeline": timeline,
            "icicle": icicle, "icicle_hz": hz, "functions": fns,
            "categories": {k: v / hz for k, v in cats.most_common()}, "perf": perf, "reconcile": reconcile,
            "kernel_leaf": [[k, v / hz] for k, v in kernel_leaf.most_common(40)],
            "bare_leaf": [[k, v / hz] for k, v in bare_leaf.most_common(25)],
            "thread_names": {str(k): v for k, v in tid_names.items()}}


def drive_path(tree):
    for p in tree.nodes:
        if p == ("cli:drive",):
            return p
    return ("cli:drive",)


# --------------------------------------------------------------------------------------------
# scaling


def fit(points):
    """Least squares T = S + P x over (x = 1/n, T), clipped to S, P >= 0."""
    n = len(points)
    if n < 2:
        return None
    sx = sum(x for x, _ in points)
    sy = sum(y for _, y in points)
    sxx = sum(x * x for x, _ in points)
    sxy = sum(x * y for x, y in points)
    den = n * sxx - sx * sx
    if den == 0:
        return None
    p = (n * sxy - sx * sy) / den
    s = (sy - p * sx) / n
    if s < 0:
        s, p = 0.0, sxy / sxx
    if p < 0:
        s, p = sy / n, 0.0
    return s, p


def analyse_scaling(scale_dirs, full_nodes):
    runs = {}
    trees = {}
    for n, d in sorted(scale_dirs.items()):
        summ = summary(d)
        runs[n] = {"wall_s": summ.get("wall_s"), "total_s": summ.get("total_s"),
                   "stages_s": summ.get("log", {}).get("stages_s"), "user_s": summ.get("user_s"),
                   "system_s": summ.get("system_s"), "others_mean": summ.get("others_mean"),
                   "cpus": summ.get("cpus")}
        if os.path.exists(os.path.join(d, "trace.tsv")):
            trees[n] = tree_for(d)
    paths = set()
    for t in trees.values():
        paths |= set(t.nodes)
    nodes = []
    for p in paths:
        walls = {n: t.nodes[p]["wall_ns"] / 1e9 for n, t in trees.items() if p in t.nodes}
        selfs = {n: t.nodes[p]["self_ns"] / 1e9 for n, t in trees.items() if p in t.nodes}
        cpus = {n: t.nodes[p]["cpu_ns"] / 1e9 for n, t in trees.items() if p in t.nodes}
        pts = [(1.0 / n, w) for n, w in walls.items() if n <= 8]
        f = fit(pts)
        rec = {"path": list(p), "label": label_of(p), "walls": walls, "selfs": selfs, "cpus": cpus}
        if f:
            s, pp = f
            rec["S"] = s
            rec["P"] = pp
            rec["serial_share"] = s / (s + pp) if s + pp > 0 else None
            rec["pred16"] = s + pp / 16
        nodes.append(rec)
    nodes.sort(key=lambda r: -max(r["walls"].values() or [0]))
    totals = [(1.0 / n, r["total_s"]) for n, r in runs.items() if r.get("total_s") and n <= 8]
    tf = fit(totals)
    return {"runs": runs, "nodes": nodes, "total_fit": {"S": tf[0], "P": tf[1]} if tf else None}


# --------------------------------------------------------------------------------------------


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--full", required=True)
    ap.add_argument("--plain")
    ap.add_argument("--scale", action="append", default=[])
    ap.add_argument("--build", action="append", default=[])
    ap.add_argument("--cmp", action="append", default=[])
    ap.add_argument("--perf-script")
    ap.add_argument("--prune", type=float, default=0.0003)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    out = {"generated_utc": subprocess.run(["date", "-u", "+%Y-%m-%dT%H:%M:%SZ"], capture_output=True,
                                           text=True).stdout.strip()}
    out["full"] = analyse_full(a.full, a.perf_script, a.prune)
    if a.plain:
        p = summary(a.plain)
        out["plain"] = p
    if a.scale:
        out["scaling"] = analyse_scaling({int(k): v for k, v in (x.split("=", 1) for x in a.scale)},
                                         out["full"]["nodes"])
    builds = {}
    for x in a.build:
        k, d = x.split("=", 1)
        builds[k] = summary(d)
    if builds:
        out["builds"] = builds
    cmps = {}
    for x in a.cmp:
        k, f = x.split("=", 1)
        try:
            txt = open(f).read()
        except OSError:
            txt = "missing"
        cmps[k] = txt[-2000:]
    out["cmp"] = cmps
    json.dump(out, open(a.out, "w"), separators=(",", ":"))
    print(f"wrote {a.out}: {os.path.getsize(a.out) / 1e6:.2f} MB; perf samples {out['full']['perf']['samples']}")


if __name__ == "__main__":
    main()
