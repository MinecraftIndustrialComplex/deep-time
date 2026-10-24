#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""PB2002 (Bird 2003) plate-boundary geometry: divider-method lengths, plate areas and perimeters,
triple-junction pieces, subduction length by plate, totals.

Usage (stdlib only, run with -I):
    python3 -I bird.py <data_dir> <paper_layout_txt> <out_json>

Everything is great-circle geometry on a sphere of radius R = 6371 km. The data files and the paper
text are untrusted input: they are only parsed, never executed.
"""
import json
import math
import os
import re
import sys
from collections import defaultdict

R = 6371.0
FOUR_PI = 4.0 * math.pi
TWO_PI = 2.0 * math.pi
RESOLUTIONS = [("full", None), ("L77", 77.0), ("L380", 380.0)]
LARGE = ["AF", "AN", "AR", "AU", "CA", "CO", "EU", "IN", "JF", "NZ", "NA", "PA", "PS", "SA"]

# Quoted from Bird (2003), Table 3 (page 46 of 52 of the PDF, paragraph [127]); lengths in km.
TABLE3 = {"CCB": 23003, "CTF": 26132, "CRB": 27472, "OSR": 67338, "OTF": 47783, "OCB": 17449,
          "SUB": 51310, "TOTAL": 260487}


# ----------------------------------------------------------------------------- vector helpers
def ll2v(lon, lat):
    lo, la = math.radians(lon), math.radians(lat)
    c = math.cos(la)
    return (c * math.cos(lo), c * math.sin(lo), math.sin(la))


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def cross(a, b):
    return (a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0])


def norm(a):
    return math.sqrt(dot(a, a))


def unit(a):
    n = norm(a)
    return (a[0] / n, a[1] / n, a[2] / n)


def ang(a, b):
    return math.atan2(norm(cross(a, b)), dot(a, b))


def gcd(a, b):
    """great-circle distance in km"""
    return R * ang(a, b)


def dedupe(vs, eps=1e-9):
    out = [vs[0]]
    for v in vs[1:]:
        if ang(out[-1], v) > eps:
            out.append(v)
    return out


# ----------------------------------------------------------------------------- divider (ruler) method
def full_length(P):
    return sum(gcd(P[i], P[i + 1]) for i in range(len(P) - 1))


def divider(P, L):
    """Walk the polyline P (list of unit vectors) with a fixed great-circle chord L km.

    Each divider step ends where the polyline first leaves the circle of radius L around the current
    point. The total is n * L plus the great-circle distance from the last divider point to the end of
    the polyline (the remainder). Returns (total km, number of full divider steps)."""
    th = L / R
    n = len(P)
    if n < 2:
        return 0.0, 0
    c = P[0]
    j = 0
    s0 = 0.0
    steps = 0
    while j < n - 1:
        A, B = P[j], P[j + 1]
        if ang(c, B) >= th:
            lo, hi = s0, 1.0
            D = (B[0] - A[0], B[1] - A[1], B[2] - A[2])
            for _ in range(60):
                mid = 0.5 * (lo + hi)
                q = unit((A[0] + mid * D[0], A[1] + mid * D[1], A[2] + mid * D[2]))
                if ang(c, q) >= th:
                    hi = mid
                else:
                    lo = mid
            s0 = hi
            c = unit((A[0] + s0 * D[0], A[1] + s0 * D[1], A[2] + s0 * D[2]))
            steps += 1
        else:
            j += 1
            s0 = 0.0
    return steps * L + gcd(c, P[-1]), steps


def lengths3(P):
    """full-resolution, L=77 and L=380 lengths of a polyline, plus the numbers of full divider steps"""
    d77, d380 = divider(P, 77.0), divider(P, 380.0)
    return {"full": full_length(P), "L77": d77[0], "L380": d380[0], "n77": d77[1], "n380": d380[1]}


def lengths3_rev(P):
    return lengths3(list(reversed(P)))


# ----------------------------------------------------------------------------- statistics
def pct(xs, p):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    k = (len(xs) - 1) * p / 100.0
    f = int(math.floor(k))
    c = min(f + 1, len(xs) - 1)
    return xs[f] + (xs[c] - xs[f]) * (k - f)


def wmean(xs, ws):
    sw = sum(ws)
    return sum(x * w for x, w in zip(xs, ws)) / sw if sw else float("nan")


def dist_stats(xs, ws=None):
    ws = ws if ws is not None else [1.0] * len(xs)
    return {"n": len(xs), "median": pct(xs, 50), "mean": sum(xs) / len(xs) if xs else float("nan"),
            "wmean": wmean(xs, ws), "p10": pct(xs, 10), "p90": pct(xs, 90),
            "min": min(xs) if xs else float("nan"), "max": max(xs) if xs else float("nan")}


def rnd(o, nd=4):
    if isinstance(o, float):
        return round(o, nd)
    if isinstance(o, dict):
        return {k: rnd(v, nd) for k, v in o.items()}
    if isinstance(o, (list, tuple)):
        return [rnd(v, nd) for v in o]
    return o


# ----------------------------------------------------------------------------- parsing
def read_dig(path):
    segs = []
    title = None
    pts = []
    num = re.compile(r"^\s*[-+]?\d")
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            line = line.rstrip("\n")
            if line.startswith("*** end of line segment"):
                segs.append((title, pts))
                title, pts = None, []
            elif title is None and not num.match(line):
                title = line
            elif num.match(line):
                a, b = line.split(",")
                pts.append((float(a), float(b)))
    return segs


def read_steps(path):
    rows = []
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            if not line.strip():
                continue
            rows.append({
                "seq": int(line[0:4]), "cont": line[5] == ":", "bid": line[6:11],
                "lon0": float(line[12:20]), "lat0": float(line[21:28]),
                "lon1": float(line[29:37]), "lat1": float(line[38:45]),
                "length": float(line[46:51]), "cls": line[92:95], "orogen": line[95:96] == "*",
            })
    return rows


def read_table1(path):
    """Plate id, name, area (sr) from the pdftotext -layout rendering of Table 1."""
    out = {}
    on = False
    rx = re.compile(r"^\s*([A-Z]{2})\s+(.+?)\s+(\d+\.\d{5})(?!\d)")
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            if line.startswith("Table 1. Plate Identifiers"):
                on = True
                continue
            if on and "All poles are expressed" in line:
                break
            if on:
                m = rx.match(line)
                if m:
                    out[m.group(1)] = {"name": m.group(2).strip(), "area_sr": float(m.group(3))}
    return out


# ----------------------------------------------------------------------------- spherical polygon area
def poly_area(V, r):
    """Area (sr) to the left of the closed counterclockwise outline V (V[-1] == V[0]), via the
    signed-triangle fan from reference point r (van Oosterom and Strackee), modulo 4 pi."""
    s = 0.0
    for a, b in zip(V[:-1], V[1:]):
        num = dot(r, cross(a, b))
        den = 1.0 + dot(r, a) + dot(a, b) + dot(b, r)
        s += 2.0 * math.atan2(num, den)
    return s % FOUR_PI


def cap_perimeter(area_sr):
    cos_t = 1.0 - area_sr / TWO_PI
    th = math.acos(max(-1.0, min(1.0, cos_t)))
    return TWO_PI * R * math.sin(th), th


# ----------------------------------------------------------------------------- self tests
def self_tests():
    out = {}
    # (1) a single great-circle arc of 1000 km, dense: divider length equals the arc length
    arc = [ll2v(0.0 + i * (1000.0 / R) * 180.0 / math.pi / 1000.0, 0.0) for i in range(1001)]
    out["arc_1000km"] = {"full": full_length(arc), "L77": divider(arc, 77.0)[0], "L380": divider(arc, 380.0)[0]}
    # (2) a small circle (geodesic radius rho around the north pole), dense, closed
    rho = 600.0 / R
    pts = [ll2v(i * 0.01, 90.0 - math.degrees(rho)) for i in range(36001)]
    out["circle_r600"] = {}
    for L in (77.0, 380.0, 1000.0):
        got, nst = divider(pts, L)
        dlon = math.acos((math.cos(L / R) - math.cos(rho) ** 2) / math.sin(rho) ** 2)
        n = int(math.floor(TWO_PI / dlon))
        rem_ang = TWO_PI - n * dlon
        rem = R * math.acos(math.cos(rho) ** 2 + math.sin(rho) ** 2 * math.cos(rem_ang))
        out["circle_r600"]["L%d" % L] = {"got": got, "expected": n * L + rem, "steps": nst, "expected_steps": n}
    # (3) polygon areas: a cap of colatitude 30 deg around either pole, counterclockwise seen from outside
    cap = 2.0 * math.pi * (1.0 - math.cos(math.radians(30.0)))
    north = [ll2v(i * 1.0, 60.0) for i in range(361)]  # increasing longitude = CCW seen from above the north pole
    south = [ll2v(-i * 1.0, -60.0) for i in range(361)]  # CCW seen from outside above the south pole
    r = unit((0.31, 0.53, 0.79))
    out["areas"] = {"cap_expected_sr": cap, "north_cap": poly_area(north, r), "south_cap": poly_area(south, r),
                    "north_reversed_expected_4pi_minus_cap": FOUR_PI - cap,
                    "north_reversed": poly_area(list(reversed(north)), r)}
    out["cap_perimeter_check"] = {"P_cap_of_30deg_cap_km": cap_perimeter(cap)[0],
                                  "expected_km": TWO_PI * R * math.sin(math.radians(30.0))}
    return out


QUOTES = {
    "citation": "Bird, P. (2003), An updated digital model of plate boundaries, Geochem. Geophys. Geosyst. 4(3), 1027, doi:10.1029/2001GC000252",
    "source_pdf": "http://peterbird.name/publications/2003_pb2002/2001GC000252.pdf (13,772,223 bytes, 52 pages)",
    "abstract": "14 large plates (NUVEL-1A) + 38 small plates = 52 plates",
    "table1": "Plate identifiers, areas (steradian) and Euler poles, PDF p. 6 of 52; 52 areas parsed from the pdftotext rendering",
    "table3_km": TABLE3,
    "table3_pct": {"CCB": 8.8, "CTF": 10.0, "CRB": 10.5, "OSR": 25.8, "OTF": 18.3, "OCB": 6.7, "SUB": 19.7},
    "table3_where": "Table 3 'Collective Properties of Plate Boundaries by Class', PDF p. 46 of 52, next to para [127]",
    "para_8": "ensured that the active plate boundaries meet at triple-junction points that are common to all digitized boundary segments, and replaced non-transform offsets on spreading ridges with idealized transforms",
    "para_111": "52 plates; 100 triple-junction points; no quadruple or higher-order junctions",
    "para_112": "150 plate boundaries, each begins at a junction and ends at a junction; neighbours: Pacific 23, Australia 18, Eurasia 11, Antarctica 10; 6 small plates with only 2 neighbours (Manus, Juan de Fuca, Altiplano, Mariana, Easter, Shetland); doubled pairs listed: AN-SA, AN-SC, AU-PA, NA-PA, NB-SB, NZ-PA, NZ-SA, PA-PS, SC-AN (2 each)",
    "para_119": "adjacent points joined by great-circle arcs; no arc between adjacent points is longer than 111 km; round-off error does not exceed 60 m",
    "para_122": "PB2002_boundaries.dig: 6,048 points in 229 segments; title bytes 1-2 plate on the left (travelling along the segment, looking down from outside the Earth), bytes 4-5 plate on the right; byte 3 '/' = right-hand plate subducts under the left-hand plate, '\\' = opposite polarity, '-' = no subduction",
    "para_123": "PB2002_plates.dig: 52 closed outlines, counterclockwise as seen from outside the Earth, last point = first",
    "para_125": "a digitization step is the short great-circle arc between adjacent digitized plate boundary points; 5,819 steps, mean length 44.7 km, length range 1 km to 109 km",
    "para_126_127": "7 step classes; SUB needs a convergent velocity component plus a Benioff zone and/or Quaternary arc; the other six by oceanic/continental and divergent/strike-slip(+-20 deg)/convergent",
    "para_128": "Earth surface area 5.10e14 m^2; total rate of lithosphere creation 0.10787 m^2/s",
    "para_116_orogens": "13 orogens, total area 0.937 sr (7.5% of the Earth)",
}


NOTES = {
    "geometry": "unit-sphere vectors from (lon, lat); great-circle distances; R = 6371 km; longitudes 0..360 and -180..180 need no special handling in 3D",
    "divider": "ruler walk with a fixed great-circle chord L: from the current point, the next divider point is where the polyline first leaves the circle of radius L (bisection on the segment); total = n*L + great-circle distance from the last divider point to the polyline end (the remainder, counted as a chord); closed outlines start at vertex 0 and end at vertex 0; shorter than L gives the end-to-end chord; closed outlines smaller than L give 0 (reported, but excluded from the group statistics)",
    "area": "signed-triangle fan from a reference point (van Oosterom-Strackee), modulo 4 pi, outlines are counterclockwise as seen from outside; two reference points agree to < 1e-9 sr; sums to 4 pi",
    "junctions": "endpoints of the 229 boundary segments are clustered (single linkage, 2 km); a cluster is a triple junction when the left/right plate ids of its segments name at least 3 distinct plates; clusters with 2 plates and 2 segment ends are mid-boundary breaks (source or SUB/non-SUB changes) and are chained through",
    "pieces": "segments are chained through non-junction nodes into pieces from one junction to the next; a piece's length is the divider walk of the concatenated polyline from one junction to the other (forward); chord = great-circle distance between its two end junctions; tau = length / chord at the same resolution",
    "sub_share": "SUB share = number of steps with class SUB / steps in the piece (the task's rule); the length-share rule gives the same 42 pieces",
    "polarity": "from the title byte 3 of the boundary segment each step belongs to; '/' right plate subducts under left plate; '\\' left plate subducts under right plate; the rule is stated in the readme (item 1) and the paper para [122]; SUB steps all carry a symbol",
    "steps_to_segments": "steps file rows are in boundary-file order; row i of segment k matches point i to i+1 within 0.0011 deg and carries the segment title",
}


def provenance(data_dir, paper_dir):
    import hashlib
    out = {}
    base = "http://peterbird.name/oldFTP/PB2002/"
    for fn in sorted(os.listdir(data_dir)):
        path = os.path.join(data_dir, fn)
        with open(path, "rb") as f:
            b = f.read()
        out[fn] = {"url": base + fn, "bytes": len(b), "sha256": hashlib.sha256(b).hexdigest()}
    pdf = os.path.join(paper_dir, "pdf", "bird2003.pdf")
    if os.path.exists(pdf):
        with open(pdf, "rb") as f:
            b = f.read()
        out["bird2003.pdf"] = {"url": "http://peterbird.name/publications/2003_pb2002/2001GC000252.pdf", "bytes": len(b),
                               "sha256": hashlib.sha256(b).hexdigest()}
    return out


# ----------------------------------------------------------------------------- main
def main(data_dir, paper_txt, out_json):
    res = {}
    res["self_tests"] = self_tests()
    res["provenance"] = provenance(data_dir, os.path.dirname(paper_txt))
    res["paper_quotes"] = QUOTES
    res["notes"] = NOTES
    T1 = read_table1(paper_txt)
    assert len(T1) == 52, "Table 1 parse: %d plates" % len(T1)

    bseg_raw = read_dig(os.path.join(data_dir, "PB2002_boundaries.dig.txt"))
    pseg_raw = read_dig(os.path.join(data_dir, "PB2002_plates.dig.txt"))
    steps = read_steps(os.path.join(data_dir, "PB2002_steps.dat.txt"))
    assert len(bseg_raw) == 229 and len(pseg_raw) == 52 and len(steps) == 5819
    res["counts"] = {"boundary_segments": len(bseg_raw), "boundary_points": sum(len(p) for _, p in bseg_raw),
                     "plate_outlines": len(pseg_raw), "plate_points": sum(len(p) for _, p in pseg_raw),
                     "steps": len(steps)}

    # ---- boundary segments, with the steps that belong to each
    segs = []
    si = 0
    for k, (title, pts) in enumerate(bseg_raw):
        t = title[:5]
        n = len(pts) - 1
        st = steps[si:si + n]
        si += n
        assert len(st) == n
        for i, s in enumerate(st):
            assert s["bid"] == t, (k, t, s["bid"])
            d0 = abs(((s["lon0"] - pts[i][0] + 180.0) % 360.0) - 180.0)
            d1 = abs(((s["lon1"] - pts[i + 1][0] + 180.0) % 360.0) - 180.0)
            assert d0 < 0.0011 and d1 < 0.0011, (k, i, s, pts[i])
            assert abs(s["lat0"] - pts[i][1]) < 0.0011 and abs(s["lat1"] - pts[i + 1][1]) < 0.0011
            assert s["cont"] == (i > 0), (k, i)
        V = [ll2v(*p) for p in pts]
        geo = [gcd(V[i], V[i + 1]) for i in range(n)]
        for s, g in zip(st, geo):
            s["geo"] = g
        segs.append({"i": k, "title": t, "left": t[0:2], "right": t[3:5], "sym": t[2], "V": V, "steps": st,
                     "geo": geo})
    assert si == len(steps)
    res["steps_to_segments"] = "all 5819 steps matched to the 229 segments in file order (id, endpoints within 0.0011 deg, continuity flag)"

    # ============================== A. plates ==============================
    ref1 = None
    plates = {}
    maxref = 0.0
    for title, pts in pseg_raw:
        pid = title.strip()[:2]
        V = dedupe([ll2v(*p) for p in pts])
        assert ang(V[0], V[-1]) < 1e-9, pid  # closed
        m = (sum(v[0] for v in V), sum(v[1] for v in V), sum(v[2] for v in V))
        r1 = unit(m) if norm(m) > 1e-3 * len(V) else (0.0, 0.0, 1.0)
        r2 = unit((0.31, 0.53, 0.79))
        a1 = poly_area(V, r1)
        a2 = poly_area(V, r2)
        maxref = max(maxref, abs(a1 - a2))
        L3 = lengths3(V)
        steps3 = {"L77": L3["n77"], "L380": L3["n380"]}
        plates[pid] = {"V": V, "area_sr": a1, "area_ref2": a2, "len": L3, "nsteps": steps3, "npts": len(V) - 1}
    assert set(plates) == set(T1), set(plates) ^ set(T1)
    rows = {}
    for pid, p in plates.items():
        a = p["area_sr"]
        pc, th = cap_perimeter(a)
        t1 = T1[pid]["area_sr"]
        rows[pid] = {
            "name": T1[pid]["name"], "large": pid in LARGE,
            "area_sr": a, "area_table1_sr": t1, "area_diff_sr": a - t1, "area_rel_diff": (a - t1) / t1,
            "area_km2": a * R * R, "cap_theta_deg": math.degrees(th), "P_cap_km": pc,
            "P_full_km": p["len"]["full"], "P_L77_km": p["len"]["L77"], "P_L380_km": p["len"]["L380"],
            "ratio_full": p["len"]["full"] / pc, "ratio_L77": p["len"]["L77"] / pc,
            "ratio_L380": p["len"]["L380"] / pc,
            "divider_steps_L77": p["nsteps"]["L77"], "divider_steps_L380": p["nsteps"]["L380"],
            "n_points": p["npts"],
        }
    sumA = sum(r["area_sr"] for r in rows.values())
    groups = {"large14": [k for k in rows if rows[k]["large"]],
              "small38": [k for k in rows if not rows[k]["large"]], "all52": list(rows)}
    gstat = {}
    for g, ids in groups.items():
        gs = {"n": len(ids), "area_sr_sum": sum(rows[i]["area_sr"] for i in ids)}
        for rk in ("full", "L77", "L380"):
            for tag, minsteps in (("", 1), ("_resolved4", 4)):
                if rk == "full":
                    ok = list(ids)
                else:
                    ok = [i for i in ids if rows[i]["divider_steps_" + rk] >= minsteps]
                if not ok:
                    continue
                xs = [rows[i]["ratio_" + rk] for i in ok]
                ws = [rows[i]["area_sr"] for i in ok]
                gs["ratio_" + rk + tag] = {"n": len(ok), "median": pct(xs, 50), "area_weighted_mean": wmean(xs, ws),
                                           "mean": sum(xs) / len(xs), "p10": pct(xs, 10), "p90": pct(xs, 90)}
        gs["P_full_km_median"] = pct([rows[i]["P_full_km"] for i in ids], 50)
        gs["fewer_than_4_divider_steps_L77"] = sum(1 for i in ids if rows[i]["divider_steps_L77"] < 4)
        gs["fewer_than_4_divider_steps_L380"] = sum(1 for i in ids if rows[i]["divider_steps_L380"] < 4)
        gstat[g] = gs
    res["A"] = {
        "plates": rows, "groups": gstat,
        "check": {"sum_area_sr": sumA, "four_pi": FOUR_PI, "sum_minus_4pi": sumA - FOUR_PI,
                  "sum_table1_sr": sum(T1[k]["area_sr"] for k in T1),
                  "max_abs_diff_vs_table1_sr": max(abs(r["area_diff_sr"]) for r in rows.values()),
                  "max_abs_rel_diff_vs_table1": max(abs(r["area_rel_diff"]) for r in rows.values()),
                  "max_diff_between_two_reference_points_sr": maxref,
                  "earth_area_km2_at_R6371": FOUR_PI * R * R},
        "perimeter_sum_half": {k: sum(p["len"][k] for p in plates.values()) / 2.0 for k in ("full", "L77", "L380")},
    }

    # ============================== B. junctions and pieces ==============================
    ends = []  # (segment index, 0 start / 1 end, vector)
    for s in segs:
        ends.append((s["i"], 0, s["V"][0]))
        ends.append((s["i"], 1, s["V"][-1]))

    def cluster(tol_km):
        parent = list(range(len(ends)))

        def find(x):
            while parent[x] != x:
                parent[x] = parent[parent[x]]
                x = parent[x]
            return x

        th = tol_km / R
        for a in range(len(ends)):
            for b in range(a + 1, len(ends)):
                if ang(ends[a][2], ends[b][2]) <= th:
                    ra, rb = find(a), find(b)
                    if ra != rb:
                        parent[ra] = rb
        groups_ = defaultdict(list)
        for a in range(len(ends)):
            groups_[find(a)].append(a)
        return list(groups_.values())

    def junction_count(tol_km):
        cl = cluster(tol_km)
        nj = 0
        for idxs in cl:
            pl = set()
            for a in idxs:
                s = segs[ends[a][0]]
                pl.add(s["left"])
                pl.add(s["right"])
            if len(pl) >= 3:
                nj += 1
        return len(cl), nj

    tol_sens = {str(t): junction_count(t) for t in (0.01, 0.1, 1.0, 2.0, 5.0, 10.0)}
    TOL = 2.0
    clusters = cluster(TOL)
    node_of = {}
    nodes = []
    for ni, idxs in enumerate(clusters):
        pl = set()
        for a in idxs:
            node_of[a] = ni
            s = segs[ends[a][0]]
            pl.add(s["left"])
            pl.add(s["right"])
        spread = max([gcd(ends[a][2], ends[b][2]) for a in idxs for b in idxs] + [0.0])
        nodes.append({"idx": idxs, "plates": sorted(pl), "deg": len(idxs), "spread_km": spread,
                      "v": ends[idxs[0]][2], "junction": len(pl) >= 3})
    min_between = min(gcd(nodes[a]["v"], nodes[b]["v"]) for a in range(len(nodes)) for b in range(a + 1, len(nodes)))
    junctions = [n for n in nodes if n["junction"]]
    anomalies = []
    for ni, n in enumerate(nodes):
        if not n["junction"] and n["deg"] != 2:
            anomalies.append({"node": ni, "plates": n["plates"], "deg": n["deg"]})
        if n["junction"] and n["deg"] != 3:
            anomalies.append({"junction_node": ni, "plates": n["plates"], "deg": n["deg"]})
    # segment endpoints near an interior vertex of another segment (a hidden junction would show here)
    hidden = 0
    hidden_detail = []
    for a in range(len(ends)):
        for s in segs:
            if s["i"] == ends[a][0]:
                continue
            for vi, v in enumerate(s["V"][1:-1]):
                if ang(ends[a][2], v) <= 1.0 / R:
                    hidden += 1
                    hidden_detail.append({"endpoint_of": segs[ends[a][0]]["title"], "end": ends[a][1],
                                          "interior_vertex_of": s["title"], "vertex_index": vi + 1,
                                          "dist_km": gcd(ends[a][2], v), "node_plates": nodes[node_of[a]]["plates"],
                                          "node_junction": nodes[node_of[a]]["junction"]})
    # union segments through non-junction nodes
    par = list(range(len(segs)))

    def f2(x):
        while par[x] != x:
            par[x] = par[par[x]]
            x = par[x]
        return x

    for ni, n in enumerate(nodes):
        if not n["junction"] and n["deg"] == 2:
            sa, sb = ends[n["idx"][0]][0], ends[n["idx"][1]][0]
            if sa != sb:
                par[f2(sa)] = f2(sb)
    comps = defaultdict(list)
    for s in segs:
        comps[f2(s["i"])].append(s["i"])

    pieces = []
    for comp in comps.values():
        # node degrees inside the component
        cnt = defaultdict(int)
        for si_ in comp:
            cnt[node_of[2 * si_]] += 1
            cnt[node_of[2 * si_ + 1]] += 1
        endnodes = [nd for nd, c in cnt.items() if c == 1]
        start = endnodes[0] if endnodes else node_of[2 * comp[0]]
        remaining = set(comp)
        cur = start
        P = []
        order = []
        while remaining:
            nxt = None
            for si_ in sorted(remaining):
                if node_of[2 * si_] == cur:
                    nxt = (si_, False)
                    break
                if node_of[2 * si_ + 1] == cur:
                    nxt = (si_, True)
                    break
            if nxt is None:
                anomalies.append({"piece_chain_break": comp})
                break
            si_, rev = nxt
            V = list(reversed(segs[si_]["V"])) if rev else list(segs[si_]["V"])
            P.extend(V if not P else V[1:])
            order.append(si_)
            remaining.discard(si_)
            cur = node_of[2 * si_] if rev else node_of[2 * si_ + 1]
        P = dedupe(P)
        st = [s for si_ in comp for s in segs[si_]["steps"]]
        pairs = {tuple(sorted((segs[si_]["left"], segs[si_]["right"]))) for si_ in comp}
        if len(pairs) != 1:
            anomalies.append({"mixed_pair_piece": sorted(pairs)})
        L3 = lengths3(P)
        L3r = lengths3_rev(P)
        chord = gcd(P[0], P[-1])
        nsub = sum(1 for s in st if s["cls"] == "SUB")
        lsub = sum(s["length"] for s in st if s["cls"] == "SUB")
        ltot = sum(s["length"] for s in st)
        pieces.append({"pair": "-".join(sorted(pairs)[0]), "segments": sorted(comp), "n_segments": len(comp),
                       "start_node": start, "end_node": cur, "len": L3, "len_rev": L3r, "chord_km": chord,
                       "tau": {k: (L3[k] / chord if chord > 0 else float("nan")) for k in ("full", "L77", "L380")},
                       "n_steps": len(st), "sub_share_steps": nsub / len(st), "sub_share_len": lsub / ltot,
                       "closed_loop": not endnodes, "P": P})
    pieces.sort(key=lambda p: (p["pair"], -p["len"]["full"]))
    pairs_count = defaultdict(int)
    for p in pieces:
        pairs_count[p["pair"]] += 1
    neigh = defaultdict(set)
    for p in pieces:
        a, b = p["pair"].split("-")
        neigh[a].add(b)
        neigh[b].add(a)
    deg_hist = defaultdict(int)
    for n in junctions:
        deg_hist[n["deg"]] += 1

    def piece_block(ps):
        out = {"n": len(ps)}
        for k in ("full", "L77", "L380"):
            ls = [p["len"][k] for p in ps]
            taus = [p["tau"][k] for p in ps]
            out[k] = {
                "length_km": dist_stats(ls), "tau": dist_stats(taus, ls),
                "tau_length_weighted_mean": wmean(taus, ls),
                "sum_length_over_sum_chord": sum(ls) / sum(p["chord_km"] for p in ps),
                "total_length_km": sum(ls),
            }
        out["chord_km"] = dist_stats([p["chord_km"] for p in ps])
        for k, nk in (("L77", "n77"), ("L380", "n380")):
            ok = [p for p in ps if p["len"][nk] >= 4]
            out[k + "_resolved4"] = {
                "n": len(ok), "n_shorter_than_one_divider_step": sum(1 for p in ps if p["len"][nk] == 0),
                "tau": dist_stats([p["tau"][k] for p in ok], [p["len"][k] for p in ok]) if ok else None,
                "tau_length_weighted_mean": wmean([p["tau"][k] for p in ok], [p["len"][k] for p in ok]) if ok else None,
                "sum_length_over_sum_chord": sum(p["len"][k] for p in ok) / sum(p["chord_km"] for p in ok) if ok else None,
                "length_share_of_all": sum(p["len"][k] for p in ok) / sum(p["len"][k] for p in ps)}
        return out

    sub_pieces = [p for p in pieces if p["sub_share_steps"] > 0.5]
    non_pieces = [p for p in pieces if p["sub_share_steps"] <= 0.5]
    sub_pieces_len = [p for p in pieces if p["sub_share_len"] > 0.5]
    res["B"] = {
        "tolerance_km": TOL, "tolerance_sensitivity_{clusters,junctions}": tol_sens,
        "n_endpoint_clusters": len(nodes), "n_junctions": len(junctions),
        "junction_degree_hist": dict(deg_hist), "n_pieces": len(pieces),
        "min_distance_between_distinct_nodes_km": min_between,
        "max_intra_node_spread_km": max(n["spread_km"] for n in nodes),
        "hidden_junction_endpoints_on_interior_vertices": hidden, "hidden_detail": hidden_detail,
        "pair_counts": {k: v for k, v in sorted(pairs_count.items()) if v > 1},
        "anomalies": anomalies, "closed_loops": sum(1 for p in pieces if p["closed_loop"]),
        "pairs_with_2_pieces": sorted(k for k, v in pairs_count.items() if v > 1),
        "pairs_max_count": max(pairs_count.values()),
        "neighbours_by_plate": {k: len(v) for k, v in sorted(neigh.items())},
        "all": piece_block(pieces), "sub_dominant_steps_gt50": piece_block(sub_pieces),
        "non_sub": piece_block(non_pieces), "sub_dominant_len_gt50_sensitivity": piece_block(sub_pieces_len),
        "n_sub_pieces_steps": len(sub_pieces), "n_sub_pieces_len": len(sub_pieces_len),
        "pieces": [{"pair": p["pair"], "n_segments": p["n_segments"], "len": p["len"], "chord_km": p["chord_km"],
                    "tau": p["tau"], "n_steps": p["n_steps"], "sub_share_steps": p["sub_share_steps"],
                    "sub_share_len": p["sub_share_len"]} for p in pieces],
    }
    res["B"]["junction_list"] = [{"plates": n["plates"], "deg": n["deg"]} for n in junctions]

    # ============================== C. subduction length by plate ==============================
    sub = defaultdict(lambda: {"subducts": 0.0, "overrides": 0.0, "subducts_geo": 0.0, "overrides_geo": 0.0,
                               "n_subducts": 0, "n_overrides": 0})
    pairs_sub = defaultdict(float)
    xtab = defaultdict(lambda: defaultdict(float))
    xtab_n = defaultdict(lambda: defaultdict(int))
    unknown_pol = 0.0
    for s in segs:
        for st in s["steps"]:
            xtab[s["sym"]][st["cls"]] += st["length"]
            xtab_n[s["sym"]][st["cls"]] += 1
            if st["cls"] != "SUB":
                continue
            if s["sym"] == "/":
                sd, ov = s["right"], s["left"]
            elif s["sym"] == "\\":
                sd, ov = s["left"], s["right"]
            else:
                unknown_pol += st["length"]
                continue
            sub[sd]["subducts"] += st["length"]
            sub[sd]["subducts_geo"] += st["geo"]
            sub[sd]["n_subducts"] += 1
            sub[ov]["overrides"] += st["length"]
            sub[ov]["overrides_geo"] += st["geo"]
            sub[ov]["n_overrides"] += 1
            pairs_sub[sd + ">" + ov] += st["length"]
    total_sub = sum(v["subducts"] for v in sub.values())
    total_sub_geo = sum(v["subducts_geo"] for v in sub.values())
    per_plate = {}
    for pid in rows:
        v = sub.get(pid, {"subducts": 0.0, "overrides": 0.0, "subducts_geo": 0.0, "overrides_geo": 0.0,
                          "n_subducts": 0, "n_overrides": 0})
        akm2 = rows[pid]["area_km2"]
        per_plate[pid] = {
            "name": rows[pid]["name"], "large": rows[pid]["large"], "area_sr": rows[pid]["area_table1_sr"],
            "area_km2_table1": rows[pid]["area_table1_sr"] * R * R,
            "sub_subducting_km": v["subducts"], "sub_overriding_km": v["overrides"],
            "sub_subducting_km_geo": v["subducts_geo"], "sub_overriding_km_geo": v["overrides_geo"],
            "subducting_km_per_Mkm2": v["subducts"] / (rows[pid]["area_table1_sr"] * R * R / 1e6),
            "overriding_km_per_Mkm2": v["overrides"] / (rows[pid]["area_table1_sr"] * R * R / 1e6),
            "subducting_share_of_perimeter": v["subducts"] / rows[pid]["P_full_km"],
            "overriding_share_of_perimeter": v["overrides"] / rows[pid]["P_full_km"],
            "P_full_km": rows[pid]["P_full_km"],
        }
    split = {}
    for kind in ("subducts", "overrides"):
        lg = sum(v[kind] for k, v in sub.items() if k in LARGE)
        sm = sum(v[kind] for k, v in sub.items() if k not in LARGE)
        split["by_%s_plate" % ("subducting" if kind == "subducts" else "overriding")] = {
            "large14_km": lg, "small38_km": sm, "large14_frac": lg / (lg + sm), "small38_frac": sm / (lg + sm)}
    pair2x2 = defaultdict(float)
    for k, v in pairs_sub.items():
        sd, ov = k.split(">")
        pair2x2[("large" if sd in LARGE else "small") + "_subducts_under_" + ("large" if ov in LARGE else "small")] += v
    top5 = sorted(sub.items(), key=lambda kv: -kv[1]["subducts"])[:5]
    top5o = sorted(sub.items(), key=lambda kv: -kv[1]["overrides"])[:5]
    class_len = defaultdict(float)
    class_n = defaultdict(int)
    class_geo = defaultdict(float)
    for s in segs:
        for st in s["steps"]:
            class_len[st["cls"]] += st["length"]
            class_geo[st["cls"]] += st["geo"]
            class_n[st["cls"]] += 1
    res["C"] = {
        "polarity_rule": "title byte 3: '/' = right-hand plate (bytes 4-5) subducts under left-hand plate (bytes 1-2); "
                         "'\\' = left-hand plate subducts under right-hand plate (Bird 2003 para [122], readme item 1)",
        "sub_total_from_steps_file_km": class_len["SUB"], "sub_total_from_geometry_km": class_geo["SUB"],
        "table3_sub_km": TABLE3["SUB"], "sub_total_attributed_km": total_sub,
        "sub_steps_with_unknown_polarity_km": unknown_pol, "n_sub_steps": class_n["SUB"],
        "symbol_by_class_km": {k: dict(v) for k, v in xtab.items()},
        "symbol_by_class_nsteps": {k: dict(v) for k, v in xtab_n.items()},
        "per_plate": per_plate, "split_large_small": split, "pair_matrix_2x2_km": dict(pair2x2),
        "top5_subducting": [{"plate": k, "name": rows[k]["name"], "km": v["subducts"]} for k, v in top5],
        "top5_overriding": [{"plate": k, "name": rows[k]["name"], "km": v["overrides"]} for k, v in top5o],
        "top10_pairs_subducting_to_overriding_km": sorted(((k, v) for k, v in pairs_sub.items()), key=lambda kv: -kv[1])[:10],
    }

    # ============================== D. totals ==============================
    step_lens = [st["length"] for s in segs for st in s["steps"]]
    geo_lens = [st["geo"] for s in segs for st in s["steps"]]
    seg_tot = {k: 0.0 for k in ("full", "L77", "L380")}
    for s in segs:
        l3 = lengths3(dedupe(s["V"]))
        for k in seg_tot:
            seg_tot[k] += l3[k]
    piece_tot = {k: sum(p["len"][k] for p in pieces) for k in ("full", "L77", "L380")}
    piece_tot_rev = {k: sum(p["len_rev"][k] for p in pieces) for k in ("full", "L77", "L380")}
    res["D"] = {
        "table3_total_km": TABLE3["TOTAL"],
        "steps_file_sum_km": sum(step_lens), "steps_geometry_R6371_sum_km": sum(geo_lens),
        "steps_mean_len_km": sum(step_lens) / len(step_lens), "steps_len_stats_km": dist_stats(step_lens),
        "steps_len_stats_geo_km": dist_stats(geo_lens),
        "boundaries_file_segments_sum_km": seg_tot, "pieces_sum_km": piece_tot, "pieces_sum_reverse_walk_km": piece_tot_rev,
        "plates_perimeter_sum_over_2_km": res["A"]["perimeter_sum_half"],
        "by_class_steps_file_km": dict(class_len), "by_class_geometry_km": dict(class_geo),
        "by_class_nsteps": dict(class_n), "by_class_table3_km": {k: v for k, v in TABLE3.items() if k != "TOTAL"},
        "ratio_to_table3_total": {
            "steps_file": sum(step_lens) / TABLE3["TOTAL"],
            "boundaries_full": seg_tot["full"] / TABLE3["TOTAL"],
            "boundaries_L77_segments": seg_tot["L77"] / TABLE3["TOTAL"],
            "boundaries_L380_segments": seg_tot["L380"] / TABLE3["TOTAL"],
            "pieces_L77": piece_tot["L77"] / TABLE3["TOTAL"],
            "pieces_L380": piece_tot["L380"] / TABLE3["TOTAL"],
        },
        "n_segments_by_length_km": dist_stats([sum(s["geo"]) for s in segs]),
    }
    res["B"]["wall_check_total_pieces_full_vs_segments_full_km"] = piece_tot["full"] - seg_tot["full"]
    # steps traced twice (identical end points in two different segments)
    seen = defaultdict(list)
    for s in segs:
        for i, st in enumerate(s["steps"]):
            a = (round(bseg_raw[s["i"]][1][i][0] % 360.0, 4), round(bseg_raw[s["i"]][1][i][1], 4))
            b = (round(bseg_raw[s["i"]][1][i + 1][0] % 360.0, 4), round(bseg_raw[s["i"]][1][i + 1][1], 4))
            seen[frozenset((a, b))].append((s["title"], st["length"], st["cls"]))
    dups = [v for v in seen.values() if len(v) > 1]
    res["D"]["steps_traced_twice"] = {"n_step_pairs": len(dups), "km_one_copy": sum(v[0][1] for v in dups),
                                      "detail": [[list(x) for x in v] for v in dups]}
    res["B"]["top5_by_tau_full"] = [{"pair": p["pair"], "length_km": p["len"]["full"], "chord_km": p["chord_km"],
                                     "tau": p["tau"]["full"], "tau_L77": p["tau"]["L77"], "tau_L380": p["tau"]["L380"]}
                                    for p in sorted(pieces, key=lambda p: -p["tau"]["full"])[:5]]
    res["B"]["top5_by_length"] = [{"pair": p["pair"], "length_km": p["len"]["full"], "chord_km": p["chord_km"],
                                   "tau": p["tau"]["full"]} for p in sorted(pieces, key=lambda p: -p["len"]["full"])[:5]]
    res["B"]["pieces_AU_PA"] = [{"length_km": p["len"]["full"], "chord_km": p["chord_km"], "n_segments": p["n_segments"]}
                                for p in pieces if p["pair"] == "AU-PA"]

    with open(out_json, "w") as f:
        json.dump(rnd(res, 6), f, indent=1, sort_keys=True)
    report(res)


# ----------------------------------------------------------------------------- printing
def fmt(x, nd=2):
    return ("%." + str(nd) + "f") % x if isinstance(x, (int, float)) else str(x)


def report(res):
    A, B, C, D = res["A"], res["B"], res["C"], res["D"]
    print("== self tests", json.dumps(rnd(res["self_tests"], 5)))
    print("== counts", res["counts"])
    print("\n== A. plates: area check")
    print({k: (round(v, 6) if isinstance(v, float) else v) for k, v in A["check"].items()})
    print("perimeter sum / 2 (km):", {k: round(v, 1) for k, v in A["perimeter_sum_half"].items()})
    print("\n%-3s %-16s %9s %9s %8s %12s %9s %9s %9s %8s %7s %7s %7s" % (
        "id", "name", "A_sr", "T1_sr", "rel%", "A_Mkm2", "P_cap", "P_full", "P_77", "P_380", "r_full", "r_77", "r_380"))
    for pid, r in sorted(A["plates"].items(), key=lambda kv: -kv[1]["area_sr"]):
        print("%-3s %-16s %9.5f %9.5f %8.3f %12.3f %9.0f %9.0f %9.0f %8.0f %7.3f %7.3f %7.3f%s" % (
            pid, r["name"][:16], r["area_sr"], r["area_table1_sr"], 100 * r["area_rel_diff"], r["area_km2"] / 1e6,
            r["P_cap_km"], r["P_full_km"], r["P_L77_km"], r["P_L380_km"], r["ratio_full"], r["ratio_L77"],
            r["ratio_L380"], " L" if r["large"] else ""))
    print("\ngroup stats (median / area-weighted mean):")
    for g, gs in A["groups"].items():
        print(g, "n=%d" % gs["n"], "area=%.4f sr" % gs["area_sr_sum"],
              " | ".join("%s %.3f / %.3f (n=%d)" % (rk, gs["ratio_" + rk]["median"], gs["ratio_" + rk]["area_weighted_mean"], gs["ratio_" + rk]["n"])
                         for rk in ("full", "L77", "L380")),
              "|| resolved (>=4 steps) L380:", "%.3f / %.3f (n=%d)" % (
                  gs["ratio_L380_resolved4"]["median"], gs["ratio_L380_resolved4"]["area_weighted_mean"],
                  gs["ratio_L380_resolved4"]["n"]) if "ratio_L380_resolved4" in gs else "none",
              "| <4 divider steps L77/L380: %d/%d" % (gs["fewer_than_4_divider_steps_L77"], gs["fewer_than_4_divider_steps_L380"]))

    print("\n== B. junctions and pieces")
    print({k: B[k] for k in ("tolerance_km", "tolerance_sensitivity_{clusters,junctions}", "n_endpoint_clusters",
                             "n_junctions", "junction_degree_hist", "n_pieces",
                             "min_distance_between_distinct_nodes_km", "max_intra_node_spread_km",
                             "hidden_junction_endpoints_on_interior_vertices", "anomalies", "closed_loops",
                             "pairs_with_2_pieces", "pairs_max_count")})
    print("neighbours:", B["neighbours_by_plate"])
    print("hidden:", B["hidden_detail"])
    print("top5 tau:", B["top5_by_tau_full"])
    print("top5 length:", B["top5_by_length"])
    print("AU-PA pieces:", B["pieces_AU_PA"])
    print("pair counts >1:", B["pair_counts"])
    for name in ("all", "sub_dominant_steps_gt50", "non_sub", "sub_dominant_len_gt50_sensitivity"):
        b = B[name]
        print("\n--", name, "n=%d" % b["n"])
        for k in ("full", "L77", "L380"):
            L, T = b[k]["length_km"], b[k]["tau"]
            print("  %-4s length km: med %.1f lwmean %.1f p10 %.1f p90 %.1f max %.1f total %.0f | tau: med %.3f lwmean %.3f"
                  " p10 %.3f p90 %.3f max %.3f | sum len/sum chord %.3f" % (
                      k, L["median"], L["wmean"], L["p10"], L["p90"], L["max"], b[k]["total_length_km"],
                      T["median"], T["wmean"], T["p10"], T["p90"], T["max"], b[k]["sum_length_over_sum_chord"]))
        for k in ("L77", "L380"):
            rv = b[k + "_resolved4"]
            if rv["tau"]:
                print("  %-4s resolved (>=4 divider steps) n=%d (%d pieces shorter than 1 step), share of length %.2f: tau med %.3f lwmean %.3f p10 %.3f p90 %.3f" % (
                    k, rv["n"], rv["n_shorter_than_one_divider_step"], rv["length_share_of_all"], rv["tau"]["median"],
                    rv["tau_length_weighted_mean"], rv["tau"]["p10"], rv["tau"]["p90"]))
        print("  chord km: med %.1f mean %.1f p10 %.1f p90 %.1f" % (
            b["chord_km"]["median"], b["chord_km"]["mean"], b["chord_km"]["p10"], b["chord_km"]["p90"]))

    print("\n== C. subduction by plate (step-file lengths)")
    print({k: C[k] for k in ("sub_total_from_steps_file_km", "sub_total_from_geometry_km", "table3_sub_km",
                             "sub_total_attributed_km", "sub_steps_with_unknown_polarity_km", "n_sub_steps")})
    print("symbol x class km:", {k: {c: round(v, 1) for c, v in d.items()} for k, d in C["symbol_by_class_km"].items()})
    print("symbol x class n:", C["symbol_by_class_nsteps"])
    print("%-3s %-16s %9s %10s %10s %9s %9s %8s %8s" % ("id", "name", "A_Mkm2", "subducts", "overrides", "sd/Mkm2",
                                                         "ov/Mkm2", "sd/P", "ov/P"))
    for pid, r in sorted(C["per_plate"].items(), key=lambda kv: -kv[1]["sub_subducting_km"]):
        if r["sub_subducting_km"] == 0 and r["sub_overriding_km"] == 0:
            continue
        print("%-3s %-16s %9.3f %10.1f %10.1f %9.1f %9.1f %8.3f %8.3f%s" % (
            pid, r["name"][:16], r["area_km2_table1"] / 1e6, r["sub_subducting_km"], r["sub_overriding_km"],
            r["subducting_km_per_Mkm2"], r["overriding_km_per_Mkm2"], r["subducting_share_of_perimeter"],
            r["overriding_share_of_perimeter"], " L" if r["large"] else ""))
    print("split:", C["split_large_small"])
    print("2x2:", C["pair_matrix_2x2_km"])
    print("top5 subducting:", C["top5_subducting"])
    print("top5 overriding:", C["top5_overriding"])
    print("top pairs:", C["top10_pairs_subducting_to_overriding_km"])

    print("\n== D. totals")
    for k in ("table3_total_km", "steps_file_sum_km", "steps_geometry_R6371_sum_km", "steps_mean_len_km",
              "boundaries_file_segments_sum_km", "pieces_sum_km", "pieces_sum_reverse_walk_km",
              "plates_perimeter_sum_over_2_km", "ratio_to_table3_total"):
        print(k, D[k])
    print("by class (steps file km | geometry km | n | Table 3):")
    for c in ("CCB", "CTF", "CRB", "OSR", "OTF", "OCB", "SUB"):
        print("  %s %10.1f %10.1f %5d %8d" % (c, D["by_class_steps_file_km"][c], D["by_class_geometry_km"][c],
                                            D["by_class_nsteps"][c], D["by_class_table3_km"][c]))
    print("step lengths:", {k: round(v, 2) for k, v in D["steps_len_stats_km"].items()})
    print("steps traced twice:", {k: v for k, v in D["steps_traced_twice"].items()})


if __name__ == "__main__":
    if len(sys.argv) != 4:
        sys.exit("usage: bird.py <data_dir> <paper_layout_txt> <out_json>")
    main(sys.argv[1], sys.argv[2], sys.argv[3])
