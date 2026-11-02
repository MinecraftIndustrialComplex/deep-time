#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Whether the model's subduction fronts stay on the rock of their overriding plates (no simulation):
frontstay.py <run folder>[@<from>:<to>] ...

(`@<from>:<to>`, Myr: only the cycles that lie between.)

Reads a run's crust-budget maps in a close snapshot window (`+DEEPTIME_SNAPSHOT_MYR=<every>:<from>:<to>`,
`plates::SNAPSHOT_ENV`) with the records' sample indices (maps/id/, written since 2026-11-01). A sample keeps its
index and rides its plate between resamplings (a resampling renumbers the lattice), so two snapshots of one
resampling cycle pair their records by index; two snapshots are of one cycle when at least 0.95 of the indices both
hold sit on the same plate in both. A snapshot taken at a resampling has no front flags and starts a new cycle.

For each front sample at t (the overriding side of a subducting front, the map flag 256, as oofronts.py takes them)
and each later snapshot t + lag of the same cycle: the share of the front still a front on the same rock (the same
sample, or a sample of its own plate within 1.5 spacings of it at t: a front's jitter of a sample across). By class
(intra-oceanic: no continental crust of its own plate within 200 km, PLT-370). What became of the rest: the sample
hidden, on another plate, or visible on its plate and no longer a front; and how far the front moved on its plate's
rock: the nearest front sample of the same plate at t + lag, taken back to t by its index, its distance from the old
front sample, km. A front fixed in its plate (PLT-710: Heuret & Lallemand 2005's Vd = 0) stays; one that moves
across its overriding plate spreads its arc's magma thin (forecrust.py's arc stock).

Then why a front ends, at a lag of one snapshot (the cadence, 2 Myr): each plate's rotation over each interval of
the cycle is fitted to its own samples paired by index (Kabsch, the plates being rigid between resamplings), and at
each front sample the subducting plate (the plate of the nearest sample of another plate) moves towards the
overriding one at v_n across the boundary (the normal: the chord from the own plate's samples within 2.5 spacings to
the other plate's, as `shortening::boundary_normal` takes it, PLT-310). The front at t + lag was classed with the
motion that brought the plates there (`dynamics::boundary_mix`), so each front sample's motion state is read with
the interval before t (as its own class was) and the interval after t (as the next class is): converging at the slab
floor or faster (10 mm/yr, PLT-682) and more than 20 degrees off the boundary's strike (PLT-311), slow, transform, or
diverging. A front that ends on rock still converging fast ended for another reason than its plates' motion. Each
front that ends is also read against its pair of plates one snapshot on: the sample on a collision front (the map
flag 512: the subducting side's crust continental), the pair's front gone everywhere (no subducting-front sample of
the overriding plate has the subducting plate across: the contact ended, or its slab broke off or is incipient
again), the pair's polarity reversed (fronts of the other plate over this one and none of this one over it), or the
pair's front still elsewhere.
"""
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

FRONT = 256
R_KM = 6371.0
ARC_KM = 200.0  # ledger: PLT-042 (the arc's distance, the class's line, PLT-370)
FIELDS = 11


def chord(km):
    return 2.0 * np.sin(km / R_KM / 2.0)


def arc_km(c):
    return 2.0 * np.arcsin(np.clip(c / 2.0, 0.0, 1.0)) * R_KM


def name_of(t):
    return f"t{t:04d}.bin" if t >= 0 else f"t-{-t:03d}.bin"


def load(run, t):
    nm = name_of(t)
    m, pf, idf = run / "maps" / nm, run / "maps" / "plate" / nm, run / "maps" / "id" / nm
    if not (m.exists() and pf.exists() and idf.exists()):
        return None
    a = np.fromfile(m, "<f4").reshape(-1, FIELDS).astype(np.float64)
    plate = np.fromfile(pf, "<u4").astype(np.int64)
    ids = np.fromfile(idf, "<u4").astype(np.int64)
    if not (len(a) == len(plate) == len(ids)):
        return None
    la, lo = np.radians(a[:, 0]), np.radians(a[:, 1])
    pos = np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)
    row = np.full(int(ids.max()) + 1, -1, np.int64)
    row[ids] = np.arange(len(ids))
    front = (a[:, 10].astype(np.int64) & FRONT) > 0
    out = {"t": t, "a": a, "plate": plate, "ids": ids, "pos": pos, "row": row, "front": front}
    af = run / "maps" / "age" / nm
    if af.exists():
        age = np.fromfile(af, "<f4").astype(np.float64)
        if len(age) == len(a):
            out["age"] = age
    return out


def row_of(s, ids):
    out = np.full(len(ids), -1, np.int64)
    ok = ids < len(s["row"])
    out[ok] = s["row"][ids[ok]]
    return out


def same_cycle(s0, s1):
    r1 = row_of(s1, s0["ids"])
    both = r1 >= 0
    if both.sum() < 0.5 * len(s0["ids"]):
        return False
    return (s0["plate"][both] == s1["plate"][r1[both]]).mean() >= 0.95


def times_of(run):
    out = []
    for f in (run / "maps").glob("t*.bin"):
        out.append(int(f.stem[1:]))
    return sorted(out)


def cycles(run):
    """Runs of consecutive close snapshots of one resampling cycle, each with fronts."""
    ts = times_of(run)
    out, cur, prev = [], [], None
    for t in ts:
        s = load(run, t)
        if s is None or not s["front"].any():
            if len(cur) > 1:
                out.append(cur)
            cur, prev = [], None
            continue
        if prev is not None and (t - prev["t"] > 10 or not same_cycle(prev, s)):
            if len(cur) > 1:
                out.append(cur)
            cur = []
        cur.append(s)
        prev = s
    if len(cur) > 1:
        out.append(cur)
    return out


def read_pair(s0, s1, spacing_km):
    idx = np.nonzero(s0["front"])[0]
    pos0, plate0 = s0["pos"], s0["plate"]
    cont0 = s0["a"][:, 4] > 0.5
    # The class at t (PLT-370): no continental crust of its own plate within the arc's distance.
    io = np.ones(len(idx), bool)
    for q in np.unique(plate0[idx]):
        m = np.nonzero(cont0 & (plate0 == q))[0]
        if not len(m):
            continue
        sel = plate0[idx] == q
        d, _ = cKDTree(pos0[m]).query(pos0[idx[sel]])
        io[sel] = arc_km(d) >= ARC_KM
    r1 = row_of(s1, s0["ids"][idx])
    hidden = r1 < 0
    other = ~hidden & (s1["plate"][np.maximum(r1, 0)] != plate0[idx])
    # Same rock: the sample or a neighbour of its plate within 1.5 spacings at t, a front at t + lag.
    tree0 = cKDTree(pos0)
    near = tree0.query_ball_point(pos0[idx], chord(1.5 * spacing_km))
    stays = np.zeros(len(idx), bool)
    for j, (i, ks) in enumerate(zip(idx, near)):
        ks = [k for k in ks if plate0[k] == plate0[i]]
        rr = row_of(s1, s0["ids"][ks])
        rr = rr[rr >= 0]
        stays[j] = bool(len(rr)) and bool((s1["front"][rr] & (s1["plate"][rr] == plate0[i])).any())
    gone = ~stays & ~hidden & ~other
    # The front's move on its plate's rock: the nearest front sample of the same plate at t + lag, back at t.
    move = np.full(len(idx), np.nan)
    f1 = np.nonzero(s1["front"])[0]
    for q in np.unique(plate0[idx]):
        m1 = f1[s1["plate"][f1] == q]
        if not len(m1):
            continue
        sel = np.nonzero(plate0[idx] == q)[0]
        # The plate's front at t + lag, taken back to t by index (only those visible at t).
        back = row_of(s0, s1["ids"][m1])
        back = back[back >= 0]
        back = back[plate0[back] == q]
        if not len(back):
            continue
        d, _ = cKDTree(pos0[back]).query(pos0[idx[sel]])
        move[sel] = arc_km(d)
    return io, stays, hidden, other, gone, move


FLOOR_MM_YR = 10.0  # ledger: PLT-682 (the slab's thermal floor)
TRANSFORM_DEG = 20.0  # ledger: PLT-311 (Bird's transforms)
STATES = ["fast", "slow", "transform", "diverging"]


def rotations(s0, s1):
    """Each plate's rotation vector from s0 to s1 (rad over the interval), fitted to its samples paired by index."""
    r1 = row_of(s1, s0["ids"])
    ok = r1 >= 0
    out = {}
    for q in np.unique(s0["plate"]):
        m = ok & (s0["plate"] == q)
        m[m] = s1["plate"][r1[m]] == q
        if m.sum() < 10:
            continue
        a, b = s0["pos"][m], s1["pos"][r1[m]]
        u, _, vt = np.linalg.svd(b.T @ a)
        d = np.sign(np.linalg.det(u @ vt))
        rot = u @ np.diag([1.0, 1.0, d]) @ vt
        ang = np.arccos(np.clip((np.trace(rot) - 1.0) / 2.0, -1.0, 1.0))
        axis = np.array([rot[2, 1] - rot[1, 2], rot[0, 2] - rot[2, 0], rot[1, 0] - rot[0, 1]])
        nrm = np.linalg.norm(axis)
        out[q] = axis / nrm * ang if nrm > 0 else np.zeros(3)
    return out


def motion_state(s0, idx, under, normal, rot, dt):
    """Each front sample's state under the rotations `rot` over `dt` Myr: an index into STATES, -1 unknown."""
    out = np.full(len(idx), -1)
    for j, i in enumerate(idx):
        q, p = s0["plate"][i], under[j]
        if p < 0 or q not in rot or p not in rot:
            continue
        x = s0["pos"][i]
        rel = np.cross(rot[p] - rot[q], x) * R_KM / dt  # km/Myr = mm/yr
        speed = np.linalg.norm(rel)
        across = rel @ normal[j]
        if speed <= 0.0 or abs(across) < np.sin(np.radians(TRANSFORM_DEG)) * speed:
            out[j] = 2
        elif across > 0.0:
            out[j] = 3
        else:
            out[j] = 0 if -across >= FLOOR_MM_YR else 1
    return out


def fronts_geometry(s0, idx, spacing_km):
    """The subducting plate across each front sample and the boundary's normal towards it."""
    pos, plate = s0["pos"], s0["plate"]
    tree = cKDTree(pos)
    under = np.full(len(idx), -1)
    normal = np.zeros((len(idx), 3))
    _, near = tree.query(pos[idx], k=12)
    around = tree.query_ball_point(pos[idx], chord(2.5 * spacing_km))
    for j, (i, ks, ball) in enumerate(zip(idx, near, around)):
        other = [k for k in ks if plate[k] != plate[i]]
        if not other:
            continue
        p = plate[other[0]]
        own = [k for k in ball if plate[k] == plate[i]]
        them = [k for k in ball if plate[k] == p]
        if not own or not them:
            continue
        x = pos[i]
        v = pos[them].mean(0) - pos[own].mean(0)
        v -= (v @ x) * x
        if np.linalg.norm(v) > 0:
            under[j] = p
            normal[j] = v / np.linalg.norm(v)
    return under, normal


def pair_fronts(s, spacing_km):
    """The (overriding, subducting) plate pairs holding subducting-front samples in snapshot `s`."""
    idx = np.nonzero(s["front"])[0]
    under, _ = fronts_geometry(s, idx, spacing_km)
    return {(int(s["plate"][i]), int(u)) for i, u in zip(idx, under) if u >= 0}


def why(c, spacing_km, acc):
    """Front samples at each middle snapshot of the cycle: their state before and after, by what became of them."""
    rots = [rotations(a, b) for a, b in zip(c, c[1:])]
    for k in range(1, len(c) - 1):
        s_prev, s0, s1 = c[k - 1], c[k], c[k + 1]
        idx = np.nonzero(s0["front"])[0]
        if not len(idx):
            continue
        io, stays, hidden, other, gone, _ = read_pair(s0, s1, spacing_km)
        under, normal = fronts_geometry(s0, idx, spacing_km)
        before = motion_state(s0, idx, under, normal, rots[k - 1], s0["t"] - s_prev["t"])
        after = motion_state(s0, idx, under, normal, rots[k], s1["t"] - s0["t"])
        tree0 = cKDTree(s0["pos"])
        pairs1 = pair_fronts(s1, spacing_km)
        # The subducting plate's nearest visible sample one snapshot on, in spacings (PLT-582's row is 2).
        row = np.full(len(idx), np.nan)
        rr = row_of(s1, s0["ids"][idx])
        for p in np.unique(under[under >= 0]):
            m1 = np.nonzero(s1["plate"] == p)[0]
            sel = np.nonzero((under == p) & (rr >= 0))[0]
            if not len(m1) or not len(sel):
                continue
            d, _ = cKDTree(s1["pos"][m1]).query(s1["pos"][rr[sel]])
            row[sel] = arc_km(d) / spacing_km
        r1 = row_of(s1, s0["ids"][idx])
        coll = np.zeros(len(idx), bool)
        ok = r1 >= 0
        coll[ok] = (s1["a"][r1[ok], 10].astype(np.int64) & 512) > 0
        pair = np.full(len(idx), 4)
        for j, i in enumerate(idx):
            q, p = int(s0["plate"][i]), int(under[j])
            if p < 0:
                continue
            if coll[j]:
                pair[j] = 0
            elif (q, p) in pairs1:
                pair[j] = 3
            elif (p, q) in pairs1:
                pair[j] = 2
            else:
                pair[j] = 1
        for cls, sel in (("intra-oceanic", io), ("continental", ~io)):
            for fate, f in (("stays", stays), ("gone", gone)):
                e = acc.setdefault((cls, fate), np.zeros((5, 5)))
                for b0, a0 in zip(before[sel & f], after[sel & f]):
                    e[b0, a0] += 1
                g = acc.setdefault((cls, fate, "pair"), np.zeros((5, 5)))
                for p0, a0 in zip(pair[sel & f], after[sel & f]):
                    g[p0, a0] += 1
                # Of those converging fast with the pair's front elsewhere: the row's distance.
                h = acc.setdefault((cls, fate, "row"), [])
                h.extend(row[sel & f & (after == 0) & (pair == 3)].tolist())
                # What lies in front of the front sample at t along the boundary's normal, 1 to 5 spacings out:
                # the subducting plate's crust (its nearest visible sample within a spacing), a gap (none of any
                # plate), another plate; and the subducting plate's floor age at the front.
                probe = acc.setdefault((cls, fate, "probe"), np.zeros((5, 3)))
                ages = acc.setdefault((cls, fate, "age"), [])
                for j in np.nonzero(sel & f & (after == 0) & (pair == 3))[0]:
                    x, nv = s0["pos"][idx[j]], normal[j]
                    for st in range(5):
                        ang = (st + 1) * spacing_km / R_KM
                        y = x * np.cos(ang) + nv * np.sin(ang)
                        d, kk = tree0.query(y)
                        if arc_km(d) > spacing_km:
                            probe[st, 1] += 1
                        elif s0["plate"][kk] == under[j]:
                            probe[st, 0] += 1
                        else:
                            probe[st, 2] += 1
                    d, kk = tree0.query(x, k=12)
                    across = [k2 for k2 in kk if s0["plate"][k2] == under[j]]
                    if across and "age" in s0:
                        ages.append(float(s0["age"][across[0]]))
                # And what became of the subducting plate's samples within the row at t: hidden, on another
                # plate or visible one snapshot on; the hidden ones' distance from the front sample, where
                # their plate's rotation over the interval takes them, spacings.
                fates = acc.setdefault((cls, fate, "fates"), {"hidden": 0, "other": 0, "visible": 0, "d": []})
                ball = cKDTree(s0["pos"]).query_ball_point(s0["pos"][idx], chord(2.0 * spacing_km))
                for j in np.nonzero(sel & f & (after == 0) & (pair == 3))[0]:
                    pj = under[j]
                    ks = np.array([k for k in ball[j] if s0["plate"][k] == pj], np.int64)
                    if not len(ks) or pj not in rots[k] or rr[j] < 0:
                        continue
                    r = row_of(s1, s0["ids"][ks])
                    hid = r < 0
                    fates["hidden"] += int(hid.sum())
                    fates["other"] += int((~hid & (s1["plate"][np.maximum(r, 0)] != pj)).sum())
                    fates["visible"] += int((~hid & (s1["plate"][np.maximum(r, 0)] == pj)).sum())
                    w = rots[k][pj]
                    ang = np.linalg.norm(w)
                    if ang > 0 and hid.any():
                        ax = w / ang
                        v = s0["pos"][ks[hid]]
                        moved = (v * np.cos(ang) + np.cross(ax, v) * np.sin(ang)
                                 + np.outer(v @ ax, ax) * (1 - np.cos(ang)))
                        dd = np.linalg.norm(moved - s1["pos"][rr[j]], axis=1)
                        fates["d"].extend((arc_km(dd) / spacing_km).tolist())


def main(argv):
    for arg in argv:
        folder, _, span = arg.partition("@")
        run = Path(folder)
        cyc = cycles(run)
        if span:
            lo, hi = (float(x) for x in span.split(":"))
            cyc = [c for c in cyc if c[0]["t"] >= lo and c[-1]["t"] <= hi]
        print(f"== {arg}: {len(cyc)} cycles of close snapshots "
              + ", ".join(f"{c[0]['t']}..{c[-1]['t']}" for c in cyc))
        if not cyc:
            continue
        n = max(int(c[0]["ids"].max()) + 1 for c in cyc)
        spacing_km = np.sqrt(4.0 * np.pi * R_KM ** 2 / n)
        acc = {}
        for c in cyc:
            for i0, s0 in enumerate(c):
                for s1 in c[i0 + 1:]:
                    lag = s1["t"] - s0["t"]
                    io, stays, hidden, other, gone, move = read_pair(s0, s1, spacing_km)
                    for cls, sel in (("intra-oceanic", io), ("continental", ~io)):
                        e = acc.setdefault((cls, lag), {"n": 0, "stays": 0, "hidden": 0, "other": 0, "gone": 0,
                                                         "move": []})
                        e["n"] += int(sel.sum())
                        e["stays"] += int(stays[sel].sum())
                        e["hidden"] += int(hidden[sel].sum())
                        e["other"] += int(other[sel].sum())
                        e["gone"] += int(gone[sel].sum())
                        mv = move[sel & ~stays]
                        e["move"].extend(mv[np.isfinite(mv)].tolist())
        causes = {}
        for c in cyc:
            if len(c) >= 3:
                why(c, spacing_km, causes)
        print(f"  spacing {spacing_km:.1f} km; shares of the front at t, by lag (Myr)")
        print(f"  {'class':<14} {'lag':>4} {'front':>7} {'stays':>6} {'hidden':>7} {'other':>6} {'gone':>6} "
              f"{'move p25 p50 p75 (km, of those not staying)':>45}")
        for (cls, lag), e in sorted(acc.items()):
            if not e["n"]:
                continue
            q = np.percentile(e["move"], [25, 50, 75]) if e["move"] else [np.nan] * 3
            print(f"  {cls:<14} {lag:>4} {e['n']:>7} {e['stays'] / e['n']:>6.3f} {e['hidden'] / e['n']:>7.3f} "
                  f"{e['other'] / e['n']:>6.3f} {e['gone'] / e['n']:>6.3f} {q[0]:>15.0f} {q[1]:>6.0f} {q[2]:>6.0f}")
        print("  why a front ends, one snapshot on: the subducting plate's motion at the front sample, in the interval"
              " before t (its own class) and after (the next class), shares of each fate")
        for key, h in sorted(causes.items()):
            if len(key) != 3 or key[2] != "row":
                continue
            h = np.array([x for x in h if np.isfinite(x)])
            if len(h):
                print(f"  {key[0]:<14} {key[1]:<6} converging fast, pair's front elsewhere ({len(h)}): the subducting"
                      f" plate's nearest visible sample one snapshot on, spacings: p25 {np.percentile(h, 25):.2f}"
                      f" p50 {np.median(h):.2f} p75 {np.percentile(h, 75):.2f}; beyond 2 (PLT-582's row) {(h > 2).mean():.3f}")
        for key, pr in sorted(causes.items()):
            if len(key) != 3 or key[2] != "probe":
                continue
            tot = pr[0].sum()
            if tot:
                print(f"  {key[0]:<14} {key[1]:<6} in front at t, 1 to 5 spacings out (the subducting plate / a gap /"
                      " another plate): " + "  ".join(f"{k + 1}: {pr[k, 0] / tot:.2f}/{pr[k, 1] / tot:.2f}/"
                                                    f"{pr[k, 2] / tot:.2f}" for k in range(5)))
            ag = np.array(causes.get((key[0], key[1], "age"), []))
            ag = ag[np.isfinite(ag)]
            if len(ag):
                print(f"  {key[0]:<14} {key[1]:<6} the subducting floor's age at the front, Myr: p25"
                      f" {np.percentile(ag, 25):.0f} p50 {np.median(ag):.0f} p75 {np.percentile(ag, 75):.0f}")
        for key, fz in sorted(causes.items()):
            if len(key) != 3 or key[2] != "fates":
                continue
            tot = fz["hidden"] + fz["other"] + fz["visible"]
            if tot:
                d = np.array(fz["d"])
                print(f"  {key[0]:<14} {key[1]:<6} the subducting plate's samples within the row at t ({tot}), one"
                      f" snapshot on: hidden {fz['hidden'] / tot:.3f}, other plate {fz['other'] / tot:.3f}, visible"
                      f" {fz['visible'] / tot:.3f}; the hidden ones from the front sample, spacings: p25"
                      f" {np.percentile(d, 25):.2f} p50 {np.median(d):.2f} p75 {np.percentile(d, 75):.2f}" if len(d) else "")
        for key, e in sorted(causes.items()):
            if len(key) != 2:
                continue
            cls, fate = key
            tot = e.sum()
            if not tot:
                continue
            after = e[:4].sum(0)[:4] / tot
            before = e.sum(1)[:4] / tot
            known = e[:4, :4].sum() / tot
            print(f"  {cls:<14} {fate:<6} {int(tot):>6}  before: "
                  + " ".join(f"{n} {x:.3f}" for n, x in zip(STATES, before))
                  + "  after: " + " ".join(f"{n} {x:.3f}" for n, x in zip(STATES, after))
                  + f"  (known {known:.2f})")
        print("  the pair one snapshot on (rows), by the motion after (columns: " + ", ".join(STATES) + ", unknown),"
              " shares of each fate")
        names = ["collision", "pair's front gone", "pair reversed", "pair's front elsewhere", "unknown"]
        for key, g in sorted(causes.items()):
            if len(key) != 3 or key[2] != "pair":
                continue
            tot = g.sum()
            if not tot:
                continue
            print(f"  {key[0]:<14} {key[1]:<6} {int(tot):>6}")
            for r, name in enumerate(names):
                if g[r].sum():
                    print(f"      {name:<24} {g[r].sum() / tot:>6.3f}  " + " ".join(f"{x / tot:.3f}" for x in g[r]))


if __name__ == "__main__":
    main(sys.argv[1:])
