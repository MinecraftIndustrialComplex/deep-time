#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-24: the tables of docs/design/faster-create-world.md from the runs of tools/review/p1-24-matrix.sh.

    python3 tools/review/p1-24-summary.py <runs dir> [run ...]   (default: every run with a result.json)

Per run: wall time, CPU time, the stages' own seconds (the run's `timings` line: plates, climate,
surface, volcanoes, impacts and structure are one coupled stage, each sub-stage timed), the bake,
peak memory and the world folder's size. Memory is the scope's anonymous memory every 2 s
(mem.log, runs from 2026-10-04 08:40 on) split into three windows by the run's stamped log lines:
the coupled stage, deposits, and young + amplify + bake (from the end of deposits: the session's
log lines of the last three come out together at the very end, so they are one window); older runs have only the whole run's
peak resident set (/usr/bin/time) and the scope's peak (which includes the page cache of what the
run wrote).
"""

import json
import re
import sys
from pathlib import Path

COUPLED = ["plates", "climate", "surface", "volcanoes", "impacts", "structure"]


def windows(run: Path):
    log = run / "run.log"
    mem = run / "mem.log"
    if not log.exists() or not mem.exists():
        return None
    samples = []
    for line in mem.read_text().splitlines():
        p = line.split()
        if len(p) == 2:
            samples.append((float(p[0]), int(p[1]) / 1048576))
    if not samples:
        return None
    marks = {}
    for raw in log.read_text(errors="replace").splitlines():
        t, _, rest = raw.partition(" ")
        try:
            t = float(t)
        except ValueError:
            continue
        m = re.search(r"stage (\w+) (started|finished)", rest)
        if m and (m.group(1), m.group(2)) not in marks:
            marks[(m.group(1), m.group(2))] = t
    t0 = marks.get(("plates", "started"), samples[0][0])
    td = marks.get(("deposits", "started"))
    ty = marks.get(("deposits", "finished"))
    end = samples[-1][0] + 1

    def peak(a, b):
        v = [g for t, g in samples if a is not None and b is not None and a <= t < b]
        return round(max(v), 2) if v else None

    return {
        "coupled": peak(t0, td or end),
        "deposits": peak(td, ty or end),
        "young_amplify_bake": peak(ty, end),
        "whole": round(max(g for _, g in samples), 2),
    }


def main() -> int:
    runs = Path(sys.argv[1])
    names = sys.argv[2:] or sorted(p.name for p in runs.iterdir() if (p / "result.json").exists())
    rows = []
    for name in names:
        r = json.loads((runs / name / "result.json").read_text())
        if "exit" not in r:
            continue
        st = r.get("stages_s") or {}
        w = windows(runs / name)
        rows.append((name, r, st, w))
    print("| run | exit | wall s | CPU s | coupled s (plates/climate/surface/volc/impacts/structure) | deposits s | young s | amplify s | bake s | peak memory GiB (coupled / deposits / young+amplify+bake) | max RSS GiB | world MB |")
    print("|---|---|---|---|---|---|---|---|---|---|---|---|")
    for name, r, st, w in rows:
        coupled = sum(st.get(k, 0) for k in COUPLED)
        parts = "/".join(f"{st.get(k, 0):.0f}" for k in COUPLED)
        cpu = (r.get("user_s") or 0) + (r.get("sys_s") or 0)
        mem = f"{w['coupled']} / {w['deposits']} / {w['young_amplify_bake']}" if w else "—"
        print(f"| {name} | {r['exit']} | {r['wall_s']:.0f} | {cpu:.0f} | {coupled:.0f} ({parts}) | "
              f"{st.get('deposits', 0):.0f} | {st.get('young', 0):.0f} | {st.get('amplify', 0):.0f} | "
              f"{r['bake_s'] if r.get('bake_s') is not None else '—'} | {mem} | {r.get('max_rss_gib')} | "
              f"{r.get('world_bytes', 0) / 1e6:.0f} |")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
