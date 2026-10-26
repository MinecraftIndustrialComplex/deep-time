"""earthsplit.py <earthisl.json> <Bird2003.txt> <bird.py dir>: Earth's island front (the model's rule)
split as the model's is split, and Stern and Scholl's named intra-oceanic arcs on Bird's SUB steps."""
import json
import sys

d = json.load(open(sys.argv[1]))
sys.path.insert(0, sys.argv[3])
import bird  # noqa: E402

areas = {k: v["area_sr"] for k, v in bird.read_table1(sys.argv[2]).items()}
steps = d["steps"]
total = sum(s["length"] for s in steps)
DIST = [154.0, 200.0, 260.0, 400.0, 600.0, 1000.0]
AREA = [0.0, 0.0377, 0.1, 0.5, 1.0]


def band(x, edges):
    i = 0
    for j, e in enumerate(edges):
        if x >= e:
            i = j
    return i


def named(s):
    """Stern and Scholl's (2010, p. 11) intra-oceanic arcs as Bird's SUB steps, by plate pair and place."""
    o, u, lat, lon = s["over"], s["under"], s["lat"], s["lon"]
    if (o, u) in (("PS", "PA"), ("MA", "PA")):
        return "Izu-Bonin-Mariana"
    if (o, u) in (("TO", "PA"), ("KE", "PA")):
        return "Tonga-Kermadec"
    if (o, u) == ("NH", "AU"):
        return "Vanuatu"
    if (o, u) == ("SW", "SA"):
        return "South Sandwich"
    if (o, u) == ("CA", "SA"):
        return "Lesser Antilles"
    if (o, u) == ("NA", "PA") and (lon < -165.0 or lon > 0.0):
        return "Aleutian west of -165"
    if (o, u) in (("PA", "AU"), ("PA", "SS"), ("PA", "WL")) and lat > -12.5:
        return "Solomon"
    if (o, u) in (("SB", "SS"), ("WL", "SS"), ("NB", "SS")):
        return "New Britain"
    if (o, u) in (("SU", "PS"), ("PS", "SU")):
        return "Philippine and Manila"
    if (o, u) in (("MS", "SU"), ("SU", "MS"), ("BH", "MS"), ("BH", "SU")):
        return "Sangihe-Halmahera"
    return None


for key in ("whole15_own", "whole20_own", "whole25_own", "cryst15_own", "cryst20_own"):
    isl = [s for s in steps if s[key] > 154.0]
    km = sum(s["length"] for s in isl)
    print(f"== {key} > 154 km: island {km:.0f} km of {total:.0f} ({km / total:.3f})")
    by = [0.0] * len(DIST)
    for s in isl:
        by[band(s[key], DIST)] += s["length"]
    print("  by distance to such crust (km): " + "  ".join(
        f"{int(e)}+: {x:.0f} ({x / km:.3f})" for e, x in zip(DIST, by)))
    for side in ("over", "under"):
        ba = [0.0] * len(AREA)
        for s in isl:
            ba[band(areas.get(s[side], 0.0), AREA)] += s["length"]
        print(f"  by {side} plate area (sr): " + "  ".join(
            f"{e}+: {x:.0f} ({x / km:.3f})" for e, x in zip(AREA, ba)))
print("== all SUB by overriding plate area (sr): " + "  ".join(
    f"{e}+: {sum(s['length'] for s in steps if band(areas.get(s['over'], 0.0), AREA) == i):.0f}"
    for i, e in enumerate(AREA)))
groups = {}
for s in steps:
    g = named(s) or "rest"
    x = groups.setdefault(g, {"km": 0.0, "w15": 0.0, "w20": 0.0, "w25": 0.0, "arc": []})
    x["km"] += s["length"]
    for t in (15, 20, 25):
        if s[f"whole{t}_own"] > 154.0:
            x[f"w{t}"] += s["length"]
    if s["arc_whole_max"] == s["arc_whole_max"]:
        x["arc"].append((s["arc_whole_max"], s["length"]))
print("== named arcs (Bird SUB km; island km by the model's rule at whole crust 15/20/25 km; "
      "CRUST1.0 whole crust 150-250 km behind the trench, length-weighted mean)")
named_km = 0.0
for g, x in sorted(groups.items(), key=lambda kv: -kv[1]["km"]):
    w = sum(l for _, l in x["arc"])
    arc = sum(a * l for a, l in x["arc"]) / w if w else float("nan")
    print(f"  {g:>24}: {x['km']:7.0f}  {x['w15']:7.0f} {x['w20']:7.0f} {x['w25']:7.0f}  arc crust {arc:5.1f} km")
    if g not in ("rest", "Philippine and Manila", "Sangihe-Halmahera"):
        named_km += x["km"]
print(f"  Stern and Scholl's list on Bird's steps (without Philippine, Manila, Sangihe-Halmahera): {named_km:.0f} km")
rest = [s for s in steps if named(s) is None]
for key in ("whole15_own", "whole20_own", "whole25_own"):
    k2 = sum(s["length"] for s in rest if s[key] > 154.0)
    print(f"  rest of SUB that the model's rule calls island at {key}: {k2:.0f} km")
