#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Where the pairs' relative poles lie against their common boundaries, model and Earth (no simulation):

    polenear.py [--chord <spacings>] [--earth <PB2002_steps.dat>] <arm folder>[@N] ...

A pair of plates whose relative rotation pole lies on or near their common boundary spreads along one stretch of it and
converges along another (the relative velocity is a rotation about that pole: its component across the boundary
changes sign where the boundary passes the pole). slabfloor.py found the model's intra-oceanic fronts beside their own
pair's spreading far more often than Earth's (earth_ridge.py: Bird 2003's SUB steps within 1,000 km of the same pair's
OSR or CRB steps); this asks whether the model's relative poles lie near the boundaries more often than Earth's, or
whether the pairs' boundaries wrap round one plate (a plate inside another: its leading edge converges and its
trailing edge spreads, whatever the pole).

The model, each run's end (the only time the record holds the plates' rotations: rec.json `prev_omega`, rad/yr, each
plate's rotation vector after the last step) with the final map (maps/t0000.bin positions, maps/plate/t0000.bin plates;
the last resampling's lattice). A boundary sample is a visible sample with a sample of another plate among its 7
nearest; at each, toward each such plate, the boundary's normal is the tangent part of the chord from the sample's own
plate's samples to the other plate's samples within 2.5 spacings (--chord; as shortening::boundary_normal takes a chord of about
five spacings, PLT-310: one lattice edge is no azimuth), the relative velocity is the other plate's less its own
(omega x p, Earth-normalized: 6,370 km, normalise.rs), and the sample stands for half a spacing of the pair's boundary
(dynamics::classify_boundary's unit).

Earth, Bird 2003's PB2002_steps.dat (5,819 steps; Table 2: the relative velocity's speed and azimuth at each step's
midpoint, and its component across the step). Each pair's relative rotation vector is fitted to its steps' velocity
vectors by least squares (Bird computed them from one pole per pair, so the fit is exact to the file's rounding; the
script prints the worst residual), the step's midpoint stands for its length.

