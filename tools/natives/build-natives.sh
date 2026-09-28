#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Cross-builds the Deep Time native library (crate deeptime-jni) for the eight platforms of plan
# §6.4, checks each file, and writes the release layout that `./gradlew build -Prelease` packs:
#
#   target/natives-dist/<platform key>/<library>   eight libraries
#   target/natives-dist/natives.json               the §6.5 manifest (schema 1), all eight
#
#   tools/natives/build-natives.sh [--only <key>[,<key>...]]
#   nix run .#natives [-- --only ...]
#
# Run from the repository root (or anywhere inside it), inside the dev shell or through
# `nix run .#natives`; CI (.github/workflows/natives.yml) runs it with rustup, zig 0.13.0,
# cargo-zigbuild, cargo-xwin and apt's clang/lld/llvm. --only builds a subset for debugging; the
# manifest then lists only that subset, which the Gradle release path refuses.
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
# Checks (any failure exits non-zero): no -C target-cpu / +avx in RUSTFLAGS (the conservative CPU
# baseline of §6.4); musl: no NEEDED libc.so.6 or ld-linux, no GLIBC_ versions; gnu: highest
# GLIBC_ version <= 2.17; macOS: minimum OS <= 11.7.1 (zig 0.13's floor) and only /usr/lib or
# /System dylibs; the file's format and machine as expected.

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"
grep -q '^\[workspace.metadata.deeptime\]' Cargo.toml 2>/dev/null \
    || { echo "build-natives: run inside the deep-time repository (no Cargo.toml workspace here)" >&2; exit 2; }

# Platform key | rustup target | builder | file (plan §6.4, in its order).
TARGETS=(
    "linux-x86_64-gnu|x86_64-unknown-linux-gnu|zig-gnu|libdeeptime_jni.so"
    "linux-aarch64-gnu|aarch64-unknown-linux-gnu|zig-gnu|libdeeptime_jni.so"
    "linux-x86_64-musl|x86_64-unknown-linux-musl|zig|libdeeptime_jni.so"
    "linux-aarch64-musl|aarch64-unknown-linux-musl|zig|libdeeptime_jni.so"
    "windows-x86_64|x86_64-pc-windows-msvc|xwin|deeptime_jni.dll"
    "windows-aarch64|aarch64-pc-windows-msvc|xwin|deeptime_jni.dll"
    "macos-x86_64|x86_64-apple-darwin|macos|libdeeptime_jni.dylib"
    "macos-aarch64|aarch64-apple-darwin|macos|libdeeptime_jni.dylib"
)
GLIBC_FLOOR=2.17

ONLY=""
while [ $# -gt 0 ]; do
    case "$1" in
        --only) ONLY="$2"; shift 2 ;;
        -h|--help) sed -n '2,27p' "$0"; exit 0 ;;
        *) echo "build-natives: unknown argument $1" >&2; exit 2 ;;
    esac
done

case "${RUSTFLAGS:-} ${CARGO_ENCODED_RUSTFLAGS:-}" in
    *target-cpu*|*avx*) echo "build-natives: RUSTFLAGS sets a CPU baseline (${RUSTFLAGS:-}); plan §6.4 forbids it" >&2; exit 1 ;;
