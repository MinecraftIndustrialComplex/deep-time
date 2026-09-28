#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Installs one of Deep Time's server sets into smoke/<profile>/server/ (plan §9.5).
# Adapted from mic-climate's smoke/install.sh.
#
#   tools/smoke/install.sh <standalone|mic|terrablender|ev>
#       [--size 16384|32768]      world circumference -> level-type deeptime:planet[_32k]
#       [--synth <kind>]          write synthetic world data before the first start (P0-09+)
#       [--test-terrain <kind>]   -Ddeeptime.testTerrain=<kind> (the analytic terrain, P0-05+)
#       [--preset <p>]            config/deeptime/server.json preset (P1-13+)
#       [--pack-worktree]         mic only: install the pack's working tree instead of its last
#                                 commit (other sessions edit the working tree; HEAD is the default)
#       [--clean]                 throw the server away first
#
# Idempotent: downloads are cached under smoke/cache/ and packwiz-installer syncs, so a
# re-run picks up exactly what the pack pins today. NOTHING under pack/ is written, and
# nothing under mods/mic-climate is read or written.
#
# The server set is also the jar folder the dev runs link for -PwithMic / -PwithTerraBlender
# / -PwithEv and the planet server (neoforge/build.gradle, linkRunMods*).

set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
PACK="$(cd "$REPO/../../pack" && pwd)"
PROFILES="$REPO/tools/smoke/profiles"
CACHE="$REPO/smoke/cache"

BOOTSTRAP_URL='https://github.com/packwiz/packwiz-installer-bootstrap/releases/download/v0.0.3/packwiz-installer-bootstrap.jar'
NEOFORGE_MAVEN='https://maven.neoforged.net/releases/net/neoforged/neoforge'

# Deep Time's own ports, clear of mic-climate's smoke (25575) and the default 25565
# (cold-read review 2, gap 24). The planet server uses +1 (gametest-planet.sh).
GAME_PORT=25665
RCON_PORT=25675

PROFILE="${1:-}"; shift || true
SIZE=16384; SYNTH=""; TEST_TERRAIN=""; PRESET=""; CLEAN=0; PACK_WORKTREE=0
while [ $# -gt 0 ]; do
    case "$1" in
        --size) SIZE="$2"; shift 2 ;;
        --synth) SYNTH="$2"; shift 2 ;;
        --test-terrain) TEST_TERRAIN="$2"; shift 2 ;;
        --preset) PRESET="$2"; shift 2 ;;
        --clean) CLEAN=1; shift ;;
        --pack-worktree) PACK_WORKTREE=1; shift ;;
        *) echo "install.sh: unknown argument '$1'" >&2; exit 2 ;;
    esac
done
case "$PROFILE" in
    standalone|mic|terrablender|ev) ;;
    *) echo "usage: install.sh <standalone|mic|terrablender|ev> [--size N] [--synth K] [--test-terrain K] [--preset P] [--clean]" >&2; exit 2 ;;
esac
case "$SIZE" in
    16384) LEVEL_TYPE='deeptime\:planet' ;;
    32768) LEVEL_TYPE='deeptime\:planet_32k' ;;
    *) echo "install.sh: --size must be 16384 or 32768" >&2; exit 2 ;;
esac

SMOKE="$REPO/smoke/$PROFILE"
SERVER="$SMOKE/server"
say() { printf '\n== %s\n' "$*"; }
dev() { if [ -n "${IN_NIX_SHELL:-}" ]; then "$@"; else nix develop -c "$@"; fi; }

[ -f "$PACK/pack.toml" ] || { echo "install.sh: no pack at $PACK" >&2; exit 1; }
if [ "$CLEAN" = 1 ]; then say "removing $SERVER"; rm -rf "$SERVER"; fi
mkdir -p "$CACHE/modrinth" "$SERVER/mods"

