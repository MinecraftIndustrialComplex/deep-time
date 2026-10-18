#!/usr/bin/env bash
# SPDX-License-Identifier: LGPL-3.0-or-later
#
# Checks Ben's running play server on the test VM (run it on the VM, after `systemctl restart deeptime-play`):
#
#   tools/remote/mac.sh --no-sync -- tools/remote/vm/play-check.sh [--dir /data/ban/play/<hash>] [--wait 240]
#
# From the server's own log (<dir>/logs/latest.log, the current start): "Done", Deep Time's natives,
# the bound world data, Distant Horizons' far terrain, and no Deep Time or Toroidal World line at WARN or
# ERROR. Over RCON: `list`, `datapack list enabled` (the MIC rock-tag datapack is on), and
# `deeptime probe palette`, whose JSON line lands in the log: the rock classes' sources and, for the
# classes below, the block each resolves to (ironstone must be Create's ochrum, not the vanilla fallback).
# Exit status 1 when any check fails. The laptop's side (the tunnel to localhost:25765) is
# tools/remote/play-ping.py.
set -uo pipefail
DIR=""; WAIT=240
while [ $# -gt 0 ]; do
    case "$1" in
        --dir) DIR="$2"; shift 2 ;;
        --wait) WAIT="$2"; shift 2 ;;
        *) echo "usage: play-check.sh [--dir <play folder>] [--wait <seconds>]" >&2; exit 2 ;;
    esac
done
[ -n "$DIR" ] || DIR="$(sed -n 's/^PLAY_DIR=//p' /etc/default/deeptime-play)"
[ -d "$DIR" ] || { echo "play-check.sh: no play folder '$DIR'" >&2; exit 2; }
LOG="$DIR/logs/latest.log"
export DIR LOG WAIT
exec python3 - <<'PY'
import json, os, re, socket, struct, sys, time

DIR, LOG, WAIT = os.environ["DIR"], os.environ["LOG"], int(os.environ["WAIT"])
# Rock classes whose MIC block the rock-tag datapack names (rock_class JSON "notes", datapack data/deeptime/tags/block/rock).
EXPECT = {"ironstone": "create:ochrum"}
bad = 0


def check(ok, what, detail=""):
    global bad
    print(("ok    " if ok else "FAIL  ") + what + (f": {detail}" if detail else ""))
    bad += 0 if ok else 1


def log_text():
    try:
        return re.sub(r"\x1b\[[0-9;]*m", "", open(LOG, errors="replace").read())
    except FileNotFoundError:
        return ""


deadline = time.time() + WAIT
while "Done (" not in log_text() and time.time() < deadline:
    time.sleep(3)
t = log_text()
m = re.search(r"Done \(([\d.]+)s\)!", t)
check(bool(m), "server reached Done", f"{m.group(1)} s" if m else f"not within {WAIT} s")
m = re.search(r"Deep Time natives: status=(\w+) version=(\S+) format=(\S+)", t)
check(bool(m) and m.group(1) == "OK", "natives", m.group(0) if m else "no line")
m = re.search(r"Bound Deep Time world data (\S+) \(format (\S+), C = (\d+)\)", t)
check(bool(m), "world data bound", f"{m.group(1)} format {m.group(2)} C = {m.group(3)}" if m else "no 'Bound Deep Time world data' line")
check("world data present at" in t and "not regenerating" in t, "world data read, not regenerated")
m = re.search(r"Distant Horizons (\S+) \(API (\S+)\) API bound", t)
check(bool(m), "Distant Horizons API bound", m.group(0) if m else "no line")
m = re.search(r"far terrain: Distant Horizons' LODs of (\S+) .*?come from the world data \(detail levels (\d+) to (\d+)", t)
check(bool(m), "far terrain from the world data", f"{m.group(1)}, levels {m.group(2)} to {m.group(3)}" if m else "no 'far terrain' line")
m = re.search(r"Rock palette: (\d+) classes \((\d+) member, (\d+) fallback, (\d+) vanilla_fallback, (\d+) unresolved", t)
print("info  " + (m.group(0) if m else "no 'Rock palette' line"))
noisy = [l for l in t.splitlines() if re.search(r"/(WARN|ERROR)\]", l) and re.search(r"\[(deeptime|co\.to\.ToroidalWorld|toroidal_world)/\]", l)]
check(not noisy, "no Deep Time or Toroidal World WARN/ERROR line", f"{len(noisy)}: {noisy[0][:200]}" if noisy else "")


def rcon(cmds):
    props = dict(l.strip().split("=", 1) for l in open(os.path.join(DIR, "server.properties")) if "=" in l and not l.startswith("#"))
    s = socket.create_connection(("127.0.0.1", int(props["rcon.port"])), timeout=20)

    def send(rid, kind, body):
        b = struct.pack("<ii", rid, kind) + body.encode() + b"\x00\x00"
        s.sendall(struct.pack("<i", len(b)) + b)

    def recv():
        n = struct.unpack("<i", s.recv(4))[0]
        data = b""
        while len(data) < n:
            data += s.recv(n - len(data))
        return struct.unpack("<ii", data[:8]) + (data[8:-2].decode(errors="replace"),)

    send(1, 3, props["rcon.password"])
    if recv()[0] == -1:
        raise RuntimeError("RCON login refused")
    out = {}
    for i, c in enumerate(cmds, 10):
        send(i, 2, c)
        out[c] = recv()[2]
        s.settimeout(0.5)
        try:                      # a long answer comes in further packets
            while True:
                out[c] += recv()[2]
        except Exception:
            pass
        s.settimeout(20)
    s.close()
    return out


try:
    r = rcon(["list", "datapack list enabled", "deeptime probe palette"])
    check(r["list"].startswith("There are"), "RCON list", r["list"].strip())
    dp = r["datapack list enabled"]
    tags = [e for e in re.findall(r"\[([^\]]+)\]", dp) if "rock-tags" in e]
    check(bool(tags), "MIC rock-tag datapack enabled", tags[0] if tags else "datapack list enabled: " + dp.strip()[:200])
    time.sleep(1)
    line = [l for l in log_text().splitlines() if "[deeptime-probe] {" in l and '"probe":"palette"' in l.replace(" ", "")]
    pal = json.loads(line[-1].split("[deeptime-probe] ", 1)[1]) if line else None
    check(pal is not None, "palette probe in the log")
    if pal:
        print("info  palette sources: " + json.dumps(pal["sources"]))
        for cls, want in EXPECT.items():
            got = pal["state"].get(cls, "")
            src = pal["source"].get(cls, "?")
            check(got.split("[")[0] == want and src == "member", f"rock class {cls}", f"{got or 'unresolved'} ({src}), expected {want} as a member")
        check(pal["sources"].get("member", 0) > 17, "member classes beyond the jar's own tags", f"{pal['sources'].get('member')} members")
except Exception as e:
    check(False, "RCON", repr(e))
sys.exit(1 if bad else 0)
PY
