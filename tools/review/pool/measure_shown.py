#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The pool's `shown` measure: the crust that went down a trench and showed again before a
resampling consumed it, by cause, from one pool run's dynamics record (`steps[].shown`, written by
crates/deeptime-core/src/plates/shown.rs since 2026-10-24; older runs have none). Printed as a JSON
object on stdout. It sizes one-way subduction (docs/science/plates.md, "The overlaps' hiding" and
"Step 6: the crust shown again").

    python3 -I measure_shown.py <run folder>

Causes (shown.rs): plate (its plate is not the one it went under with: a rift), reversal (its
contact reversed this step), on_top (its plate overrides the plate it lies under), unseen (no
contact with the plate it lies under), left (it lies under no plate's cover), collision (a terrane
transfer or a weld brought it up).

  under_km2_yr        crust that went under per year (a sample hidden that was visible), km^2/yr
  consumed_km2_yr     crust the resamplings consumed per year (hidden at a resampling), km^2/yr
  shown_<cause>       samples shown again by cause, a share of those that went under
  escaped             of the samples that went under since the last resampling, the share visible at
                      the resampling (escaped / (escaped + consumed)): what one-way subduction would
                      consume besides
  esc_<cause>         the escaped by the cause of their last showing, the same share
  esc_cont, cons_cont continental share of the escaped / of the consumed
  esc_planet          escaped samples per resampling, a share of the planet
  esc_age, ocean_age  mean age of the escaped ocean floor / of all visible ocean floor at the
                      resamplings, Myr
  cons_age            mean age of the consumed ocean floor, Myr
  ocean_age_oneway    the visible ocean floor's mean age at the resamplings with the escaped ocean
                      floor made new (age 0), Myr: one resampling's effect, not the run's

Areas are Earth-normalized (R = 6,371 km). Version 1.
"""
import json
import math
import sys
from pathlib import Path

CAUSES = ["plate", "reversal", "on_top", "unseen", "left", "collision"]
R_KM = 6371.0


def main():
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        sys.exit(2)
    run = Path(sys.argv[1])
    rec = json.loads((run / "rec.json").read_text())
    n = int(rec.get("samples", 0))
    steps = rec.get("steps", [])
    sh = [s.get("shown") for s in steps]
    if not steps or n == 0 or all(x is None for x in sh):
        print(json.dumps({"steps": len(steps)}))
        return
    nc = len(CAUSES)
    under = 0
    shown = [0] * nc
    shown_cont = [0] * nc
    consumed = consumed_cont = 0
    consumed_age = 0.0
    escaped = [0] * nc
    escaped_cont = [0] * nc
    esc_ocean = 0
    esc_age = 0.0
    ocean = 0
    ocean_age = 0.0
    resamplings = 0
    for x in sh:
        if x is None:
            continue
        under += x["under"]
        for c in range(nc):
            shown[c] += x["shown"][c]
            shown_cont[c] += x["shown_cont"][c]
            escaped[c] += x["escaped"][c]
            escaped_cont[c] += x["escaped_cont"][c]
        if x["ocean"] > 0:
            resamplings += 1
        consumed += x["consumed"]
        consumed_cont += x["consumed_cont"]
        consumed_age += x["consumed_age_myr"]
        esc_ocean += x["escaped_ocean"]
        esc_age += x["escaped_age_myr"]
        ocean += x["ocean"]
        ocean_age += x["ocean_age_myr"]
    dt = float(rec.get("dt_myr", 2.0))
    years = len(steps) * dt * 1.0e6
    area = 4.0 * math.pi * R_KM * R_KM / n
    esc = sum(escaped)
    out = {
        "steps": len(steps),
        "resamplings": resamplings,
        "under_km2_yr": under * area / years,
        "consumed_km2_yr": consumed * area / years,
        "escaped": esc / max(esc + consumed, 1),
        "esc_cont": sum(escaped_cont) / max(esc, 1),
        "cons_cont": consumed_cont / max(consumed, 1),
        "esc_planet": esc / max(resamplings, 1) / n,
        "esc_age": esc_age / esc_ocean if esc_ocean else None,
        "cons_age": consumed_age / (consumed - consumed_cont) if consumed > consumed_cont else None,
        "ocean_age": ocean_age / ocean if ocean else None,
        "ocean_age_oneway": (ocean_age - esc_age) / ocean if ocean else None,
    }
    for c, name in enumerate(CAUSES):
        out[f"shown_{name}"] = shown[c] / max(under, 1)
        out[f"esc_{name}"] = escaped[c] / max(esc + consumed, 1)
    out["shown_all"] = sum(shown) / max(under, 1)
    print(json.dumps({k: (round(v, 5) if isinstance(v, float) else v) for k, v in out.items()}))


if __name__ == "__main__":
    main()