# ---------------------------------------------------------------------------
# 0. Tools, resolved once by absolute path (run.py is stdlib-only).
# ---------------------------------------------------------------------------
if [ ! -f "$REPO/smoke/tools.env" ] || ! . "$REPO/smoke/tools.env" 2>/dev/null || [ ! -x "${JAVA:-}" ] || [ ! -x "${MCRCON:-}" ]; then
    say "resolving jdk21 and mcrcon from nixpkgs"
    JAVA="$(nix shell nixpkgs#jdk21 -c sh -c 'readlink -f "$(command -v java)"')"
    MCRCON="$(nix shell nixpkgs#mcrcon -c sh -c 'readlink -f "$(command -v mcrcon)"')"
    printf 'JAVA=%s\nMCRCON=%s\n' "$JAVA" "$MCRCON" > "$REPO/smoke/tools.env"
fi
. "$REPO/smoke/tools.env"

# ---------------------------------------------------------------------------
# 1. The pack's state: NeoForge pin, and exactly which pack was read.
# ---------------------------------------------------------------------------
NEO="$(python3 - "$PACK/pack.toml" <<'PY'
import re, sys
text = open(sys.argv[1]).read()
section = text.split('[versions]', 1)[1]
print(re.search(r'^\s*neoforge\s*=\s*"([^"]+)"', section, re.M).group(1))
PY
)"
PACK_DESC="$(git -C "$PACK" describe --always --dirty 2>/dev/null || echo unknown)"
PACK_DIRTY_HASH=""
case "$PACK_DESC" in *-dirty) PACK_DIRTY_HASH="$(git -C "$PACK" status --porcelain | sha256sum | cut -c1-16)" ;; esac
say "pack $PACK_DESC ${PACK_DIRTY_HASH:+(working-tree hash $PACK_DIRTY_HASH) }pins NeoForge $NEO"

# ---------------------------------------------------------------------------
# 2. NeoForge server.
# ---------------------------------------------------------------------------
ARGS_FILE="$SERVER/libraries/net/neoforged/neoforge/$NEO/unix_args.txt"
if [ ! -f "$ARGS_FILE" ]; then
    INSTALLER="$CACHE/neoforge-$NEO-installer.jar"
    if [ ! -f "$INSTALLER" ]; then
        say "downloading the NeoForge $NEO installer"
        curl -fsSL -o "$INSTALLER.part" "$NEOFORGE_MAVEN/$NEO/neoforge-$NEO-installer.jar"
        mv "$INSTALLER.part" "$INSTALLER"
    fi
    say "installing the NeoForge $NEO server"
    ( cd "$SERVER" && "$JAVA" -jar "$INSTALLER" --installServer "$SERVER" >/dev/null )
    [ -f "$ARGS_FILE" ] || { echo "install.sh: the installer produced no $ARGS_FILE" >&2; exit 1; }
fi

# ---------------------------------------------------------------------------
# 3. Mods.
# ---------------------------------------------------------------------------

# modrinth_fetch <version id> -> prints the cached jar path (sha512-checked).
modrinth_fetch() {
    python3 - "$1" "$CACHE/modrinth" <<'PY'
import hashlib, json, os, sys, urllib.request
vid, cache = sys.argv[1], sys.argv[2]
meta_path = os.path.join(cache, vid + ".json")
if not os.path.exists(meta_path):
    req = urllib.request.Request(f"https://api.modrinth.com/v2/version/{vid}",
                                 headers={"User-Agent": "MinecraftIndustrialComplex/deep-time smoke"})
    data = urllib.request.urlopen(req, timeout=60).read()
    open(meta_path, "wb").write(data)
meta = json.load(open(meta_path))
f = next((f for f in meta["files"] if f.get("primary")), meta["files"][0])
path = os.path.join(cache, f["filename"])
def ok(p):
    return os.path.exists(p) and hashlib.sha512(open(p, "rb").read()).hexdigest() == f["hashes"]["sha512"]
if not ok(path):
    req = urllib.request.Request(f["url"], headers={"User-Agent": "MinecraftIndustrialComplex/deep-time smoke"})
    open(path + ".part", "wb").write(urllib.request.urlopen(req, timeout=300).read())
    os.replace(path + ".part", path)
    if not ok(path):
        sys.exit(f"install.sh: sha512 mismatch for {f['filename']} ({vid})")
print(path)
PY
}

