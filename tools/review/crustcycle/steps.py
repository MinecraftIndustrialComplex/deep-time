#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The sediment's mass balance step by step (no simulation): steps.py <summary.txt> [--from T] [--to T] [--every N]

Reads a run's `cb_step` lines (the crust budget with DEEPTIME_CRUST_BUDGET_STEPS set; plates/crust_budget.rs
`end_step`): each plate step's sediment and sedimentary rock (`pool::SEDIMENT` + `pool::SEDIMENTARY`) as four stocks,
km^3 Earth-normalized,

  tr  on the visible ocean floor at the trenches (the TRENCH flag: the incoming plate at its front this step)
  ov  on all the visible ocean floor (tr is part of it)
  oh  on the hidden ocean floor (floor gone under, carried until a resampling)
  c   on continental crust, visible and hidden

with each stock's change by the step's parts (the gap budget's brackets: erosion is the surface stage's whole feedback,
subduction the fronts' flags and motion, resample_blend a resampling, ...), and the step's change of the globals the
resampling books: the sediment reaching the trenches (sed_trench, before the share) and the sediment subducted
(sed_subducted, PLT-440/PLT-660/PLT-762), the crust accreted, the one-way slab, the forearcs eroded.

The books' own check: on a resampling step the sediment that leaves the planet is sed_subducted (the share of what
reached a trench, and the one-way slab's), so the step's change of all four stocks under resample_blend plus
sed_subducted is what the resampling's blend made or lost (interpolation onto the new lattice, the dropped crust): the
"blend residual". Sediment kept in two places, or a pile counted twice, would show as a residual that grows with the
pile; the blend's own error does not grow with it.

Prints the per-step rows over [--from, --to] (every --every-th step) and, over the whole run, each stock's change by
part as km^3 a year, the resampling steps' sed_subducted and blend residual, and the trench stock's mean.
"""
import re
import sys

PART_RE = re.compile(r"(\w+):([-+0-9.eE]+)")


def parse(path):
    rows = []
    for line in open(path):
        if not line.startswith("cb_step "):
            continue
        t = float(re.search(r"t=([-0-9.]+)", line).group(1))
        st = dict(PART_RE.findall(re.search(r"stock=\[([^\]]*)\]", line).group(1).replace("=", ":")))
        row = {"t": t, "stock": {k: float(v) for k, v in st.items()}}
        for name in ("d_tr", "d_ov", "d_oh", "d_c", "g"):
            m = re.search(rf"\b{name}=\[([^\]]*)\]", line)
            row[name] = {k: float(v) for k, v in PART_RE.findall(m.group(1))} if m else {}
        rows.append(row)
    return rows


def main(argv):
    path = argv[0]
    lo, hi, every = -1e9, 1e9, 1
    i = 1
    while i < len(argv):
        if argv[i] == "--from":
            lo = float(argv[i + 1])
        elif argv[i] == "--to":
            hi = float(argv[i + 1])
        elif argv[i] == "--every":
            every = int(argv[i + 1])
        i += 2
    rows = parse(path)
    if len(rows) < 2:
        print("no cb_step lines (DEEPTIME_CRUST_BUDGET_STEPS unset?)")
        return
    dt_myr = rows[1]["t"] - rows[0]["t"]
    yr = dt_myr * 1.0e6
    print(f"# {path}: {len(rows)} steps of {dt_myr:g} Myr; km^3 per step unless marked")
    print(f"{'t':>8} {'tr':>10} {'ov':>10} {'oh':>10} {'c':>10} | {'tr:eros':>9} {'tr:subd':>9} {'tr:resm':>9}"
          f" {'tr:rest':>9} | {'all:eros':>9} {'all:resm':>9} {'all:rest':>9} | {'reach':>9} {'subd':>9} {'resid':>9}")
    for k, r in enumerate(rows):
        if not (lo <= r["t"] <= hi) or k % every:
            continue
        tr = r["d_tr"]
        eros_tr, subd_tr, resm_tr = tr.get("erosion", 0.0), tr.get("subduction", 0.0), tr.get("resample_blend", 0.0)
        rest_tr = sum(tr.values()) - eros_tr - subd_tr - resm_tr
        allp = {}
        for name in ("d_ov", "d_oh", "d_c"):
            for p, v in r[name].items():
                allp[p] = allp.get(p, 0.0) + v
        eros, resm = allp.get("erosion", 0.0), allp.get("resample_blend", 0.0)
        rest = sum(allp.values()) - eros - resm
        g = r["g"]
        resid = resm + g.get("sed_subducted", 0.0) if resm != 0.0 else 0.0
        s = r["stock"]
        print(f"{r['t']:8.1f} {s['tr']:10.4e} {s['ov']:10.4e} {s['oh']:10.4e} {s['c']:10.4e} | {eros_tr:9.2e}"
              f" {subd_tr:9.2e} {resm_tr:9.2e} {rest_tr:9.2e} | {eros:9.2e} {resm:9.2e} {rest:9.2e} |"
              f" {g.get('sed_trench', 0.0):9.2e} {g.get('sed_subducted', 0.0):9.2e} {resid:9.2e}")
    # The whole run (the first step's globals are totals so far: left out).
    body = rows[1:]
    n = len(body)
    print(f"\n# over the run ({n} steps after the first), km^3 a year, by part")
    for name in ("d_tr", "d_ov", "d_oh", "d_c"):
        tot = {}
        for r in body:
            for p, v in r[name].items():
                tot[p] = tot.get(p, 0.0) + v
        line = ", ".join(f"{p} {v / (n * yr):+.3f}" for p, v in sorted(tot.items(), key=lambda x: -abs(x[1])) if abs(v) > 0)
        print(f"  {name}: sum {sum(tot.values()) / (n * yr):+.3f} = {line}")
    res = [r for r in body if any("resample_blend" in r[nm] for nm in ("d_ov", "d_oh", "d_c"))]
    subd = sum(r["g"].get("sed_subducted", 0.0) for r in body)
    subd_res = sum(r["g"].get("sed_subducted", 0.0) for r in res)
    blend = sum(sum(r[nm].get("resample_blend", 0.0) for nm in ("d_ov", "d_oh", "d_c")) for r in res)
    reach = sum(r["g"].get("sed_trench", 0.0) for r in body)
    print(f"  resampling steps {len(res)}; sediment reaching the trenches {reach / (n * yr):.3f} km^3/yr, subducted"
          f" {subd / (n * yr):.3f} (on resampling steps {subd_res / (n * yr):.3f}); the four stocks' change under"
          f" resample_blend {blend / (n * yr):+.3f}; blend residual (that + subducted) {(blend + subd_res) / (n * yr):+.4f}"
          f" km^3/yr, {(blend + subd_res) / max(subd_res, 1e-9):+.4f} of the subducted")
    side = lambda names: sum(sum(r[nm].get("resample_blend", 0.0) for nm in names) for r in res)
    ocean, cont = side(("d_ov", "d_oh")), side(("d_c",))
    print(f"  of the residual: the ocean floor's side (hidden and visible floor under resample_blend + subducted)"
          f" {(ocean + subd_res) / (n * yr):+.4f}, the continental crust's {cont / (n * yr):+.4f} km^3/yr (crust a"
          f" collision consumed, crust dropped under another plate, the accreted share landing on continents, the"
          f" blend)")
    trs = [r["stock"]["tr"] for r in body]
    ovs = [r["stock"]["ov"] for r in body]
    print(f"  trench stock mean {sum(trs) / n:.4e} km^3 ({sum(trs) / max(sum(ovs), 1e-9):.4f} of the visible floor's"
          f" sediment); the visible floor's mean {sum(ovs) / n:.4e}")


if __name__ == "__main__":
    main(sys.argv[1:])
