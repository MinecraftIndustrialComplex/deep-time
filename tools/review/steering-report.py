#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Summarises tools/review/steering-measure.sh's runs (docs/design/playability-steering.md §8).

    python3 tools/review/steering-report.py <out> [<master run folder> ...]

Per run: its wall time, the land share and global mean temperature at its end, the steering's
report (offsets, artefact counters), the epoch store (checkpoints, bytes, write time) and, for
the resumes, where they resumed from. With master run folders: which world-data files differ
between the matching steering-off run and master's.
"""
import json
import os
import re
import sys


def manifest(d):
    with open(os.path.join(d, "manifest.json")) as f:
        return json.load(f)


def last_row(notes):
    s = notes.get("climate", {}).get("series", {})
    cols, rows = s.get("columns", []), s.get("rows", [])
    if not rows:
        return {}
    return dict(zip(cols, rows[-1]))


def seconds(log):
    try:
        txt = open(log).read()
    except OSError:
        return None
    m = re.findall(r"in ([0-9.]+)s\n", txt)
    return float(m[-1]) if m else None


def du(d):
    total = 0
    for root, _, files in os.walk(d):
        for f in files:
            total += os.path.getsize(os.path.join(root, f))
    return total


def main():
    out = sys.argv[1]
    rows = []
    for name in sorted(os.listdir(out)):
        d = os.path.join(out, name)
        if not os.path.isdir(d) or not os.path.exists(os.path.join(d, "manifest.json")):
            continue
        m = manifest(d)
        n = m.get("notes", {})
        r = last_row(n)
        st = n.get("climate", {}).get("steering")
        run = n.get("run", {})
        rec = {
            "run": name,
            "seconds": seconds(os.path.join(out, name + ".log")),
            "t_end_myr": run.get("t_years", 0) / 1e6,
            "land_frac": r.get("land_frac"),
            "t_global_c": r.get("t_global_c"),
            "co2_ppm": r.get("co2_ppm"),
            "sea_level_m": r.get("sea_level_m"),
            "snowballs": n.get("climate", {}).get("snowballs"),
            "epochs": run.get("epochs"),
            "epochs_dir_mib": du(os.path.join(d, "epochs")) / 2**20,
            "resumed": run.get("resumed"),
        }
        if st:
            rec["steering"] = {k: st.get(k) for k in (
                "tail_start", "end", "land_met", "temperature_met", "sea_offset_m",
                "earth_equivalent_volume_km3", "share_of_earths_ocean", "outgassing_factor",
                "max_sea_step_m", "max_ln_outgassing_step", "sea_reversals", "co2_reversals")}
        rows.append(rec)
    print(json.dumps(rows, indent=1))
    for base in sys.argv[2:]:
        mb = manifest(base)
        seed = "s1" if "s1" in base else "s2"
        d = os.path.join(out, "off-" + seed)
        if not os.path.exists(d):
            continue
        a = {f["path"]: f["crc32"] for f in manifest(d)["files"]}
        b = {f["path"]: f["crc32"] for f in mb["files"]}
        diff = sorted(p for p in set(a) | set(b) if a.get(p) != b.get(p))
        print(json.dumps({"off_vs_master": base, "files": len(a), "differ": diff}))


if __name__ == "__main__":
    main()