Both are classed as Bird classes them (paragraph [127], PLT-311): divergent where the component across the boundary is
more than sin 20 degrees of the speed, convergent where less than minus that, strike-slip between. For each pair: its
boundary length, the great-circle distance from its relative pole (either end of the axis) to the nearest point of its
boundary, and its divergent and convergent lengths. Prints, length-weighted, the boundary by its pair's pole distance,
the convergent boundary by the pole distance and by whether its pair also diverges (at least MIXED_KM of each), the
convergent boundary within 1,000 km of its own pair's divergent boundary (the share slabfloor.py and earth_ridge.py
read on the fronts), and the mixed pairs split by pole distance (a near pole against a wrapped boundary).
"""
import json
import math
import sys
from pathlib import Path

import numpy as np
from scipy.spatial import cKDTree

sys.path.insert(0, str(Path(__file__).resolve().parent))
import probe  # noqa: E402

R_KM = 6370.0  # tectonics.model_radius_km (normalise.rs)
SIN20 = math.sin(math.radians(20.0))  # ledger: PLT-311 (Bird 2003 [127])
COS20 = math.cos(math.radians(20.0))  # measurement only: the motion within 20 degrees of the boundary's normal
POLE_BANDS = [0.0, 500.0, 1000.0, 2000.0, 4000.0]
MIXED_KM = 400.0
NEAR_KM = 1000.0
LEN_BANDS = [0.0, 1000.0, 2000.0, 4000.0, 8000.0]


def band_of(x):
    k = 0
    for i, e in enumerate(POLE_BANDS):
        if x >= e:
            k = i
    return k


def band_names():
    out = []
    for lo, hi in zip(POLE_BANDS, POLE_BANDS[1:] + [math.inf]):
        out.append(f"{lo:g}-{hi:g}" if math.isfinite(hi) else f">{lo:g}")
    return out


def pole_km(points, w):
    n = np.linalg.norm(w)
    if n <= 0.0:
        return math.nan
    c = np.abs(points @ (w / n))
    return float(np.arccos(np.clip(c.max(), -1.0, 1.0)) * R_KM)


def classes(vn, speed):
    div = vn > SIN20 * speed
    conv = vn < -SIN20 * speed
    return div, conv


def pair_rows(points_of, snap):
    """points_of: {pair: (points (n,3), length km (n), div bool (n), conv bool (n), w (3), |across| / speed (n))}
    -> rows."""
    rows = []
    for key, (pts, ln, div, conv, w, ratio) in points_of.items():
        head_on = ratio > COS20
        d = pole_km(pts, w)
        conv_near = np.zeros(len(pts), bool)
        if div.any() and conv.any():
            t = cKDTree(pts[div])
            dist, _ = t.query(pts[conv])
            conv_near[np.nonzero(conv)[0]] = 2.0 * np.arcsin(np.clip(dist / 2.0, 0.0, 1.0)) * R_KM < NEAR_KM
        rows.append({
            "pair": key,
            "snap": snap,
            "len": float(ln.sum()),
            "div": float(ln[div].sum()),
            "conv": float(ln[conv].sum()),
            "conv_near": float(ln[conv_near].sum()),
            "div_head_on": float(ln[div & head_on].sum()),
            "conv_head_on": float(ln[conv & head_on].sum()),
            "pole_km": d,
        })
    return rows


def model_run(run, chord):
    rec = json.loads((run / "rec.json").read_text())
    omega = {int(i): np.array(w, float) for i, w in rec["prev_omega"]}
    a, _ = probe.read(run, 0)
    plate = np.fromfile(run / "maps" / "plate" / probe.name_of(0), "<u4").astype(np.int64)
    if len(plate) != len(a):
        return None
    la, lo = np.radians(a[:, 0]), np.radians(a[:, 1])
    p = np.stack([np.cos(la) * np.cos(lo), np.cos(la) * np.sin(lo), np.sin(la)], 1)
    n = len(p)
    spacing = math.sqrt(4.0 * math.pi / n)
    tree = cKDTree(p)
    _, nb = tree.query(p, k=7)
    acc = {}
    missing = 0
    for i in range(n):
        pa = plate[i]
        others = {int(plate[j]) for j in nb[i] if plate[j] != pa}
        if not others:
            continue
        near = tree.query_ball_point(p[i], chord * spacing)
        near = np.array(near)
        own = near[plate[near] == pa]
        for pb in others:
            if pa not in omega or pb not in omega:
                missing += 1
                continue
            far = near[plate[near] == pb]
            across = p[far].mean(0) - p[own].mean(0)
            across -= p[i] * p[i].dot(across)
            nrm = np.linalg.norm(across)
            if nrm <= 0.0:
                continue
            normal = across / nrm
            w = omega[pb] - omega[pa]
            v = np.cross(w, p[i]) * R_KM * 1.0e6
            key = (min(pa, pb), max(pa, pb))
            e = acc.setdefault(key, {"pts": [], "ln": [], "vn": [], "sp": [], "w": None})
            e["pts"].append(p[i])
            e["ln"].append(0.5 * spacing * R_KM)
            e["vn"].append(float(v.dot(normal)))
            e["sp"].append(float(np.linalg.norm(v)))
            e["w"] = omega[key[1]] - omega[key[0]]
    points_of = {}
    for key, e in acc.items():
        vn, sp = np.array(e["vn"]), np.array(e["sp"])
        div, conv = classes(vn, sp)
        points_of[key] = (np.array(e["pts"]), np.array(e["ln"]), div, conv, e["w"],
                          np.abs(vn) / np.maximum(sp, 1e-12))
    # A check of the frame: the record's own mean velocity across each pair's boundary after the last step
    # (dynamics::PairBound::normal_mm_yr, negative converging, zero on transform edges) against this one's mean.
    check = []
    for b in rec.get("prev_pair_bound", []):
        e = acc.get((int(b["lo"]), int(b["hi"])))
        if e and abs(b["normal_mm_yr"]) > 2.0:
            check.append((b["normal_mm_yr"], float(np.mean(e["vn"]))))
    # How the two plates are related (rec.json's plate lives): one split off the other (parent and child), both
    # off one parent (siblings), or neither; and the younger plate's origin.
    lives = {int(q["id"]): q for q in rec.get("plates", [])}
    rows = pair_rows(points_of, run.name)
    for r in rows:
        lo, hi = r["pair"]
        a, b = lives.get(lo, {}), lives.get(hi, {})
        if a.get("parent") == hi or b.get("parent") == lo:
            kin = "parent and child"
        elif a.get("parent") is not None and a.get("parent") == b.get("parent"):
            kin = "siblings"
        else:
            kin = "other"
        young = max((a, b), key=lambda q: q.get("born_myr", -1e9))
        r["kin"] = kin
        r["origin"] = young.get("origin", "?")
        r["young_born"] = young.get("born_myr", math.nan)
    return rows, missing, check


def unit(lon, lat):
    lo, la = math.radians(lon), math.radians(lat)
    return np.array([math.cos(la) * math.cos(lo), math.cos(la) * math.sin(lo), math.sin(la)])


def read_runs(path):
    """Bird's steps as contiguous runs (a step whose sequence number carries ':' continues the one before)."""
    runs = []
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            t = line.split()
            if len(t) < 15:
                continue
            bid = t[1].lstrip(":")
            lon0, lat0, lon1, lat1 = map(float, t[2:6])
            st = {
                "pair": (bid[0:2], bid[3:5]), "a": unit(lon0, lat0), "b": unit(lon1, lat1),
                "len": float(t[6]), "vel": float(t[8]), "vaz": float(t[9]), "div": float(t[10]),
                "cls": t[14].lstrip(":"),
            }
            if t[1].startswith(":") and runs and runs[-1][-1]["pair"] == st["pair"]:
                runs[-1].append(st)
            else:
                runs.append([st])
    return runs


