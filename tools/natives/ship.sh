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
