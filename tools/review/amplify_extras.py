#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Adds the drama-slider figure to an amplification review bundle (P1-06).

`deeptime stage amplify --viewer <dir>` writes the before/after views; the drama slider needs the
same mountains three times, so run the stage twice more with `--set surface.drama=0.25` and
`--set surface.drama=2.0` into other viewer folders, then

    tools/review/amplify_extras.py <bundle> <drama-0.25 bundle> <drama-2 bundle>

copies each run's `amplify/mountains-after.png` next to the bundle's own and lists the three
under the amplification section (`window.DT_REPORT.amplify.extras`, drawn by viewer.js). The
mean detail amplitude of each run (printed by the stage as `mean_detail_blocks=`) can be given
with --detail 0.29,1.16,2.32.
"""

import argparse
import json
import shutil
import sys
from pathlib import Path


def load(bundle: Path):
    text = (bundle / "report.js").read_text()
    head, _, body = text.partition("=")
    return head, json.loads(body.strip().rstrip(";"))


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("bundle", type=Path)
    ap.add_argument("low", type=Path, help="bundle of the run with surface.drama=0.25")
    ap.add_argument("high", type=Path, help="bundle of the run with surface.drama=2")
    ap.add_argument("--detail", help="mean detail amplitude in blocks of the three runs, low,1,high")
    args = ap.parse_args()

    head, report = load(args.bundle)
    am = report.get("amplify")
    if am is None:
        print("no amplify section in", args.bundle, file=sys.stderr)
        return 1
    ids = {r["id"] for r in am["regions"]}
    if "mountains" not in ids:
        print("the bundle has no mountains view", file=sys.stderr)
        return 1
    detail = args.detail.split(",") if args.detail else [None, None, None]
    images = []
    for label, src, value, d in [
        ("drama 0.25", args.low, "0.25", detail[0]),
        ("drama 1 (the default)", args.bundle, "1", detail[1]),
        ("drama 2", args.high, "2", detail[2]),
    ]:
        name = f"amplify/drama-{value}.png"
        shutil.copyfile(src / "amplify" / "mountains-after.png", args.bundle / name)
        text = f"{label}" + (f": mean detail amplitude {d} blocks" if d else "")
        images.append({"label": text, "file": name})
    am["extras"] = [
        {
            "title": "The drama slider (surface.drama)",
            "note": "The roughest mountains again at drama 0.25, 1 and 2. Drama scales the thermal critical "
            "slope (0.25 + 0.75 × drama times the base slope) and the amplitude of the sub-cell detail "
            "term the game adds at chunk time; the terrain here changes little because the amplifier's "
            "slumping reaches only a few dozen cells, the detail amplitude changes in proportion.",
            "images": images,
        }
    ]
    (args.bundle / "report.js").write_text(f"{head}= {json.dumps(report)};\n")
    print("drama figure added:", ", ".join(i["file"] for i in images))
    return 0


if __name__ == "__main__":
    sys.exit(main())
