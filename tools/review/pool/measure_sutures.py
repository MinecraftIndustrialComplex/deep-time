#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `sutures` measure (V1 item 1, the docking lane, 2026-10-23): the collision record
counted as sutures, what the docked crust was, and which collisions hold the front, from one pool
run folder's dynamics record, printed as a JSON object on stdout.

    python3 -I measure_sutures.py <run folder>

Over the run after its first 100 Myr, as `measure_collcoast.py`.

Why (ledger PLT-512). `collcoast` counts collision states: a state begins when continental crust
of the subducting plate stands at a front and ends in a hand-off (a transfer or weld) or a parting
(`shortening.rs`). A suture of Macdonald et al. (2019) is one mapped polyline, the trace of one
closed ocean between an arc and a margin (median 1,019 km long), active over its exhumation
interval. In the model one contact carries a train of states: a hand-off at 100 km of shortening
(PLT-068) is followed by a new state the next step on the same contact, so one suture is counted
2.7 times on average. A suture here is the states on one contact (its two plates and its onset)
whose gaps are at most `shortening::GRACE_MYR` (6 Myr, PLT-225: the model's own time for a
front to come back), with at least one hand-off; its length is its longest front over 1.84 (the
lattice boundary's excess over a geodesic one, `measure_boundary.py`).

  states_per100        hand-off states per 100 Myr (collcoast's hand-off count, all sizes)
  sutures_per100       sutures per 100 Myr (Macdonald et al. 2019: 104 in 540 Myr, 19 per 100 Myr,
                       39 in the last 100 Myr; counted from their data by exhumation start)
  sutures_ge50_per100  sutures that docked 50 samples or more (about 255,000 km2)
  suture_km_per100     their length per 100 Myr, geodesic km (Macdonald: 23,235 over the
                       Phanerozoic, 50,919 over 0 to 100 Ma)
  suture_km_med        a suture's median length, geodesic km (Macdonald: 1,019)
  suture_myr_med       a suture's median duration, Myr
  states_per_suture    the states one suture holds, mean
  parting_eps_per100   episodes on a contact with no hand-off (partings only), per 100 Myr

Which collisions hold the front: each state's (duration + a step) x longest front, as a share of
all states' (an upper bound per state):
  ft_small, ft_mid, ft_large   transfers of terranes under 50, 50 to 1000, 1000 or more samples
  ft_weld, ft_parting          welds and partings (open at the end included)

What docked (transfers, not welds, by samples), from the steps' per-plate continental counts:
the subducting plate's continental crust at the hand-off is split into what it had at birth
(`endowed`), what it received in transfers and welds (`received`) and what it made itself, net of
what it handed out and its rift children took (`made`: arcs matured, thick floor made continental,
crust growth in place); a terrane is of the kind holding half or more of that supply, else `mixed`.
  made, received, endowed, mixed     shares of the transferred samples
  origin_rift, origin_initiation, origin_initial
                                     the subducting plate's origin, shares of the samples

The collisions under way at the end (`open` partings):
  open_front_km        their longest fronts, summed, geodesic km
  open_old60           the share of that front on contacts older than 60 Myr
  open_conv_med        their median peak convergence, mm/yr

Plain Python. Version 1.
"""
import json
import statistics as st
import sys
from collections import defaultdict
from pathlib import Path

LATTICE = 1.84  # the lattice boundary over a geodesic one (measure_boundary.py)
GRACE_MYR = 6.0  # shortening::GRACE_MYR, ledger PLT-225


def key(c):
    return (c["under"], c["over"], round(c["contact_onset_myr"], 3))


def sutures(rec, t0, span):
    by = defaultdict(list)
    for c in rec["collisions"]:
        if c["end_myr"] >= t0:
            by[key(c)].append((c["start_myr"], c["end_myr"], c["front_km"], 1, c["terrane_samples"]))
    for p in rec["partings"]:
        if p["end_myr"] >= t0:
            by[key(p)].append((p["start_myr"], p["end_myr"], p["front_km"], 0, 0))
    dt = rec["dt_myr"]
    eps = []
    for v in by.values():
        v.sort()
        cur = None
        for s, e, f, h, ts in v:
            if cur is None or s - cur["e"] > GRACE_MYR + 1e-6:
                if cur is not None:
                    eps.append(cur)
                cur = {"s": s, "e": e, "f": f, "h": h, "ts": ts, "n": 1}
            else:
                cur["e"] = max(cur["e"], e)
                cur["f"] = max(cur["f"], f)
                cur["h"] += h
                cur["ts"] += ts
                cur["n"] += 1
        if cur is not None:
            eps.append(cur)
    he = [x for x in eps if x["h"] > 0]
    out = {
        "states_per100": 100.0 * sum(len([x for x in v if x[3]]) for v in by.values()) / span,
        "sutures_per100": 100.0 * len(he) / span,
        "sutures_ge50_per100": 100.0 * sum(1 for x in he if x["ts"] >= 50) / span,
        "suture_km_per100": 100.0 * sum(x["f"] for x in he) / LATTICE / span,
        "suture_km_med": st.median([x["f"] / LATTICE for x in he]) if he else None,
        "suture_myr_med": st.median([x["e"] - x["s"] + dt for x in he]) if he else None,
        "states_per_suture": st.mean([x["n"] for x in he]) if he else None,
        "parting_eps_per100": 100.0 * sum(1 for x in eps if x["h"] == 0) / span,
    }
    return out


def front_time(rec, t0, t_end):
    dt = rec["dt_myr"]
    ft = defaultdict(float)

    def held(c):
        return (min(c["end_myr"], t_end) - max(c["start_myr"], t0) + dt) * c["front_km"]

    for c in rec["collisions"]:
        if c["end_myr"] < t0:
            continue
        if c["sutured"]:
            k = "ft_weld"
        elif c["terrane_samples"] < 50:
            k = "ft_small"
        elif c["terrane_samples"] < 1000:
            k = "ft_mid"
        else:
            k = "ft_large"
        ft[k] += held(c)
    for p in rec["partings"]:
        if p["end_myr"] >= t0:
            ft["ft_parting"] += held(p)
    tot = sum(ft.values()) or 1e-9
    return {k: ft[k] / tot for k in ("ft_small", "ft_mid", "ft_large", "ft_weld", "ft_parting")}


def provenance(rec, t0):
    """Per plate, from the steps' continental counts: endowment at birth, received, made in place."""
    steps = rec["steps"]
    tix = {round(s["t"], 6): i for i, s in enumerate(steps)}
    cont = defaultdict(dict)
    for i, s in enumerate(steps):
        for p in s["plates"]:
            cont[int(p[0])][i] = p[2]
    ins, outs, rift_out = defaultdict(float), defaultdict(float), defaultdict(float)
    for c in rec["collisions"]:
        i = tix.get(round(c["end_myr"], 6))
        if i is None:
            continue
        if c["sutured"]:
            died = c.get("died")
            surv = c["over"] if died == c["under"] else c["under"]
            if died is not None and i - 1 in cont[died]:
                ins[(surv, i)] += cont[died][i - 1]
        else:
            ins[(c["over"], i)] += c["terrane_samples"]
            outs[(c["under"], i)] += c["terrane_samples"]
    life_of = defaultdict(list)
    for L in rec["plates"]:
        life_of[L["id"]].append(L)
        i = tix.get(round(L["born_myr"], 6))
        if i is not None and L.get("parent") is not None:
            rift_out[(L["parent"], i)] += cont[L["id"]].get(i, 0)
    led = {}
    for pid, series in cont.items():
        idx = sorted(series)
        if not idx:
            continue
        e_, r_, g_ = float(series[idx[0]]), 0.0, 0.0
        hist, prev = {}, None
        for i in idx:
            if prev is not None and i == prev + 1:
                g = series[i] - series[prev] - ins[(pid, i)] + outs[(pid, i)] + rift_out[(pid, i)]
                r_ += ins[(pid, i)]
                g_ += max(g, 0.0)
            elif prev is not None:
                e_ += series[i]
            hist[i] = (e_, r_, g_)
            prev = i
        led[pid] = hist
    kinds, origins = defaultdict(float), defaultdict(float)
    for c in rec["collisions"]:
        if c["end_myr"] < t0 or c["sutured"]:
            continue
        i = tix.get(round(c["end_myr"], 6))
        h = led.get(c["under"], {}).get(i - 1) if i is not None else None
        if h is None:
            continue
        e_, r_, g_ = h
        supply = max(e_ + r_ + g_, 1e-9)
        k = "made" if g_ / supply >= 0.5 else "received" if r_ / supply >= 0.5 else "endowed" if e_ / supply >= 0.5 else "mixed"
        w = float(c["terrane_samples"])
        kinds[k] += w
        life = [x for x in life_of[c["under"]] if abs(x["born_myr"] - c["under_born_myr"]) < 1e-6]
        o = life[0].get("origin") if life else None
        o = {"rift": "origin_rift", "subduction_initiation": "origin_initiation", "initial": "origin_initial"}.get(o)
        if o:
            origins[o] += w
    tk, to = sum(kinds.values()) or 1e-9, sum(origins.values()) or 1e-9
    out = {k: kinds[k] / tk for k in ("made", "received", "endowed", "mixed")}
    out.update({k: origins[k] / to for k in ("origin_rift", "origin_initiation", "origin_initial")})
    return out


def open_states(rec, t_end):
    opens = [p for p in rec["partings"] if p.get("reason") == "open"]
    tot = sum(p["front_km"] for p in opens) / LATTICE
    old = sum(p["front_km"] for p in opens if t_end - p["contact_onset_myr"] >= 60.0) / LATTICE
    conv = [p["peak_convergence_mm_yr"] for p in opens]
    return {
        "open_front_km": tot,
        "open_old60": old / tot if tot > 0 else None,
        "open_conv_med": st.median(conv) if conv else None,
    }


def main():
    run = Path(sys.argv[1])
    rec = json.loads((run / "rec.json").read_text())
    t_end = rec["steps"][-1]["t"]
    t0 = rec["t0_myr"] + 100.0
    span = max(t_end - t0, 1e-9)
    out = sutures(rec, t0, span)
    out.update(front_time(rec, t0, t_end))
    out.update(provenance(rec, t0))
    out.update(open_states(rec, t_end))
    print(json.dumps({k: (None if v is None else float(v)) for k, v in out.items()}, sort_keys=True))


if __name__ == "__main__":
    main()
