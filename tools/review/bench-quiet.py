#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""One `deeptime bench` run held to a set of CPUs, with the machine's load from everyone else
recorded beside it (P1-19 item 1 / the P1-17 gate on the shared test VM).

    tools/review/bench-quiet.py --cpus 0,2,4,6,8,10,12,14 --out <dir> <bin> <preset> <start> <myr> [threads]

Runs `env RAYON_NUM_THREADS=<threads> /usr/bin/time -v <bin> bench --preset ... --threads <threads>
--out <dir>/bench.json --work <dir>/work` with its CPU affinity set to --cpus (the default: one
logical CPU of each of the first eight physical cores, from /sys's SMT sibling lists), and every
5 s samples /proc/stat, /proc/loadavg and the run's own CPU time, so the busy CPUs of everyone else
(the machine's busy time less the run's) are known for the whole run. Writes <dir>/log.txt (the
bench output and time -v), <dir>/load.tsv (the samples) and <dir>/summary.txt (wall time, stages,
others' load: mean, p90, max; the busy share of the run's CPUs' SMT siblings). Run it admitted
(`detach.sh start <name> --kind sim:<preset>:<threads>t tools/review/bench-quiet.py ...`).
Standard library only.
"""
import argparse
import os
import re
import subprocess
import sys
import time

HZ = os.sysconf("SC_CLK_TCK")


def physical_cores():
    """One logical CPU per physical core, in order: the first of each sibling list."""
    seen, out = set(), []
    for c in sorted(int(d[3:]) for d in os.listdir("/sys/devices/system/cpu") if re.fullmatch(r"cpu\d+", d)):
        try:
            sib = open(f"/sys/devices/system/cpu/cpu{c}/topology/thread_siblings_list").read().strip()
        except OSError:
            continue
        if sib not in seen:
            seen.add(sib)
            out.append(c)
    return out


def siblings(cpu):
    txt = open(f"/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list").read().strip()
    out = []
    for part in txt.split(","):
        a, _, b = part.partition("-")
        out += list(range(int(a), int(b or a) + 1))
    return [c for c in out if c != cpu]


def stat():
    """{cpu label: (busy jiffies, total jiffies)} from /proc/stat ('cpu' is the machine)."""
    out = {}
    for line in open("/proc/stat"):
        if not line.startswith("cpu"):
            break
        f = line.split()
        v = [int(x) for x in f[1:]]
        idle = v[3] + (v[4] if len(v) > 4 else 0)
        out[f[0]] = (sum(v[:8]) - idle, sum(v[:8]))
    return out


def tree_jiffies(root):
    """utime + stime of `root` and every descendant (threads are counted in their process)."""
    kids, times = {}, {}
    for p in os.listdir("/proc"):
        if not p.isdigit():
            continue
        try:
            s = open(f"/proc/{p}/stat").read()
        except OSError:
            continue
        rest = s[s.rfind(")") + 2:].split()
        kids.setdefault(int(rest[1]), []).append(int(p))
        times[int(p)] = int(rest[11]) + int(rest[12])
    total, todo = 0, [root]
    while todo:
        p = todo.pop()
        total += times.get(p, 0)
        todo += kids.get(p, [])
    return total


def pct(xs, q):
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(q * len(xs)))] if xs else float("nan")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cpus", default="")
    ap.add_argument("--out", required=True)
    ap.add_argument("--interval", type=float, default=5.0)
    ap.add_argument("bin")
    ap.add_argument("preset")
    ap.add_argument("start")
    ap.add_argument("myr")
    ap.add_argument("threads", nargs="?", default="8")
    a = ap.parse_args()
    t = int(a.threads)
    cpus = [int(c) for c in a.cpus.split(",")] if a.cpus else physical_cores()[:t]
    sibs = sorted({s for c in cpus for s in siblings(c)} - set(cpus))
    os.makedirs(a.out, exist_ok=True)
    cmd = ["env", f"RAYON_NUM_THREADS={t}", "/usr/bin/time", "-v", a.bin, "bench", "--preset", a.preset,
           "--start", a.start, "--length-myr", a.myr, "--threads", str(t),
           "--out", os.path.join(a.out, "bench.json"), "--work", os.path.join(a.out, "work")]
    log = open(os.path.join(a.out, "log.txt"), "w")
    log.write(f"# cpus {','.join(map(str, cpus))} (siblings {','.join(map(str, sibs))})\n# {' '.join(cmd)}\n")
    log.flush()
    tsv = open(os.path.join(a.out, "load.tsv"), "w")
    tsv.write("t_s\tload1\tmachine_busy_cpus\trun_cpus\tothers_cpus\trun_set_busy\tsibling_set_busy\n")
    t0 = time.time()
    proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT, preexec_fn=lambda: os.sched_setaffinity(0, cpus))
    prev, prev_run, prev_t = stat(), 0, time.time()
    others, loads, runs, sib_busy = [], [], [], []
    while proc.poll() is None:
        time.sleep(a.interval)
        now, now_run, now_t = stat(), tree_jiffies(proc.pid), time.time()
        dt = max(1e-6, now_t - prev_t)
        machine = (now["cpu"][0] - prev["cpu"][0]) / HZ / dt
        run = max(0, now_run - prev_run) / HZ / dt
        def busy(cs):
            b = sum(now[f"cpu{c}"][0] - prev[f"cpu{c}"][0] for c in cs)
            tot = sum(now[f"cpu{c}"][1] - prev[f"cpu{c}"][1] for c in cs)
            return b / tot if tot else 0.0
        load1 = float(open("/proc/loadavg").read().split()[0])
        oth = max(0.0, machine - run)
        others.append(oth); loads.append(load1); runs.append(run); sib_busy.append(busy(sibs))
        tsv.write(f"{now_t - t0:.0f}\t{load1:.2f}\t{machine:.2f}\t{run:.2f}\t{oth:.2f}\t{busy(cpus):.3f}\t{busy(sibs):.3f}\n")
        tsv.flush()
        prev, prev_run, prev_t = now, now_run, now_t
    wall = time.time() - t0
    log.close()
    txt = open(os.path.join(a.out, "log.txt"), errors="replace").read()
    m = re.search(r"threads=\d+ total_ms=([\d.]+) peak_bytes=(\d+) stages=(\S+)", txt)
    lines = [f"exit {proc.returncode}, wall {wall:.1f} s, cpus {','.join(map(str, cpus))}"]
    if m:
        lines.append(f"total {float(m.group(1)) / 1000:.1f} s, peak {int(m.group(2)) / 2**20:.0f} MiB")
        lines.append("stages " + ", ".join(f"{k} {float(v) / 1000:.1f}" for k, v in (kv.split(":") for kv in m.group(3).split(","))))
    if others:
        mean = sum(others) / len(others)
        lines.append(f"others' busy CPUs: mean {mean:.1f}, p90 {pct(others, 0.9):.1f}, max {max(others):.1f}; "
                     f"load1 mean {sum(loads) / len(loads):.1f}, max {max(loads):.1f}; the run's CPUs mean {sum(runs) / len(runs):.2f}; "
                     f"its cores' SMT siblings busy {100 * sum(sib_busy) / len(sib_busy):.0f} % on average")
    for k in ("User time", "System time", "Percent of CPU", "Maximum resident"):
        mm = re.search(rf"{k}[^:]*: (.*)", txt)
        if mm:
            lines.append(f"{k}: {mm.group(1)}")
    open(os.path.join(a.out, "summary.txt"), "w").write("\n".join(lines) + "\n")
    print("\n".join(lines))
    sys.exit(proc.returncode)


if __name__ == "__main__":
    main()
