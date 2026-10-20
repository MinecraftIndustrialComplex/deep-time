#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Makes the PGO profile the shipped natives and the gate bench are built with (STATUS.md build log
# 2026-10-19): an instrumented `deeptime` (the `ship` profile plus `-C profile-generate`), a short
# training run of a representative preset on it, and `llvm-profdata merge`.
#
#   tools/natives/pgo-profile.sh <out dir> [--preset P] [--start S] [--myr N] [--threads T]
#                                [--features F]
#
#   <out dir>/deeptime.profdata   the merged profile: pass it to gate-bins.sh --pgo and to
#                                 build-natives.sh --pgo (an absolute path)
#   <out dir>/raw/                the instrumented run's raw profiles (removed with --clean)
#   <out dir>/train.log           the training run's output
#   <out dir>/pgo-times.tsv       wall seconds: the instrumented build, the training run, the merge
#
# The training run is `deeptime bench` of preset P (default earthlike_full_16k, scattered
# continents) for N Myr (default 40) on T threads (default 8); the instrumented binary runs about
# twice as slow as a ship build. Counters do not depend on the CPU, so the instrumented build is
# for the host's baseline CPU and its profile serves the x86-64-v3 builds as well; the profile
# also fits other targets' builds of the same source as far as the code is the same (see the
# build log). llvm-profdata must come from rustc's own LLVM (the `llvm-tools` rustup component,
# in the Nix dev shell too): another LLVM version may not read the raw profiles.
#
# Run from the repository root inside the dev shell, on the test VM and admitted (a cargo:build,
# then a sim:<preset>:<threads>t; a script that starts both wraps each in `deeptime-admit run`),
# e.g.
#   tools/remote/mac.sh --kind light -- tools/natives/pgo-profile.sh /data/ban/deeptime-runs/<task>/pgo
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"
# shellcheck source=tools/natives/ship.sh
. tools/natives/ship.sh

OUT=""; PRESET=earthlike_full_16k; START=scattered_continents; MYR=40; THREADS=8; FEATURES=""
while [ $# -gt 0 ]; do
    case "$1" in
        --preset) PRESET="$2"; shift 2 ;;
        --start) START="$2"; shift 2 ;;
        --myr) MYR="$2"; shift 2 ;;
        --threads) THREADS="$2"; shift 2 ;;
        --features) FEATURES="$2"; shift 2 ;;
        -h|--help) sed -n '2,29p' "$0"; exit 0 ;;
        -*) echo "pgo-profile: unknown option $1" >&2; exit 2 ;;
        *) OUT="$1"; shift ;;
    esac
done
[ -n "$OUT" ] || { sed -n '2,29p' "$0"; exit 2; }
if [ -n "${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}" ]; then
    echo "pgo-profile: RUSTFLAGS is set; it would replace .cargo/config.toml's flags. Unset it." >&2
    exit 1
fi
HOST="$(rustc -vV | sed -n 's/^host: //p')"
[[ "$HOST" == x86_64-* ]] || { echo "pgo-profile: the host is $HOST; the profile is made on x86-64 Linux" >&2; exit 1; }
SYSROOT_BIN="$(rustc --print sysroot)/lib/rustlib/$HOST/bin"
PROFDATA="$SYSROOT_BIN/llvm-profdata"
[ -x "$PROFDATA" ] || { echo "pgo-profile: no $PROFDATA; add the llvm-tools component (rustup component add llvm-tools; the Nix dev shell has it)" >&2; exit 1; }

mkdir -p "$OUT"
OUT="$(cd "$OUT" && pwd)"
rm -rf "$OUT/raw"; mkdir -p "$OUT/raw"
printf 'step\tseconds\n' > "$OUT/pgo-times.tsv"
now_ms() { date +%s%N | cut -b1-13; }
note() { # step, start ms
    local ms=$(( $(now_ms) - $2 ))
    printf '%s\t%d.%d\n' "$1" $((ms / 1000)) $(((ms % 1000) / 100)) >> "$OUT/pgo-times.tsv"
}

FEAT=(); [ -z "$FEATURES" ] || FEAT=(--features "$FEATURES")
GEN_DIR=target/pgo-gen
t0=$(now_ms)
cargo build --locked -p deeptime-cli "${FEAT[@]}" --profile "$SHIP_PROFILE" --target "$HOST" \
    --target-dir "$GEN_DIR" \
    --config "target.$HOST.rustflags=[\"-C\", \"profile-generate=$OUT/raw\"]"
note instrumented_build "$t0"

t0=$(now_ms)
# --work: the run's world data, removed afterwards; one thread count, so one run.
WORK="$OUT/work"; rm -rf "$WORK"
RAYON_NUM_THREADS="$THREADS" LLVM_PROFILE_FILE="$OUT/raw/deeptime-%p.profraw" \
    "$GEN_DIR/$HOST/$SHIP_PROFILE/deeptime" bench --preset "$PRESET" --start "$START" \
    --length-myr "$MYR" --threads "$THREADS" --out "$OUT/train-bench.json" --work "$WORK" \
    > "$OUT/train.log" 2>&1 || { tail -n 20 "$OUT/train.log" >&2; echo "pgo-profile: the training run failed" >&2; exit 1; }
rm -rf "$WORK"
note training_run "$t0"

t0=$(now_ms)
ls "$OUT"/raw/*.profraw > /dev/null
"$PROFDATA" merge -o "$OUT/deeptime.profdata" "$OUT"/raw/*.profraw
note merge "$t0"
echo "pgo-profile: $OUT/deeptime.profdata ($(stat -c %s "$OUT/deeptime.profdata") bytes)"
cat "$OUT/pgo-times.tsv"
