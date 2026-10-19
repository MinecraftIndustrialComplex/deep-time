#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""One `deeptime bench` run held to a CPU set, with every thread's busy time sampled over the run,
optionally under `perf record` and with the span log of a trace build (the sim-time page,
2026-10-18). Standard library only; run on the test VM, admitted.

    tools/review/sim-time-run.py --out <dir> --cpus LIST [--perf HZ] [--trace] [--lull]
                                 <bin> <preset> <start> <myr> <threads>

- `--cpus`: the run's CPU affinity (taskset, innermost, so perf itself is not held to the set).
- `--lull`: first wait until everyone else's busy CPUs (from /proc/stat) have stayed under 1.5 on
  average for `--lull-window` seconds (120), at most `--lull-max` seconds (1,800) (bench-quiet.py's
  rule, tightened: no other agents run).
- `--perf HZ`: `sudo perf record -k CLOCK_MONOTONIC -F HZ -g` around the run (frame pointers; the
  binary must be built with `-C force-frame-pointers=yes`), into <dir>/perf.data.
- `--trace`: sets DEEPTIME_TRACE=<dir>/trace.tsv (a build with the sim-time span patch writes one
  line per timer span; others ignore it).

Writes <dir>/log.txt (bench output and `/usr/bin/time -v`), <dir>/threads.tsv (every 0.5 s: each
thread's utime+stime jiffies, by tid and name, CLOCK_MONOTONIC ns), <dir>/load.tsv (every 2 s: the
machine's busy CPUs, the run's, everyone else's, the run's CPU set's and its SMT siblings' busy
shares) and <dir>/summary.json.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time

HZ = os.sysconf("SC_CLK_TCK")


def stat():
    out = {}
    for line in open("/proc/stat"):
        if not line.startswith("cpu"):
            break
        f = line.split()
        v = [int(x) for x in f[1:]]
        idle = v[3] + (v[4] if len(v) > 4 else 0)
        out[f[0]] = (sum(v[:8]) - idle, sum(v[:8]))
    return out


def siblings(cpu):
    txt = open(f"/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list").read().strip()
    out = []
    for part in txt.split(","):
        a, _, b = part.partition("-")
        out += list(range(int(a), int(b or a) + 1))
    return [c for c in out if c != cpu]


def children():
    kids = {}
    for p in os.listdir("/proc"):
        if not p.isdigit():
            continue
        try:
            s = open(f"/proc/{p}/stat").read()
        except OSError:
            continue
        rest = s[s.rfind(")") + 2:].split()
        kids.setdefault(int(rest[1]), []).append(int(p))
    return kids


def tree(root):
    kids, out, todo = children(), [], [root]
    while todo:
        p = todo.pop()
        out.append(p)
        todo += kids.get(p, [])
    return out


def comm(pid):
    try:
        return open(f"/proc/{pid}/comm").read().strip()
    except OSError:
        return ""


def find_bin(root, name):
    for p in tree(root):
        if comm(p) == name[:15]:
            return p
    return None


def threads(pid):
    """{tid: (name, utime+stime jiffies)} of every thread of `pid`."""
    out = {}
    try:
        tids = os.listdir(f"/proc/{pid}/task")
    except OSError:
        return out
    for t in tids:
        try:
            s = open(f"/proc/{pid}/task/{t}/stat").read()
        except OSError:
            continue
        name = s[s.find("(") + 1:s.rfind(")")]
        rest = s[s.rfind(")") + 2:].split()
        out[int(t)] = (name, int(rest[11]) + int(rest[12]))
    return out


def tree_jiffies(root):
    total = 0
    for p in tree(root):
        try:
            s = open(f"/proc/{p}/stat").read()
        except OSError:
            continue
        rest = s[s.rfind(")") + 2:].split()
        total += int(rest[11]) + int(rest[12])
    return total


def mono_ns():
    return time.clock_gettime_ns(time.CLOCK_MONOTONIC)