def along(pts, cum, x):
    """The point at distance x (km) along a polyline (pts, cumulative km), clamped to its ends."""
    x = min(max(x, 0.0), cum[-1])
    k = int(np.searchsorted(cum, x, side="right")) - 1
    k = min(max(k, 0), len(pts) - 2)
    seg = cum[k + 1] - cum[k]
    f = 0.0 if seg <= 0.0 else (x - cum[k]) / seg
    q = pts[k] * (1.0 - f) + pts[k + 1] * f
    return q / np.linalg.norm(q)


def earth_steps(path, half_km):
    """Bird's steps by pair, each (midpoint, length km, speed mm/yr, across component mm/yr, velocity vector, Bird's
    class): half_km 0 takes the step's own azimuth for its normal (Bird's classes); otherwise the chord of its run from
    half_km before the step's midpoint to half_km after (clamped to the run's ends), as the model's normal is a chord
    of about five spacings. Also the sign check: (steps whose own across component agrees with Bird's, steps read)."""
    steps = {}
    sign_ok, sign_n = 0, 0
    for run in read_runs(path):
        pts = [run[0]["a"]] + [s["b"] for s in run]
        cum = np.concatenate([[0.0], np.cumsum([s["len"] for s in run])])
        for k, s in enumerate(run):
            m = s["a"] + s["b"]
            m /= np.linalg.norm(m)
            lat, lon = math.asin(m[2]), math.atan2(m[1], m[0])
            east = np.array([-math.sin(lon), math.cos(lon), 0.0])
            north = np.array([-math.sin(lat) * math.cos(lon), -math.sin(lat) * math.sin(lon), math.cos(lat)])
            az = math.radians(s["vaz"])
            v = s["vel"] * (math.cos(az) * north + math.sin(az) * east)
            mid = 0.5 * (cum[k] + cum[k + 1])
            if half_km > 0.0:
                t = along(pts, cum, mid + half_km) - along(pts, cum, mid - half_km)
            else:
                t = s["b"] - s["a"]
            t = t - m * m.dot(t)
            nt = np.linalg.norm(t)
            # The step's own across component against Bird's (calibrates the sign: the left plate first).
            t0 = s["b"] - s["a"]
            t0 = t0 - m * m.dot(t0)
            own = float(v.dot(np.cross(m, t0 / max(np.linalg.norm(t0), 1e-12))))
            if abs(s["div"]) > 1.0:
                sign_n += 1
                sign_ok += int((own > 0) == (s["div"] > 0))
            vn = float(v.dot(np.cross(m, t / nt))) if nt > 0.0 else 0.0
            # One pair whichever order Bird names it in (20 pairs appear both ways, EU-NA and NA-EU among them): the
            # velocity is the second plate's relative to the first, so a step named the other way turns it round to
            # fit one rotation a pair. Its class does not change: its step runs the other way too.
            pair = tuple(sorted(s["pair"]))
            steps.setdefault(pair, []).append((m, s["len"], s["vel"], vn, v if pair == s["pair"] else -v, s["cls"]))
    return steps, (sign_ok, sign_n)


