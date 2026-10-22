#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""TEST ONLY: does Distant Horizons' `/dh pregen` stall on a square that crosses the seam of
Toroidal World / Exotic Worlds? (Ben, decisions log 2026-10-17, "Test on 1.0.0 first".)

Nothing here ships and nothing here changes Deep Time's pins: Deep Time stays on Toroidal World
0.19.3 (MIT, neoforge/gradle.properties, tools/smoke/profiles/standalone.txt). This script builds
its own server sets, outside smoke/, from the dh profile's pins with the Toroidal World jar (and
optionally Distant Horizons) swapped for another version, and measures the pregeneration. The
default `--tw` is the standalone profile's pin, i.e. 0.19.3. Exotic Worlds 1.0.0 is PolyForm Shield
1.0.0: it is downloaded to the scratch folder of a test run and nowhere else.

Deep Time cannot load next to Exotic Worlds 1.0.0 (the mod id and the API package were renamed), so
the sets here have NO Deep Time jar: the world is Toroidal World's own preset (`large`, in 1.0.0
`torus_large`: x and z loop from chunk -128 to 128, edge E = 2048) and Distant Horizons generates
its own LODs. That is the draft issue's reproduction (docs/upstream/toroidal-world-dh-pregen-issue.md)
without our mod. `--deeptime-jar` adds one, to show the refusal.

    ew-dh-pregen-repro.py install --work W --set NAME [--tw ID] [--dh ID] [--jars dh-set|two-mod]
                          [--deeptime-jar PATH]
    ew-dh-pregen-repro.py run     --work W --set NAME [--squares control,cross] [--stall-s 200]
                          [--limit-s 1500] [--boot-only]

`install` builds W/sets/NAME/server (NeoForge shared in W/base, jars cached in W/cache). `run`
boots a fresh private copy of the set once per square (control: a /dh pregen square of radius 32
chunks whose last 64-block section ends exactly at the edge; cross: centred 320 blocks before it,
as the 2026-10-03 measurement) and polls `dh pregen status` until it finishes, stops moving for
--stall-s seconds, or --limit-s passes. Results: W/results/NAME.json and NAME.<square>.log.

Server runs go through the test VM's admission tool (tools/smoke/admission.py) and refuse to run
anywhere else. Standard library only.
"""

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
import urllib.request
import zipfile
from datetime import datetime, timezone

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
sys.path.insert(0, os.path.join(REPO, "tools", "smoke"))
import admission  # noqa: E402
import instance  # noqa: E402
import run as smoke_run  # noqa: E402  (Server, Rcon: the smoke harness's server process and RCON)

PROFILES = os.path.join(REPO, "tools", "smoke", "profiles")
NEOFORGE = "21.1.249"   # neo_version in neoforge/gradle.properties
NEOFORGE_MAVEN = "https://maven.neoforged.net/releases/net/neoforged/neoforge"
UA = "MinecraftIndustrialComplex/deep-time ew-dh-pregen-repro (test only)"
EDGE = 2048             # `large` / `torus_large`: chunks -128..128 on both axes
RADIUS_CHUNKS = 32      # the smallest /dh pregen takes
# Centres (blocks) as in the draft issue: the square covers [centre - 512, centre + 512).
SQUARES = {"cross": EDGE - 320, "control": EDGE - 576}


def say(msg):
    print(f"ew-repro: {msg}", flush=True)


def profile_pins(name):
    """[(slug, version id)] of tools/smoke/profiles/<name>.txt, @include followed, duplicates once."""
    out = []
    for line in open(os.path.join(PROFILES, name + ".txt")):
        parts = line.split("#", 1)[0].split()
        if not parts:
            continue
        if parts[0] == "@include":
            out.extend(profile_pins(parts[1]))
        else:
            out.append((parts[0], parts[1]))
    seen, uniq = set(), []
    for p in out:
        if p not in seen:
            seen.add(p)
            uniq.append(p)
    return uniq


def modrinth_fetch(vid, cache):
    """The cached jar of a Modrinth version id (sha512-checked), as install.sh's modrinth_fetch."""
    os.makedirs(cache, exist_ok=True)
    meta_path = os.path.join(cache, vid + ".json")
    if not os.path.exists(meta_path):
        req = urllib.request.Request(f"https://api.modrinth.com/v2/version/{vid}", headers={"User-Agent": UA})
        open(meta_path, "wb").write(urllib.request.urlopen(req, timeout=60).read())
    meta = json.load(open(meta_path))
    f = next((f for f in meta["files"] if f.get("primary")), meta["files"][0])
    path = os.path.join(cache, f["filename"])

    def ok(p):
        return os.path.exists(p) and hashlib.sha512(open(p, "rb").read()).hexdigest() == f["hashes"]["sha512"]
    if not ok(path):
        req = urllib.request.Request(f["url"], headers={"User-Agent": UA})
        open(path + ".part", "wb").write(urllib.request.urlopen(req, timeout=300).read())
        os.replace(path + ".part", path)
        if not ok(path):
            sys.exit(f"ew-repro: sha512 mismatch for {f['filename']} ({vid})")
    return path, meta, f


