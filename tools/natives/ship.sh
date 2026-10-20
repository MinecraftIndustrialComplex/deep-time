# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Sourced by tools/natives/build-natives.sh and tools/natives/gate-bins.sh: how a shipped build is
# made (decisions log 2026-10-18; plan §6.4).
#
#   SHIP_PROFILE      the cargo profile of everything that ships and of the gate bench: release
#                     plus fat LTO in one codegen unit (Cargo.toml [profile.ship])
#   V3_LEVEL          the name of the second x86-64 build, `-C target-cpu=x86-64-v3`
#   v3_cargo_args T   sets V3_ARGS to the extra cargo arguments that make a build of target T the
#                     x86-64-v3 variant: its own target dir (target/x86-64-v3, so the baseline's
#                     artifacts are never rebuilt or overwritten) and `-C target-cpu=x86-64-v3`
#                     added to target.T.rustflags through --config. --config merges with
#                     .cargo/config.toml's target flags (musl's -crt-static stays) where RUSTFLAGS
#                     would replace them, and with --target it never reaches build scripts, so a
#                     build host without v3 still builds it.
#   cpu_level_of F    prints the cpu level a deeptime_jni file was compiled for (`x86-64-v3` or
#                     `baseline`), from the literal its version() string carries (deeptime-jni
#                     CPU_LEVEL); empty if neither is found
#   v3_insn_count F   how many BMI1/BMI2 instructions F has (shlx, shrx, sarx, rorx, andn, mulx,
#                     bzhi, with or without AT&T's l/q suffix), by llvm-objdump or objdump. LLVM
#                     emits the BMI2 shifts for nearly every variable shift once it may, so a v3
#                     build has them by the thousand (measured 2026-10-18: deeptime_jni 1,620, the
#                     CLI 3,796) and a baseline build none (0 and 0). FMA and AVX2 are no signal:
#                     runtime-dispatched code in dependencies carries them in a baseline build, and
#                     Rust never contracts a multiply and an add by itself
SHIP_PROFILE=ship
V3_LEVEL=x86-64-v3
V3_TARGET_DIR=target/x86-64-v3

v3_cargo_args() {
    V3_ARGS=(--target-dir "$V3_TARGET_DIR" --config "target.$1.rustflags=[\"-C\", \"target-cpu=$V3_LEVEL\"]")
}

# PGO_PKGS: the package set of the instrumented build and of every profile-guided build, the CLI
# alone. A function's profile entry is found by its symbol name, which carries the crate's id, and
# cargo's crate ids depend on the roots of the build and on the target triple: the CLI built alone
# and the library built alone share no names (13,000 functions without data), and no other target's
# build found the Linux profile's names either (build log 2026-10-19). So a profile serves the
# build that made it, on the host triple, and the x86-64-v3 build of the same package.
PGO_PKGS=(-p deeptime-cli)

# pgo_cargo_args T LEVEL PROFDATA: sets PGO_ARGS and PGO_TARGET_DIR for a profile-guided build of
# target T at LEVEL (`baseline` or the x86-64-v3 name) from the merged profile PROFDATA (an
# absolute path: llvm-profdata merge, tools/natives/pgo-profile.sh). `-C profile-use` goes into
# target.T.rustflags through --config, as the v3 flag does (with --target it never reaches build
# scripts); `-pgo-warn-missing-function` makes rustc say, for every function the profile has no
# data for, so a profile that does not fit the source or the target shows. Its own target dir per
# level, so the plain builds' artifacts are never rebuilt.
pgo_cargo_args() {
    local flags='"-C", "profile-use='"$3"'", "-C", "llvm-args=-pgo-warn-missing-function"'
    PGO_TARGET_DIR=target/pgo
    if [ "$2" = "$V3_LEVEL" ]; then
        flags="\"-C\", \"target-cpu=$V3_LEVEL\", $flags"
        PGO_TARGET_DIR=target/pgo-$V3_LEVEL
    fi
    PGO_ARGS=(--target-dir "$PGO_TARGET_DIR" --config "target.$1.rustflags=[$flags]")
}

cpu_level_of() {
    if grep -aq "$V3_LEVEL" "$1"; then echo "$V3_LEVEL"
    elif grep -aq baseline "$1"; then echo baseline
    fi
}

v3_insn_count() {
    local dis
    dis="$(command -v llvm-objdump || command -v objdump || true)"
    [ -n "$dis" ] || { echo "?"; return 0; }
    "$dis" -d --no-show-raw-insn "$1" 2>/dev/null \
        | grep -cE '[[:space:]](shlx|shrx|sarx|rorx|andn|mulx|bzhi)[lq]?[[:space:]]' || true
}
