#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Boot a Deep Time server set headlessly, drive it over RCON, and write a report (plan §9.5).

    tools/smoke/run.py --profile <standalone|mic|terrablender>
        [--fresh]                 delete the world first
        [--level-type <preset>]   override server.properties' level-type (boot-only runs,
                                  e.g. minecraft:normal before Deep Time's preset exists)
        [--chunky <radius>]       pregenerate a square of this radius around 0,0 (and the seam
                                  once the planet preset exists)
        [--restart]               stop, start again and confirm the world data was not regenerated
        [--timeout <s>]           how long to wait for "Done (" (default 1500)

Adapted from mic-climate's smoke/run.py (MIT, same author). Standard library only; the JDK and
mcrcon come from smoke/tools.env, which install.sh resolves out of nixpkgs once.

What it proves, in order:
 1. The set boots: no mixin of Deep Time or Toroidal World fails (defaultRequire 1 kills
    the launch otherwise). Anything that looks like a mixin failure or a crash ends the run at
    once instead of after the timeout.
 2. Every Deep Time mixin was applied, by name (TW's are counted): the server runs with
    -Dmixin.debug.verbose=true and each "Mixing X from Y into Z" line is matched against the
    config's declared list. A declared mixin that never applied is listed.
 3. The Deep Time probes answer (from P0-05 on) and nothing is written outside the canonical span.

The exit code is 0 only if every check passed. smoke/<profile>/report.md is written either way.
"""

import argparse
import io
import json
import os
import re
import signal
import subprocess
import sys
import threading
import time
import zipfile
from datetime import datetime, timezone

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

FATAL = re.compile(
    r"InjectionError|MixinTransformerError|MixinApplyError|Mixin apply failed"
    r"|InvalidInjectionException|Critical injection failure|Preparing crash report"
    r"|Failed to start the minecraft server|Mod [Ll]oading has failed|Cannot find class"
)
DONE = re.compile(r"Done \(")
# RCON comes up after "Done (", sometimes tens of seconds later: wait for it before any command.
RCON_UP = re.compile(r"RCON running on")
MIXING = re.compile(r"Mixing (\S+) from (\S+\.mixins\.json) into (\S+)")
PROBE = re.compile(r"\[deeptime-probe\]\s*(\{.*\})")
# Configs whose every declared mixin must have applied by "Done (": Deep Time's own, whose
# targets are all loaded during boot (worldgen, commands, servers). Toroidal World's configs are
# recorded only: many of its mixins target classes an idle server never loads (villager AI,
# dragon phases, raids), and Mixin applies a mixin when its target class loads.
REQUIRED_CONFIGS = ["deeptime.mixins.json"]
RECORDED_CONFIGS = ["toroidal_world.mixins.json", "toroidal_world.shape.mixins.json"]


def load_env(path):
    env = {}
    for line in open(path):
        if "=" in line:
            k, v = line.strip().split("=", 1)
            env[k] = v
    return env


class Server:
    def __init__(self, server_dir, java, neoforge_version, log_path):
        args_file = os.path.join(server_dir, "libraries", "net", "neoforged", "neoforge",
                                 neoforge_version, "unix_args.txt")
        if not os.path.exists(args_file):
            sys.exit(f"run.py: {args_file} is missing -- run tools/smoke/install.sh")
        self.dir = server_dir
        self.command = [java, "@user_jvm_args.txt", "-Dmixin.debug.verbose=true",
                        f"@{args_file}", "--nogui"]
        self.log_path = log_path
        self.lines, self.fatal, self.done_at, self.started, self._proc = [], None, None, None, None

    def start(self):
        self.lines, self.fatal, self.done_at, self.rcon_up = [], None, None, False
        self.started = time.time()
        self._log = open(self.log_path, "a", buffering=1, errors="replace")
        self._log.write(f"\n===== start {datetime.now(timezone.utc).isoformat()} =====\n")
        self._proc = subprocess.Popen(self.command, cwd=self.dir, stdout=subprocess.PIPE,
                                      stderr=subprocess.STDOUT, text=True, errors="replace",
                                      bufsize=1, start_new_session=True)
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        for line in self._proc.stdout:
            self._log.write(line)
            self.lines.append(line)
            if self.done_at is None and DONE.search(line):
                self.done_at = time.time()
            if RCON_UP.search(line):
                self.rcon_up = True
            if self.fatal is None and FATAL.search(line):
                self.fatal = line.rstrip()

    def peak_rss_kb(self):
        try:
            for line in open(f"/proc/{self._proc.pid}/status"):
                if line.startswith("VmHWM:"):
                    return int(line.split()[1])
        except (OSError, AttributeError):
            pass
        return None

    def wait_for_done(self, timeout):
        deadline, last = time.time() + timeout, 0.0
        while time.time() < deadline:
            if self.done_at and self.rcon_up:
                return True
            if self.fatal:
                return False
            if self._proc.poll() is not None:
                self.fatal = f"the server exited with status {self._proc.returncode} before it finished starting"
                return False
            if time.time() - last > 30:
                last = time.time()
                print(f"   ... {int(last - self.started)}s, {len(self.lines)} log lines", flush=True)
            time.sleep(0.25)
        self.fatal = f'timed out after {timeout}s waiting for "Done ("'
        return False

    def stop(self, rcon):
        if self._proc is None or self._proc.poll() is not None:
            return
        try:
            rcon("stop", check=False)
        except Exception:
            pass
        try:
            self._proc.wait(timeout=180)
            return
        except subprocess.TimeoutExpired:
            pass
        os.killpg(os.getpgid(self._proc.pid), signal.SIGTERM)
        try:
            self._proc.wait(timeout=60)
        except subprocess.TimeoutExpired:
            os.killpg(os.getpgid(self._proc.pid), signal.SIGKILL)

    def applied(self, config):
        found = {}
        for line in self.lines:
            m = MIXING.search(line)
            if m and m.group(2) == config:
                found[m.group(1)] = m.group(3)
        return found

    def grep(self, pattern, limit=10):
        rx = re.compile(pattern)
        return [l.rstrip() for l in self.lines if rx.search(l)][:limit]


def declared_mixins(mods_dir, config):
    """The `mixins` (+ `server`) arrays of a config, searched through jars and their jar-in-jars."""
    def search(zf, name):
        names = zf.namelist()
        if config in names:
            data = json.loads(zf.read(config))
            return name, list(data.get("mixins", [])) + list(data.get("server", []))
        for inner in names:
            if inner.startswith("META-INF/jarjar/") and inner.endswith(".jar"):
                with zipfile.ZipFile(io.BytesIO(zf.read(inner))) as z2:
                    hit = search(z2, f"{name}!{os.path.basename(inner)}")
                    if hit:
                        return hit
        return None
    for name in sorted(os.listdir(mods_dir)):
        if name.endswith(".jar"):
            try:
                with zipfile.ZipFile(os.path.join(mods_dir, name)) as zf:
                    hit = search(zf, name)
                    if hit:
                        return hit
            except (zipfile.BadZipFile, ValueError):
                continue
    return None, []


class Rcon:
    def __init__(self, mcrcon, port, password):
        self.mcrcon, self.port, self.password = mcrcon, str(port), password

    def __call__(self, command, check=True):
        r = subprocess.run([self.mcrcon, "-H", "127.0.0.1", "-P", self.port, "-p", self.password,
                            "-c", command], capture_output=True, text=True, errors="replace",
                           timeout=300)
        if check and r.returncode != 0:
            raise RuntimeError(f"mcrcon failed for {command!r}: {r.stderr.strip() or r.stdout.strip()}")
        return r.stdout.strip()


class Steps:
    def __init__(self):
        self.rows = []

    def check(self, name, expected, actual, ok, note=""):
        self.rows.append((name, expected, str(actual), bool(ok), note))
        print(f"   [{'PASS' if ok else 'FAIL'}] {name}" + ("" if ok else f": expected {expected}; got {actual}"), flush=True)
        return ok

    def record(self, name, value, note=""):
        self.rows.append((name, "(recorded)", str(value), True, note))
        print(f"   [ -- ] {name}: {value}", flush=True)

    @property
    def failed(self):
        return [r for r in self.rows if not r[3]]


def port_free(port):
    import socket
    with socket.socket() as s:
        return s.connect_ex(("127.0.0.1", port)) != 0


def region_span_check(world, circumference):
    """Every region file of the overworld must satisfy -C/1024 <= rx < C/1024 (plan §9.5)."""
    region_dir = os.path.join(world, "region")
    half = circumference // 1024
    outside, total = [], 0
    if os.path.isdir(region_dir):
        for f in os.listdir(region_dir):
            m = re.match(r"r\.(-?\d+)\.(-?\d+)\.mca$", f)
            if m:
                total += 1
                if not (-half <= int(m.group(1)) < half):
                    outside.append(f)
    return total, outside


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--profile", required=True)
    ap.add_argument("--fresh", action="store_true")
    ap.add_argument("--level-type")
    ap.add_argument("--chunky", type=int, default=0)
    ap.add_argument("--restart", action="store_true")
    ap.add_argument("--timeout", type=int, default=1500)
    a = ap.parse_args()

    smoke = os.path.join(REPO, "smoke", a.profile)
    server_dir = os.path.join(smoke, "server")
    if not os.path.exists(os.path.join(smoke, "install.json")):
        sys.exit(f"run.py: no {smoke}/install.json -- run tools/smoke/install.sh {a.profile}")
    install = json.load(open(os.path.join(smoke, "install.json")))
    tools = load_env(os.path.join(REPO, "smoke", "tools.env"))
    password = open(os.path.join(smoke, "rcon.pw")).read().strip()
    world = os.path.join(server_dir, "planet")

    props_path = os.path.join(server_dir, "server.properties")
    props = open(props_path).read()
    level_type = a.level_type or install["level_type"]
    props = re.sub(r"(?m)^level-type=.*$", "level-type=" + level_type.replace(":", "\\:"), props)
    open(props_path, "w").write(props)
    is_planet = level_type.startswith("deeptime:")

    if a.fresh:
        import shutil
        # Keep world data written by --synth; drop only what the game generated.
        for sub in ("region", "entities", "poi", "DIM-1", "DIM1", "dimensions", "data", "level.dat", "level.dat_old", "session.lock"):
            p = os.path.join(world, sub)
            if os.path.isdir(p):
                shutil.rmtree(p)
            elif os.path.exists(p):
                os.remove(p)

    for port in (install["game_port"], install["rcon_port"]):
        if not port_free(port):
            owners = subprocess.run(["ss", "-ltnp", f"sport = :{port}"], capture_output=True, text=True).stdout
            sys.exit(f"run.py: port {port} is in use -- another server (a peer session?) holds it:\n{owners}")

    os.makedirs(os.path.join(smoke, "logs"), exist_ok=True)
    log_path = os.path.join(smoke, "logs", "server.log")
    open(log_path, "w").close()
    rcon = Rcon(tools["MCRCON"], install["rcon_port"], password)
    server = Server(server_dir, tools["JAVA"], install["neoforge"], log_path)
    steps = Steps()
    started = datetime.now(timezone.utc)

    print(f"== booting {a.profile} (level-type {level_type})", flush=True)
    server.start()
    ok = server.wait_for_done(a.timeout)
    boot_s = (server.done_at or time.time()) - server.started
    steps.check("server reaches Done (", "Done (", "Done" if ok else server.fatal, ok)
    try:
        if ok:
            mods_dir = os.path.join(server_dir, "mods")
            for config in REQUIRED_CONFIGS:
                jar, declared = declared_mixins(mods_dir, config)
                applied = server.applied(config)
                short = {k.rsplit(".", 1)[-1] for k in applied} | set(applied)
                missing = [m for m in declared if m not in short and m.rsplit(".", 1)[-1] not in short]
                steps.check(f"{config}: every declared mixin applied", f"{len(declared)} of {len(declared)}",
                            f"{len(declared) - len(missing)} of {len(declared)}" + (f" (missing: {', '.join(missing)})" if missing else ""),
                            jar is not None and not missing, note=jar or "config not found")
            for config in RECORDED_CONFIGS:
                jar, declared = declared_mixins(mods_dir, config)
                steps.record(f"{config}: mixins applied by Done (", f"{len(server.applied(config))} of {len(declared)} declared (the rest target classes not loaded yet)")
            other = sorted({m.group(2) for l in server.lines if (m := MIXING.search(l)) and m.group(2).startswith("toroidal_world.compat.")})
            steps.record("Toroidal World compat configs that applied", ", ".join(other) or "none")

            if is_planet:
                out = rcon("deeptime probe tw", check=False)
                m = PROBE.search(out) or next((PROBE.search(l) for l in reversed(server.lines) if PROBE.search(l)), None)
                steps.check("/deeptime probe tw answers", "[deeptime-probe] JSON", out[:200] if not m else m.group(1), m is not None)
                if m:
                    tw = json.loads(m.group(1))
                    for k in ("stamped", "loopsX", "loopsZ", "widthChunks", "climateCompression"):
                        steps.record(f"tw.{k}", tw.get(k))
                out = rcon("deeptime probe column 0 0", check=False)
                m = PROBE.search(out)
                steps.check("/deeptime probe column 0 0 answers", "[deeptime-probe] JSON", out[:200], m is not None)

            if a.chunky:
                rcon("chunky center 0 0")
                rcon(f"chunky radius {a.chunky}")
                rcon("chunky start")
                deadline = time.time() + 1800
                while time.time() < deadline:
                    time.sleep(10)
                    if server.grep(r"Task finished for|Task complete", 1):
                        break
                steps.check("chunky pregeneration finished", "finished", "finished" if server.grep(r"Task finished|Task complete", 1) else "timeout",
                            bool(server.grep(r"Task finished|Task complete", 1)))

            rcon("save-all flush")
            time.sleep(3)
            if is_planet:
                total, outside = region_span_check(world, install["size"])
                steps.check("region files inside the canonical span", "outside=0", f"outside={len(outside)} of {total}", not outside)

            steps.record("peak RSS (MB)", (server.peak_rss_kb() or 0) // 1024)
            server.stop(rcon)

            if a.restart:
                manifest = os.path.join(world, "deeptime", "manifest.json")
                before = os.path.getmtime(manifest) if os.path.exists(manifest) else None
                server.start()
                ok2 = server.wait_for_done(a.timeout)
                steps.check("restart reaches Done (", "Done (", "Done" if ok2 else server.fatal, ok2)
                after = os.path.getmtime(manifest) if os.path.exists(manifest) else None
                steps.check("world data not regenerated on restart", before, after, before == after)
                server.stop(rcon)
    finally:
        server.stop(rcon)

    passed = not steps.failed
    write_report(smoke, a, install, steps, level_type, boot_s, started, passed, server)
    print(f"\n{'PASS' if passed else 'FAIL'} -- report: {os.path.join(smoke, 'report.md')}", flush=True)
    sys.exit(0 if passed else 1)


def write_report(smoke, a, install, steps, level_type, boot_s, started, passed, server):
    excluded = open(os.path.join(smoke, "excluded.txt")).read().split() if os.path.exists(os.path.join(smoke, "excluded.txt")) else []
    mods = sorted(f for f in os.listdir(os.path.join(smoke, "server", "mods")) if f.endswith(".jar"))
    with open(os.path.join(smoke, "report.md"), "w") as r:
        r.write(f"# Deep Time smoke report — {a.profile}\n\n")
        r.write(f"**Result: {'PASS' if passed else 'FAIL'}** · {started.strftime('%Y-%m-%d %H:%M UTC')}\n\n")
        r.write("| Field | Value |\n|---|---|\n")
        for k, v in [("Profile", a.profile), ("Level type", level_type), ("Circumference", install["size"]),
                     ("NeoForge", install["neoforge"]), ("MIC pack", install["pack"] + (f" (working tree {install['pack_dirty_hash']})" if install.get("pack_dirty_hash") else "")),
                     ("Deep Time jar", install["deeptime_jar"]), ("Mod jars", len(mods)),
                     ("Boot time", f"{boot_s:.0f} s"), ("Excluded jars", ", ".join(excluded) or "none")]:
            r.write(f"| {k} | {v} |\n")
        r.write("\n## Checks\n\n| Check | Expected | Actual | Result | Note |\n|---|---|---|---|---|\n")
        for name, exp, act, ok, note in steps.rows:
            act = act.replace("|", "\\|")
            r.write(f"| {name} | {exp} | {act} | {'PASS' if ok else 'FAIL'} | {note} |\n")
        if server.fatal:
            r.write(f"\n## First fatal line\n\n```\n{server.fatal}\n```\n")
        r.write("\n## Mod jars\n\n" + "\n".join(f"- {m}" for m in mods) + "\n")


if __name__ == "__main__":
    main()
