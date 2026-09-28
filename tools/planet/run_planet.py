#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Drive the dev planet server: a real Deep Time + Toroidal World overworld (plan §9.4).

Called by ./gametest-planet.sh; see there for the flags. Standard library only.

The server is ModDevGradle's `runPlanetServer` (a dev dedicated server, so /test exists) in
run-planet/, on Deep Time's planet ports (game 25666, RCON 25676). GameTest results come from
Deep Time's own reporter (DeepTimeTestReporter): a headless 1.21.1 server logs no passes and no
summary of its own, so after `/test runall <class>` the harness polls
`/deeptime gametest summary` until every expected test has a result, with a timeout.
"""

import argparse
import json
import os
import re
import secrets
import shutil
import signal
import subprocess
import sys
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RUN = os.path.join(REPO, "run-planet")
GAME_PORT, RCON_PORT = 25666, 25676
DONE = re.compile(r"Done \(")
# RCON comes up after "Done (" (38 s later on the first planet boot), so wait for it too.
RCON_UP = re.compile(r"RCON running on")
FATAL = re.compile(r"InjectionError|MixinTransformerError|InvalidInjectionException|Critical injection failure"
                   r"|Preparing crash report|Failed to start the minecraft server|Mod [Ll]oading has failed")
PROBE = re.compile(r"\[deeptime-probe\]\s*(\{.*\})")
SUMMARY = re.compile(r"\[gametest-summary\]\s*(\{.*\})")


def tools():
    path = os.path.join(REPO, "smoke", "tools.env")
    if not os.path.exists(path):
        sys.exit("run_planet.py: no smoke/tools.env -- run tools/smoke/install.sh standalone first")
    return dict(l.strip().split("=", 1) for l in open(path) if "=" in l)


class Planet:
    def __init__(self, gradle_args, log_path):
        dev = [] if os.environ.get("IN_NIX_SHELL") else ["nix", "develop", "-c"]
        self.cmd = [*dev, "./gradlew", "runPlanetServer", "--console=plain", *gradle_args]
        self.log_path = log_path
        self.lines, self.fatal, self.done, self.rcon_up = [], None, False, False

    def start(self):
        self.lines, self.fatal, self.done, self.rcon_up = [], None, False, False
        self.log = open(self.log_path, "a", buffering=1, errors="replace")
        self.proc = subprocess.Popen(self.cmd, cwd=os.path.join(REPO, "neoforge"), stdin=subprocess.PIPE,
                                     stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                                     errors="replace", bufsize=1, start_new_session=True)
        threading.Thread(target=self._read, daemon=True).start()

    def _read(self):
        for line in self.proc.stdout:
            self.log.write(line)
            self.lines.append(line)
            if DONE.search(line):
                self.done = True
            if RCON_UP.search(line):
                self.rcon_up = True
            if self.fatal is None and FATAL.search(line):
                self.fatal = line.rstrip()

    def wait_done(self, timeout):
        end = time.time() + timeout
        while time.time() < end:
            if self.done and self.rcon_up:
                return True
            if self.fatal or self.proc.poll() is not None:
                self.fatal = self.fatal or f"exited with {self.proc.returncode} before Done"
                return False
            time.sleep(0.5)
        self.fatal = f"timeout after {timeout}s waiting for Done and RCON"
        return False

    def stop(self, rcon):
        if self.proc.poll() is not None:
            return
        try:
            rcon("stop")
        except Exception:
            pass
        try:
            self.proc.wait(timeout=180)
        except subprocess.TimeoutExpired:
            os.killpg(os.getpgid(self.proc.pid), signal.SIGTERM)
            self.proc.wait(timeout=60)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--boot-only", action="store_true")
    ap.add_argument("--size", type=int, default=16384, choices=(16384, 32768))
    ap.add_argument("--test-terrain", default="wrap-test")
    ap.add_argument("--run", action="append", default=[], metavar="CLASS@X,Y,Z",
                    help="run a test class positioned at X,Y,Z (repeatable)")
    ap.add_argument("--timeout", type=int, default=1200)
    ap.add_argument("gradle_args", nargs="*")
    a = ap.parse_args()

    t = tools()
    # Fresh world every run; mods/ is re-linked by Gradle's linkRunModsPlanetServer.
    shutil.rmtree(os.path.join(RUN, "world"), ignore_errors=True)
    os.makedirs(RUN, exist_ok=True)
    pw_path = os.path.join(RUN, "rcon.pw")
    if not os.path.exists(pw_path):
        open(pw_path, "w").write(secrets.token_hex(16))
        os.chmod(pw_path, 0o600)
    pw = open(pw_path).read().strip()
    open(os.path.join(RUN, "eula.txt"), "w").write("eula=true\n")  # accepted for test servers (plan §2.7)
    level_type = "deeptime\\:planet" if a.size == 16384 else "deeptime\\:planet_32k"
    open(os.path.join(RUN, "server.properties"), "w").write(
        "# Written by tools/planet/run_planet.py\n"
        f"server-port={GAME_PORT}\nenable-rcon=true\nrcon.port={RCON_PORT}\nrcon.password={pw}\n"
        f"level-type={level_type}\nlevel-name=world\nonline-mode=false\nmax-tick-time=-1\n"
        "spawn-protection=0\nview-distance=6\nsimulation-distance=4\nsync-chunk-writes=false\n")
    # The terrain source comes from a JVM property; ModDevGradle passes -P... through to the run
    # configuration only via the build script, so it is set with an environment variable that
    # the planetServer run reads (neoforge/build.gradle).
    os.environ["DEEPTIME_TEST_TERRAIN"] = a.test_terrain

    def rcon(cmd):
        r = subprocess.run([t["MCRCON"], "-H", "127.0.0.1", "-P", str(RCON_PORT), "-p", pw, "-c", cmd],
                           capture_output=True, text=True, timeout=600)
        return (r.stdout + r.stderr).strip()

    log_path = os.path.join(RUN, "planet.log")
    open(log_path, "w").close()
    srv = Planet(a.gradle_args, log_path)
    print(f"== planet server: C={a.size}, test terrain {a.test_terrain}", flush=True)
    srv.start()
    ok = srv.wait_done(a.timeout)
    failures = []
    try:
        if not ok:
            failures.append(f"boot: {srv.fatal}")
        else:
            print("   Done", flush=True)
            out = rcon("deeptime probe column 0 0")
            m = PROBE.search(out)
            print(f"   probe column 0 0: {m.group(1) if m else out}", flush=True)
            if not m:
                failures.append("probe column 0 0 did not answer")
            for spec in a.run:
                cls, _, pos = spec.partition("@")
                x, y, z = (pos or "0,100,0").split(",")
                rcon("deeptime gametest reset")
                rcon(f"forceload add {int(x) - 32} {int(z) - 32} {int(x) + 32} {int(z) + 32}")
                print(f"   /test runall {cls} at {x},{y},{z}", flush=True)
                print("     " + rcon(f"execute positioned {x} {y} {z} run test runall {cls}"), flush=True)
                end = time.time() + a.timeout
                summary = None
                while time.time() < end:
                    time.sleep(5)
                    m = SUMMARY.search(rcon("deeptime gametest summary"))
                    if m:
                        summary = json.loads(m.group(1))
                        if not any("Running" in l and "tests" in l for l in srv.lines[-5:]) and summary["passed"] + summary["failed"] > 0:
                            # Settle: a later poll with the same counts means the batch is done.
                            time.sleep(10)
                            again = json.loads(SUMMARY.search(rcon("deeptime gametest summary")).group(1))
                            if again == summary:
                                break
                print(f"   [gametest-summary] {cls}: {json.dumps(summary)}", flush=True)
                if not summary or summary["failed"] or summary["passed"] == 0:
                    failures.append(f"{cls}: {summary}")
    finally:
        srv.stop(rcon)
    print("PASS" if not failures else "FAIL: " + "; ".join(failures), flush=True)
    sys.exit(0 if not failures else 1)


if __name__ == "__main__":
    main()
