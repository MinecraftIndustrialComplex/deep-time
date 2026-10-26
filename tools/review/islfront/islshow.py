"""islshow.py <islmap jsonl> [...]: the island map split, summed over runs and snapshots, as shares of
the island (or continental) front's Voronoi km, and the per-snapshot lengths as means."""
import json
import statistics
import sys

EDGES = {
    "area": ["<0.038", "0.038-0.1", "0.1-0.5", "0.5-1", ">=1 sr"],
    "cont": ["0", "0.01", "0.05", "0.15", "0.3", ">=0.5"],
    "hops": ["3", "4", "5-8", "9-16", ">=17/none"],
    "path_km": ["<200", "200-260", "260-400", "400-600", "600-1000", ">=1000/none"],
    "eucl_km": ["<200", "200-400", "400-600", "600-1000", "1000-2000", ">=2000"],
    "age": ["<10", "10-30", "30-60", "60-100", ">=100 Myr"],
    "pair": ["<500", "500-1500", "1500-3000", "3000-6000", ">=6000 km"],
    "chain": ["<150", "150-300", "300-600", "600-1200", "1200-2400", ">=2400 km"],
    "bool": ["no", "yes"],
    "kind": ["oceanic", "continental"],
}


def edges_for(name):
    for k, e in (("area", "area"), ("any_cont", "eucl_km"), ("cont_made", "bool"), ("hascont", "bool"),
                 ("_cont", "cont"), ("hops", "hops"), ("native_path", "path_km"), ("path_km", "path_km"),
                 ("age", "age"), ("pair", "pair"), ("chain", "chain"), ("kind", "kind")):
        if k in name:
            return EDGES[e]
    return EDGES["bool"]


def main():
    for path in sys.argv[1:]:
        rows = [json.loads(line) for line in open(path)]
        print(f"== {path}: {len(rows)} runs, {sum(len(r['snaps']) for r in rows)} snapshots")
        acc = {}
        for r in rows:
            for k, v in r["acc"].items():
                a = acc.setdefault(k, [0.0] * len(v))
                for i, x in enumerate(v):
                    a[i] += x
        for k in sorted(acc):
            v = acc[k]
            s = sum(v)
            e = edges_for(k)
            print(f"  {k:>22}: " + "  ".join(f"{lab}: {x / s:.3f}" for lab, x in zip(e, v)) if s else f"  {k}: -")
        snaps = [s for r in rows for s in r["snaps"]]
        keys = sorted({k for s in snaps for k in s if isinstance(s[k], (int, float))})
        for k in keys:
            xs = [s[k] for s in snaps if k in s]
            print(f"  {k:>26}: mean {statistics.mean(xs):10.1f}  median {statistics.median(xs):10.1f}")


main()
