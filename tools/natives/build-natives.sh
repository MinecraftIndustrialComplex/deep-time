#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cross-builds the Deep Time native library (crate deeptime-jni) for the eight platforms of plan
# §6.4, twice on each x86-64 platform (the baseline and x86-64-v3, decisions log 2026-10-18) with
# the CPU probe (crate deeptime-cpuprobe) beside them, checks each file, and writes the release
# layout that `./gradlew build -Prelease` packs:
#
#   target/natives-dist/<platform key>/baseline/<library>    all eight platforms
#   target/natives-dist/<platform key>/x86-64-v3/<library>   the four x86-64 platforms
#   target/natives-dist/<platform key>/probe/<cpu probe>     the four x86-64 platforms
#   target/natives-dist/natives.json                         the §6.5 manifest (schema 2), all 16
#
#   tools/natives/build-natives.sh [--only <key>[,<key>...]]
#   nix run .#natives [-- --only ...]
#
# Run from the repository root (or anywhere inside it), inside the dev shell or through
# `nix run .#natives`; CI (.github/workflows/natives.yml) runs it with rustup, zig 0.13.0,
# cargo-zigbuild, cargo-xwin and apt's clang/lld/llvm. --only builds a subset for debugging; the
# manifest then lists only that subset, which the Gradle release path refuses.
#
# Everything builds with the `ship` profile (release plus fat LTO in one codegen unit; Cargo.toml,
# tools/natives/ship.sh). The x86-64-v3 build adds `-C target-cpu=x86-64-v3` to the target's
# rustflags through --config and builds into target/x86-64-v3/; the probe is built only for the
# baseline CPU.
#
# Builder per target (plan §6.4):
#   linux gnu     cargo zigbuild --target <triple>.2.17   (glibc floor 2.17)
#   linux musl    cargo zigbuild --target <triple>        (.cargo/config.toml: -crt-static)
#   windows       cargo xwin build --target <triple>       (Microsoft CRT/SDK: NOTICE.md)
#   macos         cargo zigbuild --target <triple>        on every host, macOS included: a plain
#                 `cargo build` in the Nix dev shell on a Mac links the Nix store's libiconv
#                 (/nix/store/…/libiconv.2.dylib), which no player's Mac has. SDKROOT is unset so
#                 a Mac build links zig's bundled stubs, exactly as CI's Linux build does.
#
# Checks (any failure exits non-zero): RUSTFLAGS unset (it would replace the per-target flags of
# .cargo/config.toml and the v3 build's target-cpu); each library reports the CPU level of its
# directory (cpu= in its version string) and, with a disassembler, the x86-64-v3 build carries
# at least 500 BMI1/BMI2 instructions, over four times the baseline's (none); the probe exports
# its JNI function; musl: no NEEDED libc.so.6 or ld-linux, no GLIBC_ versions; gnu: highest GLIBC_
# version <= 2.17; macOS: minimum OS <= 11.7.1 (zig 0.13's floor), install name @rpath/<file> and
# only /usr/lib or /System dylibs; the file's format and machine as expected.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"
grep -q '^\[workspace.metadata.deeptime\]' Cargo.toml 2>/dev/null \
    || { echo "build-natives: run inside the deep-time repository (no Cargo.toml workspace here)" >&2; exit 2; }
# The repository's copy (`nix run .#natives` runs this script from the Nix store).
# shellcheck source=tools/natives/ship.sh
. tools/natives/ship.sh

# Platform key | rustup target | builder | library file | CPU probe file (x86-64 only)
# (plan §6.4, in its order).
TARGETS=(
    "linux-x86_64-gnu|x86_64-unknown-linux-gnu|zig-gnu|libdeeptime_jni.so|libdeeptime_cpuprobe.so"
    "linux-aarch64-gnu|aarch64-unknown-linux-gnu|zig-gnu|libdeeptime_jni.so|"
    "linux-x86_64-musl|x86_64-unknown-linux-musl|zig|libdeeptime_jni.so|libdeeptime_cpuprobe.so"
    "linux-aarch64-musl|aarch64-unknown-linux-musl|zig|libdeeptime_jni.so|"
    "windows-x86_64|x86_64-pc-windows-msvc|xwin|deeptime_jni.dll|deeptime_cpuprobe.dll"
    "windows-aarch64|aarch64-pc-windows-msvc|xwin|deeptime_jni.dll|"
    "macos-x86_64|x86_64-apple-darwin|macos|libdeeptime_jni.dylib|libdeeptime_cpuprobe.dylib"
    "macos-aarch64|aarch64-apple-darwin|macos|libdeeptime_jni.dylib|"
)
GLIBC_FLOOR=2.17