def wait_lull(log, mean_below=1.5, window=120.0, most=1800.0):
    t0, prev, hist = time.time(), stat(), []
    while time.time() - t0 < most:
        time.sleep(5)
        now = stat()
        busy = (now["cpu"][0] - prev["cpu"][0]) / HZ / 5.0
        prev = now
        hist.append(busy)
        n = int(window / 5)
        if len(hist) >= n and sum(hist[-n:]) / n < mean_below:
            log.write(f"# lull after {time.time() - t0:.0f} s (others' busy CPUs {sum(hist[-n:]) / n:.2f})\n")
            return
    log.write(f"# no lull in {most:.0f} s; running anyway\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--cpus", required=True)
    ap.add_argument("--perf", type=int, default=0)
    ap.add_argument("--trace", action="store_true")
    ap.add_argument("--lull", action="store_true")
    ap.add_argument("--lull-window", type=float, default=120.0)
    ap.add_argument("--lull-max", type=float, default=1800.0)
    ap.add_argument("--set", action="append", default=[], help="a preset override passed on as --set")
    ap.add_argument("bin")
    ap.add_argument("preset")
    ap.add_argument("start")
    ap.add_argument("myr")
    ap.add_argument("threads")
    a = ap.parse_args()
    os.makedirs(a.out, exist_ok=True)
    cpus = [int(c) for c in a.cpus.split(",")]
    sibs = sorted({s for c in cpus for s in siblings(c)} - set(cpus))
    log = open(os.path.join(a.out, "log.txt"), "w")
    if a.lull:
        wait_lull(log, window=a.lull_window, most=a.lull_max)
        log.flush()
    env = [f"RAYON_NUM_THREADS={a.threads}"]
    if a.trace:
        env.append(f"DEEPTIME_TRACE={os.path.join(a.out, 'trace.tsv')}")
    bench = [a.bin, "bench", "--preset", a.preset, "--start", a.start, "--length-myr", a.myr,
             "--threads", a.threads, "--out", os.path.join(a.out, "bench.json"),
             "--work", os.path.join(a.out, "work")]
    for s in a.set:
        bench += ["--set", s]
    inner = ["env", *env, "taskset", "-c", a.cpus, *bench]
    user = os.environ.get("USER") or subprocess.run(["id", "-un"], capture_output=True, text=True).stdout.strip()
    if a.perf:
        cmd = ["/usr/bin/time", "-v", "sudo", "perf", "record", "-k", "CLOCK_MONOTONIC", "-F", str(a.perf), "-g",
               "-o", os.path.join(a.out, "perf.data"), "--", "sudo", "-u", user, *inner]
    else:
        cmd = ["/usr/bin/time", "-v", *inner]
    log.write(f"# cpus {a.cpus} (siblings {','.join(map(str, sibs))})\n# {' '.join(cmd)}\n")
    log.flush()
    th = open(os.path.join(a.out, "threads.tsv"), "w")
    th.write("mono_ns\ttid\tname\tjiffies\n")
    ld = open(os.path.join(a.out, "load.tsv"), "w")
    ld.write("mono_ns\tload1\tmachine_busy\trun_cpus\tothers\tset_busy\tsibling_busy\n")
    t0, m0 = time.time(), mono_ns()
    proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT)
    name = os.path.basename(a.bin)
    pid = None
    prev, prev_run, prev_t = stat(), 0, time.time()
    last_load = time.time()
    others, runs, sib_busy, set_busy = [], [], [], []
    while proc.poll() is None:
        time.sleep(0.5)
        if pid is None:
            pid = find_bin(proc.pid, name)
        if pid is not None:
            now_ns = mono_ns()
            for tid, (tn, j) in threads(pid).items():
                th.write(f"{now_ns}\t{tid}\t{tn}\t{j}\n")
        if time.time() - last_load >= 2.0:
            last_load = time.time()
            now, now_run, now_t = stat(), tree_jiffies(proc.pid), time.time()
            dt = max(1e-6, now_t - prev_t)
            machine = (now["cpu"][0] - prev["cpu"][0]) / HZ / dt
            run = max(0, now_run - prev_run) / HZ / dt

            def busy(cs):
                b = sum(now[f"cpu{c}"][0] - prev[f"cpu{c}"][0] for c in cs)
                tot = sum(now[f"cpu{c}"][1] - prev[f"cpu{c}"][1] for c in cs)
                return b / tot if tot else 0.0
            oth = max(0.0, machine - run)
            others.append(oth); runs.append(run); sib_busy.append(busy(sibs)); set_busy.append(busy(cpus))
            load1 = float(open("/proc/loadavg").read().split()[0])
            ld.write(f"{mono_ns()}\t{load1:.2f}\t{machine:.2f}\t{run:.2f}\t{oth:.2f}\t{set_busy[-1]:.3f}\t{sib_busy[-1]:.3f}\n")
            prev, prev_run, prev_t = now, now_run, now_t
    wall = time.time() - t0
    th.close(); ld.close(); log.close()
    txt = open(os.path.join(a.out, "log.txt"), errors="replace").read()
    out = {"exit": proc.returncode, "wall_s": wall, "start_mono_ns": m0, "cpus": cpus, "siblings": sibs,
           "bin": a.bin, "preset": a.preset, "start": a.start, "myr": float(a.myr), "threads": int(a.threads),
           "perf_hz": a.perf, "trace": a.trace}
    m = re.search(r"threads=\d+ total_ms=([\d.]+) peak_bytes=(\d+) stages=(\S+)", txt)
    if m:
        out["total_s"] = float(m.group(1)) / 1000
        out["peak_bytes"] = int(m.group(2))
        out["stages_s"] = {k: float(v) / 1000 for k, v in (kv.split(":") for kv in m.group(3).split(","))}
    if others:
        out["others_mean"] = sum(others) / len(others)
        out["others_p90"] = sorted(others)[int(0.9 * len(others))]
        out["run_cpus_mean"] = sum(runs) / len(runs)
        out["siblings_busy_mean"] = sum(sib_busy) / len(sib_busy)
    for k, key in (("User time (seconds)", "user_s"), ("System time (seconds)", "system_s"),
                   ("Maximum resident set size (kbytes)", "max_rss_kb"),
                   ("Major (requiring I/O) page faults", "major_faults"),
                   ("Minor (reclaiming a frame) page faults", "minor_faults"),
                   ("Voluntary context switches", "voluntary_cs"),
                   ("Involuntary context switches", "involuntary_cs"),
                   ("File system outputs", "fs_outputs"), ("File system inputs", "fs_inputs")):
        mm = re.search(rf"{re.escape(k)}: (\S+)", txt)
        if mm:
            out[key] = float(mm.group(1))
    json.dump(out, open(os.path.join(a.out, "summary.json"), "w"), indent=1)
    print(json.dumps(out))
    sys.exit(proc.returncode)


if __name__ == "__main__":
    main()
