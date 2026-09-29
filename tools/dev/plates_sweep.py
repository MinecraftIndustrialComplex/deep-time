#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Parameter sweep for the plates stage (P1-03 tuning): runs `deeptime stage plates` on a built-in
preset with tectonics overrides and prints each run's summary line.

    python3 tools/dev/plates_sweep.py [--preset earthlike_quick_16k] [--start scattered]
        [--myr 500] [--samples N] 'name:key=value,key=value' ...

Run it on the Mac (tools/remote/mac.sh) after `cargo build --release -p deeptime-cli`.
"""

import argparse
import json
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, ROOT)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--preset", default="earthlike_quick_16k")
    ap.add_argument("--start", default="scattered")
    ap.add_argument("--myr", type=float, default=500.0)
    ap.add_argument("--samples", type=int, default=0)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("variants", nargs="*", default=["default:"])
    a = ap.parse_args()
    base = json.load(open(os.path.join(ROOT, "crates/deeptime-core/presets", a.preset + ".json")))
    exe = os.path.join(ROOT, "target/release/deeptime")
    for v in a.variants:
        name, _, rest = v.partition(":")
        p = json.loads(json.dumps(base))
        p["timeline"]["start"] = a.start
        p["timeline"]["length_myr"] = a.myr
        if a.samples:
            p["resolution"]["l0_samples"] = a.samples
        if a.seed:
            p["seed"] = a.seed
        for kv in filter(None, rest.split(",")):
            k, _, val = kv.partition("=")
            p["tectonics"][k] = json.loads(val)
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "p.json")
            json.dump(p, open(path, "w"))
            out = subprocess.run([exe, "stage", "plates", "--preset", path, "--work", os.path.join(d, "w")],
                                 capture_output=True, text=True)
            lines = [l for l in out.stdout.splitlines() if "plates_final" in l]
            print(f"{name:>14}: {lines[-1] if lines else out.stderr.strip()[-300:]}", flush=True)


if __name__ == "__main__":
    main()
