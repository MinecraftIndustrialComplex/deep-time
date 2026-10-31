#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The rift stretching's run totals (`plates/extension.rs`, PLT-781 to PLT-784) from each run's
`fields/globals.json` (the dump at the run's end; the totals are cumulative):

  python3 -I tools/review/extension/rift_globals.py <run folder> ...

Per run and as the mean over the runs: the crust the rifts stretched off their columns, the part
laid as rift floor at a resampling, the part laid by the fallback, what still waits (stretched less
laid), all km^3 a year over the run's length (Earth-normalized as the other ledgers, 1 AU = 1 km^3
a year), and the steps with a rift.
"""
import json
import os
import sys

KEYS = (
    ("stretched", "plates/rift_stretched_km3"),
    ("floor", "plates/rift_floor_km3"),
    ("fallback", "plates/rift_fallback_km3"),
)


def length_myr(folder):
    args = os.path.join(os.path.dirname(folder.rstrip("/")), "args.json")
    try:
        with open(args) as f:
            return float(json.load(f).get("length_myr", 1000))
    except (OSError, ValueError):
        return 1000.0


def main(folders):
    rows = []
    for d in folders:
        path = os.path.join(d, "fields", "globals.json")
        try:
            with open(path) as f:
                g = json.load(f)
        except OSError as e:
            print(f"{d}: {e}")
            continue
        g = g.get("globals", g)
        years = length_myr(d) * 1.0e6
        v = {k: float(g.get(name, 0.0)) / years for k, name in KEYS}
        v["waiting"] = v["stretched"] - v["floor"] - v["fallback"]
        v["steps"] = float(g.get("plates/rift_steps", 0.0))
        rows.append(v)
        print(
            f"{os.path.basename(d.rstrip('/')):28} stretched {v['stretched']:.4f} floor {v['floor']:.4f} "
            f"fallback {v['fallback']:.4f} waiting {v['waiting']:.4f} km^3/yr; rift steps {v['steps']:.0f}"
        )
    if rows:
        m = {k: sum(r[k] for r in rows) / len(rows) for k in rows[0]}
        print(
            f"{'mean of ' + str(len(rows)):28} stretched {m['stretched']:.4f} floor {m['floor']:.4f} "
            f"fallback {m['fallback']:.4f} waiting {m['waiting']:.4f} km^3/yr; rift steps {m['steps']:.0f}"
        )


if __name__ == "__main__":
    main(sys.argv[1:])
