"""The last cb_globals line of each run, seeds up to N: cbread.py <folder>[@N] [<folder>[@N] ...]

Per folder the medians (and the range) of the arcs' rate per km by type (arc_c / front_c, arc_o / front_o,
km3/km/Myr against Stern & Scholl's 35 and 100), the mean normal convergence at the arcs (front_n / front,
mm/yr, against PLT-666's 58.6 and 86.6), the sediment reaching the trenches and its split by the crust on top,
the sediment subducted, the sediment margins turned ocean floor (PLT-667), all over the run's 1000 Myr as
km3 a year."""
import statistics
import sys
from pathlib import Path


def last_globals(summary):
    line = None
    for x in summary.read_text().splitlines():
        if x.startswith("cb_globals "):
            line = x
    if line is None:
        return None
    out = {}
    for tok in line.split()[1:]:
        k, _, v = tok.partition("=")
        try:
            out[k] = float(v)
        except ValueError:
            pass
    return out


def derived(g):
    yr = 1.0e9
    d = {
        "arc_rate_c": g["arc_c"] / g["front_c"] if g.get("front_c") else float("nan"),
        "arc_rate_o": g["arc_o"] / g["front_o"] if g.get("front_o") else float("nan"),
        "vn_c": g.get("front_n_c", 0.0) / g["front_c"] if g.get("front_c") else float("nan"),
        "vn_o": g.get("front_n_o", 0.0) / g["front_o"] if g.get("front_o") else float("nan"),
        # PLT-670: the share of each front whose arc is younger than the 10 Myr early pulse.
        "young_c": g.get("front_young_c", 0.0) / g["front_c"] if g.get("front_c") else float("nan"),
        "young_o": g.get("front_young_o", 0.0) / g["front_o"] if g.get("front_o") else float("nan"),
        "arc_c_yr": g["arc_c"] / yr,
        "arc_o_yr": g["arc_o"] / yr,
        "front_c_km": g["front_c"] / 1000.0,
        "front_o_km": g["front_o"] / 1000.0,
        "sed_trench_yr": g.get("sed_trench", 0.0) / yr,
        "sed_trench_c_yr": g.get("sed_trench_c", 0.0) / yr,
        "sed_trench_o_yr": g.get("sed_trench_o", 0.0) / yr,
        "sed_slab_yr": (g.get("sed_trench", 0.0) - g.get("sed_trench_c", 0.0) - g.get("sed_trench_o", 0.0)) / yr,
        "sed_subducted_yr": g.get("sed_subducted", 0.0) / yr,
        "margins_left_yr": g.get("margins_left", 0.0) / yr,
        "transition_moved_yr": g.get("transition_moved", 0.0) / yr,
    }
    return d


def main(argv):
    for arg in argv:
        folder, _, top = arg.partition("@")
        rows = []
        for run in sorted(p for p in Path(folder).glob("*-*") if (p / "summary.txt").exists()):
            if top and int(run.name.rsplit("-", 1)[1]) > int(top):
                continue
            g = last_globals(run / "summary.txt")
            if g is None:
                continue
            rows.append((run.name, derived(g)))
        print(f"{folder}: {len(rows)} runs")
        if not rows:
            continue
        for k in rows[0][1]:
            v = [r[1][k] for r in rows if r[1][k] == r[1][k]]
            if not v:
                continue
            print(f"  {k:22} median {statistics.median(v):10.4g}  range {min(v):10.4g} .. {max(v):10.4g}  "
                  f"runs {' '.join(f'{x:.3g}' for x in v)}")


if __name__ == "__main__":
    main(sys.argv[1:])