def tools():
    """JAVA and MCRCON from smoke/tools.env, resolved out of nixpkgs the first time (install.sh's way)."""
    env_path = os.path.join(REPO, "smoke", "tools.env")
    env = {}
    if os.path.exists(env_path):
        env = smoke_run.load_env(env_path)
    if not (os.access(env.get("JAVA", ""), os.X_OK) and os.access(env.get("MCRCON", ""), os.X_OK)):
        say("resolving jdk21 and mcrcon from nixpkgs")
        def resolve(pkg, exe):
            return subprocess.run(["nix", "shell", f"nixpkgs#{pkg}", "-c", "sh", "-c", f'readlink -f "$(command -v {exe})"'],
                                  capture_output=True, text=True, check=True).stdout.strip()
        env = {"JAVA": resolve("jdk21", "java"), "MCRCON": resolve("mcrcon", "mcrcon")}
        os.makedirs(os.path.dirname(env_path), exist_ok=True)
        open(env_path, "w").write("".join(f"{k}={v}\n" for k, v in env.items()))
    return env


def install_neoforge(work, java):
    base = os.path.join(work, "base", "server")
    args = os.path.join(base, "libraries", "net", "neoforged", "neoforge", NEOFORGE, "unix_args.txt")
    if os.path.exists(args):
        return base
    os.makedirs(base, exist_ok=True)
    installer = os.path.join(work, "cache", f"neoforge-{NEOFORGE}-installer.jar")
    if not os.path.exists(installer):
        say(f"downloading the NeoForge {NEOFORGE} installer")
        os.makedirs(os.path.dirname(installer), exist_ok=True)
        req = urllib.request.Request(f"{NEOFORGE_MAVEN}/{NEOFORGE}/neoforge-{NEOFORGE}-installer.jar", headers={"User-Agent": UA})
        open(installer + ".part", "wb").write(urllib.request.urlopen(req, timeout=300).read())
        os.replace(installer + ".part", installer)
    say(f"installing the NeoForge {NEOFORGE} server")
    subprocess.run([java, "-jar", installer, "--installServer", base], cwd=base, check=True, capture_output=True)
    if not os.path.exists(args):
        sys.exit(f"ew-repro: the installer produced no {args}")
    return base


