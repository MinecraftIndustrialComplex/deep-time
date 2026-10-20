#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The continental crust budget of runs (crates/deeptime-core/src/plates/crust_budget.rs): where
the continents' area and volume go and why their land drowns (build log 2026-10-19).

    DEEPTIME_CRUST_BUDGET=1 [DEEPTIME_CRUST_BUDGET_MAPS=<dir>] deeptime stage volcanoes ... > run.txt
    python3 tools/review/continent-budget.py [--out DIR] [--maps NAME=DIR ...] run.txt ...

Prints, per run and as the mean over the runs given:
  series   every snapshot: continental crust's share of the planet (visible, hidden), its volume
           (10^9 km^3), land at the climate's sea level, the dry share of the continental crust,
           the land's share above 2 km, the median distance from land to the coast (km), the mean
           thickness of the continental crust (km)
  bands    at the last snapshot: the thickness bands' share of the continental area, median
           elevation (the freeboard) and mean Airy height
  parts    each process's change of the stocks over the run (area and land as shares of the
           planet, volume in 10^6 km^3)
  moves    each process's single-sample moves over the run (ocean made continental, with the mean
           elevation and thickness it had then; hidden; surfacing; drowned; emerging)
With --out (needs numpy and matplotlib; on the VM: nix shell --impure --expr '(builtins.getFlake
"nixpkgs").legacyPackages.${builtins.currentSystem}.python3.withPackages (p: [p.numpy
p.matplotlib])'), also draws series.png (area, land, volume over time), bands.png (thickness
distribution over time), freeboard.png (elevation against thickness), parts.png, and per --maps
folder (written by DEEPTIME_CRUST_BUDGET_MAPS) maps-NAME.png at chosen times.
"""
import argparse
import re
import sys
from pathlib import Path

NUM = r"(-?[0-9.]+(?:e[+-]?[0-9]+)?|NaN|nan|inf)"


def kv(text):
    return {k: float(v) for k, v in re.findall(r"([A-Za-z0-9_/]+)=" + NUM + r"(?=[\s\]]|$)", text)}


def read(path):
    """Snapshots by time: dict t -> {'t': {...}, 'bins': [...], 'parts': {...}, 'moves': {...},
    'globals': {...}}."""
    snaps = {}
    with open(path) as f:
        for line in f:
            line = line.rstrip("\n")
            m = re.match(r"(cb_t|cb_bins|cb_parts|cb_moves|cb_globals) t=(-?[0-9.]+)", line)
            if not m:
                continue
            kind, t = m.group(1), float(m.group(2))
            s = snaps.setdefault(t, {})
            body = line[m.end():]
            if kind == "cb_t":
                s["t"] = kv(body)
            elif kind == "cb_bins":
                bins = []
                for b in re.findall(r"\[([0-9.]+)-([0-9.]+)km ([^\]]*)\]", body):
                    d = kv(b[2])
                    z = re.search(r"z10/50/90=(-?[0-9.]+|NaN|nan)/(-?[0-9.]+|NaN|nan)/(-?[0-9.]+|NaN|nan)", b[2])
                    if z:
                        d["z10"], d["z50"], d["z90"] = (float(x) for x in z.groups())
                    d["lo"], d["hi"] = float(b[0]), float(b[1])
                    bins.append(d)
                s["bins"] = bins
            elif kind == "cb_parts":
                parts = {}
                for name, inner in re.findall(r"(\w+)=\[([^\]]*)\]", body):
                    parts[name] = {
                        p: float(x) for p, x in re.findall(r"(\w+):" + NUM, inner)
                    }
                s["parts"] = parts
            elif kind == "cb_moves":
                s["moves"] = {name: kv(inner) for name, inner in re.findall(r"(\w+)=\[([^\]]*)\]", body)}
            else:
                s["globals"] = kv(body)
    return dict(sorted(snaps.items()))


SERIES = [
    ("cont", "cont", "{:.3f}"),
    ("hidden", "cont_hidden", "{:.3f}"),
    ("vol", "vol_km3", "{:.2f}"),
    ("land", "land", "{:.3f}"),
    ("land0", "land0", "{:.3f}"),
    ("dry", "dry", "{:.2f}"),
    (">2km", "above_2km", "{:.3f}"),
    ("coast", "coast_p50_km", "{:.0f}"),
    ("T", "mean_t_km", "{:.1f}"),
    ("sea", "sea_m", "{:.0f}"),
]


def series_value(s, key):
    v = s["t"].get(key)
    if v is not None and key == "vol_km3":
        v /= 1e9
    return v


def mean(xs):
    xs = [x for x in xs if x is not None and x == x]
    return sum(xs) / len(xs) if xs else float("nan")


def print_series(name, snaps):
    print(f"\n== {name}: series")
    print("   t  " + "  ".join(f"{c:>6s}" for c, _, _ in SERIES))
    for t, s in snaps.items():
        if "t" not in s:
            continue
        vals = [series_value(s, k) for _, k, _ in SERIES]
        print(f"{t:5.0f}  " + "  ".join(f"{f.format(v):>6s}" if v is not None else "   n/a" for (_, _, f), v in zip(SERIES, vals)))


def print_bands(name, snaps):
    t, s = list(snaps.items())[-1]
    print(f"\n== {name}: bands at {t:.0f} Myr (visible continental crust)")
    print("  band km     area   vol    z10    z50    z90    iso    dry")
    for b in s.get("bins", []):
        print(
            f"  {b['lo']:3.0f}-{b['hi']:3.0f}  {b.get('a', 0):6.3f} {b.get('v', 0):6.3f} {b.get('z10', 0):6.0f} "
            f"{b.get('z50', 0):6.0f} {b.get('z90', 0):6.0f} {b.get('iso', 0):6.0f} {b.get('dry', 0):6.3f}"
        )


def print_parts(name, runs):
    """The parts' changes over the run (last snapshot), mean over runs."""
    stocks = ["cont", "cont_hidden", "vol", "vol_hidden", "land", "dry"]
    names = set()
    for snaps in runs:
        last = list(snaps.values())[-1]
        for st in stocks:
            names |= set(last.get("parts", {}).get(st, {}))
    print(f"\n== {name}: each process's change over the run (mean of {len(runs)} runs; area, land, dry: share of the planet; vol: 10^6 km^3)")
    print("  part              " + "  ".join(f"{s:>11s}" for s in stocks))
    rows = []
    for p in names:
        vals = [mean([list(sn.values())[-1].get("parts", {}).get(st, {}).get(p, 0.0) for sn in runs]) for st in stocks]
        rows.append((p, vals))
    rows.sort(key=lambda r: r[1][0])
    for p, vals in rows:
        print(f"  {p:16s}  " + "  ".join(f"{v:11.4f}" for v in vals))
    tot = [sum(r[1][k] for r in rows) for k in range(len(stocks))]
    print("  " + "total".ljust(16) + "  " + "  ".join(f"{v:11.4f}" for v in tot))


