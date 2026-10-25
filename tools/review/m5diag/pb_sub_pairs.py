"""PB2002's subduction steps by plate pair: pb_sub_pairs.py <PB2002_steps.dat.txt>. Prints the format's first
lines, then the SUB pairs (upper and lower plate from the boundary name's slash), their lengths, the
count of pairs and of upper plates, and the length percentiles."""
import sys
from collections import defaultdict

lines = open(sys.argv[1]).read().splitlines()
for x in lines[:3]:
    print(x)
by = defaultdict(float)
for x in lines:
    v = x.split()
    if len(v) < 10 or "SUB" not in x:
        continue
    name = v[1].lstrip(":")
    cls = [t for t in v if t.strip("*:") in ("SUB",)]
    if not cls:
        continue
    try:
        km = float(v[6])
    except ValueError:
        continue
    by[name] += km
print(len(by), "SUB boundary names")
lens = sorted(by.values(), reverse=True)
tot = sum(lens)
print(f"total {tot:.0f} km")
uppers = set()
for name in by:
    for sep in ("/", "\\"):
        if sep in name:
            a, b = name.split(sep)
            uppers.add((a, b, sep))
print("names:", " ".join(f"{n}:{by[n]:.0f}" for n in sorted(by, key=lambda k: -by[k])))


def pct(x, q):
    x = sorted(x)
    i = (len(x) - 1) * q / 100
    lo = int(i)
    hi = min(lo + 1, len(x) - 1)
    return x[lo] + (x[hi] - x[lo]) * (i - lo)


print("pair km " + " ".join(f"p{q} {pct(lens, q):.0f}" for q in (25, 50, 75, 90)), f"max {lens[0]:.0f}",
      f">=500 km {sum(1 for x in lens if x >= 500)}", f"top5 share {sum(lens[:5]) / tot:.2f}")
