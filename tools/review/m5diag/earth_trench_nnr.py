"""Earth's upper-plate and trench normal motions from Becker's NNR re-expression of Heuret &
Lallemand 2005 (trench_heuret05_nnr.dat): earth_trench_nnr.py <dat> (columns lon lat tazi tvx tvy tvn pvx pvy pvn).
Prints the count, the percentiles and the shares in backarc.py's Vup bands (negative = retreat)."""
import sys

rows = []
for line in open(sys.argv[1]):
    s = line.strip()
    if not s or s.startswith("#"):
        continue
    v = s.split()
    if len(v) < 9:
        continue
    rows.append([float(x) for x in v[:9]])
print(len(rows), "rows")


def pct(x, q):
    x = sorted(x)
    i = (len(x) - 1) * q / 100
    lo = int(i)
    hi = min(lo + 1, len(x) - 1)
    return x[lo] + (x[hi] - x[lo]) * (i - lo)


for name, col in (("pvn (upper plate)", 8), ("tvn (trench)", 5)):
    x = [r[col] for r in rows]
    print(name, " ".join(f"p{q} {pct(x, q):.1f}" for q in (10, 25, 50, 75, 90)), f"mean {sum(x) / len(x):.1f}")
    edges = (-20.0, -5.0, 5.0, 20.0)
    names = ("ret_fast", "ret", "neutral", "adv", "adv_fast")
    cnt = [0] * 5
    for v in x:
        cnt[sum(v >= e for e in edges)] += 1
    print("  bands " + " ".join(f"{n} {c / len(x):.3f}" for n, c in zip(names, cnt)),
          f" negative {sum(v < 0 for v in x) / len(x):.3f}")
