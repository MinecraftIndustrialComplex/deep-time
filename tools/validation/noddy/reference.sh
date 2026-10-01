#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Regenerates crates/deeptime-noddy/tests/data/reference.txt: the original Noddy kinematics
# (Loop3D/noddyverse, MIT) run on random events and points by harness.c, for the Rust port's
# tests/reference.rs to compare against (plan P2-01).
#
#   tools/remote/mac.sh --pull crates/deeptime-noddy/tests/data/reference.txt -- \
#       tools/validation/noddy/reference.sh
#
# Runs on the test machine (it compiles C). The noddyverse checkout is a scratch clone at the
# pinned commit; nothing of it is vendored into this repository. Only the source files the harness
# reaches are compiled (unEvents.c, taste.c, utils.c, ellFault.c, proflib.c for calcProfile and
# nodLib3.c for initSineFourierCoeff); every other symbol they mention (windows, DXF surfaces) is
# unreachable from the harness and left unresolved on purpose. The script fails if a symbol the
# harness does reach is among the unresolved ones.
set -euo pipefail

COMMIT=6a21b2374516f8506c19cabcbe3ead00be9cc45d
REPO="$(cd "$(dirname "$0")/../../.." && pwd)"
WORK="${NODDY_WORK:-/Volumes/Graphide/ban/deeptime-runs/p2-01/noddy-reference}"
OUT="$REPO/crates/deeptime-noddy/tests/data/reference.txt"

mkdir -p "$WORK"
if [ ! -d "$WORK/noddyverse/.git" ]; then
    git clone --quiet https://github.com/Loop3D/noddyverse.git "$WORK/noddyverse"
fi
git -C "$WORK/noddyverse" checkout --quiet "$COMMIT"
# The checkout's src/malloc.h is a copy of Apple's and shadows the system header (xvt.h includes
# it by name): compile from a scratch copy of src/ whose malloc.h is the standard library's.
SRC="$WORK/src"
rm -rf "$SRC"
cp -r "$WORK/noddyverse/src" "$SRC"
printf '#include <stdlib.h>\n' > "$SRC/malloc.h"

CC="${CC:-gcc}"
command -v "$CC" >/dev/null 2>&1 || CC="nix shell nixpkgs#gcc -c gcc"

# K&R C from the 1990s: an old dialect, and no warnings-as-errors.
$CC -O1 -std=gnu89 -w -fcommon \
    -Wno-error=implicit-function-declaration -Wno-error=implicit-int \
    -Wno-error=int-conversion -Wno-error=incompatible-pointer-types \
    -I"$SRC" -o "$WORK/harness" \
    "$REPO/tools/validation/noddy/harness.c" \
    "$SRC/unEvents.c" "$SRC/taste.c" "$SRC/utils.c" "$SRC/ellFault.c" "$SRC/proflib.c" "$SRC/nodLib3.c" \
    -lm -Wl,--unresolved-symbols=ignore-all

# An unresolved symbol that is reached would be a jump to nowhere (or, silently, garbage).
for sym in reverseEvents unfold unfault unplan undyke unplug unrot taste convrt rotset plane paxis \
           irotate fourier calcProfile initSineFourierCoeff NewtonRaphson local4x4From3x3 matmlt; do
    nm "$WORK/harness" | grep -q " [TtDdBb] $sym\$" \
        || { echo "reference.sh: $sym is not linked into the harness" >&2; exit 1; }
done

mkdir -p "$(dirname "$OUT")"
"$WORK/harness" > "$OUT.tmp"
{
    echo "# Written by tools/validation/noddy/reference.sh from Loop3D/noddyverse $COMMIT (MIT)."
    cat "$OUT.tmp"
} > "$OUT"
rm -f "$OUT.tmp"
echo "cases: $(grep -c '^case ' "$OUT")  points: $(grep -c '^[-0-9]' "$OUT")  bytes: $(wc -c < "$OUT")"
