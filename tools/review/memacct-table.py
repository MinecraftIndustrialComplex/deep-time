#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""memacct: per-stage memory of traced runs, as a Markdown table.

    python3 tools/review/memacct-table.py <run dir>...

Each <run dir> is a `tools/review/p1-24-profile.sh` output folder: its run.log (the run's
`estimate` line) and memtrace.jsonl (DEEPTIME_MEM_TRACE: one line per phase occurrence). For every
phase: what the run reported (the counting allocator's peak while it ran, the figure the memory cap
and the editor use), the process's resident peak in the same phase (VmHWM, reset at the phase's
start), the most a reservation asked for, and the estimate (the stage's figure; the keyframes
and a stage's end are listed after the stage that stepped before them, with its figure; the bake
has the run's peak). Then the run's maxima and the sampled
anonymous memory of its cgroup (mem.log).
"""
import json
import re
import sys

M = 1048576.0


def table(d):
    log = open(f"{d}/run.log", errors="replace").read()
    est = {}
    peak_est = None
    m = re.search(r"deeptime run: estimate peak_mib=(\d+) .*stages_mib\[([^\]]*)\]", log)
    if m:
        peak_est = float(m.group(1))
        for name, v in re.findall(r"(\w+)=(\d+)", m.group(2)):
            est[name] = float(v)
    rows, order = {}, []
    stepped = None
    for line in open(f"{d}/memtrace.jsonl"):
        r = json.loads(line)
        p = r["phase"]
        # The keyframes and a stage's end belong to the stage that stepped before them.
        if p in ("keyframes", "finish") and stepped:
            p = f"{p} after {stepped}"
        elif p != "bake":
            stepped = p
        if p not in rows:
            rows[p] = {"live": 0, "rss": 0, "reserved": 0, "n": 0}
            order.append(p)
        a = rows[p]
        a["n"] += 1
        a["live"] = max(a["live"], r["live_peak"])
        a["rss"] = max(a["rss"], r["rss_peak"])
        a["reserved"] = max(a["reserved"], r.get("reserved", 0))
    anon = 0.0
    try:
        for line in open(f"{d}/mem.log"):
            p = line.split()
            if len(p) == 2:
                anon = max(anon, int(p[1]) / 1024.0)
    except OSError:
        pass
    total = re.search(r"\) in ([\d.]+)s", log)
    print(f"### {d}")
    print()
    print("| phase | steps | reported MiB | RSS MiB | reported/RSS | reserved MiB | estimate MiB | estimate/RSS |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|")
    for p in order:
        a = rows[p]
        live, rss = a["live"] / M, a["rss"] / M
        owner = p.split(" after ")[-1]
        e = est.get(owner, peak_est)
        es = "-" if e is None else f"{e:.0f}{'' if owner in est else ' (run)'}"
        er = "-" if e is None else f"{e / rss:.2f}"
        print(
            f"| {p} | {a['n']} | {live:.0f} | {rss:.0f} | {live / rss:.2f} | "
            f"{a['reserved'] / M:.0f} | {es} | {er} |"
        )
    live = max(a["live"] for a in rows.values()) / M
    rss = max(a["rss"] for a in rows.values()) / M
    pe = "-" if peak_est is None else f"**{peak_est:.0f}**"
    pr = "-" if peak_est is None else f"{peak_est / rss:.2f}"
    print(f"| **run** | | **{live:.0f}** | **{rss:.0f}** | {live / rss:.2f} | | {pe} | {pr} |")
    print()
    print(f"Sampled cgroup anonymous memory peak: {anon:.0f} MiB; run time {total.group(1) if total else '?'} s.")
    print()


for d in sys.argv[1:]:
    table(d.rstrip("/"))
