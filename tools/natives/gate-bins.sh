#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Builds the `deeptime` binaries the P1-17/P1-19 gate bench times, exactly as the natives ship
# (decisions log 2026-10-18; docs/design/p1-17-benchmark-gate.md, "Running the gate"): the `ship`
# profile (fat LTO, one codegen unit), once for the x86-64 baseline and once for x86-64-v3, the two
# builds a player's x86-64 machine can load. The gate reports both.
#
#   tools/natives/gate-bins.sh <out dir> [--plain] [--only baseline|x86-64-v3] [--features F]
#                              [--pgo <merged profile>]
#
#   <out dir>/deeptime-baseline     --profile ship, the target's default CPU
#   <out dir>/deeptime-x86-64-v3    --profile ship, -C target-cpu=x86-64-v3
#   <out dir>/deeptime-release      with --plain: the everyday `release` profile, for comparisons
#   <out dir>/build-times.tsv       each build's wall seconds
#
#   --features F   cargo features of deeptime-cli (e.g. mimalloc)
#   --pgo FILE     profile-guided: both builds use the merged profile FILE (an absolute path;
#                  tools/natives/pgo-profile.sh makes it), each in its own target dir
#
# Run from the repository root inside the dev shell, on the test VM and admitted (cargo:build), e.g.
#   tools/remote/mac.sh --kind cargo:build -- tools/natives/gate-bins.sh /data/ban/deeptime-runs/<task>/bins
# The host must be x86-64 (the v3 build runs here). Both builds name the host triple with --target,
# so build scripts stay on the baseline CPU.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"
# shellcheck source=tools/natives/ship.sh
. tools/natives/ship.sh

OUT=""; PLAIN=0; ONLY=""; FEATURES=""; PGO=""
while [ $# -gt 0 ]; do
    case "$1" in
        --plain) PLAIN=1; shift ;;
        --only) ONLY="$2"; shift 2 ;;
        --features) FEATURES="$2"; shift 2 ;;
        --pgo) PGO="$2"; shift 2 ;;
        -h|--help) sed -n '2,26p' "$0"; exit 0 ;;
        -*) echo "gate-bins: unknown option $1" >&2; exit 2 ;;
        *) OUT="$1"; shift ;;
    esac
done
[ -n "$OUT" ] || { sed -n '2,26p' "$0"; exit 2; }
if [ -n "${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}" ]; then
    echo "gate-bins: RUSTFLAGS is set (${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}); it would replace" \
         ".cargo/config.toml's flags and the x86-64-v3 build's target-cpu. Unset it." >&2
    exit 1
fi
HOST="$(rustc -vV | sed -n 's/^host: //p')"
[[ "$HOST" == x86_64-* ]] || { echo "gate-bins: the host is $HOST; the gate's two builds need an x86-64 host" >&2; exit 1; }
[ -z "$PGO" ] || [[ "$PGO" == /* && -f "$PGO" ]] || { echo "gate-bins: --pgo needs the absolute path of an existing merged profile" >&2; exit 2; }
PKGS=("${PGO_PKGS[@]}")
FEAT=(); [ -z "$FEATURES" ] || FEAT=(--features "$FEATURES")
mkdir -p "$OUT"
printf 'build\tseconds\n' > "$OUT/build-times.tsv"

timed() { # name, cargo args...: builds and records the wall time
    local name=$1 t0 ms; shift
    t0=$(date +%s%N)
    cargo build --locked "${PKGS[@]}" "${FEAT[@]}" "$@"
    ms=$(( ($(date +%s%N) - t0) / 1000000 ))
    printf '%s\t%d.%d\n' "$name" $((ms / 1000)) $(((ms % 1000) / 100)) >> "$OUT/build-times.tsv"
}

build_level() { # level: baseline or the v3 name
    local level=$1 dir
    if [ -n "$PGO" ]; then
        pgo_cargo_args "$HOST" "$level" "$PGO"; dir=$PGO_TARGET_DIR
        timed "$level" --profile "$SHIP_PROFILE" --target "$HOST" "${PGO_ARGS[@]}"
    elif [ "$level" = baseline ]; then
        dir=target
        timed "$level" --profile "$SHIP_PROFILE" --target "$HOST"
    else
        v3_cargo_args "$HOST"; dir=$V3_TARGET_DIR
        timed "$level" --profile "$SHIP_PROFILE" --target "$HOST" "${V3_ARGS[@]}"
    fi
    cp "$dir/$HOST/$SHIP_PROFILE/deeptime" "$OUT/deeptime-$level"
}
if [ -z "$ONLY" ] || [ "$ONLY" = baseline ]; then build_level baseline; fi
if [ -z "$ONLY" ] || [ "$ONLY" = "$V3_LEVEL" ]; then build_level "$V3_LEVEL"; fi
if [ "$PLAIN" = 1 ]; then
    timed release --release
    cp target/release/deeptime "$OUT/deeptime-release"
fi
cat "$OUT/build-times.tsv"
for f in "$OUT"/deeptime-*; do
    echo "$(basename "$f"): $(stat -c %s "$f") bytes, $(v3_insn_count "$f") BMI1/BMI2 instructions (none in a baseline build, thousands in a v3 one)"
done