def cmd_install(a):
    work = os.path.abspath(a.work)
    env = tools()
    base = install_neoforge(work, env["JAVA"])
    pins = profile_pins("dh")                      # standalone's pins plus Distant Horizons
    swapped = []
    for slug, vid in pins:
        if slug == "toroidal-world" and a.tw:
            vid = a.tw
        if slug == "distanthorizons" and a.dh:
            vid = a.dh
        swapped.append((slug, vid))
    if a.jars == "two-mod":
        swapped = [(s, v) for s, v in swapped if s in ("toroidal-world", "distanthorizons")]
    set_dir = os.path.join(work, "sets", a.set)
    server = os.path.join(set_dir, "server")
    shutil.rmtree(server, ignore_errors=True)
    os.makedirs(os.path.join(server, "mods"))
    os.symlink(os.path.join(base, "libraries"), os.path.join(server, "libraries"))
    cache = os.path.join(work, "cache", "modrinth")
    jars = []
    for slug, vid in swapped:
        path, meta, f = modrinth_fetch(vid, cache)
        shutil.copy2(path, os.path.join(server, "mods"))
        jars.append({"slug": slug, "version_id": vid, "file": os.path.basename(path), "version_number": meta.get("version_number"),
                     "sha512": f["hashes"]["sha512"]})
        say(f"{slug} {vid} -> {os.path.basename(path)}")
    if a.deeptime_jar:
        shutil.copy2(a.deeptime_jar, os.path.join(server, "mods"))
        jars.append({"slug": "deeptime", "file": os.path.basename(a.deeptime_jar),
                     "sha256": hashlib.sha256(open(a.deeptime_jar, "rb").read()).hexdigest()})
    tw_file = next(j["file"] for j in jars if j["slug"] == "toroidal-world")
    level_type = "exotic_worlds:torus_large" if tw_file.startswith("exotic_worlds") else "toroidal_world:large"
    with open(os.path.join(server, "eula.txt"), "w") as f:
        f.write("eula=true\n")   # Ben accepted the Minecraft EULA for test servers (plan §2.7)
    with open(os.path.join(server, "user_jvm_args.txt"), "w") as f:
        f.write("-Xmx8G\n-XX:+UseG1GC\n")
    with open(os.path.join(set_dir, "set.json"), "w") as f:
        json.dump({"set": a.set, "jars": jars, "level_type": level_type, "neoforge": NEOFORGE, "jar_set": a.jars,
                   "installed": datetime.now(timezone.utc).isoformat()}, f, indent=1)
    say(f"ready: {server} ({len(jars)} jars, level-type {level_type})")


def props_text(level_type, ports, password):
    return "\n".join([
        "# Written by tools/review/ew-dh-pregen-repro.py (test only).",
        f"server-port={ports['game']}", "enable-rcon=true", f"rcon.port={ports['rcon']}", f"rcon.password={password}",
        "broadcast-rcon-to-ops=false", "online-mode=false", f"level-type={level_type.replace(':', chr(92) + ':')}",
        "level-name=world", "spawn-protection=0", "max-tick-time=-1", "view-distance=6", "simulation-distance=4",
        "sync-chunk-writes=false", "motd=Exotic Worlds pregen repro", ""])


# DH's status line: "Generated radius: 21.07 / 32 chunks (136 cps, 43%), ETA: 16s", then "Pregen is not running".
PCT = re.compile(r"(\d+(?:\.\d+)?)\s*%")
RADIUS = re.compile(r"radius:\s*(\d+(?:\.\d+)?)\s*/\s*(\d+)")
NOT_RUNNING = re.compile(r"not running", re.I)
INTERESTING = re.compile(r"(?i)pregen|expired|exoticworld|exotic_worlds|toroidal|DhFold|Exception|\bERROR\b|\bWARN\b.*(distant|dh)")


def parse_status(text):
    """(percent, radius done in chunks) of a status line; None where it does not say."""
    pct = PCT.search(text)
    rad = RADIUS.search(text)
    return (float(pct.group(1)) if pct else None), (float(rad.group(1)) if rad else None)


