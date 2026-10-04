#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Plate task 6: fold trains and thrusts started in collision belts and in forelands, by run.

    python3 tools/review/p2-01-formed.py <runs-folder> <name> [<name>...]

For each world `<runs-folder>/<name>-world` (a `p2-01-planet.sh world` or `again` run) prints what
the structure stage's note says about where folds and thrusts formed (`formed_by`: the collision's
belt and wedge, a lifted Andean margin, the foreland of an Andean margin or of the old
distance rule, the foreland basin of a collision), the totals, the share of continental cells of
each regime under a structure, and the sections the run drew.
"""
import json
import sys
from pathlib import Path


def note(world: Path) -> dict:
    m = json.loads((world / "manifest.json").read_text())
    d = m["notes"]["structure"]["deformation"]
    return json.loads(d) if isinstance(d, str) else d


def main() -> None:
    runs = Path(sys.argv[1])
    for name in sys.argv[2:]:
        d = note(runs / f"{name}-world")
        by = d["formed_by"]
        print(f"== {name}")
        total = {"fold_train": 0, "thrust": 0}
        for k, v in by.items():
            print(f"  formed in {k:9s}: folds {v['fold_train']:5d}  thrusts {v['thrust']:5d}")
        forelands = ("foreland", "basin")
        for kind in total:
            total[kind] = sum(by[k][kind] for k in forelands)
        print(f"  forelands (old rule + basin): folds {total['fold_train']}  thrusts {total['thrust']}")
        print(f"  all formed: {json.dumps(d['formed'])}")
        cov = d["coverage"]
        for reg in ("collision", "back_arc", "arc", "craton", "passive_margin"):
            c = cov.get(reg)
            if c:
                print(f"  coverage {reg:14s}: cells {c['cells']:6d} covered {c['covered']:.3f} "
                      f"young {c['young']:.3f} mean {c['mean_structures']:.2f}")
        sec = runs / f"{name}-sections" / "sections.json"
        if sec.exists():
            s = json.loads(sec.read_text())
            for sc in s.get("sections", []):
                print("  section:", sc.get("title") or sc)


if __name__ == "__main__":
    main()
