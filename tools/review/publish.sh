#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Publish a review page as a PRIVATE spacesheep space (plan §2.6).
#
#   tools/review/publish.sh <name> <folder> [-m <version label>] [--description <one sentence>]
#                           [--emoji <one emoji>]
#
#   <name>          short id matching [a-z0-9][a-z0-9-]*: the key in tools/review/spaces.json,
#                   the output folder review/out/<name>/ and, for a new space, the slug deeptime-<name>
#   <folder>        a folder holding index.html and any local files it loads
#   -m              version label shown in the space's history (default: "<name> <today>")
#   --description   one sentence for the space (default: the page's <meta name="description">)
#   --emoji         the space's dashboard icon (optional; spacesheep uses a placeholder without it)
#
# The space title is the page's own <title>, so the two never drift apart.
#
# Steps:
#   1. copy <folder> to the gitignored review/out/<name>/ (a .spacesheep.json left there by an
#      earlier deploy is kept);
#   2. deploy it with spacesheep 1.17.0 (pinned) through `nix shell nixpkgs#nodejs_22`, passing
#      --space when tools/review/spaces.json already records this page (1.17.0 refuses to update a
#      pinned folder whose <title> changed unless --space or --new is given; we never pass --new);
#      --slug is passed only when creating the space, so a later deploy cannot move Ben's link;
#   3. record {"<name>": {"space": "<uuid>", "url": "<url>"}} in tools/review/spaces.json
#      (committed), so the link stays stable on a fresh clone.
#
# Visibility: new spaces are private by default. This script never passes --visibility, --org or
# --new and never runs `spacesheep share`. It never signs in: if a deploy fails on authentication
# it stops with spacesheep's error, and Ben signs in himself (never run login/connect here).
# Verify afterwards with `spacesheep list` (exists, private) and `spacesheep read <space>`.

set -euo pipefail

usage() {
    echo "usage: tools/review/publish.sh <name> <folder> [-m <label>] [--description <sentence>] [--emoji <emoji>]" >&2
    exit 2
}
die() { echo "publish.sh: $*" >&2; exit 1; }

[ $# -ge 2 ] || usage
NAME=$1; SRC=$2; shift 2
LABEL=""; DESC=""; EMOJI=""
while [ $# -gt 0 ]; do
    case "$1" in
        -m|--message) [ $# -ge 2 ] || usage; LABEL=$2; shift 2 ;;
        --description) [ $# -ge 2 ] || usage; DESC=$2; shift 2 ;;
        --emoji) [ $# -ge 2 ] || usage; EMOJI=$2; shift 2 ;;
        *) usage ;;
    esac
done
[[ $NAME =~ ^[a-z0-9][a-z0-9-]*$ ]] || die "name must match [a-z0-9][a-z0-9-]*: $NAME"
[ -f "$SRC/index.html" ] || die "no index.html in $SRC"
SRC=$(cd "$SRC" && pwd)

REPO=$(git rev-parse --show-toplevel)
OUT="$REPO/review/out/$NAME"
RESULT="$REPO/review/out/$NAME.deploy.json"   # outside the deployed folder
SPACES="$REPO/tools/review/spaces.json"
NODE=(nix shell nixpkgs#nodejs_22 -c node)
SPACESHEEP=(nix shell nixpkgs#nodejs_22 -c npx -y spacesheep@1.17.0)
[ -z "$LABEL" ] && LABEL="$NAME $(date +%F)"
[ -f "$SPACES" ] || echo '{}' > "$SPACES"

# Title and description from the page; the space id, if this page was published before.
mapfile -t META < <("${NODE[@]}" -e '
    const fs = require("fs");
    const [page, spaces, name] = process.argv.slice(1);
    const html = fs.readFileSync(page, "utf8").slice(0, 16384);
    const dec = (s) => s.replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&quot;/g, "\"")
        .replace(/&#39;/g, "\x27").replace(/&amp;/g, "&").replace(/\s+/g, " ").trim();
    const title = html.match(/<title>([\s\S]*?)<\/title>/i);
    const desc = html.match(/<meta\s+name="description"\s+content="([^"]*)"/i);
    const rec = JSON.parse(fs.readFileSync(spaces, "utf8"))[name] || {};
    console.log(title ? dec(title[1]) : "");
    console.log(desc ? dec(desc[1]) : "");
    console.log(rec.space || "");
' "$SRC/index.html" "$SPACES" "$NAME")
TITLE=${META[0]:-}; PAGE_DESC=${META[1]:-}; SPACE=${META[2]:-}
[ -n "$TITLE" ] || die "$SRC/index.html has no <title>"
[ -n "$DESC" ] || DESC=$PAGE_DESC
[ -n "$DESC" ] || die "no --description and no <meta name=\"description\"> in the page"

# Fresh copy of the page; keep the pin file an earlier deploy wrote.
mkdir -p "$OUT"
find "$OUT" -mindepth 1 -maxdepth 1 ! -name .spacesheep.json -exec rm -rf -- {} +
cp -R "$SRC"/. "$OUT"/

ARGS=(deploy . --title "$TITLE" --description "$DESC" -m "$LABEL" --json)
[ -n "$EMOJI" ] && ARGS+=(--emoji "$EMOJI")
if [ -n "$SPACE" ]; then
    ARGS+=(--space "$SPACE")
    echo "publish.sh: deploying review/out/$NAME as a new version of space $SPACE" >&2
else
    ARGS+=(--slug "deeptime-$NAME")
    echo "publish.sh: deploying review/out/$NAME as a new private space (slug deeptime-$NAME)" >&2
fi
if ! (cd "$OUT" && "${SPACESHEEP[@]}" "${ARGS[@]}") > "$RESULT"; then
    cat "$RESULT" >&2
    die "deploy failed (spacesheep's output above). On an authentication error, stop: Ben signs in himself."
fi

# Record the space id and URL: the pin file spacesheep writes, else the --json result.
"${NODE[@]}" -e '
    const fs = require("fs");
    const [spaces, name, result, pinFile] = process.argv.slice(1);
    const readJson = (p) => { try { const t = fs.readFileSync(p, "utf8"); return JSON.parse(t.slice(t.indexOf("{"))); } catch (e) { return {}; } };
    const res = readJson(result), pin = readJson(pinFile);
    const pick = (...v) => v.find((x) => typeof x === "string" && x) || "";
    const sp = res.space && typeof res.space === "object" ? res.space : {};
    const space = pick(pin.space, res.uuid, sp.id, res.space, res.spaceId, res.id);
    const url = pick(pin.url, res.url, sp.url, res.pageUrl);
    if (!space) { console.error("publish.sh: no space id in .spacesheep.json or the deploy result"); process.exit(1); }
    const all = JSON.parse(fs.readFileSync(spaces, "utf8"));
    all[name] = { space, url };
    const sorted = Object.fromEntries(Object.keys(all).sort().map((k) => [k, all[k]]));
    fs.writeFileSync(spaces, JSON.stringify(sorted, null, 2) + "\n");
    console.log(url || space);
' "$SPACES" "$NAME" "$RESULT" "$OUT/.spacesheep.json"
