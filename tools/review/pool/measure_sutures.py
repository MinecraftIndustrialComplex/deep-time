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
about twice. A suture here is the states on one contact (its two plates and its onset)
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
  ride_myr_med       how long the docked crust rode the subducting plate: the hand-off's time less
                     its supply's mean time of arrival (birth, receipt, making), median by samples
  under_age_myr_med  the subducting plate's age at the hand-off, median by samples
  ft30_made, ft30_received, ft30_endowed, ft30_mixed, ft30_weld, ft30_parting
                     the last 30 Myr's front-time (`collision_orogeny_myr`: the collisions whose
                     belts the final fields still mark as collision) by the transfer's kind, the
                     welds and the partings: each kind's share of what holds the collision coast

The collisions under way at the end (`open` partings):
  open_front_km        their longest fronts, summed, geodesic km
  open_old60           the share of that front on contacts older than 60 Myr (the contact's age
                       counts its subduction before the collision)
  open_train_myr_med   how long the collisions open at the end have gone on (the train of states
                       on the contact with gaps of at most the grace), median by front
  open_conv_med        their median peak convergence, mm/yr

Plain Python. Version 2.
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


def ledger(rec):
    """Per plate and step, from the steps' continental counts: the samples endowed at birth,
    received and made in place so far, and each part's mean time of arrival."""
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
    for L in rec["plates"]:
        i = tix.get(round(L["born_myr"], 6))
        if i is not None and L.get("parent") is not None:
            rift_out[(L["parent"], i)] += cont[L["id"]].get(i, 0)
    led = {}
    for pid, series in cont.items():
        idx = sorted(series)
        if not idx:
            continue
        t0 = steps[idx[0]]["t"]
        e_, r_, g_ = float(series[idx[0]]), 0.0, 0.0
        et, rt, gt = e_ * t0, 0.0, 0.0
        hist, prev = {}, None
        for i in idx:
            t = steps[i]["t"]
            if prev is not None and i == prev + 1:
                g = max(series[i] - series[prev] - ins[(pid, i)] + outs[(pid, i)] + rift_out[(pid, i)], 0.0)
                r_ += ins[(pid, i)]
                rt += ins[(pid, i)] * t
                g_ += g
                gt += g * t
            elif prev is not None:
                e_ += series[i]
                et += series[i] * t
            hist[i] = (e_, r_, g_, et + rt + gt)
            prev = i
        led[pid] = hist
    return tix, led


def kind_of(h):
    e_, r_, g_, _ = h
    supply = max(e_ + r_ + g_, 1e-9)
    if g_ / supply >= 0.5:
        return "made"
    if r_ / supply >= 0.5:
        return "received"
    if e_ / supply >= 0.5:
        return "endowed"
    return "mixed"


def wmedian(pairs):
    pairs = sorted(p for p in pairs if p[1] > 0)
    tot = sum(w for _v, w in pairs)
    if tot <= 0:
        return None
    acc = 0.0
    for v, w in pairs:
        acc += w
        if acc >= 0.5 * tot:
            return v
    return pairs[-1][0]


def provenance(rec, t0, t_end):
    """What docked: each transfer's kind (the subducting plate's supply at the hand-off), the
    plate's origin, the crust's ride (the hand-off's time less the supply's mean arrival) and the
    plate's age; and the last 30 Myr's front-time by the kind of collision."""
    tix, led = ledger(rec)
    life_of = defaultdict(list)
    for L in rec["plates"]:
        life_of[L["id"]].append(L)
    dt = rec["dt_myr"]
    kinds, origins, ride, age = defaultdict(float), defaultdict(float), [], []
    recent = defaultdict(float)
    for c in rec["collisions"]:
        if c["end_myr"] < t0:
            continue
        held = (min(c["end_myr"], t_end) - max(c["start_myr"], t_end - 30.0) + dt) * c["front_km"]
        if c["sutured"]:
            if c["end_myr"] >= t_end - 30.0:
                recent["weld"] += held
            continue
        i = tix.get(round(c["end_myr"], 6))
        h = led.get(c["under"], {}).get(i - 1) if i is not None else None
        if h is None:
            continue
        k = kind_of(h)
        w = float(c["terrane_samples"])
        kinds[k] += w
        if c["end_myr"] >= t_end - 30.0:
            recent[k] += held
        supply = max(h[0] + h[1] + h[2], 1e-9)
        ride.append((c["end_myr"] - h[3] / supply, w))
        age.append((c["end_myr"] - c["under_born_myr"], w))
        life = [x for x in life_of[c["under"]] if abs(x["born_myr"] - c["under_born_myr"]) < 1e-6]
        o = life[0].get("origin") if life else None
        o = {"rift": "origin_rift", "subduction_initiation": "origin_initiation", "initial": "origin_initial"}.get(o)
        if o:
            origins[o] += w
    for p in rec["partings"]:
        if p["end_myr"] >= t_end - 30.0:
            recent["parting"] += (min(p["end_myr"], t_end) - max(p["start_myr"], t_end - 30.0) + dt) * p["front_km"]
    tk, to, tr = sum(kinds.values()) or 1e-9, sum(origins.values()) or 1e-9, sum(recent.values()) or 1e-9
    out = {k: kinds[k] / tk for k in ("made", "received", "endowed", "mixed")}
    out.update({k: origins[k] / to for k in ("origin_rift", "origin_initiation", "origin_initial")})
    out["ride_myr_med"] = wmedian(ride)
    out["under_age_myr_med"] = wmedian(age)
    out.update({f"ft30_{k}": recent[k] / tr for k in ("made", "received", "endowed", "mixed", "weld", "parting")})
    return out


def open_states(rec, t_end):
    by = defaultdict(list)
    for c in rec["collisions"]:
        by[key(c)].append((c["start_myr"], c["end_myr"]))
    for q in rec["partings"]:
        by[key(q)].append((q["start_myr"], q["end_myr"]))
    opens = [q for q in rec["partings"] if q.get("reason") == "open"]
    train = []
    for q in opens:
        # the start of the train of states on this contact that the open state ends
        s0 = q["start_myr"]
        for s, e in sorted(by[key(q)], reverse=True):
            if e <= s0 + 1e-6 and s0 - e <= GRACE_MYR + 1e-6:
                s0 = min(s0, s)
        train.append((t_end - s0, q["front_km"]))
    tot = sum(q["front_km"] for q in opens) / LATTICE
    old = sum(q["front_km"] for q in opens if t_end - q["contact_onset_myr"] >= 60.0) / LATTICE
    conv = [q["peak_convergence_mm_yr"] for q in opens]
    return {
        "open_front_km": tot,
        "open_old60": old / tot if tot > 0 else None,
        "open_train_myr_med": wmedian(train),
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
    out.update(provenance(rec, t0, t_end))
    out.update(open_states(rec, t_end))
    print(json.dumps({k: (None if v is None else float(v)) for k, v in out.items()}, sort_keys=True))


if __name__ == "__main__":
    main()