# profile_lines <profile> -> "<slug> <id>" lines, following @include.
profile_lines() {
    local file="$PROFILES/$1.txt"
    while IFS= read -r line; do
        line="${line%%#*}"
        set -- $line
        [ $# -eq 0 ] && continue
        if [ "$1" = "@include" ]; then profile_lines "$2"; else echo "$1 $2"; fi
    done < "$file"
}

# Deep Time and TW are always re-placed; nothing else of ours is left behind.
find "$SERVER/mods" -maxdepth 1 \( -name 'deeptime-*.jar' -o -name 'toroidal_world*.jar' \) -delete

if [ "$PROFILE" = mic ]; then
    BOOTSTRAP="$CACHE/packwiz-installer-bootstrap.jar"
    if [ ! -f "$BOOTSTRAP" ]; then
        say "downloading packwiz-installer-bootstrap v0.0.3"
        curl -fsSL -o "$BOOTSTRAP.part" "$BOOTSTRAP_URL"; mv "$BOOTSTRAP.part" "$BOOTSTRAP"
    fi
    # Install from a snapshot, never from pack/ itself: other sessions edit the pack's working
    # tree, and packwiz-installer rejects any file whose hash the index does not match. The
    # snapshot is the last commit (or the working tree with --pack-worktree), re-indexed with
    # `packwiz refresh` so a stale committed hash cannot block the install; every hash the
    # refresh changed is recorded in install.json as a pack finding.
    SNAP="$CACHE/pack-snapshot"
    rm -rf "$SNAP" && mkdir -p "$SNAP"
    if [ "$PACK_WORKTREE" = 1 ]; then
        PACK_REF="worktree:$PACK_DESC"
        ( cd "$PACK" && git ls-files -z --cached --others --exclude-standard | tar --null -T - -cf - ) | tar -xf - -C "$SNAP"
    else
        PACK_REF="$(git -C "$PACK" rev-parse --short HEAD)"
        git -C "$PACK" archive HEAD | tar -xf - -C "$SNAP"
    fi
    cp "$SNAP/index.toml" "$CACHE/pack-index.toml.orig"
    ( cd "$SNAP" && nix run nixpkgs#packwiz -- refresh >/dev/null )
    PACK_STALE="$(python3 - "$CACHE/pack-index.toml.orig" "$SNAP/index.toml" <<'PY2'
import re, sys
def entries(p):
    return dict(re.findall(r'file = "([^"]+)"\nhash = "([^"]+)"', open(p).read()))
a, b = entries(sys.argv[1]), entries(sys.argv[2])
print(",".join(sorted(f for f in b if a.get(f) != b[f])))
PY2
)"
    [ -n "$PACK_STALE" ] && echo "   PACK FINDING: the pack index had stale hashes for: $PACK_STALE"
    say "installing the MIC pack (side = server) from a snapshot of $PACK_REF"
    ( cd "$SERVER" && "$JAVA" -jar "$BOOTSTRAP" -g -s server "$SNAP/pack.toml" )
    # A pack copy of Deep Time or Toroidal World (plan Q11 recommends pinning TW in the
    # pack) would be a duplicate mod id next to ours.
    find "$SERVER/mods" -maxdepth 1 \( -iname 'deeptime-*.jar' -o -iname 'toroidal*world*.jar' \) -print -delete
else
    # A non-pack set is exactly its pin file: stale jars from an older pin list go.
    find "$SERVER/mods" -maxdepth 1 -name '*.jar' -delete
fi

say "installing pinned jars from profiles/$PROFILE.txt"
while read -r slug vid; do
    jar="$(modrinth_fetch "$vid")"
    cp "$jar" "$SERVER/mods/"
    echo "   $slug $vid -> $(basename "$jar")"
    # Pins that follow the pack must stay equal to the pack's.
    if [ "$PROFILE" != mic ] && [ -f "$PACK/mods/$slug.pw.toml" ]; then
        packvid="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$PACK/mods/$slug.pw.toml")"
        [ -n "$packvid" ] && [ "$packvid" != "$vid" ] && echo "   WARNING: the pack pins $slug at $packvid, profiles/$PROFILE.txt at $vid"
    fi
