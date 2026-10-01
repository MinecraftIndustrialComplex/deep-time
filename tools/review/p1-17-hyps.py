#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""P1-17 hypsometry recalibration: the summary of tools/review/p1-17-hyps.sh's runs.

    p1-17-hyps.py <run folder> <length Myr> "<seeds>" "<starts>" <label> ...

Prints one line per run and one per label (mean [min, max] over its runs) and writes every run's
figures and hypsographic curve to <run folder>/summary-<label>-<length>.json for the plots.
Earth (GMT earth_relief, SRTM15+, cos-weighted; STATUS.md P1-17): land 0.289, median 427 m,
p90 2,022 m, above 2 km 0.102, sea median depth 4,062 m.
"""
import json
import re
import sys

KEYS = {
    "hypsometry": ["land_frac_area", "land_z_p50_m", "land_z_p90_m", "land_above_1km", "land_above_2km",
                   "sea_depth_p50_m", "shelf_frac_of_sea", "continental_frac", "continental_dry"],
    "climate_final": ["t_surface_c", "t_land_c", "glaciated_land_frac", "co2_ppm", "snowballs"],
    "margins records": ["active_mean"],
    "volcanoes_final": ["arc_base_p50_mean_m", "arc_base_p50_final_m"],
}
SHOW = [("land", "land_frac_area", "{:.3f}"), ("z50", "land_z_p50_m", "{:.0f}"), ("z90", "land_z_p90_m", "{:.0f}"),
        (">2km", "land_above_2km", "{:.3f}"), ("sea50", "sea_depth_p50_m", "{:.0f}"),
        ("crust", "continental_frac", "{:.3f}"), ("dry", "continental_dry", "{:.2f}"),
        ("T", "t_surface_c", "{:.1f}"), ("ice", "glaciated_land_frac", "{:.3f}"), ("E", "koppen_E", "{:.2f}"),
        ("B", "koppen_B", "{:.2f}"), ("active", "active_mean", "{:.2f}"), ("arc", "arc_base_p50_mean_m", "{:.0f}"),
        ("snow", "snowballs", "{:.0f}"), ("s", "seconds", "{:.0f}")]


def parse(path: str) -> dict:
    text = open(path).read()
    out: dict = {}
    for tag, ks in KEYS.items():
        line = next((ln for ln in text.splitlines() if tag in ln), "")
        for k in ks:
            m = re.search(r"(?:^|\s)" + re.escape(k) + r"=([-0-9.eE+]+|nan|NaN)", line)
            if m:
                out[k] = float(m.group(1))
    m = re.search(r"koppen_land=A([0-9.]+)/B([0-9.]+)/C([0-9.]+)/D([0-9.]+)/E([0-9.]+)", text)
    if m:
        for name, x in zip("ABCDE", m.groups()):
            out["koppen_" + name] = float(x)
    m = re.search(r"deeptime stage \w+: steps=\S+ .* seconds=([0-9.]+)", text)
    if m:
        out["seconds"] = float(m.group(1))
    m = re.search(r"bands_km\[-7\.\.6\]=([0-9./]+)", text)
    if m:
        out["bands"] = [float(x) for x in m.group(1).split("/")]
    m = re.search(r"hypsometry_curve pct=0\.\.100 z_m=([-0-9/]+)", text)
    if m:
        out["curve"] = [float(x) for x in m.group(1).split("/")]
    m = re.search(r"timings (.*)", text)
    if m:
        out["timings"] = m.group(1).strip()
    m = re.search(r"evolution t_myr:land/land_z_p50_m (.*)", text)
    if m:
        out["evolution"] = m.group(1).strip()
    return out


def fmt(fmt_s: str, x) -> str:
    return "-" if x is None else fmt_s.format(x)


def main() -> int:
    r, length, seeds, starts = sys.argv[1], sys.argv[2], sys.argv[3].split(), sys.argv[4].split()
    for label in sys.argv[5:]:
        runs = {}
        for start in starts:
            for seed in seeds:
                path = f"{r}/{label}-{start}-{seed}-{length}.txt"
                try:
                    runs[f"{start}-{seed}"] = parse(path)
                except FileNotFoundError:
                    print(f"{label} {start} {seed}: missing {path}")
        for name, v in runs.items():
            cells = " ".join(f"{h}={fmt(f, v.get(k))}" for h, k, f in SHOW)
            print(f"{label} {name}: {cells}")
        agg = []
        for h, k, f in SHOW:
            xs = [v[k] for v in runs.values() if k in v]
            if xs:
                agg.append(f"{h}={f.format(sum(xs) / len(xs))}[{f.format(min(xs))},{f.format(max(xs))}]")
        print(f"== {label} n={len(runs)} {length} Myr: " + " ".join(agg), flush=True)
        with open(f"{r}/summary-{label}-{length}.json", "w") as fh:
            json.dump({"label": label, "length_myr": float(length), "runs": runs}, fh)
    return 0


if __name__ == "__main__":
    sys.exit(main())