ONLY=""
while [ $# -gt 0 ]; do
    case "$1" in
        --only) ONLY="$2"; shift 2 ;;
        -h|--help) sed -n '2,42p' "$0"; exit 0 ;;
        *) echo "build-natives: unknown argument $1" >&2; exit 2 ;;
    esac
done

if [ -n "${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}" ]; then
    echo "build-natives: RUSTFLAGS is set (${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}); it would replace" \
         ".cargo/config.toml's per-target flags and the x86-64-v3 build's target-cpu (plan §6.4). Unset it." >&2
    exit 1
fi

# rustc's own minimum macOS versions (x86_64 10.12, aarch64 11.0), set explicitly so a dev shell
# that exports a newer MACOSX_DEPLOYMENT_TARGET cannot raise it. zig 0.13 then raises both to its
# own supported minimum, 11.7.1.
macos_min() { case "$1" in x86_64-*) echo 10.12 ;; *) echo 11.0 ;; esac; }
MACOS_FLOOR=11.7.1

READELF="$(command -v readelf || command -v llvm-readelf || true)"
[ -n "$READELF" ] || { echo "build-natives: needs readelf (binutils) or llvm-readelf" >&2; exit 1; }
OBJDUMP="$(command -v llvm-objdump || true)"

DIST=target/natives-dist
rm -rf "$DIST"
mkdir -p "$DIST"

fail() { echo "build-natives: $*" >&2; exit 1; }

# Highest GLIBC_x.y[.z] version the ELF file requires (empty if none).
max_glibc() { "$READELF" -V --wide "$1" | grep -o 'GLIBC_[0-9][0-9.]*' | sed 's/^GLIBC_//' | sort -V | tail -1; }
version_le() { [ "$(printf '%s\n%s\n' "$1" "$2" | sort -V | head -1)" = "$1" ]; }

check_elf() { # file key triple
    local f="$1" key="$2" triple="$3" needed machine
    machine="$("$READELF" -h "$f" | sed -n 's/^ *Machine: *//p')"
    case "$triple" in
        x86_64-*) [[ "$machine" == *X86-64* ]] || fail "$key: ELF machine is '$machine', expected X86-64" ;;
        aarch64-*) [[ "$machine" == *AArch64* ]] || fail "$key: ELF machine is '$machine', expected AArch64" ;;
    esac
    needed="$("$READELF" -d --wide "$f" | sed -n 's/.*(NEEDED).*\[\(.*\)\]/\1/p' | tr '\n' ' ')"
    echo "  NEEDED: $needed"
    # On the triple, not the key: place() passes "<key>/<level dir>", which no *-musl or *-gnu
    # pattern matched, so neither check ever ran (found by the mic-native copy, 2026-10-20).
    case "$triple" in
        *-musl)
            [[ " $needed " != *" libc.so.6 "* ]] || fail "$key: NEEDED libc.so.6 (glibc) in a musl build"
            [[ "$needed" != *ld-linux* ]] || fail "$key: NEEDED ld-linux in a musl build"
            [ -z "$(max_glibc "$f")" ] || fail "$key: GLIBC_ symbol versions in a musl build"
            ;;
        *-gnu)
            local g; g="$(max_glibc "$f")"
            echo "  highest GLIBC_ version: ${g:-none}"
            [ -z "$g" ] || version_le "$g" "$GLIBC_FLOOR" || fail "$key: needs GLIBC_$g, above the $GLIBC_FLOOR floor"
            ;;
    esac
}

check_pe() { # file key triple
    local f="$1" key="$2" triple="$3" magic
    # PE: "MZ", e_lfanew at 0x3c, then "PE\0\0" and the COFF machine (0x8664 x64, 0xaa64 arm64).
    magic="$(python3 - "$f" <<'EOF'
import struct, sys
b = open(sys.argv[1], 'rb').read(4096)
off = struct.unpack_from('<I', b, 0x3c)[0]
assert b[:2] == b'MZ' and b[off:off + 4] == b'PE\0\0', 'not a PE file'
print(hex(struct.unpack_from('<H', b, off + 4)[0]))
EOF
)" || fail "$key: not a PE file"
    case "$triple" in
        x86_64-*) [ "$magic" = 0x8664 ] || fail "$key: PE machine $magic, expected 0x8664 (x86-64)" ;;
        aarch64-*) [ "$magic" = 0xaa64 ] || fail "$key: PE machine $magic, expected 0xaa64 (arm64)" ;;
    esac
    [ -n "$OBJDUMP" ] && echo "  imports: $("$OBJDUMP" -p "$f" | sed -n 's/^ *DLL Name: //p' | tr '\n' ' ')"
    return 0
}

