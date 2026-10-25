#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The thick crust's denudation by class, from runs with the front trace on: m4erode.py <run folder> ...

The front trace's attribution (`att_t<Myr>.bin`, plates/front_trace.rs) records for every visible continental
sample that is or was 45 km thick the last part that changed its thickness by 1 m or more (`touch_part`, a
`gap_budget::Part`; 1 is `Erosion`, the surface feedback's rock: erosion less deposition), that change
(`touch_dm`, m) and when (`touch_t`). Where that part is `Erosion` in the snapshot's own step, -touch_dm over the
step (`time_step_myr`, 2 Myr in earthlike_quick_16k) is the column's net denudation rate in that step, m/Myr.

Per class (collision, Andean, untagged; on a keel K >= 100 km or not; by Myr since the stamp) over the samples
45 km thick or more: samples, the share whose last change was this step's erosion, the denudation rate p50 and
mean (m/Myr, lowering positive), the height over the sea p50 H, and the rate against Pinet & Souriau 1988's law
for old orogens, Ds = 61e-6 H m/kyr (0.061 H m/Myr; read by a Sonnet sub-agent), as the p50 of E / (0.061 H).
Pooled over the runs given and over each run's snapshots. Needs numpy only."""
import json
import sys
from pathlib import Path

import numpy as np

ATT = 27
DT_MYR = 2.0
EROSION = 1
AGES = [0, 10, 20, 30, 50, 100, 200, 1e9]


def load(run):
    rec = json.loads((run / "rec.json").read_text())
    ts = np.array([s["t"] for s in rec["steps"]])
    seas = np.array([s.get("sea_m", 0.0) or 0.0 for s in rec["steps"]])
    out = []
    for line in (run / "trace" / "att_index.txt").read_text().splitlines():
        kv = dict(p.split("=", 1) for p in line.split())
        t = float(kv["t"])
        a = np.fromfile(run / "trace" / kv["file"], "<f4").reshape(-1, ATT).astype(np.float64)
        out.append((t, float(seas[int(np.argmin(np.abs(ts - t)))]), a))
    return out


def main():
    snaps = [s for r in sys.argv[1:] for s in load(Path(r))]
    print(f"{len(sys.argv) - 1} runs, {len(snaps)} snapshots")
    T = np.concatenate([a[:, 2] for _, _, a in snaps])
    H = np.concatenate([a[:, 3] - sea for _, sea, a in snaps])
    tag = np.concatenate([np.rint(a[:, 4]) for _, _, a in snaps])
    age = np.concatenate([a[:, 5] for _, _, a in snaps])
    K = np.concatenate([a[:, 7] for _, _, a in snaps])
    tp = np.concatenate([a[:, 14] for _, _, a in snaps])
    dm = np.concatenate([a[:, 15] for _, _, a in snaps])
    tt = np.concatenate([a[:, 16] for _, _, a in snaps])
    tnow = np.concatenate([np.full(len(a), t) for t, _, a in snaps])
    thick = T >= 45_000
    fresh = (np.rint(tp) == EROSION) & (np.abs(tt - tnow) <= DT_MYR + 1e-3)
    E = -dm / DT_MYR
    print("thick (>= 45 km) samples:", int(thick.sum()), "; last change this step's erosion:",
          f"{(fresh & thick).sum() / max(thick.sum(), 1):.2f}")
    other = thick & ~fresh
    if other.any():
        parts, counts = np.unique(np.rint(tp[other]).astype(int), return_counts=True)
        top = sorted(zip(counts, parts), reverse=True)[:6]
        print("  the others' last part (gap_budget::Part id: share of the thick):",
              ", ".join(f"{p}: {c / thick.sum():.2f}" for c, p in top))
    print("\nclass, keel, age Myr: n thick, share fresh, E p50 / mean m/Myr (lowering +), H p50 m, E/(0.061 H) p50,"
          " share of fresh with E < 0 (deposition)")
    for tg, name in ((2, "collision"), (1, "andean"), (0, "untagged")):
        for kname, km in (("keel", K >= 100), ("no keel", K < 100)):
            for a0, a1 in zip(AGES[:-1], AGES[1:]):
                m = thick & (tag == tg) & km
                if tg != 0:
                    m &= (age >= a0) & (age < a1)
                elif a0 > 0:
                    break
                n = int(m.sum())
                if n < 20:
                    continue
                f = m & fresh
                e = E[f]
                hp = np.median(H[f]) if f.any() else float("nan")
                ratio = e / np.maximum(0.061 * H[f], 1.0) if f.any() else np.array([np.nan])
                label = "any" if tg == 0 else f"{a0:g}-{a1:g}"
                print(f"  {name:9s} {kname:7s} {label:>8s}: n {n:6d}  fresh {f.sum() / n:.2f}"
                      f"  E {np.median(e) if e.size else float('nan'):6.1f} / {e.mean() if e.size else float('nan'):6.1f}"
                      f"  H {hp:6.0f}  E/PS {np.median(ratio):5.2f}  dep {np.mean(e < 0) if e.size else float('nan'):.2f}")
    # Against the height alone, all thick continental samples: E by H band (Pinet & Souriau's form).
    print("\nall thick samples whose last change was this step's erosion, by H over the sea: n, E p50 / mean m/Myr,"
          " 0.061 H at the band's middle")
    for h0, h1 in ((0, 500), (500, 1000), (1000, 1500), (1500, 2000), (2000, 3000), (3000, 4000), (4000, 9000)):
        m = thick & fresh & (H >= h0) & (H < h1)
        if m.sum() < 20:
            continue
        print(f"  {h0:5d}-{h1:5d} m: n {int(m.sum()):6d}  E {np.median(E[m]):6.1f} / {E[m].mean():6.1f}"
              f"  PS {0.061 * (h0 + min(h1, 5000)) / 2:6.1f}")


if __name__ == "__main__":
    main()