def print_moves(name, runs):
    keys = ["o2c", "o2c_z", "o2c_t_km", "c2o", "hide", "surface", "drown", "emerge"]
    names = set()
    for snaps in runs:
        names |= set(list(snaps.values())[-1].get("moves", {}))
    print(f"\n== {name}: single-sample moves over the run (mean of {len(runs)} runs; counts as shares of the planet)")
    print("  part              " + "  ".join(f"{k:>9s}" for k in keys))
    for p in sorted(names):
        vals = [mean([list(sn.values())[-1].get("moves", {}).get(p, {}).get(k) for sn in runs]) for k in keys]
        print(f"  {p:16s}  " + "  ".join(f"{v:9.4f}" if abs(v) < 100 else f"{v:9.0f}" for v in vals))


def print_globals(name, runs, length):
    keys = list(list(runs[0].values())[-1].get("globals", {}))
    print(f"\n== {name}: globals over the run, km^3 a year (mean of {len(runs)} runs)")
    for k in keys:
        v = mean([list(sn.values())[-1].get("globals", {}).get(k) for sn in runs])
        print(f"  {k:20s} {v / (length * 1e6):10.3f}")


def print_compare(groups):
    """With several groups: the series at chosen times, each group's mean over its runs."""
    if len(groups) < 2:
        return
    print("\n== groups compared: mean over each group's runs")
    times = sorted({t for runs in groups.values() for sn in runs for t, s in sn.items() if "t" in s})
    picks = sorted({times[0], times[len(times) // 10], times[len(times) // 3], times[len(times) // 2], times[-1]})
    print("  group           t  " + "  ".join(f"{c:>6s}" for c, _, _ in SERIES))
    for gname, runs in groups.items():
        for t in picks:
            vals = [mean([series_value(sn[t], k) if t in sn and "t" in sn[t] else None for sn in runs])
                    for _, k, _ in SERIES]
            print(f"  {gname:12s} {t:5.0f}  " + "  ".join(f"{f.format(v):>6s}" for (_, _, f), v in zip(SERIES, vals)))


def plots(out, groups, maps):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np

    out.mkdir(parents=True, exist_ok=True)
    colors = plt.rcParams["axes.prop_cycle"].by_key()["color"]
    # Series.
    fig, axes = plt.subplots(2, 3, figsize=(15, 8))
    panels = [
        ("cont", "continental crust, share of planet", 0.41, "Earth incl. shelves 0.41 (Cogley 1984)"),
        ("land", "land at sea level, share of planet", 0.289, "Earth 0.289 (HYP-001)"),
        ("dry", "dry share of continental crust", 0.71, "Earth 0.71 (HYP-012)"),
        ("vol_km3", "continental volume, 10^9 km^3", None, None),
        ("above_2km", "land above 2 km, share of land", 0.102, "Earth 0.102 (HYP-004)"),
        ("coast_p50_km", "median distance from land to the coast, km", 433, "Earth 433 (HYP-035)"),
    ]
    for ax, (key, title, earth, label) in zip(axes.flat, panels):
        for gi, (gname, runs) in enumerate(groups.items()):
            for ri, snaps in enumerate(runs):
                ts = [t for t, s in snaps.items() if "t" in s]
                ys = [series_value(snaps[t], key) for t in ts]
                ax.plot(ts, ys, color=colors[gi % len(colors)], alpha=0.8, lw=1.2,
                        label=gname if ri == 0 else None)
        if earth is not None:
            ax.axhline(earth, color="k", ls=":", lw=1, label=label)
        ax.set_title(title)
        ax.set_xlabel("Myr since start")
        ax.legend(fontsize=7)
    fig.tight_layout()
    fig.savefig(out / "series.png", dpi=110)
    plt.close(fig)
    # Thickness bands over time (first group, mean of runs) and freeboard at the end.
    for gname, runs in groups.items():
        fig, (a1, a2) = plt.subplots(1, 2, figsize=(14, 5))
        ts = [t for t, s in runs[0].items() if "bins" in s]
        nb = len(runs[0][ts[0]]["bins"])
        shares = np.array([[mean([sn.get(t, {}).get("bins", [{}] * nb)[b].get("a") * sn.get(t, {}).get("t", {}).get("cont", 0)
                                  if sn.get(t, {}).get("bins") else None for sn in runs]) for b in range(nb)] for t in ts])
        labels = [f"{runs[0][ts[0]]['bins'][b]['lo']:.0f}-{runs[0][ts[0]]['bins'][b]['hi']:.0f} km" for b in range(nb)]
        a1.stackplot(ts, shares.T, labels=labels, colors=plt.cm.viridis(np.linspace(0, 1, nb)))
        a1.set_title(f"{gname}: continental crust by thickness, share of planet")
        a1.set_xlabel("Myr since start")
        a1.legend(fontsize=7, loc="upper right")
        for ti, t in enumerate([ts[0], ts[len(ts) // 4], ts[len(ts) // 2], ts[-1]]):
            mids, z50, iso = [], [], []
            for b in range(nb):
                lo, hi = runs[0][t]["bins"][b]["lo"], runs[0][t]["bins"][b]["hi"]
                mids.append(0.5 * (lo + min(hi, 100.0)))
                z50.append(mean([sn[t]["bins"][b].get("z50") for sn in runs if t in sn and sn[t].get("bins")]))
                iso.append(mean([sn[t]["bins"][b].get("iso") for sn in runs if t in sn and sn[t].get("bins")]))
            a2.plot(mids, z50, "o-", color=colors[ti], label=f"median z, {t:.0f} Myr")
            a2.plot(mids, iso, "x--", color=colors[ti], alpha=0.6, label=f"Airy height, {t:.0f} Myr")
        a2.axhline(0, color="k", lw=0.8)
        a2.set_xlabel("crust thickness, km")
        a2.set_ylabel("elevation, m")
        a2.set_title(f"{gname}: freeboard (elevation against thickness)")
        a2.legend(fontsize=7)
        fig.tight_layout()
        fig.savefig(out / f"bands-{gname}.png", dpi=110)
        plt.close(fig)
    # Maps.
    for name, folder in maps:
        files = sorted(Path(folder).glob("t*.bin"), key=lambda f: float(f.stem[1:]))
        if not files:
            continue
        pick = [files[0], files[min(len(files) - 1, 2)], files[len(files) // 3], files[len(files) // 2],
                files[(2 * len(files)) // 3], files[-1]]
        fig, axes = plt.subplots(len(pick), 2, figsize=(16, 3.3 * len(pick)))
        for row, f in zip(axes, pick):
            a = np.fromfile(f, dtype="<f4").reshape(-1, 5)
            lat, lon, z, thick, kind = a.T
            land = z > 0
            cont = kind > 0.5
            # Left: the sea floor in blues, the land from pale (low) to brown (high).
            row[0].scatter(lon[~land], lat[~land], c=np.clip(z[~land], -6000, 0), cmap="Blues_r",
                           vmin=-7000, vmax=500, s=0.6, lw=0)
            sc0 = row[0].scatter(lon[land], lat[land], c=np.clip(z[land], 0, 5000), cmap="YlOrBr",
                                 vmin=-800, vmax=5000, s=0.6, lw=0)
            row[0].set_title(f"{name} {float(f.stem[1:]):.0f} Myr: elevation; land {np.mean(land):.2f}")
            fig.colorbar(sc0, ax=row[0], fraction=0.02, label="land, m")
            # Right: continental crust, drowned in grey, dry coloured by its thickness.
            wet = cont & ~land
            dry = cont & land
            row[1].scatter(lon[wet], lat[wet], color="#9aa7b4", s=0.6, lw=0)
            sc = row[1].scatter(lon[dry], lat[dry], c=np.clip(thick[dry] / 1000, 20, 70),
                                cmap="viridis", vmin=20, vmax=70, s=0.6, lw=0)
            row[1].set_title(f"continental crust {np.mean(cont):.2f}: grey under the sea, dry ones by thickness "
                             f"(dry share {np.sum(dry) / max(1, np.sum(cont)):.2f})")
            for ax in row:
                ax.set_xlim(-180, 180)
                ax.set_ylim(-90, 90)
                ax.set_aspect("equal")
                ax.set_facecolor("#e4edf5")
            fig.colorbar(sc, ax=row[1], fraction=0.02, label="dry crust, km")
        fig.tight_layout()
        fig.savefig(out / f"maps-{name}.png", dpi=90)
        plt.close(fig)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out")
    ap.add_argument("--length", type=float, default=1000.0)
    ap.add_argument("--maps", action="append", default=[], help="NAME=DIR")
    ap.add_argument("--group", action="append", default=[], help="NAME=run.txt,run.txt (instead of positional runs)")
    ap.add_argument("runs", nargs="*")
    a = ap.parse_args()
    groups = {}
    for g in a.group:
        name, files = g.split("=", 1)
        groups[name] = [read(p) for p in files.split(",") if p]
    if a.runs:
        groups["runs"] = [read(p) for p in a.runs]
    for gname, runs in groups.items():
        runs = [r for r in runs if r]
        groups[gname] = runs
        if not runs:
            print(f"{gname}: no crust budget lines", file=sys.stderr)
            continue
        for i, snaps in enumerate(runs):
            print_series(f"{gname}[{i}]", snaps)
        print_bands(f"{gname}[0]", runs[0])
        print_parts(gname, runs)
        print_moves(gname, runs)
        print_globals(gname, runs, a.length)
    print_compare({k: v for k, v in groups.items() if v})
    if a.out:
        plots(Path(a.out), {k: v for k, v in groups.items() if v}, [m.split("=", 1) for m in a.maps])


if __name__ == "__main__":
    main()
