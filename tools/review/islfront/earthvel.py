"""earthvel.py <PB2002_steps.dat> <earthisl.json>: Bird 2003's convergence rate (minus the step's
divergence, mm/a, PB2002_steps.dat) on SUB and OCB steps, in the dynamics record's normal-speed bands,
by length; for all SUB, for SUB that the model's rule calls island (CRUST1.0 whole crust 20 km within
154 km) and for OCB."""
import json
import sys

EDGES = [0.0, 5.0, 10.0, 20.0, 40.0, 80.0]
rows = []
for line in open(sys.argv[1], encoding="utf-8", errors="replace"):
    if not line.strip():
        continue
    tok = line[12:].split()
    rows.append({"seq": int(line[0:4]), "length": float(tok[4]), "div": float(tok[8]), "cls": line[92:95]})
isl = {s["k"] for s in json.load(open(sys.argv[2]))["steps"] if s["whole20_own"] > 154.0}


def band(x):
    i = 0
    for j, e in enumerate(EDGES):
        if x >= e:
            i = j
    return i


def show(name, sel):
    km = [0.0] * len(EDGES)
    neg = 0.0
    for r in sel:
        c = -r["div"]
        if c < 0.0:
            neg += r["length"]
            c = 0.0
        km[band(c)] += r["length"]
    tot = sum(km)
    print(f"  {name:>34} {tot:7.0f} km: " + "  ".join(f"{int(e)}+: {x / tot:.3f}" for e, x in zip(EDGES, km))
          + f"  (diverging {neg / tot:.3f})")


print("== convergence rate bands (mm/a), shares of length")
show("SUB", [r for r in rows if r["cls"] == "SUB"])
show("SUB island by the model's rule", [r for k, r in enumerate(rows) if r["cls"] == "SUB" and k in isl])
show("SUB not island", [r for k, r in enumerate(rows) if r["cls"] == "SUB" and k not in isl])
show("OCB", [r for r in rows if r["cls"] == "OCB"])
show("CCB", [r for r in rows if r["cls"] == "CCB"])

# Stern and Scholl's named intra-oceanic arcs (earthsplit.py's mapping) and Leat and Larter's wider list.
import os  # noqa: E402

src = open(os.path.join(os.path.dirname(os.path.abspath(__file__)), "earthsplit.py")).read()
ns = {}
exec(src[src.index("def named(s):"):src.index("\nfor key in (")], ns)
steps = {s["k"]: s for s in json.load(open(sys.argv[2]))["steps"]}
ss = {k for k, s in steps.items() if ns["named"](s) not in (None, "Philippine and Manila", "Sangihe-Halmahera")}
ll = {k for k, s in steps.items() if ns["named"](s) not in (None, "Philippine and Manila")}
ll |= {k for k, s in steps.items() if (s["over"], s["under"]) == ("ON", "PS")}
ll |= {k for k, s in steps.items() if (s["over"], s["under"]) == ("PA", "AU") and s["lat"] < -40.0}
show("Stern and Scholl's named arcs", [r for k, r in enumerate(rows) if k in ss])
show("Leat and Larter's 13 systems (approx.)", [r for k, r in enumerate(rows) if k in ll])