def run_square(a, set_dir, meta, square, tool_env, password):
    centre = SQUARES[square]
    log_path = os.path.join(a.work, "results", f"{a.set}.{square}.log")
    os.makedirs(os.path.dirname(log_path), exist_ok=True)
    ports = admission.ports()
    server_dir = instance.make(os.path.join(set_dir, "server"), os.path.join(admission.server_dir(), f"server-{square}"),
                               ports=ports, say=say)
    props = os.path.join(server_dir, "server.properties")
    open(props, "w").write(props_text(meta["level_type"], ports, password))
    open(log_path, "w").close()
    server = smoke_run.Server(server_dir, tool_env["JAVA"], meta["neoforge"], log_path)
    rcon = smoke_run.Rcon(tool_env["MCRCON"], ports["rcon"], password)
    out = {"square": square, "centre_x": centre, "centre_z": 0, "radius_chunks": RADIUS_CHUNKS,
           "x_from": centre - 16 * RADIUS_CHUNKS, "x_to": centre + 16 * RADIUS_CHUNKS, "edge": EDGE}
    say(f"[{square}] booting {a.set} (level-type {meta['level_type']}), square x {out['x_from']}..{out['x_to']} (edge {EDGE})")
    server.start()
    try:
        ok = server.wait_for_done(a.boot_timeout)
        out["boot_ok"] = ok
        out["boot_s"] = round((server.done_at or time.time()) - server.started, 1)
        if not ok:
            out["boot_failure"] = server.fatal
            out["boot_log_tail"] = [l.rstrip() for l in server.lines[-40:]]
            say(f"[{square}] the server did not start: {server.fatal}")
            return out
        time.sleep(a.settle_s)   # DH starts its level after the server is up
        started = rcon(f"dh pregen start minecraft:overworld {centre} 0 {RADIUS_CHUNKS}", check=False)
        say(f"[{square}] dh pregen start: {started[:200]}")
        out["start_reply"] = started
        t0 = time.time()
        timeline, last_change, last_text, last_pos = [], t0, None, None
        verdict = None
        while True:
            time.sleep(a.poll_s)
            now = time.time() - t0
            status = rcon("dh pregen status", check=False)
            pct, radius = parse_status(status)
            timeline.append({"t": round(now, 1), "status": status[:300], "pct": pct, "radius_chunks_done": radius})
            # The status carries an ETA that ticks down, so "moved" means the percent or the radius changed.
            if (pct, radius) != last_pos or NOT_RUNNING.search(status) != NOT_RUNNING.search(last_text or ""):
                say(f"[{square}] t={now:6.1f}s  {status[:200]}")
                last_pos, last_change = (pct, radius), time.time()
            last_text = status
            if now > 6 and NOT_RUNNING.search(status):
                verdict = "finished"
                break
            if time.time() - last_change >= a.stall_s:
                verdict = "stalled"
                break
            if now >= a.limit_s:
                verdict = "limit"
                break
        out.update({"verdict": verdict, "elapsed_s": round(time.time() - t0, 1), "timeline": timeline,
                    "last_status": last_text, "last_change_t": round(last_change - t0, 1)})
        # Where it got to: the last status that reported a position (the final one says "not running").
        reached = [t for t in timeline if t["pct"] is not None]
        out["last_pct"] = reached[-1]["pct"] if reached else None
        out["last_radius_chunks_done"] = reached[-1]["radius_chunks_done"] if reached else None
        # The expiry message the draft says never appears, and DH's own lines about the pregeneration.
        out["expired_lines"] = len(server.grep(r"has expired", 10**6))
        out["log_lines"] = [l for l in (x.rstrip() for x in server.lines) if INTERESTING.search(l)][:80]
        if verdict != "finished":
            say(f"[{square}] {verdict}: stopping the pregeneration; log lines about it follow")
            out["stop_reply"] = rcon("dh pregen stop", check=False)
            time.sleep(3)
            out["status_after_stop"] = rcon("dh pregen status", check=False)
        say(f"[{square}] {verdict} after {out['elapsed_s']} s; last status {last_text!r}")
    finally:
        server.stop(rcon)
    return out


