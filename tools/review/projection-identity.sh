#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The projection-neutral refactor's identity check (docs/design/equal-area-projection.md §8 step 1):
# a build must write byte-for-byte what master writes. Runs one `deeptime` binary over the JUnit
# fixtures' worlds and pictures and prints one sha256 per output file (JSON with the run's clock
# and timings stripped), sorted, so two builds compare with `diff`:
#
#   tools/review/projection-identity.sh <deeptime binary> <output folder> > hashes.txt
#
# What it writes: the wrap-test and lake-test synthetic worlds (C 1,024) and their sample dumps;
# the real test planet (tests/presets/planet_world_4k.json: every stage, baked) with its viewer
# bundle (the keyframe pictures), its column and climate parity fixtures and a satellite picture
# of its last checkpoint; the deposit world (deposits_world_4k) and its dump. On the test VM via
# tools/remote/mac.sh; the output folder is deleted by the caller.
#
#   tools/review/projection-identity.sh --hash <output folder>   # hash an existing folder again
set -euo pipefail
if [ "$1" = --hash ]; then
    cd "$2"
else
BIN=$(realpath "$1"); OUT=$2
REPO=$(cd "$(dirname "$0")/../.." && pwd)
P=$REPO/crates/deeptime-core/tests/presets
rm -rf "$OUT"; mkdir -p "$OUT"
cd "$OUT"
q() { "$@" > /dev/null 2>> log.txt; }
q "$BIN" synth --kind wrap-test --size 1024 --out wrap1024 --viewer wrap1024-viewer --dump-samples wrap1024-samples.json
q "$BIN" synth --kind lake-test --size 1024 --out lake1024 --dump-samples lake1024-samples.json
q "$BIN" parity --world-data lake1024 --columns 300
q "$BIN" run --preset "$P/planet_world_4k.json" --out planet4k --viewer planet4k-viewer --checkpoints
q "$BIN" parity --world-data planet4k --columns 500
last=$(ls planet4k/checkpoints/*.dtcp | sort -V | tail -1)
q "$BIN" satellite --in "$last" --out planet4k-satellite.png --width 512
q "$BIN" run --preset "$P/deposits_world_4k.json" --out deposits4k
q "$BIN" dump-deposits --world deposits4k --out deposits4k-samples.json
# Checkpoints carry the stages' timings; their pictures and the bake are what is compared.
rm -rf planet4k/checkpoints
fi
python3 - <<'EOF'
import hashlib, json, os, re
# The clock and the run's timings and memory peaks, which differ from run to run of one build.
SKIP = re.compile(r"(millis|_ms$|^passes$|seconds|_secs|_s$|_ns_point|elapsed|created_utc|^time$|timing|^wall|reserved.*bytes)", re.I)
def clean(v):
    if isinstance(v, dict):
        return {k: clean(x) for k, x in v.items() if not SKIP.search(k)}
    if isinstance(v, list):
        return [clean(x) for x in v]
    return v
for root, _, files in os.walk("."):
    for f in files:
        p = os.path.join(root, f)
        # The run's log, and the viewer bundle's code (the page, its scripts): code, not output.
        if f == "log.txt" or f in ("index.html", "viewer.js", "projection.js"):
            continue
        b = open(p, "rb").read()
        if f == "bundle.js" and b.startswith(b"window.DT_BUNDLE = "):
            b = b.split(b"= ", 1)[1].rstrip().rstrip(b";")
            f = "bundle.json"
        if f.endswith(".json"):
            try:
                v = json.loads(b)
                # The parity fixture's own check of the projection (added with the module): the
                # columns are what is compared.
                if f == "columns.json" and isinstance(v, dict):
                    v.pop("projection", None)
                b = json.dumps(clean(v), sort_keys=True).encode()
            except ValueError:
                pass
        print(hashlib.sha256(b).hexdigest(), p)
EOF
