#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The impact stage's census on the quick 16k planet (the invocation of P2-08's evidence run, without
# the pictures): runs 500 Myr from scattered continents, prints the stage's summary line and counts
# the craters of the present from impacts/craters.json by state and size. Run it detached on the
# test machine (from the repo root):
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start impacts-census tools/review/impacts-census.sh base
#   tools/remote/mac.sh --no-sync -- tools/remote/detach.sh wait impacts-census
#
#   tools/review/impacts-census.sh [<name> [<extra deeptime run arguments>...]]
#
# The run stays under /Volumes/Graphide/ban/deeptime-runs/wp1-impacts/<name>/ until deleted by hand.
set -uo pipefail
cd "$(dirname "$0")/../.."
NAME="${1:-base}"
[ $# -gt 0 ] && shift
RUNS=/Volumes/Graphide/ban/deeptime-runs/wp1-impacts
RUN="$RUNS/$NAME"
mkdir -p "$RUNS"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-8}"
# DEEPTIME_BIN: a binary built earlier (for example from the code before a change), else built here.
if [ -z "${DEEPTIME_BIN:-}" ]; then
    cargo build --release -q -p deeptime-cli || exit 1
    DEEPTIME_BIN=target/release/deeptime
fi
rm -rf "$RUN"
# structure.deformation.density=0: on seed 1 the structure stage panics ("a history holds at most
# 128 events", deeptime-noddy history.rs; STATUS.md, 2026-10-02 plate-check entry); the impact stage
# runs before it and does not read what it makes.
/usr/bin/time -v "$DEEPTIME_BIN" run --preset earthlike_quick_16k --start scattered_continents \
    --length-myr 500 --threads 4 --set structure.deformation.density=0 "$@" --out "$RUN" > "$RUNS/$NAME.log" 2>&1
status=$?
grep -a -E "impacts_final|volcanoes_final|Elapsed|Maximum resident" "$RUNS/$NAME.log"
[ "$status" = 0 ] || { tail -n 20 "$RUNS/$NAME.log"; exit "$status"; }
CR=$(find "$RUN" -name craters.json -path '*impacts*' | head -n 1)
echo "craters file: $CR"
python3 - "$CR" <<'PY'
import json, sys, collections
d = json.load(open(sys.argv[1]))
cs = d["craters"]
edges = [0, 5, 10, 20, 50, 100]
def cls(km):
    k = 0
    for i, e in enumerate(edges):
        if km >= e:
            k = i
    return k
labels = ["<5", "5-10", "10-20", "20-50", "50-100", ">100"]
tab = collections.defaultdict(lambda: [0] * 6)
for c in cs:
    tab[c["fate"]][cls(c["diameter_m"] / 1000.0)] += 1
print("present craters:", len(cs), " (columns", labels, ")")
for f in ["fresh", "degraded", "buried", "eroded_to_roots"]:
    print(f"  {f:16s}", tab[f], sum(tab[f]))
show = [c for c in cs if c["fate"] in ("fresh", "degraded")]
print("showing (fresh or degraded):", len(show))
if show:
    print("oldest showing, Myr:", max(c["age_myr"] for c in show),
          " largest showing, km:", max(c["diameter_m"] for c in show) / 1000.0)
print("by morphology:", dict(collections.Counter(c["morphology"] for c in cs)))
PY