esac

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
    case "$key" in
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
    local f="$1" key="$2" triple="$3" minos libs
    [ -n "$OBJDUMP" ] || { echo "  (no llvm-objdump: Mach-O checks skipped)"; return 0; }
    "$OBJDUMP" --macho --private-header "$f" | grep -q "$( [[ $triple == x86_64-* ]] && echo X86_64 || echo ARM64 )" \
        || fail "$key: Mach-O CPU type is not $triple's"
    minos="$("$OBJDUMP" --macho --private-headers "$f" | awk '/LC_BUILD_VERSION/{b=1} b&&/minos/{print $2; exit} /LC_VERSION_MIN_MACOSX/{v=1} v&&/version/{print $2; exit}')"
    echo "  minimum macOS: ${minos:-unknown}"
    [ -z "$minos" ] || version_le "$minos" "$MACOS_FLOOR" || fail "$key: minimum macOS $minos is above $MACOS_FLOOR"
    # The first entry is the library's own install name.
    libs="$("$OBJDUMP" --macho --dylibs-used "$f" | tail -n +3 | awk '{print $1}' | tr '\n' ' ')"
    echo "  dylibs: $libs"
    for l in $libs; do
        case "$l" in /usr/lib/*|/System/Library/*) ;; *) fail "$key: links $l, which is not part of macOS" ;; esac
    done
}

built=0
for row in "${TARGETS[@]}"; do
    IFS='|' read -r key triple builder file <<<"$row"
    if [ -n "$ONLY" ] && [[ ",$ONLY," != *",$key,"* ]]; then continue; fi
    echo "== $key ($triple)"
    args=(--release --locked -p deeptime-jni)
    case "$builder" in
        zig-gnu) cargo zigbuild "${args[@]}" --target "$triple.$GLIBC_FLOOR" ;;
        zig) cargo zigbuild "${args[@]}" --target "$triple" ;;
        xwin) cargo xwin build "${args[@]}" --target "$triple" ;;
        macos) env -u SDKROOT MACOSX_DEPLOYMENT_TARGET="$(macos_min "$triple")" \
                   cargo zigbuild "${args[@]}" --target "$triple" ;;
    esac
    src="target/$triple/release/$file"
    [ -f "$src" ] || fail "$key: cargo produced no $src"
    mkdir -p "$DIST/$key"
    cp "$src" "$DIST/$key/$file"
    case "$triple" in
        *-linux-*) check_elf "$DIST/$key/$file" "$key" "$triple" ;;
        *-windows-*) check_pe "$DIST/$key/$file" "$key" "$triple" ;;
        *-darwin) check_macho "$DIST/$key/$file" "$key" "$triple" ;;
    esac
    built=$((built + 1))
done
[ "$built" -gt 0 ] || fail "--only '$ONLY' names no platform key"

# natives.json, plan §6.5 schema 1 (the schema Gradle's hostNatives writes and NativeLoader reads).
# glibc_floor uses the same scan as hostNatives: the highest GLIBC_x.y[.z] string in the file.
GIT_SHA="$(git rev-parse --short HEAD 2>/dev/null || true)"
python3 - "$DIST" "$GIT_SHA" "${TARGETS[@]}" <<'EOF'
import hashlib, json, os, re, sys

dist, git_sha, rows = sys.argv[1], sys.argv[2], sys.argv[3:]

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

libs = []
for row in rows:
    key, triple, _builder, name = row.split('|')
    path = os.path.join(dist, key, name)
    if not os.path.isfile(path):
        continue
    data = open(path, 'rb').read()
    floor = None
    if triple.endswith('-linux-gnu'):
        vs = [tuple(int(p or 0) for p in m) for m in re.findall(rb'GLIBC_(\d+)\.(\d+)(?:\.(\d+))?', data)]
        if vs:
            a, b, c = max(vs)
            floor = f'{a}.{b}' if c == 0 else f'{a}.{b}.{c}'
    libs.append({
        'platform': key,
        'file': name,
        'rust_target': triple,
        'glibc_floor': floor,
        'sha256': hashlib.sha256(data).hexdigest(),
        'size': len(data),
    })

manifest = {
    'schema': 1,
    'version': cargo_value('workspace.package', 'version'),
    'format_version': cargo_value('workspace.metadata.deeptime', 'format_version'),
}
if git_sha:
    manifest['git_sha'] = git_sha
manifest['libraries'] = libs
with open(os.path.join(dist, 'natives.json'), 'w', encoding='utf-8') as f:
    json.dump(manifest, f, indent=2)
    f.write('\n')
print(f"natives.json: {len(libs)} libraries, version {manifest['version']}, format {manifest['format_version']}")
for l in libs:
    print(f"  {l['platform']:<20} {l['file']:<24} {l['size']:>9} bytes  sha256 {l['sha256'][:12]}  glibc {l['glibc_floor'] or '-'}")
EOF

if command -v file >/dev/null; then
    (cd "$DIST" && file */*)
fi