def earth(path, half_km):
    """Each pair's steps classed as earth_steps reads them, its relative rotation fitted, and its rows."""
    steps, (sign_ok, sign_n) = earth_steps(path, half_km)
    points_of = {}
    worst = 0.0
    for pair, ss in steps.items():
        a = np.zeros((3 * len(ss), 3))
        b = np.zeros(3 * len(ss))
        for k, (m, _, _, _, v, _) in enumerate(ss):
            # v = R (w x m) = -R [m]x w
            mx = np.array([[0.0, -m[2], m[1]], [m[2], 0.0, -m[0]], [-m[1], m[0], 0.0]])
            a[3 * k:3 * k + 3] = -mx * R_KM * 1.0e6
            b[3 * k:3 * k + 3] = v
        w, *_ = np.linalg.lstsq(a, b, rcond=None)
        res = np.abs(a @ w - b).max() if len(ss) else 0.0
        worst = max(worst, float(res))
        pts = np.array([s[0] for s in ss])
        ln = np.array([s[1] for s in ss])
        vel = np.array([s[2] for s in ss])
        div_c = np.array([s[3] for s in ss])
        div, conv = classes(div_c, vel)
        points_of[pair] = (pts, ln, div, conv, w, np.abs(div_c) / np.maximum(vel, 1e-12))
    rows = pair_rows(points_of, "earth")
    bird = {}
    for pair, ss in steps.items():
        for s in ss:
            bird[s[5]] = bird.get(s[5], 0.0) + s[1]
    return rows, worst, bird, (sign_ok, sign_n)