done < <(profile_lines "$PROFILE")

JAR="$(find "$REPO/neoforge/build/libs" -maxdepth 1 -name 'deeptime-1.21.1-*.jar' ! -name '*-sources.jar' 2>/dev/null | sort | tail -1)"
[ -n "$JAR" ] || { echo "install.sh: no Deep Time jar -- run (cd neoforge && nix develop -c ./gradlew build) first" >&2; exit 1; }
cp "$JAR" "$SERVER/mods/"
say "installed $(basename "$JAR")"

# Exclusions -- recorded, never silent.
mkdir -p "$SERVER/mods-excluded"
find "$SERVER/mods-excluded" -maxdepth 1 -name '*.jar' -exec mv -t "$SERVER/mods" {} + 2>/dev/null || true
: > "$SMOKE/excluded.txt"
while IFS= read -r pattern; do
    case "$pattern" in ''|'#'*) continue ;; esac
    while IFS= read -r hit; do
        mv "$hit" "$SERVER/mods-excluded/"; echo "$(basename "$hit")" >> "$SMOKE/excluded.txt"
    done < <(find "$SERVER/mods" -maxdepth 1 -name "$pattern")
done < "$PROFILES/$PROFILE.excluded"

# ---------------------------------------------------------------------------
# 4. Server configuration.
# ---------------------------------------------------------------------------
echo 'eula=true' > "$SERVER/eula.txt"   # Ben accepted the EULA for test servers (plan §2.7)
if [ ! -f "$SMOKE/rcon.pw" ]; then
    python3 -c "import secrets; print(secrets.token_hex(16))" > "$SMOKE/rcon.pw"
    chmod 600 "$SMOKE/rcon.pw"
fi
cat > "$SERVER/server.properties" <<EOF
# Written by deep-time/tools/smoke/install.sh -- not a pack file.
server-port=$GAME_PORT
enable-rcon=true
rcon.port=$RCON_PORT
rcon.password=$(cat "$SMOKE/rcon.pw")
broadcast-rcon-to-ops=false
online-mode=false
level-type=$LEVEL_TYPE
level-name=planet
spawn-protection=0
max-tick-time=-1
view-distance=6
simulation-distance=4
enable-command-block=true
sync-chunk-writes=false
motd=Deep Time smoke test ($PROFILE)
EOF
{
    echo '-Xmx6G'
    echo '-XX:+UseG1GC'
    [ -n "$TEST_TERRAIN" ] && echo "-Ddeeptime.testTerrain=$TEST_TERRAIN"
} > "$SERVER/user_jvm_args.txt"
if [ -n "$PRESET" ]; then
    mkdir -p "$SERVER/config/deeptime"
    printf '{"preset": "%s", "memory_cap_gib": 4, "threads": 0}\n' "$PRESET" > "$SERVER/config/deeptime/server.json"
fi

if [ -n "$SYNTH" ]; then
    say "writing synthetic world data ($SYNTH, C=$SIZE)"
    rm -rf "$SERVER/planet"
    ( cd "$REPO" && dev cargo run --release -q -p deeptime-cli -- synth --kind "$SYNTH" --size "$SIZE" --out "$SERVER/planet/deeptime" )
fi

cat > "$SMOKE/install.json" <<EOF
{"profile": "$PROFILE", "size": $SIZE, "level_type": "$(echo "$LEVEL_TYPE" | tr -d '\\')", "synth": "$SYNTH",
 "test_terrain": "$TEST_TERRAIN", "neoforge": "$NEO", "pack": "$PACK_DESC", "pack_dirty_hash": "$PACK_DIRTY_HASH",
 "pack_ref": "${PACK_REF:-}", "pack_stale_hashes": "${PACK_STALE:-}",
 "deeptime_jar": "$(basename "$JAR")", "game_port": $GAME_PORT, "rcon_port": $RCON_PORT}
EOF
echo "$NEO" > "$SMOKE/neoforge.version"
say "ready: $SERVER (mods: $(find "$SERVER/mods" -maxdepth 1 -name '*.jar' | wc -l), excluded: $(wc -l < "$SMOKE/excluded.txt"))"
