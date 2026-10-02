#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Writes the game's built-in presets from the calibrated pipeline presets (P1-08).

The four built-ins the game ships (crates/deeptime-core/presets/earthlike_*.json, compiled into
the core and copied into the jar) are the planet pipeline presets under their game names:

    earthlike_quick_16k  <- presets/pipeline/pipeline_quick_16k.json
    earthlike_full_16k   <- presets/pipeline/pipeline_full_16k.json
    earthlike_quick_32k  <- presets/pipeline/pipeline_quick_32k.json
    earthlike_full_32k   <- presets/pipeline/pipeline_full_32k.json

Only `name` and `description` differ. Tuning happens in the pipeline files (the calibration and
the stages' agents edit those); run this afterwards. `cargo test -p deeptime-core preset::`
fails while the two drift (`builtins_are_the_pipeline_presets`).

    python3 tools/dev/sync_builtin_presets.py [--check]
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PRESETS = ROOT / "crates/deeptime-core/presets"

MAPPING = {
    "earthlike_quick_16k": "pipeline_quick_16k",
    "earthlike_full_16k": "pipeline_full_16k",
    "earthlike_quick_32k": "pipeline_quick_32k",
    "earthlike_full_32k": "pipeline_full_32k",
}


def description(name: str, p: dict) -> str:
    quick = "_quick_" in name
    c = p["scale"]["circumference_blocks"]
    r = p["resolution"]

    def w(k: str) -> int:
        return c // r[k]

    return (
        f"An Earth-like planet, {'quick' if quick else 'full'} "
        f"({'singleplayer' if c == 16384 else 'server'} size, C = {c} blocks): plates and crust, "
        f"climate, deep-time erosion and strata, volcanoes, meteor impacts, rock and metamorphism, ore deposits, the young "
        f"landscape and its amplification, baked for the game. L0 {r['l0_samples']} samples, "
        f"L1 {w('l1_r')}x{w('l1_r') // 2}, L2 {w('l2_r')}x{w('l2_r') // 2}, "
        f"L3 {w('l3_r')}x{w('l3_r') // 2}. The same parameters as {MAPPING[name]} (calibrated "
        f"2026-10-01, P1-17). It names no timeline: choose the start and the "
        f"length in the editor or config/deeptime/server.json."
    )


def render(name: str) -> str:
    src = (PRESETS / "pipeline" / f"{MAPPING[name]}.json").read_text()
    p = json.loads(src)
    out, n1 = re.subn(r'("name":\s*)"[^"]*"', lambda m: m.group(1) + json.dumps(name), src, count=1)
    out, n2 = re.subn(
        r'("description":\s*)"(?:[^"\\]|\\.)*"',
        lambda m: m.group(1) + json.dumps(description(name, p), ensure_ascii=False),
        out,
        count=1,
    )
    if n1 != 1 or n2 != 1:
        sys.exit(f"{MAPPING[name]}.json: no name or description to replace")
    back = json.loads(out)
    for k in ("name", "description"):
        back.pop(k)
        p.pop(k)
    assert back == p, name
    return out


def main() -> int:
    check = "--check" in sys.argv[1:]
    stale = []
    for name in MAPPING:
        path = PRESETS / f"{name}.json"
        want = render(name)
        if path.exists() and path.read_text() == want:
            continue
        stale.append(name)
        if not check:
            path.write_text(want)
            print(f"wrote {path.relative_to(ROOT)}")
    if check and stale:
        print("stale: " + ", ".join(stale) + " (run tools/dev/sync_builtin_presets.py)")
        return 1
    if not stale:
        print("the built-in presets are current")
    return 0


if __name__ == "__main__":
    sys.exit(main())
