#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Markdown tables from tools/review/p1-17-bench.sh logs (P1-17, docs/design/p1-17-benchmark-gate.md).

    p1-17-table.py pair  <out root> <before label> <after label> <length_myr> [threads]
        per preset (quick, full 16k): every stage and sub-timer, both starts, before → after
    p1-17-table.py scale <out root> <length_myr> <label>:<threads>...
        per preset: one column per thread count (scattered continents)
"""
import os
import re
import sys

PRESETS = ["earthlike_quick_16k", "earthlike_full_16k"]
STARTS = ["scattered_continents", "supercontinent"]
STAGES = ["plates", "climate", "surface", "volcanoes", "impacts", "structure", "deposits",
          "young", "amplify", "finalize", "total"]
SUBS = ["surface.diffusion", "surface.diffusion_solve", "surface.slope_failure", "surface.flexure",
        "surface.advect", "surface.glacial", "surface.fields_events", "surface.spl",
        "surface.pflood", "climate.ebm2d", "climate.clock", "climate.cells", "climate.ice",
        "climate.present", "structure.outputs", "structure.deformation",
        "structure.frame_fields", "volcanoes.births", "volcanoes.measure"]
EXTRA = ["user s", "cpu %", "peak GiB (gauge)", "max RSS GiB", "estimate GiB", "estimate s"]


def parse(path):
    if not os.path.exists(path):
        return None
    txt = open(path, errors="replace").read()
    m = re.search(r"threads=\d+ total_ms=([\d.]+) peak_bytes=(\d+) stages=(\S+)", txt)
    if not m:
        return None
    r = {"total": float(m.group(1)) / 1000, "peak GiB (gauge)": int(m.group(2)) / 2**30}
    for kv in m.group(3).split(","):
        k, v = kv.split(":")
        r[k] = float(v) / 1000
    m = re.search(r"timings (.*)", txt)
    if m:
        for grp in re.finditer(r"(\w+)\[([^\]]*)\]", m.group(1)):
            for kv in grp.group(2).split(","):
                if "=" in kv:
                    k, v = kv.split("=", 1)
                    try:
                        r[grp.group(1) + "." + k] = float(v.rstrip("s"))
                    except ValueError:
                        pass
    m = re.search(r"estimate peak_bytes=(\d+) seconds=(\d+)", txt)
    if m:
        r["estimate GiB"] = int(m.group(1)) / 2**30
        r["estimate s"] = int(m.group(2))
    m = re.search(r"User time \(seconds\): ([\d.]+)", txt)
    if m:
        r["user s"] = float(m.group(1))
    m = re.search(r"Percent of CPU this job got: (\d+)%", txt)
    if m:
        r["cpu %"] = int(m.group(1))
    m = re.search(r"Maximum resident set size \(kbytes\): (\d+)", txt)
    if m:
        r["max RSS GiB"] = int(m.group(1)) * 1024 / 2**30
    return r


def plate_keys(runs):
    """The plate step's parts (plates.*), in the order the first run that has them lists them."""
    for r in runs:
        if r and any(k.startswith("plates.") for k in r):
            return [k for k in r if k.startswith("plates.")]
    return []


def fmt(k, z):
    if z is None:
        return "–"
    if "GiB" in k:
        return f"{z:.2f}"
    return f"{z:.1f}" if isinstance(z, float) else str(z)


def pair(root, a, b, length, threads):
    for preset in PRESETS:
        rs = {s: (parse(f"{root}/{a}-{preset}-{s}-{length}-t{threads}/log.txt"),
                  parse(f"{root}/{b}-{preset}-{s}-{length}-t{threads}/log.txt")) for s in STARTS}
        print(f"\n#### {preset}, {length} Myr, {threads} threads (seconds; before {a}, after {b})\n")
        print("| stage | scattered before | after | × | supercontinent before | after | × |")
        print("|---|---:|---:|---:|---:|---:|---:|")
        for k in STAGES + SUBS + plate_keys([r for p in rs.values() for r in p]) + EXTRA:
            row = [k if k in STAGES or k in EXTRA else "  " + k]
            seen = False
            for s in STARTS:
                x, y = rs[s]
                u = x.get(k) if x else None
                v = y.get(k) if y else None
                seen = seen or u is not None or v is not None
                ratio = f"{u / v:.2f}" if (u and v and k not in EXTRA) else ""
                row += [fmt(k, u), fmt(k, v), ratio]
            if seen:
                print("| " + " | ".join(row) + " |")


def scale(root, length, cols):
    for preset in PRESETS:
        runs = []
        for c in cols:
            lab, t = c.split(":")
            r = parse(f"{root}/{lab}-{preset}-scattered_continents-{length}-t{t}/log.txt")
            if r:
                runs.append((int(t), r))
        if not runs:
            continue
        runs.sort(key=lambda x: x[0])
        ts = [t for t, _ in runs]
        print(f"\n#### {preset}, scattered continents, {length} Myr: seconds by thread count\n")
        print("| stage | " + " | ".join(f"{t} t" for t in ts) + f" | {ts[0]} → {ts[-1]} t |")
        print("|---|" + "---:|" * (len(ts) + 1))
        for k in STAGES + SUBS + plate_keys([r for _, r in runs]) + ["user s", "cpu %"]:
            v = [r.get(k) for _, r in runs]
            if any(x is None for x in v):
                continue
            sp = f"{v[0] / v[-1]:.2f}×" if k != "cpu %" and v[-1] else ""
            print(f"| {k} | " + " | ".join(f"{x:.0f}" for x in v) + f" | {sp} |")


if __name__ == "__main__":
    if len(sys.argv) >= 6 and sys.argv[1] == "pair":
        pair(sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5],
             sys.argv[6] if len(sys.argv) > 6 else "8")
    elif len(sys.argv) >= 5 and sys.argv[1] == "scale":
        scale(sys.argv[2], sys.argv[3], sys.argv[4:])
    else:
        print(__doc__)
        sys.exit(2)