def cmd_run(a):
    if not admission.available():
        sys.exit("ew-repro: server runs go to the test VM (tools/remote/mac.sh -- ...); this machine has no admission tool")
    if not admission.server_dir():
        # Started as a plain shell job (mac.sh and detach.sh classify this script as `shell:...`, and bash
        # execs it in place, so it would count as the admitted process without a slot): ask for a server one.
        os.environ.pop("DEEPTIME_ADMIT_PID", None)
    admission.ensure("server:ew-dh-pregen-repro", label=a.set)
    if not admission.server_dir():
        sys.exit("ew-repro: no server slot from the admission tool (DEEPTIME_SERVER_DIR is unset)")
    a.work = os.path.abspath(a.work)
    set_dir = os.path.join(a.work, "sets", a.set)
    meta = json.load(open(os.path.join(set_dir, "set.json")))
    tool_env = tools()
    password = hashlib.sha256(os.urandom(16)).hexdigest()[:24]
    results = {"set": a.set, "jars": meta["jars"], "level_type": meta["level_type"], "neoforge": meta["neoforge"],
               "started": datetime.now(timezone.utc).isoformat(), "squares": []}
    squares = ["cross"] if a.boot_only else a.squares.split(",")
    for sq in squares:
        r = run_square(a, set_dir, meta, sq, tool_env, password)
        results["squares"].append(r)
        out_path = os.path.join(a.work, "results", f"{a.set}.json")
        json.dump(results, open(out_path, "w"), indent=1)
        if a.boot_only or not r.get("boot_ok"):
            break
    say("== summary")
    for r in results["squares"]:
        # A finished run's last status ("Pregen is not running") carries no percent, so the figure is the last
        # one sampled before it, not where the run ended: say so instead of printing a partial-looking value.
        sampled = "last sampled " if r.get("verdict") == "finished" else "reached "
        say(f"{a.set} {r['square']}: {r.get('verdict', 'no boot')}, {sampled}{r.get('last_pct')} % "
            f"(radius {r.get('last_radius_chunks_done')} of {RADIUS_CHUNKS} chunks) after {r.get('elapsed_s')} s, "
            f"last movement at {r.get('last_change_t')} s; last status {r.get('last_status')!r}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    i = sub.add_parser("install")
    i.add_argument("--work", required=True, help="scratch folder, e.g. /Volumes/Graphide/ban/deeptime-runs/ew1-dh-repro/work")
    i.add_argument("--set", required=True)
    i.add_argument("--tw", help="Modrinth version id of the Toroidal World / Exotic Worlds jar (default: the standalone profile's 0.19.3)")
    i.add_argument("--dh", help="Modrinth version id of Distant Horizons (default: the dh profile's pin, 3.3.3 since pack 62dd14e)")
    i.add_argument("--jars", choices=["dh-set", "two-mod"], default="dh-set",
                   help="dh-set: every jar of the dh profile (without Deep Time); two-mod: Toroidal World / Exotic Worlds and Distant Horizons only")
    i.add_argument("--deeptime-jar", help="add this Deep Time jar (to show it cannot load next to Exotic Worlds 1.0.0)")
    r = sub.add_parser("run")
    r.add_argument("--work", required=True)
    r.add_argument("--set", required=True)
    r.add_argument("--squares", default="control,cross")
    r.add_argument("--stall-s", type=float, default=200.0, help="no movement for this long is a stall (the draft: 200 s)")
    r.add_argument("--limit-s", type=float, default=1500.0)
    r.add_argument("--poll-s", type=float, default=3.0)
    r.add_argument("--settle-s", type=float, default=15.0)
    r.add_argument("--boot-timeout", type=int, default=900)
    r.add_argument("--boot-only", action="store_true", help="boot once, report whether the server starts, and stop")
    a = ap.parse_args()
    {"install": cmd_install, "run": cmd_run}[a.cmd](a)


if __name__ == "__main__":
    main()
