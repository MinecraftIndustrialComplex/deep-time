#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-17: where a bench run's serial time goes. Builds this checkout with frame pointers (its own
# target dir), records a call-graph profile of one `deeptime bench` run with perf, and lists per
# symbol the samples of the busiest thread (the one driving the stages) less those of the next
# busiest (a Rayon worker): what the driver does while the workers wait.
#
#   tools/review/p1-17-profile.sh <out dir> <preset> <start> <length_myr> [threads]
#   tools/review/p1-17-profile.sh analyse <out dir>     (the reports again, from <out>/perf.data)
#
# Run on the test VM (perf needs sudo there); the run is admitted as sim:<preset>:<threads>t.
# Writes <out>/{log.txt,bench.json,perf.data,threads.txt,serial.txt,self.txt}.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
if [ "${1:-}" = analyse ]; then
    out=$2
else
[ $# -ge 4 ] || { sed -n 4,14p "$0"; exit 2; }
out=$1; preset=$2; start=$3; len=$4; t=${5:-8}
mkdir -p "$out"
tdir="$out/../target-fp"
( cd "$REPO" && RUSTFLAGS="-C force-frame-pointers=yes" CARGO_TARGET_DIR="$tdir" \
    deeptime-admit run --kind cargo:build -- cargo build --release -q -p deeptime-cli ) || exit 1
bin="$tdir/release/deeptime"
deeptime-admit run --kind "sim:$preset:${t}t" --label "p1-17-profile-$preset-$len" -- \
    sudo perf record -F 199 -g -o "$out/perf.data" -- sudo -u "$(id -un)" env RAYON_NUM_THREADS="$t" \
    "$bin" bench --preset "$preset" --start "$start" --length-myr "$len" --threads "$t" \
    --out "$out/bench.json" > "$out/log.txt" 2>&1
echo "exit $?" >> "$out/log.txt"
sudo chown "$(id -un)" "$out/perf.data"
fi
cd "$out" || exit 1
# Samples per thread (perf's "pid" key is the thread id).
perf report -i perf.data --no-children --sort pid --stdio -g none -n > tid-report.txt 2>&1
grep "%" tid-report.txt | head -12 > threads.txt
perf report -i perf.data --no-children --sort symbol --stdio -g none --percent-limit 0.3 2>/dev/null | grep "%" > self.txt
mapfile -t tids < <(grep "%" tid-report.txt | sort -k2 -n -r | grep -o "[0-9]*:" | tr -d : | head -2)
for tid in "${tids[@]}"; do
    perf report -i perf.data --no-children --tid "$tid" --sort symbol --stdio -g none -n --percent-limit 0 2>/dev/null \
        | grep "%" | awk '{n=$2; $1="";$2="";$3=""; print n"\t"substr($0,1,140)}' > "tid-$tid.txt"
done
python3 - "${tids[@]}" > serial.txt <<'PY'
import sys
def load(t):
    d = {}
    for line in open(f"tid-{t}.txt"):
        n, s = line.split("\t", 1)
        d[s.strip()] = d.get(s.strip(), 0) + int(n)
    return d
a, b = load(sys.argv[1]), load(sys.argv[2])
print(f"driver {sys.argv[1]}: {sum(a.values()) / 199:.0f} s, worker {sys.argv[2]}: {sum(b.values()) / 199:.0f} s (199 Hz)")
rows = sorted(((a[k] - b.get(k, 0), a[k], k) for k in a), reverse=True)[:60]
for d, n, k in rows:
    print(f"{d / 199:7.1f}s {n / 199:7.1f}s  {k[:130]}")
PY
cat threads.txt; head -45 serial.txt
