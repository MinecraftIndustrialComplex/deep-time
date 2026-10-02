#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Installs one of Deep Time's server sets into smoke/<profile>/server/ (plan §9.5).
# Adapted from mic-climate's smoke/install.sh.
#
#   tools/smoke/install.sh <standalone|mic|mic-ev|terrablender|ev|ev-client|dh|dh-next>
#       [--size 16384|32768]      world circumference -> level-type deeptime:planet[_32k]
#       [--synth <kind>]          write synthetic world data before the first start (P0-09+)
#       [--strata <kind>]         the synthetic horizons: placeholder (default), every-class (P1-12) or basin
#       [--datapack <folder>]     copy a datapack folder into the world's datapacks/ (e.g. the MIC
#                                 rock tags, …/mic/Docs/datapacks/deeptime-mic-rock-tags, until the
#                                 pack ships them; P1-12)
#       [--test-terrain <kind>]   -Ddeeptime.testTerrain=<kind> (the analytic terrain, P0-05+)
#       [--preset <p>]            config/deeptime/server.json preset (P1-13+): a built-in name, a
#                                 file or a share code; the server generates on its first start
#       [--start <s>]             its timeline.start (supercontinent | scattered_continents | random)
#       [--length-myr <n>]        its timeline.length_myr. The mod has no default story; this
#                                 harness names supercontinent / 1000 when --preset comes alone
#       [--memory-cap-gib <n>]    its memory_cap_gib (default 4)
#       [--pack-worktree]         mic only: install the pack's working tree instead of its last
#                                 commit (other sessions edit the working tree; HEAD is the default)
#       [--seed <n>]              server.properties level-seed (default: none, a random seed). Two
#                                 sets compared side by side (Q10: mic vs mic-ev) need the same seed,
#                                 or vanilla's features, caves and scatter differ between them
#       [--clean]                 throw the server away first
#
# DEEPTIME_PACK=<dir> installs mic/mic-ev from that pack checkout instead of ../../pack (a git
# checkout with a .git directory: its HEAD, or its working tree with --pack-worktree).
#
# mic-ev is the mic set plus Excavated Variants and Dynamic Asset Generator (profiles/ev.txt's pins),
# for the plan Q10 comparison; ev-client is the ev set plus Create, the smallest set a client needs to
# generate EV's variant textures for every ore and stone the MIC pack has (tools/client/ev-assets.sh).
# dh is the standalone set plus Distant Horizons at the pack's pin, dh-next the same with the newest
# Distant Horizons checked (P3-04: far terrain from the world data; run.py --dh).
#
# install.json records the Deep Time jar's sha256 and the source it was built from (jarstate.py). The
# jar copy goes stale as soon as the sources change; tools/workbench/preview.py refreshes it itself
# (build, compare, replace), so a preview needs no re-install for Java or Rust changes.
#
# Idempotent: downloads are cached under smoke/cache/ and packwiz-installer syncs, so a
# re-run picks up exactly what the pack pins today. NOTHING under pack/ is written, and
# nothing under mods/mic-climate is read or written.
#
# The server set is also the jar folder the dev runs link for -PwithMic / -PwithTerraBlender
# / -PwithEv and the planet server (neoforge/build.gradle, linkRunMods*).

set -euo pipefail

REPO="$(cd "$(dirname "$0")/../.." && pwd)"
# DEEPTIME_PACK: another pack checkout (e.g. a copy of the pack's deep-time branch); default ../../pack.
PACK="$(cd "${DEEPTIME_PACK:-$REPO/../../pack}" && pwd)"
PROFILES="$REPO/tools/smoke/profiles"
CACHE="$REPO/smoke/cache"

BOOTSTRAP_URL='https://github.com/packwiz/packwiz-installer-bootstrap/releases/download/v0.0.3/packwiz-installer-bootstrap.jar'
NEOFORGE_MAVEN='https://maven.neoforged.net/releases/net/neoforged/neoforge'

# Deep Time's own ports, clear of mic-climate's smoke (25575) and the default 25565
# (cold-read review 2, gap 24). The planet server uses +1 (gametest-planet.sh).
GAME_PORT=25665
RCON_PORT=25675