def report(name, rows, nruns):
    tot = sum(r["len"] for r in rows)
    conv = sum(r["conv"] for r in rows)
    div = sum(r["div"] for r in rows)
    print(f"# {name}: {nruns} snapshot(s), {len(rows)} pairs, boundary {tot / nruns:,.0f} km a snapshot "
          f"(divergent {div / max(tot, 1e-9):.3f}, convergent {conv / max(tot, 1e-9):.3f})")
    names = band_names()
    by = [0.0] * len(names)
    cby = [0.0] * len(names)
    pairs_by = [0] * len(names)
    for r in rows:
        if not math.isfinite(r["pole_km"]):
            continue
        k = band_of(r["pole_km"])
        by[k] += r["len"]
        cby[k] += r["conv"]
        pairs_by[k] += 1
    print("  pole distance to the pair's boundary, km: " + "  ".join(
        f"{n} {b / max(tot, 1e-9):.3f} (conv {c / max(conv, 1e-9):.3f}, {p} pairs)"
        for n, b, c, p in zip(names, by, cby, pairs_by)))
    xs = sorted((r["pole_km"], r["len"]) for r in rows if math.isfinite(r["pole_km"]))
    acc, med = 0.0, math.nan
    for x, w in xs:
        acc += w
        if acc >= tot / 2:
            med = x
            break
    xs = sorted((r["pole_km"], r["conv"]) for r in rows if math.isfinite(r["pole_km"]) and r["conv"] > 0)
    acc, cmed = 0.0, math.nan
    for x, w in xs:
        acc += w
        if acc >= conv / 2:
            cmed = x
            break
    print(f"  length-weighted median pole distance: boundary {med:,.0f} km, convergent boundary {cmed:,.0f} km")
    mixed = [r for r in rows if r["div"] >= MIXED_KM and r["conv"] >= MIXED_KM]
    mixed_len = sum(r["len"] for r in mixed)
    mixed_conv = sum(r["conv"] for r in mixed)
    near_conv = sum(r["conv_near"] for r in rows)
    print(f"  pairs with >= {MIXED_KM:g} km both divergent and convergent: {len(mixed)}, their boundary "
          f"{mixed_len / max(tot, 1e-9):.3f} of all, their convergent {mixed_conv / max(conv, 1e-9):.3f} of the "
          f"convergent ({mixed_conv / nruns:,.0f} km a snapshot)")
    print(f"  convergent boundary within {NEAR_KM:g} km of its own pair's divergent boundary: "
          f"{near_conv / max(conv, 1e-9):.3f} ({near_conv / nruns:,.0f} km a snapshot)")
    mb = [0.0] * len(names)
    for r in mixed:
        if math.isfinite(r["pole_km"]):
            mb[band_of(r["pole_km"])] += r["conv"]
    print("  the mixed pairs' convergent boundary by pole distance: " + "  ".join(
        f"{n} {b / max(mixed_conv, 1e-9):.3f}" for n, b in zip(names, mb)))
    snaps = {}
    for r in rows:
        snaps.setdefault(r["snap"], set()).update(r["pair"])
    plates = sum(len(s) for s in snaps.values()) / max(len(snaps), 1)
    print(f"  plates on a boundary {plates:.1f} a snapshot, pairs {len(rows) / nruns:.1f}, boundary a pair "
          f"{tot / max(len(rows), 1):,.0f} km")
    dh = sum(r["div_head_on"] for r in rows)
    ch = sum(r["conv_head_on"] for r in rows)
    print(f"  head-on (the motion within 20 degrees of the boundary's normal): of the divergent "
          f"{dh / max(div, 1e-9):.3f}, of the convergent {ch / max(conv, 1e-9):.3f}")
    # How much of each plate's outline it shares with one neighbour (a plate wrapped by another spreads at its
    # trailing edge and converges at its leading edge whatever the pole).
    outline = {}
    for r in rows:
        for q in r["pair"]:
            outline.setdefault((r["snap"], q), []).append((r["len"], r))
    wrapped, wrapped_mixed, n_pl, top_share = 0, 0, 0, []
    for (_, q), ls in outline.items():
        whole = sum(x for x, _ in ls)
        big, r = max(ls, key=lambda t: t[0])
        top_share.append(big / max(whole, 1e-9))
        n_pl += 1
        if big >= 0.5 * whole:
            wrapped += 1
            wrapped_mixed += int(r["div"] >= MIXED_KM and r["conv"] >= MIXED_KM)
    print(f"  a plate's outline shared with its main neighbour: median {np.median(top_share):.2f}; plates with half "
          f"or more of it on one neighbour {wrapped}/{n_pl}, and that pair mixed for {wrapped_mixed} of them")
    for field in ("kin", "origin"):
        if not any(field in r for r in rows):
            continue
        groups = {}
        for r in rows:
            g = groups.setdefault(r.get(field, "?"), [0.0, 0.0, 0.0, 0.0])
            g[0] += r["len"]
            g[1] += r["len"] if r["div"] >= MIXED_KM and r["conv"] >= MIXED_KM else 0.0
            g[2] += r["conv"]
            g[3] += r["conv_near"]
        print(f"  by the pair's {'relation' if field == 'kin' else 'younger plate origin'} (share of the boundary; "
              "of it mixed; of its convergent near its divergent): " + "  ".join(
                  f"{k} {g[0] / max(tot, 1e-9):.3f} ({g[1] / max(g[0], 1e-9):.2f}, {g[3] / max(g[2], 1e-9):.2f})"
                  for k, g in sorted(groups.items(), key=lambda kv: -kv[1][0])))
    # The longest pairs (the last length band): wrapped (one of the two has half its outline or more on the other) or
    # not, each with its mixed share; and for the model their relation, the younger plate's origin and birth.
    whole = {k: sum(x for x, _ in ls) for k, ls in outline.items()}
    longest = [r for r in rows if r["len"] >= LEN_BANDS[-1]]
    if longest:
        lt = sum(r["len"] for r in longest)

        def mixed_share(rs):
            ln = sum(r["len"] for r in rs)
            return ln / max(lt, 1e-9), sum(r["len"] for r in rs if r["div"] >= MIXED_KM and r["conv"] >= MIXED_KM) / max(
                ln, 1e-9)

        wr = [r for r in longest if max(r["len"] / max(whole[(r["snap"], q)], 1e-9) for q in r["pair"]) >= 0.5]
        nw = [r for r in longest if r not in wr]
        (sw, mw), (sn, mn) = mixed_share(wr), mixed_share(nw)
        line = (f"  pairs over {LEN_BANDS[-1]:g} km ({len(longest)}, {lt / nruns:,.0f} km a snapshot): wrapped {sw:.3f} of "
                f"them (mixed {mw:.2f}), not wrapped {sn:.3f} (mixed {mn:.2f})")
        if any("kin" in r for r in longest):
            for field in ("kin", "origin"):
                groups = {}
                for r in longest:
                    groups.setdefault(r.get(field, "?"), []).append(r)
                line += f"; by {field}: " + ", ".join(
                    f"{k} {mixed_share(g)[0]:.2f} ({mixed_share(g)[1]:.2f})"
                    for k, g in sorted(groups.items(), key=lambda kv: -mixed_share(kv[1])[0]))
            born = [r["young_born"] for r in longest if math.isfinite(r.get("young_born", math.nan))]
            if born:
                line += f"; the younger plate born at a median {np.median(born):,.0f} Myr"
        print(line)
    cells = []
    for lo, hi in zip(LEN_BANDS, LEN_BANDS[1:] + [math.inf]):
        rs = [r for r in rows if lo <= r["len"] < hi]
        ln = sum(r["len"] for r in rs)
        cv = sum(r["conv"] for r in rs)
        mx = sum(r["len"] for r in rs if r["div"] >= MIXED_KM and r["conv"] >= MIXED_KM)
        nr = sum(r["conv_near"] for r in rs)
        label = f"{lo:g}-{hi:g}" if math.isfinite(hi) else f">{lo:g}"
        cells.append(f"{label} {ln / max(tot, 1e-9):.3f} (mixed {mx / max(ln, 1e-9):.2f}, near "
                     f"{nr / max(cv, 1e-9):.2f}, {len(rs)} pairs)")
    print("  by the pair's boundary length, km (share of the boundary; of it on mixed pairs; of its convergent "
          "within 1,000 km of the pair's divergent): " + "  ".join(cells))


