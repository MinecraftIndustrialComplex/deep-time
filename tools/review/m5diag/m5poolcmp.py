#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""pool.sh table on matched seeds (VM): m5poolcmp.py <measure> <base folder> <arm folder> [<arm folder> ...]

A folder is <pool>/<key>/<preset>-<length>/<argkey>. The base keeps only the runs whose (start, seed) the first arm
has, so a 24-run base reads against an 8-run arm on its own seeds (coordinator, 2026-10-25: "Read the arm against
base15 seeds 1 to 4 at matching envs"). The table is pooltable.render's, as `pool.sh table` prints it, from each
run's cached measures/<measure>.json at the registry's version (tools/review/pool/measures.json)."""
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "pool"))
import pooltable  # noqa: E402


def runs(folder, name, version):
    out = {}
    for d in sorted(Path(folder).iterdir()):
        f = d / "measures" / f"{name}.json"
        if d.is_dir() and f.exists():
            m = json.loads(f.read_text())
            if m.get("version") == version:
                out[d.name] = m["values"]
    return out


def main():
    name, base, arms = sys.argv[1], sys.argv[2], sys.argv[3:]
    reg = json.loads((HERE.parent / "pool" / "measures.json").read_text())
    spec = reg["measures"][name]
    got = [(Path(a).parts[-3][:6] + "/" + Path(a).parts[-1], runs(a, name, spec["version"])) for a in arms]
    keep = set(got[0][1])
    b = {k: v for k, v in runs(base, name, spec["version"]).items() if k in keep}
    print(f"base {base}: {len(b)} runs ({', '.join(sorted(b))})")
    for n, r in got:
        print(f"arm {n}: {len(r)} runs")
    text, _ = pooltable.render(spec, ("base", list(b.values())), [(n, list(r.values())) for n, r in got], name)
    print(text, end="")


if __name__ == "__main__":
    main()