check_macho() { # file key triple
    local f="$1" key="$2" triple="$3" minos libs id
    [ -n "$OBJDUMP" ] || { echo "  (no llvm-objdump: Mach-O checks skipped)"; return 0; }
    "$OBJDUMP" --macho --private-header "$f" | grep -q "$( [[ $triple == x86_64-* ]] && echo X86_64 || echo ARM64 )" \
        || fail "$key: Mach-O CPU type is not $triple's"
    minos="$("$OBJDUMP" --macho --private-headers "$f" | awk '/LC_BUILD_VERSION/{b=1} b&&/minos/{print $2; exit} /LC_VERSION_MIN_MACOSX/{v=1} v&&/version/{print $2; exit}')"
    echo "  minimum macOS: ${minos:-unknown}"
    [ -z "$minos" ] || version_le "$minos" "$MACOS_FLOOR" || fail "$key: minimum macOS $minos is above $MACOS_FLOOR"
    # Its own install name (set per library by its crate's build.rs), never the build machine's path.
    id="$("$OBJDUMP" --macho --dylib-id "$f" | tail -n 1)"
    echo "  install name: $id"
    [ "$id" = "@rpath/$(basename "$f")" ] || fail "$key: install name $id, expected @rpath/$(basename "$f")"
    # The first entry is the library's own install name.
    libs="$("$OBJDUMP" --macho --dylibs-used "$f" | tail -n +3 | awk '{print $1}' | tr '\n' ' ')"
    echo "  dylibs: $libs"
    for l in $libs; do
        case "$l" in /usr/lib/*|/System/Library/*) ;; *) fail "$key: links $l, which is not part of macOS" ;; esac
    done
}

build() { # builder triple cargo args...
    local builder=$1 triple=$2; shift 2
    local args=(--profile "$SHIP_PROFILE" --locked "$@")
    case "$builder" in
        zig-gnu) cargo zigbuild "${args[@]}" --target "$triple.$GLIBC_FLOOR" ;;
        zig) cargo zigbuild "${args[@]}" --target "$triple" ;;
        xwin) cargo xwin build "${args[@]}" --target "$triple" ;;
        macos) env -u SDKROOT MACOSX_DEPLOYMENT_TARGET="$(macos_min "$triple")" \
                   cargo zigbuild "${args[@]}" --target "$triple" ;;
    esac
}

place() { # src key dir triple: copies a built file to $DIST/<key>/<dir>/ and checks its format
    local src="$1" key="$2" dir="$3" triple="$4" dest
    [ -f "$src" ] || fail "$key/$dir: cargo produced no $src"
    mkdir -p "$DIST/$key/$dir"
    dest="$DIST/$key/$dir/$(basename "$src")"
    cp "$src" "$dest"
    echo "  $dest: $(stat -c %s "$dest" 2>/dev/null || wc -c <"$dest") bytes"
    case "$triple" in
        *-linux-*) check_elf "$dest" "$key/$dir" "$triple" ;;
        *-windows-*) check_pe "$dest" "$key/$dir" "$triple" ;;
        *-darwin) check_macho "$dest" "$key/$dir" "$triple" ;;
    esac
    PLACED="$dest"
}

check_level() { # file key variant: the library reports the CPU level of its directory
    local level; level="$(cpu_level_of "$1")"
    echo "  cpu level (version string): ${level:-none}"
    [ "$level" = "$3" ] || fail "$2/$3: the library reports cpu=${level:-nothing}, expected $3"
}

built=0
for row in "${TARGETS[@]}"; do
    IFS='|' read -r key triple builder file probe <<<"$row"
    if [ -n "$ONLY" ] && [[ ",$ONLY," != *",$key,"* ]]; then continue; fi
    pkgs=(-p deeptime-jni)
    [ -z "$probe" ] || pkgs+=(-p deeptime-cpuprobe)

    echo "== $key baseline ($triple, profile $SHIP_PROFILE)"
    build "$builder" "$triple" "${pkgs[@]}"
    place "target/$triple/$SHIP_PROFILE/$file" "$key" baseline "$triple"
    check_level "$PLACED" "$key" baseline
    base_count="$(v3_insn_count "$PLACED")"
    echo "  BMI1/BMI2 instructions: $base_count"

    if [ -n "$probe" ]; then
        echo "== $key CPU probe"
        place "target/$triple/$SHIP_PROFILE/$probe" "$key" probe "$triple"
        grep -aq Java_com_minecraftindustrialcomplex_deeptime_nativebridge_CpuProbe_probe "$PLACED" \
            || fail "$key/probe: no JNI function CpuProbe.probe in $PLACED"

        echo "== $key $V3_LEVEL ($triple, profile $SHIP_PROFILE)"
        v3_cargo_args "$triple"
        build "$builder" "$triple" -p deeptime-jni "${V3_ARGS[@]}"
        place "$V3_TARGET_DIR/$triple/$SHIP_PROFILE/$file" "$key" "$V3_LEVEL" "$triple"
        check_level "$PLACED" "$key" "$V3_LEVEL"
        v3_count="$(v3_insn_count "$PLACED")"
        echo "  BMI1/BMI2 instructions: $v3_count (baseline: $base_count)"
        # A v3 build whose target-cpu was dropped (a builder that replaced the rustflags) has the
        # baseline's count, none; a real one has them by the thousand (1,620 on linux-x86_64-gnu).
        if [ "$v3_count" != "?" ] && [ "$base_count" != "?" ]; then
            [ "$v3_count" -ge 500 ] && [ "$v3_count" -gt $((base_count * 4)) ] \
                || fail "$key/$V3_LEVEL: $v3_count BMI1/BMI2 instructions against the baseline's $base_count; was -C target-cpu=$V3_LEVEL dropped?"
        fi
    fi
    built=$((built + 1))
done
[ "$built" -gt 0 ] || fail "--only '$ONLY' names no platform key"

# natives.json, plan §6.5 schema 2 (the schema Gradle's hostNatives writes and NativeLoader reads).
# glibc_floor uses the same scan as hostNatives: the highest GLIBC_x.y[.z] string in the file.
GIT_SHA="$(git rev-parse --short HEAD 2>/dev/null || true)"
python3 - "$DIST" "$GIT_SHA" "$V3_LEVEL" "${TARGETS[@]}" <<'EOF'
import hashlib, json, os, re, sys

dist, git_sha, v3, rows = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4:]

def cargo_value(section, key):
    inside = False
    for line in open('Cargo.toml', encoding='utf-8'):
        line = line.strip()
        if line.startswith('['):
            inside = line == f'[{section}]'
            continue
        m = re.match(rf'^{key}\s*=\s*"([^"]*)"', line) if inside else None
        if m:
            return m.group(1)
    sys.exit(f'build-natives: no {key} in [{section}] of Cargo.toml')

def entry(key, triple, path, variant):
    data = open(path, 'rb').read()
    floor = None
    if triple.endswith('-linux-gnu'):
        vs = [tuple(int(p or 0) for p in m) for m in re.findall(rb'GLIBC_(\d+)\.(\d+)(?:\.(\d+))?', data)]
        if vs:
            a, b, c = max(vs)
            floor = f'{a}.{b}' if c == 0 else f'{a}.{b}.{c}'
    e = {'platform': key}
    if variant:
        e['variant'] = variant
    e.update({
        'file': os.path.basename(path),
        'rust_target': triple,
        'glibc_floor': floor,
        'sha256': hashlib.sha256(data).hexdigest(),
        'size': len(data),
    })
    return e

libs, probes = [], []
for row in rows:
    key, triple, _builder, name, probe = row.split('|')
    for variant in ('baseline', v3):
        path = os.path.join(dist, key, variant, name)
        if os.path.isfile(path):
            libs.append(entry(key, triple, path, variant))
    if probe and os.path.isfile(os.path.join(dist, key, 'probe', probe)):
        probes.append(entry(key, triple, os.path.join(dist, key, 'probe', probe), None))

manifest = {
    'schema': 2,
    'version': cargo_value('workspace.package', 'version'),
    'format_version': cargo_value('workspace.metadata.deeptime', 'format_version'),
}
if git_sha:
    manifest['git_sha'] = git_sha
manifest['libraries'] = libs
manifest['probes'] = probes
with open(os.path.join(dist, 'natives.json'), 'w', encoding='utf-8') as f:
    json.dump(manifest, f, indent=2)
    f.write('\n')
print(f"natives.json: {len(libs)} libraries and {len(probes)} CPU probes, version {manifest['version']}, "
      f"format {manifest['format_version']}")
for l in libs + probes:
    print(f"  {l['platform']:<20} {l.get('variant', 'probe'):<10} {l['file']:<26} {l['size']:>9} bytes  "
          f"sha256 {l['sha256'][:12]}  glibc {l['glibc_floor'] or '-'}")
EOF

if command -v file >/dev/null; then
    (cd "$DIST" && file */*/*)
fi