def main(argv):
    chord = 2.5
    if len(argv) >= 2 and argv[0] == "--chord":
        chord = float(argv[1])
        argv = argv[2:]
    half_km = chord * math.sqrt(4.0 * math.pi / 16000.0) * R_KM
    if len(argv) >= 2 and argv[0] == "--earth":
        for h, label in ((0.0, "each step's own azimuth (Bird's classes)"),
                         (half_km, f"the chord {half_km:,.0f} km either side (the model's {chord:g} spacings at 16k)")):
            rows, worst, bird, (ok, nsign) = earth(argv[1], h)
            report(f"Earth, Bird 2003 PB2002, {label}", rows, 1)
            if h == 0.0:
                tot = sum(bird.values())
                print(f"  pole fit's worst velocity residual {worst:.2f} mm/yr; the step's own across component has "
                      f"Bird's sign at {ok}/{nsign} steps; Bird's own classes: " + "  ".join(
                          f"{k} {v / tot:.3f}" for k, v in sorted(bird.items())))
        argv = argv[2:]
    for arg in argv:
        rows, nruns, missing, check = [], 0, 0, []
        for run in probe.runs_of(arg):
            try:
                got = model_run(run, chord)
            except (FileNotFoundError, KeyError, ValueError) as e:
                print(f"  {run.name}: {e}")
                continue
            if got is None:
                continue
            r, m, c = got
            rows += r
            missing += m
            check += c
            nruns += 1
        report(f"{Path(arg.split('@')[0]).parent.parent.name} (runs' ends, chord {chord:g} spacings)", rows,
               max(nruns, 1))
        if missing:
            print(f"  boundary samples of plates with no recorded rotation: {missing}")
        if check:
            rec_v, own_v = np.array(check).T
            r = float(np.corrcoef(rec_v, own_v)[0, 1]) if len(check) > 2 else math.nan
            same = float(np.mean(np.sign(rec_v) == np.sign(own_v)))
            print(f"  frame check against the record's mean velocity across each pair (|v| > 2 mm/yr, {len(check)} "
                  f"pairs): same sign {same:.2f}, correlation {r:.2f}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
