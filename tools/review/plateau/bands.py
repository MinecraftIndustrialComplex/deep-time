# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau lane: what wears the high ground, by process, from the cached runs' summaries (VM, no simulation).

Usage: python3 -I bands.py <arm folder> [<arm folder> ...]

Each run's `summary.txt` carries the surface stage's `erosion_bands` line (`surface::erosion_bands`, printed by
`deeptime-cli/src/run.rs`): per class `share/net/spl/threshold/diffusion/glacial`, the class's share of the land's
area x time and its mean rates in m/Myr over the whole run. `spl` is the stream-power solve's gross lowering and
includes the critical slope's cut (`threshold`); `net` is h_end - h_start of the surface step (every process, the
dissolved load and the dust included, deposition counted against it; no isostasy). Printed per arm: the medians over
the runs of each rate, and the shares of the gross lowering by process for the flat high ground (above 2 km over the
step's sea, 3 x 3 relief under 500 m at the step's end), the steep high ground and the 2 to 3, 3 to 4 and above 4 km
bands: stream power without the cut, the cut, diffusion, ice. `rest` is net - (spl + diffusion + glacial): the
dissolved load and the dust less the deposition on the class (negative: deposition outweighs them).
"""
import statistics
import sys
from pathlib import Path

CLASSES = ["flat", "steep", "z2000", "z3000", "z4000", "z0", "z1000"]


def parse(summary):
    for line in open(summary, errors="replace"):
        if "erosion_bands " not in line:
            continue
        out = {}
        for tok in line.split("erosion_bands ", 1)[1].split():
            if ":" not in tok or tok.startswith("wrate"):
                continue
            name, vals = tok.split(":", 1)
            v = vals.split("/")
            if len(v) != 6:
                continue
            try:
                share, net, spl, thr, dif, gla = (float(x) for x in v)
            except ValueError:
                continue
            out[name] = dict(share=share, net=net, spl=spl, thr=thr, dif=dif, gla=gla)
        return out
    return None


def med(v):
    v = [x for x in v if x == x]
    return statistics.median(v) if v else float("nan")


def main():
    for arm in sys.argv[1:]:
        runs = []
        for r in sorted(Path(arm).iterdir()):
            s = r / "summary.txt"
            if r.is_dir() and s.exists():
                b = parse(s)
                if b:
                    runs.append((r.name, b))
        print(f"== {arm} ({len(runs)} runs)")
        for c in CLASSES:
            rows = [b[c] for _, b in runs if c in b]
            if not rows:
                continue
            m = {k: med([x[k] for x in rows]) for k in ("share", "net", "spl", "thr", "dif", "gla")}
            # Shares of the gross lowering, run by run, then the median.
            sh = {"spl_nocut": [], "cut": [], "dif": [], "gla": []}
            rest = []
            for x in rows:
                gross = x["spl"] + x["dif"] + x["gla"]
                if gross <= 0:
                    continue
                sh["spl_nocut"].append((x["spl"] - x["thr"]) / gross)
                sh["cut"].append(x["thr"] / gross)
                sh["dif"].append(x["dif"] / gross)
                sh["gla"].append(x["gla"] / gross)
                rest.append(x["net"] - gross)
            print(f"  {c:6s} land share {m['share']:.4f}; m/Myr net {m['net']:.1f} = stream power {m['spl']:.1f} "
                  f"(of it the cut {m['thr']:.1f}) + diffusion {m['dif']:.1f} + ice {m['gla']:.1f} + rest "
                  f"{med(rest):.1f}; gross shares: stream power without the cut {med(sh['spl_nocut']):.3f}, the cut "
                  f"{med(sh['cut']):.3f}, diffusion {med(sh['dif']):.3f}, ice {med(sh['gla']):.3f}")
        flat = [b["flat"]["net"] for _, b in runs if "flat" in b]
        if flat:
            print(f"  flat net per run: min {min(flat):.1f}, max {max(flat):.1f}")
        # The land's mean lowering (the bands' area x time shares times their net), and each class's
        # part of it, run by run, then the median.
        mean, parts = [], {c: [] for c in ("flat", "steep", "z0", "z1000", "z2000", "z3000", "z4000")}
        for _, b in runs:
            total = sum(b[z]["share"] * b[z]["net"] for z in ("z0", "z1000", "z2000", "z3000", "z4000") if z in b)
            if total <= 0:
                continue
            mean.append(total)
            for c in parts:
                if c in b:
                    parts[c].append(b[c]["share"] * b[c]["net"] / total)
        if mean:
            print(f"  the land's mean net lowering {med(mean):.1f} m/Myr; parts of it: "
                  + ", ".join(f"{c} {med(v):.3f}" for c, v in parts.items() if v))
        # What the surface sent to the sea and eroded over the run, Earth-normalized km3 a year
        # (the map's km3 x Earth's area over the L1 map's, 1024 x 328 cells of 4.8 km; a 1000-Myr run).
        scale = 5.10e8 / (1024 * 328 * 4.8 * 4.8) / 1.0e9
        sea, ero = [], []
        for r, _ in runs:
            for line in open(Path(arm) / r / "summary.txt", errors="replace"):
                if "surface_final=" in line:
                    kv = dict(t.split("=", 1) for t in line.split() if "=" in t)
                    sea.append(float(kv.get("to_sea_km3", "nan")) * scale)
                    ero.append(float(kv.get("eroded_km3", "nan")) * scale)
                    break
        if sea:
            print(f"  to the sea {med(sea):.2f} km3/yr, eroded {med(ero):.2f} km3/yr (Earth-normalized, medians)")


if __name__ == "__main__":
    main()
