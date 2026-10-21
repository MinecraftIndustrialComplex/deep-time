#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# The continent bisect (build log 2026-10-21): the same earthlike_quick_16k runs on each of several
# commits of master, each commit's own release `deeptime` and its own shipped preset, read by one
# script (tools/review/continent-bisect.py). Everything lives under
# /data/ban/deeptime-runs/continent-bisect/ (ROOT): src/<commit>/ (a `git archive` of the commit,
# with its own target/), bin/<commit>/deeptime, runs/<commit>/<start>-<seed>-<stop>.txt (the stage's
# stdout) and fields/<commit>/<start>-<seed>-<stop>/ (`deeptime dump-fields` of its checkpoint).
#
#   tools/review/continent-bisect.sh ship <commit>...        on the laptop: each commit's `git archive`
#                                                            onto the VM, into its own src/<commit>/
#   tools/remote/mac.sh -- tools/remote/detach.sh start <job> tools/review/continent-bisect.sh \
#       all <commit> <seeds,> <starts,> <stops,> [<length Myr>]
#                                                            on the VM (detach.sh puts it in the dev shell):
#                                                            build (once), then the runs
#   ... continent-bisect.sh build <commit>                   only the build
#   ... continent-bisect.sh runs <commit> <seeds,> <starts,> <stops,> [<length>]   only the runs
#   ... continent-bisect.sh control <commit> <seeds,> <starts,> <stops,>   the runs without the crust
#                                                            budget's env (into $ROOT/ctl, to cmp with runs/)
#
# A stop below the length is `--set timeline.stop_myr=<stop>` on the run planned for the length:
# the world at that time of the longer run (docs/design/playability-steering.md: a run's world
# after plate step k is the same for every stop whose own tail starts after k), with its own tail.
# Every run is admitted by the VM's admission tool on its own (kind sim:earthlike_quick_16k:<T>t).
# Environment: T (threads per run, default 2), SEED_TARGET (a recent target/ to copy first, so the
# dependencies are not built again), CB_ROOT. The kernels run on the CPU (DEEPTIME_COMPUTE=cpu:
# the VM has no GPU, and a CPU world does not depend on whether a commit has the GPU path), and
# DEEPTIME_CRUST_BUDGET=1 prints the crust budget's measurement lines (island-arc share of the land;
# measurement only, and a run without it is a control).
set -uo pipefail
ROOT=${CB_ROOT:-/data/ban/deeptime-runs/continent-bisect}
SEED_TARGET=${SEED_TARGET:-/data/ban/mic/mods/agent-a7e8aa9425fbce4eb/target}
T=${T:-2}
PRESET=${PRESET:-earthlike_quick_16k}
CB_BUDGET=${CB_BUDGET:-1}
SELF=$(readlink -f "$0")
verb=${1:-}; shift || true

case "$verb" in
ship)
  HOST=${DEEPTIME_MAC:-ban@deeptime-test-ban.westus2.cloudapp.azure.com}
  for c in "$@"; do
    echo "ship $c"
    git archive "$c" | ssh -o BatchMode=yes "$HOST" \
      "rm -rf '$ROOT/src/$c' && mkdir -p '$ROOT/src/$c' && tar -x -m -C '$ROOT/src/$c'" || exit 1
  done
  ;;

build)
  c=$1
  src=$ROOT/src/$c
  cd "$src" || exit 1
  mkdir -p target "$ROOT/bin/$c"
  if [ ! -d target/release ] && [ -d "$SEED_TARGET/release" ]; then
    # ext4 has no reflinks: a plain copy. tar -m gave every source file the extraction time, so
    # every workspace crate rebuilds (cargo trusts file times, not contents).
    cp -a --reflink=auto "$SEED_TARGET/release" target/release
    for f in CACHEDIR.TAG .rustc_info.json; do [ -e "$SEED_TARGET/$f" ] && cp -a "$SEED_TARGET/$f" target/; done
  fi
  export CARGO_TARGET_DIR=$src/target
  env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID deeptime-admit run --kind cargo:build --label "cb-build-$c" -- \
    cargo build --release -p deeptime-cli > "$ROOT/bin/$c/build.log" 2>&1 || { tail -n 30 "$ROOT/bin/$c/build.log"; echo "BUILD FAILED $c"; exit 1; }
  cp target/release/deeptime "$ROOT/bin/$c/deeptime" || exit 1
  "$ROOT/bin/$c/deeptime" version | tee "$ROOT/bin/$c/version.txt"
  echo "BUILD_OK $c"
  ;;

runs)
  c=$1; SEEDS=${2//,/ }; STARTS=${3//,/ }; STOPS=${4//,/ }; LEN=${5:-1000}
  bin=$ROOT/bin/$c/deeptime
  [ -x "$bin" ] || { echo "no binary $bin"; exit 1; }
  mkdir -p "$ROOT/runs/$c" "$ROOT/fields/$c" "$ROOT/tmp/$c"
  pids=()
  for stop in $STOPS; do
    for start in $STARTS; do
      for seed in $SEEDS; do
        name=$start-$seed-$stop
        out=$ROOT/runs/$c/$name.txt
        fields=$ROOT/fields/$c/$name
        if [ -s "$out" ] && [ -s "$fields/globals.json" ]; then continue; fi
        (
          ck=$ROOT/tmp/$c/$name.ck
          work=$ROOT/tmp/$c/$name.work
          sets=(--set "seed=$seed")
          if [ "$stop" != "$LEN" ]; then sets+=(--set "timeline.stop_myr=$stop"); fi
          budget=()
          if [ "$CB_BUDGET" = 1 ]; then budget=(DEEPTIME_CRUST_BUDGET=1); fi
          env -u DEEPTIME_ADMIT_JOB -u DEEPTIME_ADMIT_PID DEEPTIME_COMPUTE=cpu "${budget[@]}" \
            deeptime-admit run --kind "sim:$PRESET:${T}t" --label "cb-$c-$name" -- \
            nice -n 10 "$bin" stage volcanoes --preset "$PRESET" --start "$start" --length-myr "$LEN" \
            --threads "$T" "${sets[@]}" --work "$work" --out "$ck" > "$out.part" 2>&1 \
          && "$bin" dump-fields --in "$ck" --out "$fields.part" --fields surface/elevation_m,plates/crust_type,elevation >> "$out.part" 2>&1 \
          && rm -rf "$fields" && mv "$fields.part" "$fields" && mv "$out.part" "$out" \
          || echo "FAILED $c $name" >&2
          rm -rf "$ck" "$ck".* "$work"
        ) &
        pids+=($!)
        sleep 2
      done
    done
  done
  for p in "${pids[@]}"; do wait "$p"; done
  echo "RUNS_DONE $c $SEEDS"
  ;;

control)
  # The same runs with DEEPTIME_CRUST_BUDGET unset, in $ROOT/ctl (its bin/ is the main one): the
  # measurement env must change no field.
  c=$1; shift
  mkdir -p "$ROOT/ctl" && ln -sfn ../bin "$ROOT/ctl/bin"
  CB_ROOT=$ROOT/ctl CB_BUDGET=0 "$SELF" runs "$c" "$@"
  ;;

all)
  c=$1
  if [ ! -x "$ROOT/bin/$c/deeptime" ]; then "$SELF" build "$c" || exit 1; fi
  shift
  "$SELF" runs "$c" "$@"
  echo ALLDONE
  ;;

*)
  sed -n '3,30p' "$0"
  exit 2
  ;;
esac
