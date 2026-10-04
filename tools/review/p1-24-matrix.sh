#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# P1-24: the measurement matrix, one run after another (so the runs do not share CPUs with each
# other), each through tools/review/p1-24-profile.sh. Start it detached and admitted once for the
# whole matrix:
#
#   tools/remote/mac.sh -- tools/remote/detach.sh start p124-matrix --kind sim:earthlike_quick_16k:11t \
#       tools/review/p1-24-matrix.sh <runs dir> [timing|quality|all]
#
# timing:  the player profile (mid6) on both presets and both starts at 500 Myr, the quick preset
#          at 1000 Myr, and the low4 profile on the quick preset.
# quality: the faster-quick-mode candidates (option a) on the quick preset, scattered continents,
#          500 Myr, each world kept for tools/review/p1-24-views.py.
# A run whose result.json exists is skipped, so the matrix resumes after a stop.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
runs="${1:?runs dir}"; what="${2:-all}"
P="$REPO/tools/review/p1-24-profile.sh"
one() {   # name profile preset start length [extra run args...]
    local name="$1"; shift
    if [ -f "$runs/$name/result.json" ]; then echo "skip $name"; return; fi
    echo "=== $name $(date -u +%H:%M:%S)"
    "$P" "$1" "$2" "$3" "$4" "$runs/$name" "" -- "${@:5}" | tail -1
}
keep() { P124_KEEP=1 one "$@"; }
Q=earthlike_quick_16k; F=earthlike_full_16k; SC=scattered_continents; SU=supercontinent

if [ "$what" = timing ] || [ "$what" = all ]; then
    one t-mid6-quick-$SU-500 mid6 $Q $SU 500
    keep t-mid6-quick-$SC-500 mid6 $Q $SC 500          # also option (a)'s baseline picture
    one t-mid6-full-$SC-500 mid6 $F $SC 500
    one t-mid6-full-$SU-500 mid6 $F $SU 500
    one t-mid6-quick-$SC-1000 mid6 $Q $SC 1000
    one t-mid6-quick-$SU-1000 mid6 $Q $SU 1000
    one t-low4-quick-$SC-500 low4 $Q $SC 500
fi
if [ "$what" = quality ] || [ "$what" = all ]; then
    keep a-l1r32 mid6 $Q $SC 500 --set resolution.l1_r=32
    keep a-l0-50k mid6 $Q $SC 500 --set resolution.l0_samples=50000
    keep a-step4 mid6 $Q $SC 500 --set tectonics.time_step_myr=4 --set climate.snapshot_every_myr=20
    keep a-kf100 mid6 $Q $SC 500 --set keyframes.every_myr=100
    keep a-len250 mid6 $Q $SC 250
    keep a-express mid6 $Q $SC 500 --set resolution.l1_r=32 --set resolution.l0_samples=50000 \
        --set tectonics.time_step_myr=4 --set climate.snapshot_every_myr=20 --set keyframes.every_myr=100
fi
if [ "$what" = long ] || [ "$what" = all ]; then
    one t-mid6-full-$SC-1000 mid6 $F $SC 1000
fi
echo "matrix done $(date -u +%H:%M:%S)"
