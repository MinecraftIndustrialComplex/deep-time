#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `front` measure (the front lane, 2026-10-24): why the model's plate boundary and its
subduction front are longer than Earth's, from the dynamics record alone (`rec.json`), printed as a JSON
object on stdout. It splits the boundary's length into the plates' sizes and the plates' shapes:

    total boundary = E x (sum over plates of P_cap) / 2

where P_cap is the perimeter of a spherical cap of the plate's area (the shortest closed curve round that
area) and E = sum of perimeters / sum of P_cap is how far the plates' outlines are from caps. Bird 2003's
PB2002 (52 plates, from PB2002_plates.dig, Bird2003/PB2002_geometry.json): sum of P_cap 345,081 km,
E 1.510 at full resolution, 1.418 measured with a 77 km divider and 1.274 with a 380 km one.

    python3 -I measure_front.py <run folder>

A plate's perimeter is its record's `boundary_km` (one sample spacing per visible sample with a neighbour
of another plate or one hidden under another plate) over the lattice's factor 1.043 (one spacing per
boundary sample over the straight length on the 100,000-sample lattice, the share test in
`plates/shortening.rs`, ledger PLT-530); its area is its visible samples' share of the sphere. Steps after
the first 100 Myr; each step's plates pooled over the run. Plates under 10 samples are left out.

  plates           plates alive per step (mean)
  bnd_km           the boundary, km per step (sum of perimeters / 2; Bird 2003 Table 3: 260,487)
  pcap_sum_km      sum of P_cap per step (Bird 345,081)
  excess           E, sum of perimeters over sum of P_cap (Bird 1.510; 1.418 at 77 km)
  ppcap_p50, ppcap_aw   each plate's P / P_cap: median over plate-steps, area-weighted mean
  excess_small, excess_mid, excess_large   E of plates under 0.0377 sr, 0.0377 to 0.5 sr, 0.5 sr or more
                   (Bird full 1.459 / 1.424 / 1.579, at 77 km 1.383 / 1.347 / 1.473)
  bnd_small, bnd_mid, bnd_large   their shares of the sum of perimeters (Bird 0.180 / 0.285 / 0.535)
  front_km         the subduction fronts, km per step (the plates' `front_km` over the lattice's factor;
                   Bird's SUB 51,310)
  front_small, front_mid, front_large   its shares by the subducting plate's size (Bird: SUB by
                   subducting plate in Bird2003/PB2002_geometry.json C.per_plate: 0.112 / 0.339 / 0.549)
  oo_front_km      the ocean-ocean (island-arc) front, km per step (the boundary mix's ocean-ocean class,
                   counted on the overriding side, over the lattice's factor)
  excess_t1 ... excess_t9   E in each 100 Myr after the first (t0 + 100 to 200, ..., 900 to 1000)
  bnd_t1, bnd_t9, front_t1, front_t9, plates_t1, plates_t9   the boundary, front and plates in the
                   first and last of those windows
  excess_young, excess_mature, excess_old   E of plates under 50 Myr old, 50 to 200, 200 or more
  excess_initial, excess_rift, excess_init   E by the plate's origin: the start's, a rift's, a
                   subduction initiation's (the life records' `origin`)
  excess_guard, excess_free   E of plates whose rotation the speed guard holds, and of the rest
  share_guard      the guard-held plates' share of the sum of perimeters
  excess_spin      E of spinning plates (pole within the record's spin angle of the centroid)
  excess_cont, excess_ocean   E of plates with a continental share of 0.3 or more, and of the rest
  excess_slow, excess_fast    E of plates under 20 mm/yr at the centroid, and of the rest
  front_per_pcap   each subducting plate's front over its P_cap, the area-weighted mean over plates with a
                   front (Bird: SUB by subducting plate over its P_cap, 0.194 area-weighted)

Version 1.
"""
import json
import math
import sys
from pathlib import Path

R = 6371.0
LATTICE = 1.043  # ledger: PLT-530 (one spacing per boundary sample over the straight length)
SKIP_MYR = 100.0
MIN_SAMPLES = 10
SMALL_SR, LARGE_SR = 0.0377, 0.5
OO = 0  # dynamics::mix class ocean_ocean


def cap_perimeter(area_km2):
    cos_t = max(-1.0, min(1.0, 1.0 - area_km2 / (2.0 * math.pi * R * R)))
    return 2.0 * math.pi * R * math.sin(math.acos(cos_t))


class Sum:
    """Sums of perimeter and P_cap for a group of plate-steps."""

    def __init__(self):
        self.p = 0.0
        self.c = 0.0

    def add(self, p, c):
        self.p += p
        self.c += c

    def e(self):
        return self.p / self.c if self.c > 0 else None


def main():
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    rec = json.loads((Path(sys.argv[1]) / "rec.json").read_text())
    d = rec.get("dynamics", rec)
    steps = d["steps"]
    t0 = d.get("t0_myr", steps[0]["t"])
    born = {p["id"]: p.get("born_myr", t0) for p in d.get("plates", [])}
    origin = {p["id"]: p.get("origin", "initial") for p in d.get("plates", [])}
    use = [s for s in steps if s["t"] > t0 + SKIP_MYR]
    n = max(len(use), 1)
    groups = {k: Sum() for k in ("all", "small", "mid", "large", "young", "mature", "old", "initial", "rift",
                                 "init", "guard", "free", "spin", "cont", "ocean", "slow", "fast")}
    windows = [Sum() for _ in range(9)]
    wb = [[0.0, 0.0, 0.0, 0] for _ in range(9)]  # boundary, front, plates, steps
    ratios, weights = [], []
    front = {"small": 0.0, "mid": 0.0, "large": 0.0}
    fp_num, fp_w = 0.0, 0.0
    plates_n, oo = 0, 0.0
    for s in use:
        vis_total = max(float(s.get("visible", 0)), 1.0)
        a_sample = 4.0 * math.pi * R * R / vis_total
        k = min(8, max(0, int((s["t"] - t0 - SKIP_MYR) // 100.0)))
        oo += s.get("mix_km", [0.0])[OO] / LATTICE
        for row in s["plates"]:
            pid, vis, cont, speed, fkm, bkm, _, flags = row
            pid, vis, flags = int(pid), int(vis), int(flags)
            if vis < MIN_SAMPLES:
                continue
            area = vis * a_sample
            sr = area / (R * R)
            p = bkm / LATTICE
            c = cap_perimeter(area)
            f = fkm / LATTICE
            plates_n += 1
            size = "small" if sr < SMALL_SR else ("large" if sr >= LARGE_SR else "mid")
            age = s["t"] - born.get(pid, t0)
            agek = "young" if age < 50.0 else ("old" if age >= 200.0 else "mature")
            o = origin.get(pid, "initial")
            ok = {"initial": "initial", "rift": "rift"}.get(o, "init")
            for g in ("all", size, agek, ok, "guard" if flags & 2 else "free",
                      "cont" if cont >= 0.3 * vis else "ocean", "slow" if speed < 20.0 else "fast"):
                groups[g].add(p, c)
            if flags & 8:
                groups["spin"].add(p, c)
            windows[k].add(p, c)
            wb[k][0] += p / 2.0
            wb[k][1] += f
            wb[k][2] += 1
            ratios.append(p / c)
            weights.append(area)
            front[size] += f
            if f > 0.0:
                fp_num += area * f / c
                fp_w += area
        wb[k][3] += 1
    out = {}
    if not ratios:
        print(json.dumps(out))
        return
    order = sorted(range(len(ratios)), key=lambda i: ratios[i])
    out["plates"] = plates_n / n
    out["bnd_km"] = groups["all"].p / 2.0 / n
    out["pcap_sum_km"] = groups["all"].c / n
    out["excess"] = groups["all"].e()
    out["ppcap_p50"] = ratios[order[len(order) // 2]]
    out["ppcap_aw"] = sum(r * w for r, w in zip(ratios, weights)) / sum(weights)
    for g in ("small", "mid", "large"):
        out["excess_" + g] = groups[g].e()
        out["bnd_" + g] = groups[g].p / groups["all"].p
    ftot = sum(front.values())
    out["front_km"] = ftot / n
    for g in ("small", "mid", "large"):
        out["front_" + g] = front[g] / ftot if ftot > 0 else None
    out["oo_front_km"] = oo / n
    for i, w in enumerate(windows):
        out[f"excess_t{i + 1}"] = w.e()
    for i in (0, 8):
        b, f, pl, ns = wb[i]
        if ns:
            out[f"bnd_t{i + 1}"] = b / ns
            out[f"front_t{i + 1}"] = f / ns
            out[f"plates_t{i + 1}"] = pl / ns
    for g in ("young", "mature", "old", "initial", "rift", "init", "guard", "free", "spin", "cont", "ocean",
              "slow", "fast"):
        out["excess_" + g] = groups[g].e()
    out["share_guard"] = groups["guard"].p / groups["all"].p
    out["front_per_pcap"] = fp_num / fp_w if fp_w > 0 else None
    print(json.dumps({k: (round(v, 5) if isinstance(v, float) else v) for k, v in out.items()}))


if __name__ == "__main__":
    main()