PROFILE="${1:-}"; shift || true
SIZE=16384; SYNTH=""; STRATA=placeholder; DATAPACK=""; TEST_TERRAIN=""; PRESET=""; CLEAN=0; PACK_WORKTREE=0; SEED=""
START=""; LENGTH_MYR=""; MEMORY_CAP_GIB=4
while [ $# -gt 0 ]; do
    case "$1" in
        --size) SIZE="$2"; shift 2 ;;
        --synth) SYNTH="$2"; shift 2 ;;
        --strata) STRATA="$2"; shift 2 ;;
        --datapack) DATAPACK="$2"; shift 2 ;;
        --test-terrain) TEST_TERRAIN="$2"; shift 2 ;;
        --preset) PRESET="$2"; shift 2 ;;
        --start) START="$2"; shift 2 ;;
        --length-myr) LENGTH_MYR="$2"; shift 2 ;;
        --memory-cap-gib) MEMORY_CAP_GIB="$2"; shift 2 ;;
        --seed) SEED="$2"; shift 2 ;;
        --clean) CLEAN=1; shift ;;
        --pack-worktree) PACK_WORKTREE=1; shift ;;
        *) echo "install.sh: unknown argument '$1'" >&2; exit 2 ;;
    esac
done
case "$PROFILE" in
    standalone|mic|mic-ev|terrablender|ev|ev-client|dh|dh-next) ;;
    *) echo "usage: install.sh <standalone|mic|mic-ev|terrablender|ev|ev-client|dh|dh-next> [--size N] [--synth K] [--strata S] [--datapack DIR] [--test-terrain K] [--preset P [--start S] [--length-myr N] [--memory-cap-gib N]] [--seed N] [--clean]" >&2; exit 2 ;;
esac
# The sets installed from the MIC pack (packwiz-installer); the others are their pin files alone.
case "$PROFILE" in mic|mic-ev) FROM_PACK=1 ;; *) FROM_PACK=0 ;; esac
case "$SEED" in ''|-[0-9]*|[0-9]*) ;; *) echo "install.sh: --seed must be a number" >&2; exit 2 ;; esac
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

