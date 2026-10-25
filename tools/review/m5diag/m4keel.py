#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The keel test of the M4 diagnosis, from a run with the front trace on: m4keel.py <run folder>

The crust-budget maps hold no lithosphere or keel, so the belts' slow decay cannot be put on the
lithosphere's thermal state from them. The front trace's thickening attribution
(`DEEPTIME_FRONT_TRACE`, plates/front_trace.rs, `att_t<Myr>.bin`: lat, lon, thickness m, elevation m,
orogeny tag, Myr since the stamp, lithosphere km, keel km, plate, ... 27 f32 per visible continental
sample) holds both. For the collision (tag 2) and Andean (tag 1) samples by age since the stamp, split
by the keel (K >= 100 km, K < 100 km): samples, T p50 km, the share above 2 km over the run's sea, the
lithosphere p50 and the sheet's viscosity factor at it (PLT-415: eta / eta_0 = exp(t_eq / 62.8),
t_eq = 203 ln(165 / (225 - L)), 0 at L <= 60). Needs numpy only."""
import json
import sys
from pathlib import Path

import numpy as np

ATT = 27
AGES = [0, 10, 20, 30, 50, 100, 1e9]


def eta_factor(L):
    L = np.clip(L, 60.0, 224.9)
    t_eq = np.where(L <= 60.0, 0.0, 203.0 * np.log(165.0 / (225.0 - L)))
    return np.exp(t_eq / 62.8)


def main():
    run = Path(sys.argv[1])
    rec = json.loads((run / "rec.json").read_text())
    ts = np.array([s["t"] for s in rec["steps"]])
    seas = np.array([s.get("sea_m", 0.0) or 0.0 for s in rec["steps"]])
    rows = []
    for line in (run / "trace" / "att_index.txt").read_text().splitlines():
        kv = dict(p.split("=", 1) for p in line.split())
        t = float(kv["t"])
        a = np.fromfile(run / "trace" / kv["file"], "<f4").reshape(-1, ATT).astype(np.float64)
        sea = float(seas[int(np.argmin(np.abs(ts - t)))])
        rows.append((t, sea, a))
    print(f"{run}: snapshots {[r[0] for r in rows]}, seas {[round(r[1]) for r in rows]}")
    allA = np.vstack([a for _, _, a in rows])
    K, L = allA[:, 7], allA[:, 6]
    print(f"all visible continental samples: {len(allA)}; keel >= 100 km {np.mean(K >= 100):.3f},"
          f" >= 190 km {np.mean(K >= 190):.3f}; lithosphere p10/p50/p90 {np.percentile(L, 10):.0f}/"
          f"{np.median(L):.0f}/{np.percentile(L, 90):.0f} km")
    # The high ground (continental, above 2 km over the sea) as a share of the continental land, by tag and keel.
    print("\nM4 (continental land only) by snapshot: all; collision on a keel / not; Andean on a keel / not;"
          " untagged on a keel / not; the high ground on a keel")
    for t, sea, a in rows:
        zs = a[:, 3] - sea
        land, hi, keel, tag = zs > 0, zs > 2000, a[:, 7] >= 100, np.rint(a[:, 4])
        nl = max(int(land.sum()), 1)

        def s(m):
            return f"{(hi & m).sum() / nl:.4f}"
        print(f"  t {t:6.0f}: {s(hi)}  coll {s((tag == 2) & keel)} / {s((tag == 2) & ~keel)}"
              f"  andean {s((tag == 1) & keel)} / {s((tag == 1) & ~keel)}  untagged {s((tag == 0) & keel)}"
              f" / {s((tag == 0) & ~keel)}  on a keel {(hi & keel).sum() / max(int(hi.sum()), 1):.2f}")
    for tag, name in ((2, "collision"), (1, "andean"), (0, "untagged, T >= 49.5 km")):
        print(f"\n{name}: age Myr, keel class: samples, T p50 km, above 2 km over the sea, L p50 km,"
              f" eta/eta0 at L p50, K p50 km")
        for a0, a1 in zip(AGES[:-1], AGES[1:]):
            for kname, kcut in (("K>=100", lambda k: k >= 100), ("K<100", lambda k: k < 100)):
                n = 0
                Ts, hi, Ls, Ks = [], [], [], []
                for t, sea, a in rows:
                    m = (np.rint(a[:, 4]) == tag) & kcut(a[:, 7])
                    if tag == 0:
                        m &= a[:, 2] >= 49_500
                    else:
                        m &= (a[:, 5] >= a0) & (a[:, 5] < a1)
                    n += int(m.sum())
                    Ts.append(a[m, 2] / 1000)
                    hi.append(a[m, 3] - sea > 2000)
                    Ls.append(a[m, 6])
                    Ks.append(a[m, 7])
                if n == 0:
                    continue
                Ts, hi, Ls, Ks = (np.concatenate(x) for x in (Ts, hi, Ls, Ks))
                lp = float(np.median(Ls))
                label = "any age" if tag == 0 else f"{a0:g}-{a1:g}"
                print(f"  {label:>8s} {kname:7s} n {n:6d}  T {np.median(Ts):5.1f}  >2km {hi.mean():.3f}"
                      f"  L {lp:5.0f}  eta/eta0 {float(eta_factor(np.array([lp]))[0]):8.1f}  K {np.median(Ks):5.0f}")
            if tag == 0:
                break


if __name__ == "__main__":
    main()
