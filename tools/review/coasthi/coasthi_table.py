#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Medians over an arm's runs of coasthi.py's output, beside Earth's: coasthi_table.py <runs.jsonl> <earth.json>
[--corr]. Lists are medianed element by element; dicts key by key. --corr adds Spearman rho of M5
with each scalar over the runs."""
import json
import sys

import numpy as np


def flatten(d, pre=""):
    out = {}
    for k, v in d.items():
        if isinstance(v, dict):
            out.update(flatten(v, f"{pre}{k}."))
        elif isinstance(v, list) and v and isinstance(v[0], dict):
            for i, e in enumerate(v):
                tag = e.get("bin_Mkm2") or e.get("bin_dmax_km") or str(i)
                out.update(flatten({kk: vv for kk, vv in e.items() if not kk.startswith("bin")}, f"{pre}{k}.{tag}."))
        elif isinstance(v, list):
            for i, e in enumerate(v):
                out[f"{pre}{k}[{i}]"] = e
        elif isinstance(v, (int, float)) or v is None:
            out[f"{pre}{k}"] = v
    return out


def main():
    runs = [flatten(json.loads(line)) for line in open(sys.argv[1]) if line.strip()]
    earth = json.load(open(sys.argv[2]))
    en = flatten(earth["no_sheets"])
    ea = flatten(earth["all"])
    keys = []
    for r in runs:
        for k in r:
            if k not in keys:
                keys.append(k)
    m5 = np.array([r["M5"] for r in runs])
    corr = "--corr" in sys.argv
    if corr:
        from scipy.stats import spearmanr
    print(f"{len(runs)} runs; columns: median [p25, p75] | Earth no sheets | Earth all" + (" | rho with M5" if corr else ""))
    for k in keys:
        v = np.array([np.nan if r.get(k) is None else float(r[k]) for r in runs])
        if np.all(np.isnan(v)):
            continue
        q = np.nanpercentile(v, [50, 25, 75])
        e1 = en.get(k)
        e2 = ea.get(k)
        line = f"{k:48s} {q[0]:9.4f} [{q[1]:8.4f}, {q[2]:8.4f}]"
        line += f" | {e1:8.4f}" if isinstance(e1, (int, float)) else " |      n/a"
        line += f" | {e2:8.4f}" if isinstance(e2, (int, float)) else " |      n/a"
        if corr and np.isfinite(v).sum() > 5 and np.nanstd(v) > 0:
            ok = np.isfinite(v)
            rho, p = spearmanr(m5[ok], v[ok])
            line += f" | {rho:+.2f} (p {p:.3f})"
        print(line)


if __name__ == "__main__":
    main()