# profile_lines <profile> -> "<slug> <id>" lines, following @include; profile_pins lists a pin that
# two includes share once (mic-ev includes mic and ev, both of which pin Toroidal World).
profile_lines() {
    local file="$PROFILES/$1.txt"
    while IFS= read -r line; do
        line="${line%%#*}"
        set -- $line
        [ $# -eq 0 ] && continue
        if [ "$1" = "@include" ]; then profile_lines "$2"; else echo "$1 $2"; fi
    done < "$file"
}
profile_pins() { profile_lines "$1" | awk '!seen[$0]++'; }

# Deep Time and TW are always re-placed; nothing else of ours is left behind.
find "$SERVER/mods" -maxdepth 1 \( -name 'deeptime-*.jar' -o -name 'toroidal_world*.jar' \) -delete

if [ "$FROM_PACK" = 1 ]; then
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
    # Fix stale hashes only: recompute the sha256 of every file the index already lists, then the
    # index's own hash in pack.toml. (A full `packwiz refresh` would also add committed .pw.toml files
    # that the index leaves out -- the pack has one, mods/rpl.pw.toml -- and so install jars real
    # players never get.)
    PACK_STALE="$(python3 - "$SNAP" "$PACK" <<'PY2'
import hashlib, os, re, shutil, sys
snap, pack = sys.argv[1], sys.argv[2]
idx_path = os.path.join(snap, "index.toml")
idx = open(idx_path).read()
stale = []
# The index lists a few raw jars that the pack's .gitignore keeps out of git (Destroy, Dice,
# Playing Cards on 2026-09-28): they exist only in the pack's working tree, so copy them in.
for rel in re.findall(r'file = "([^"]+)"', idx):
    dst = os.path.join(snap, rel)
    src = os.path.join(pack, rel)
    if not os.path.exists(dst) and os.path.exists(src):
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        shutil.copy2(src, dst)
        print(f"   copied untracked pack file {rel}", file=sys.stderr)
def fix(m):
    rel, old = m.group(1), m.group(2)
    path = os.path.join(snap, rel)
    if not os.path.exists(path):
        return m.group(0)
    new = hashlib.sha256(open(path, "rb").read()).hexdigest()
    if new != old:
        stale.append(rel)
    return m.group(0).replace(old, new)
idx = re.sub(r'file = "([^"]+)"\nhash = "([0-9a-f]+)"', fix, idx)
open(idx_path, "w").write(idx)
pack_path = os.path.join(snap, "pack.toml")
pack = open(pack_path).read()
idx_hash = hashlib.sha256(idx.encode()).hexdigest()
pack = re.sub(r'(\[index\][^\[]*?hash = ")[0-9a-f]+(")', lambda m: m.group(1) + idx_hash + m.group(2), pack, flags=re.S)
open(pack_path, "w").write(pack)
print(",".join(stale))
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
    if [ "$FROM_PACK" = 0 ] && [ -f "$PACK/mods/$slug.pw.toml" ]; then
        packvid="$(sed -n 's/^version *= *"\(.*\)"/\1/p' "$PACK/mods/$slug.pw.toml")"
        [ -n "$packvid" ] && [ "$packvid" != "$vid" ] && echo "   WARNING: the pack pins $slug at $packvid, profiles/$PROFILE.txt at $vid"
    fi
done < <(profile_pins "$PROFILE")

# Always the current sources: an old jar left in build/libs once installed pre-P0-10 code.
say "building the Deep Time jar"
( cd "$REPO/neoforge" && dev ./gradlew jar -q --console=plain )
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
[ -n "$SEED" ] && echo "level-seed=$SEED" >> "$SERVER/server.properties"
{
    echo '-Xmx6G'
    echo '-XX:+UseG1GC'
    [ -n "$TEST_TERRAIN" ] && echo "-Ddeeptime.testTerrain=$TEST_TERRAIN"
} > "$SERVER/user_jvm_args.txt"
# config/deeptime/server.json (plan §7.13, P1-13): exactly what this install asked for, so a stale
# file from an earlier install never decides a first start.
rm -f "$SERVER/config/deeptime/server.json"
if [ -n "$PRESET" ]; then
    [ -n "$START" ] || START=supercontinent
    [ -n "$LENGTH_MYR" ] || LENGTH_MYR=1000
    mkdir -p "$SERVER/config/deeptime"
    python3 - "$SERVER/config/deeptime/server.json" "$PRESET" "$START" "$LENGTH_MYR" "$MEMORY_CAP_GIB" <<'PY'
import json, sys
path, preset, start, length, cap = sys.argv[1:]
cfg = {"_about": "Written by deep-time/tools/smoke/install.sh (P1-13 smoke).",
       "preset": preset,
       "timeline": {"start": start, "length_myr": float(length)},
       "memory_cap_gib": float(cap), "threads": 0}
open(path, "w").write(json.dumps(cfg, indent=2) + "\n")
PY
    echo "   config/deeptime/server.json: preset $PRESET, timeline $START / $LENGTH_MYR Myr"
fi

if [ -n "$SYNTH" ]; then
    say "writing synthetic world data ($SYNTH, C=$SIZE)"
    rm -rf "$SERVER/planet"
    ( cd "$REPO" && dev cargo run --release -q -p deeptime-cli -- synth --kind "$SYNTH" --strata "$STRATA" --size "$SIZE" --out "$SERVER/planet/deeptime" )
fi
if [ -n "$DATAPACK" ]; then
    [ -f "$DATAPACK/pack.mcmeta" ] || { echo "install.sh: --datapack $DATAPACK has no pack.mcmeta" >&2; exit 1; }
    say "world datapack $(basename "$DATAPACK")"
    mkdir -p "$SERVER/planet/datapacks"
    rm -rf "$SERVER/planet/datapacks/$(basename "$DATAPACK")"
    cp -R "$DATAPACK" "$SERVER/planet/datapacks/$(basename "$DATAPACK")"
fi

cat > "$SMOKE/install.json" <<EOF
{"profile": "$PROFILE", "size": $SIZE, "level_type": "$(echo "$LEVEL_TYPE" | tr -d '\\')", "synth": "$SYNTH", "strata": "$STRATA", "datapack": "$DATAPACK",
 "preset": "$PRESET", "timeline_start": "$START", "timeline_length_myr": "$LENGTH_MYR",
 "test_terrain": "$TEST_TERRAIN", "seed": "$SEED", "neoforge": "$NEO", "pack": "$PACK_DESC", "pack_dirty_hash": "$PACK_DIRTY_HASH",
 "pack_ref": "${PACK_REF:-}", "pack_stale_hashes": "${PACK_STALE:-}",
 "deeptime_jar": "$(basename "$JAR")", "game_port": $GAME_PORT, "rcon_port": $RCON_PORT}
EOF
# What the jar was built from (its sha256, the source revision and a hash of the source trees), so a
# smoke run can report it and tools/workbench/preview.py can see when the copy went stale.
python3 "$REPO/tools/smoke/jarstate.py" record "$SMOKE/install.json" "$SERVER/mods/$(basename "$JAR")" --repo "$REPO"
echo "$NEO" > "$SMOKE/neoforge.version"
say "ready: $SERVER (mods: $(find "$SERVER/mods" -maxdepth 1 -name '*.jar' | wc -l), excluded: $(wc -l < "$SMOKE/excluded.txt"))"
