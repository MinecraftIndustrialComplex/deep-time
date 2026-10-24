#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Drive the dev planet server: a real Deep Time + Toroidal World overworld (plan §9.4).

Called by ./gametest-planet.sh; see there for the flags. Standard library only.

The server is ModDevGradle's `runPlanetServer` (a dev dedicated server, so /test exists). On the test
VM it runs through the admission tool (tools/remote/vm/admit.py, kind planet:<set>; Ben 2026-10-03:
load-based admission, no server lock) in the job's own folder (DEEPTIME_SERVER_DIR/run-planet) on the
job's ports, with a copy of the set's jars; elsewhere in run-planet/ on the old fixed ports (game
25666, RCON 25676). The log is always run-planet/planet.log in the checkout. GameTest results come from
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
sys.path.insert(0, os.path.join(REPO, "tools", "smoke"))
import admission  # noqa: E402  (tools/smoke/admission.py: the test VM's admission tool)
import instance  # noqa: E402  (tools/smoke/instance.py: jar snapshots, ports, the launch lock)

RUN = os.path.join(REPO, "run-planet")
LOG_DIR = RUN
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
        # Gradle writes the run's argument files into neoforge/build just before it starts the JVM:
        # two runs of this checkout must not interleave there (instance.launch_lock).
        with instance.launch_lock(REPO):
            self.proc = subprocess.Popen(self.cmd, cwd=os.path.join(REPO, "neoforge"), stdin=subprocess.PIPE,
                                         stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
                                         errors="replace", bufsize=1, start_new_session=True)
            threading.Thread(target=self._read, daemon=True).start()
            instance.wait_jvm(RUN, self.proc.pid, timeout=1800)

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
    ap.add_argument("--cube-4k", action="store_true",
                    help="the overworld is planet_world_4k baked as a cube world (format 0.7, s = 1,024, period "
                         "4,096; cube plan phase 2) under the gametest level type deeptime:test_planet_cube_4k, "
                         "with the probes compared against inspect-column on every face")
    ap.add_argument("--run", action="append", default=[], metavar="TEST[,TEST...]@X,Y,Z[~DX,DZ]",
                    help="run tests by name (lower-case method names) positioned at X,Y,Z, forceloading "
                         "X±DX, Z±DZ (default 32,32) (repeatable)")
    ap.add_argument("--scenario", action="append", default=[], choices=("sable_straddle", "refuse_client"),
                    help="a scripted scenario after the tests (plan §9.4; repeatable)")
    ap.add_argument("--timeout", type=int, default=1200)
    ap.add_argument("gradle_args", nargs="*")
    a = ap.parse_args()

    run_set = "mic" if "-PwithMic" in a.gradle_args else "standalone"
    admission.ensure(f"planet:{run_set}")
    global RUN, GAME_PORT, RCON_PORT
    ports = admission.ports()
    if ports:
        GAME_PORT, RCON_PORT = ports["game"], ports["rcon"]
    if admission.server_dir():
        RUN = os.path.join(admission.server_dir(), "run-planet")
        os.environ["DEEPTIME_PLANET_DIR"] = RUN
    t = tools()
    # Fresh world every run; mods/ is re-linked by Gradle's linkRunModsPlanetServer, from a copy of
    # the set's jars (an install.sh meanwhile cannot change them under the server).
    shutil.rmtree(os.path.join(RUN, "world"), ignore_errors=True)
    os.makedirs(RUN, exist_ok=True)
    os.makedirs(LOG_DIR, exist_ok=True)
    set_server = os.path.join(REPO, "smoke", run_set, "server")
    if admission.server_dir() and os.path.isdir(os.path.join(set_server, "mods")):
        os.environ["DEEPTIME_RUN_MODS"] = instance.snapshot_mods(set_server, os.path.join(RUN, "set-mods"))
    if ports:
        instance.write_ports(RUN, ports)
    pw_path = os.path.join(RUN, "rcon.pw")
    if not os.path.exists(pw_path):
        open(pw_path, "w").write(secrets.token_hex(16))
        os.chmod(pw_path, 0o600)
    pw = open(pw_path).read().strip()
    open(os.path.join(RUN, "eula.txt"), "w").write("eula=true\n")  # accepted for test servers (plan §2.7)
    # The analytic test terrains are Lambert worlds (C itself): the Lambert level types (format 0.5;
    # deeptime:planet[_32k] are the Petroff-Guyou worlds, 13,312 / 26,112 blocks round).
    level_type = "deeptime\\:planet_lambert" if a.size == 16384 else "deeptime\\:planet_lambert_32k"
    if a.cube_4k:
        # World data, not a test terrain: the bake writes world/deeptime before the server starts.
        level_type, a.test_terrain = "deeptime\\:test_planet_cube_4k", ""
        dev = [] if os.environ.get("IN_NIX_SHELL") else ["nix", "develop", "-c"]
        bake = subprocess.run([*dev, "cargo", "run", "--release", "-q", "-p", "deeptime-cli", "--", "run", "--preset",
                               "crates/deeptime-core/tests/presets/planet_world_4k.json", "--set", "scale.projection=cube",
                               "--out", os.path.join(RUN, "world", "deeptime")], cwd=REPO)
        if bake.returncode != 0:
            sys.exit("run_planet.py: the 4k cube bake failed")
    open(os.path.join(RUN, "server.properties"), "w").write(
        "# Written by tools/planet/run_planet.py\n"
        f"server-port={GAME_PORT}\nenable-rcon=true\nrcon.port={RCON_PORT}\nrcon.password={pw}\n"
        + (f"query.port={ports['query']}\nenable-query=false\n" if ports else "") +
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

    log_path = os.path.join(LOG_DIR, "planet.log")
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
            out = rcon("deeptime probe tw")
            m = PROBE.search(out)
            print(f"   probe tw: {m.group(1) if m else out}", flush=True)
            if not m:
                failures.append("probe tw did not answer")
            if a.cube_4k:
                failures += cube_probes(rcon, os.path.join(RUN, "world", "deeptime"))
            for spec in a.run:
                failures += run_tests(spec, rcon, a.timeout, a.size)
            for scenario in a.scenario:
                failures += SCENARIOS[scenario](a, srv, rcon)
    finally:
        srv.stop(rcon)
    print("PASS" if not failures else "FAIL: " + "; ".join(failures), flush=True)
    sys.exit(0 if not failures else 1)


CUBE_4K_SIDE = 1024
# Columns on every face of the 4k cube (TW frame [-2s, 2s)): F0's centre and off it, F1, F3, F2 on
# both sides of the slide, N and S, and one by each of F0's corners inside the border.
CUBE_4K_COLUMNS = [(0, 0), (40, 25), (1024, -100), (-1100, 200), (2030, 10), (-2030, 10), (100, -1200),
                   (-200, 1300), (400, -400), (-400, 400)]


def cube_probes(rcon, world_data):
    """`/deeptime probe column` on the cube's faces against the CLI's `inspect-column --parity` (the
    Rust reader the bake was written by): the ground height must be the same number."""
    cli = os.path.join(REPO, "target", "release", "deeptime")
    failures, rows = [], []
    for x, z in CUBE_4K_COLUMNS:
        out = rcon(f"deeptime probe column {x} {z}")
        m = PROBE.search(out)
        ref = subprocess.run([cli, "inspect-column", "--world-data", world_data, "--parity", str(x), str(z)],
                             capture_output=True, text=True)
        if not m or ref.returncode != 0:
            failures.append(f"cube probe ({x},{z}) no answer: {out[:80] or ref.stderr[:80]}")
            continue
        probe, col = json.loads(m.group(1)), json.loads(ref.stdout)
        h_java, h_rust = probe.get("terrainH"), col.get("surface")
        same = h_java is not None and h_rust is not None and abs(h_java - h_rust) <= 0.01
        rows.append(f"({x},{z}) terrainH {h_java} vs {h_rust}, surfaceY {probe.get('surfaceY')} vs top_y {col.get('top_y')}")
        if not same:
            failures.append("cube probe " + rows[-1])
    print("   cube probes: " + "; ".join(rows), flush=True)
    return failures


def forceload(rcon, c, x0, z0, x1, z1, wait=180, points=()):
    """Forceload the block box [x0, x1] x [z0, z1] on the planet, then wait until it is loaded.

    Toroidal World's /forceload accepts canonical coordinates only (-C/2 <= x < C/2: "8217 is
    outside this world"), so a box across the seam is folded and split into two canonical boxes.
    Forceloading is asynchronous; `/test run` puts a test on the WORLD_SURFACE heightmap of its
    position, which reads y = min build height (-512) until the chunk is there, so the harness
    waits (execute if loaded, every corner) before running anything. Re-adding forced chunks is
    harmless (the command reports that nothing changed), so this runs before every test.
    """
    half = c // 2
    a = (x0 + half) % c - half
    b = a + (x1 - x0)
    boxes = [(a, b)] if b < half else [(a, half - 1), (-half, b - c)]
    for bx0, bx1 in boxes:
        print(f"   forceload add {bx0} {z0} {bx1} {z1}: " + rcon(f"forceload add {bx0} {z0} {bx1} {z1}"), flush=True)
    # Every corner, and every extra point (the test spots): the box's chunks do not all arrive
    # at once, and a test whose own chunk is late lands at the bottom of the world.
    corners = [(x, z) for bx0, bx1 in boxes for x in (bx0, bx1) for z in (z0, z1)] + list(points)
    end = time.time() + wait
    while time.time() < end:
        missing = [(x, z) for x, z in corners if "passed" not in rcon(f"execute if loaded {x} 0 {z}").lower()]
        if not missing:
            return True
        time.sleep(2)
    print(f"   forceload: not loaded after {wait}s: {missing}", flush=True)
    return False


def run_tests(spec, rcon, timeout, c):
    """Run one `--run` spec and return its failures.

    Spec: "<test>[,<test>...]@x,y,z[~dx,dz]". A test's name is its method name in lower case:
    Deep Time's holders use @PrefixGameTestTemplate(false), which also drops the class prefix
    that `/test runall <class>` would match, so tests are run by name. The harness forceloads
    x±dx, z±dz (default 32, 32) around the spot plus the 8-block z steps between tests. `/test
    run` ignores y: vanilla puts the test on the surface at x, z + 3 (TestCommand
    .createTestPositionAround), so tests place things relative to their own position.
    """
    names_part, _, where = spec.partition("@")
    names = [n.strip().lower() for n in names_part.split(",") if n.strip()]
    pos, _, extent = (where or "0,100,0").partition("~")
    x, y, z = (int(v) for v in pos.split(","))
    dx, dz = (int(v) for v in (extent or "32,32").split(","))
    rcon("deeptime gametest reset")
    last_z = z + 8 * (len(names) - 1)
    # `/test run` at (x, z) puts the test at (x, surface + 1, z + 3).
    spots = [(x, z + 8 * i + 3) for i in range(len(names))]
    # One test at a time: 1.21.1's `/test run` begins with TestCommand.stopTests(), which cancels
    # every test still running, so a second /test run issued before the first test reported
    # silently kills it (P0-06 finding). Each test also gets its own spot, 8 blocks apart in z,
    # so no test's structure lands on another's leftovers.
    summary = {}
    for n_i, name in enumerate(names):
        tz = z + 8 * n_i
        # Forceload again before every test: when a test batch ends, vanilla's GameTestRunner
        # un-forces every forced chunk of the level (level.getForcedChunks()), the harness's too.
        if not forceload(rcon, c, x - dx, z - dz, x + dx, last_z + dz, points=spots):
            return [f"{name}: forceloaded chunks never loaded"]
        m = PROBE.search(rcon(f"deeptime probe column {x} {tz + 3}"))
        where = json.loads(m.group(1)) if m else {}
        print(f"   spot {x},{tz + 3}: chunkLoaded={where.get('chunkLoaded')} worldSurface={where.get('worldSurface')} "
              f"surfaceY={where.get('surfaceY')}", flush=True)
        print(f"   /test run {name} at {x},{y},{tz}: " + rcon(f"execute positioned {x} {y} {tz} run test run {name}"), flush=True)
        end = time.time() + timeout
        while time.time() < end:
            time.sleep(2)
            m = SUMMARY.search(rcon("deeptime gametest summary"))
            summary = json.loads(m.group(1)) if m else {}
            r = summary.get("tests", {}).get(name)
            # A repeated test reports its attempts before its one final result: wait for the result.
            if r and r["pass"] + r["fail"] > 0:
                break
    failures = []
    tests = summary.get("tests", {})
    for n in names:
        r = tests.get(n)
        print(f"   [gametest-summary] {n}: {r if r else 'NO RESULT (timeout)'}", flush=True)
        if not r or r["pass"] + r["fail"] == 0:
            failures.append(f"{n}: no result")
            continue
        passed = r["pass"] > 0 and r["fail"] == 0
        # Repeated tests: the one PASS vanilla reports means requiredSuccesses passing attempts;
        # when the body reports its attempts, they must agree.
        if passed and "attempts" in r and r["attempts"] > 0 and r["attemptsOk"] < r["requiredSuccesses"]:
            passed = False
        if "attempts" in r:
            print(f"   {n}: {r.get('attemptsOk', 0)}/{r.get('requiredSuccesses')} attempts passed "
                  f"({r.get('attempts', 0)} run, max {r.get('maxAttempts')})", flush=True)
        if not passed:
            if r.get("required", True):
                failures.append(f"{n}: {r}")
            else:
                print(f"   {n}: optional test did not pass (informational)", flush=True)
    return failures


def scenario_sable_straddle(a, srv, rcon):
    """Plan P0-06 step 4: a Sable hull straddling the seam survives save, stop and start.

    Assembles a hull across x = C/2 (/deeptime gametest sable straddle), lets it settle, records
    /deeptime probe sable, saves, stops, starts the same world again and waits until the same
    sub-level id is back; prints `sable_straddle: OK` or the difference.
    """
    c = a.size
    z = 0
    # The hull and its neighbourhood stay loaded before and after the restart (forced chunks
    # persist in the world), so the sub-level's holding chunk loads with the world.
    if not forceload(rcon, c, c // 2 - 32, z - 32, c // 2 + 31, z + 32):
        return ["sable_straddle: the seam chunks never loaded"]
    out = rcon(f"deeptime gametest sable straddle {z}")
    m = PROBE.search(out)
    print(f"   assemble: {m.group(1) if m else out}", flush=True)
    if not m or "id" not in json.loads(m.group(1)):
        return [f"sable_straddle: assembly failed: {out[:300]}"]
    made = json.loads(m.group(1))
    hull = made["id"]
    time.sleep(10)   # let it fall and settle before saving

    def probe():
        m = PROBE.search(rcon("deeptime probe sable"))
        return json.loads(m.group(1)) if m else None

    before = probe()
    print(f"   before: {before}", flush=True)
    mine = next((s for s in (before or {}).get("subLevels", []) if s["id"] == hull), None)
    if mine is None:
        return [f"sable_straddle: hull {hull} not in the probe before saving: {before}"]
    print("   save-all flush: " + rcon("save-all flush"), flush=True)
    srv.stop(rcon)
    print("   restarting", flush=True)
    srv.start()
    if not srv.wait_done(a.timeout):
        return [f"sable_straddle: restart failed: {srv.fatal}"]
    after, again = None, None
    end = time.time() + 120
    while time.time() < end:
        after = probe()
        again = next((s for s in (after or {}).get("subLevels", []) if s["id"] == hull), None)
        if again is not None:
            break
        time.sleep(3)
    print(f"   after: {after}", flush=True)
    if again is None:
        # Diagnostics: what Sable itself has on disk.
        print("   /sable storage find_all_sub_levels: " + rcon("sable storage find_all_sub_levels")[:600], flush=True)
        print(f"sable_straddle: FAIL (sub-level {hull} is gone after the restart)", flush=True)
        return [f"sable_straddle: sub-level {hull} missing after restart"]
    if again["blocks"] != mine["blocks"]:
        print(f"sable_straddle: FAIL (blocks {mine['blocks']} before, {again['blocks']} after)", flush=True)
        return [f"sable_straddle: block count {mine['blocks']} -> {again['blocks']}"]
    print(f"sable_straddle: OK (sub-level {hull}, {again['blocks']} blocks, "
          f"x {mine['x']:.2f} before, {again['x']:.2f} after)", flush=True)
    return []


def scenario_refuse_client(a, srv, rcon):
    """P1-18 (plan §7.15): a client without Deep Time cannot join; one with it gets past the check.

    Reads the server's own channel list (/deeptime probe network), then connects over TCP three
    times with tools/planet/mcclient.py: a NeoForge client offering every channel except Deep
    Time's must be refused with exactly Deep Time's required channels named; one offering every
    channel must pass the negotiation (neoforge:network); a vanilla client must be refused.
    Prints `refuse_client: OK` or what differed.
    """
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import mcclient
    # ~200 channels make a probe line of ~20 kB, and RCON returns only its first 4 kB: read the
    # server's own log line instead.
    rcon("deeptime probe network")
    channels, end = None, time.time() + 30
    while channels is None and time.time() < end:
        for line in reversed(srv.lines):
            m = PROBE.search(line)
            if m and '"probe":"network"' in line:
                channels = json.loads(m.group(1))["channels"]
                break
        else:
            time.sleep(0.5)
    if channels is None:
        return ["refuse_client: /deeptime probe network did not answer"]
    ours = [c for c in channels if c["id"].startswith("deeptime:")]
    required_ours = sorted(c["id"] for c in ours if not c["optional"])
    print(f"   server channels: {len(channels)} ({len(ours)} Deep Time: "
          + ", ".join(f"{c['id']} {c['protocol']} v{c['version']} {'optional' if c['optional'] else 'required'}" for c in ours)
          + ")", flush=True)
    failures = []
    if not required_ours:
        failures.append("refuse_client: Deep Time has no required channel")
    without = [c for c in channels if not c["id"].startswith("deeptime:")]
    host = "127.0.0.1"
    r = mcclient.attempt(host, GAME_PORT, "neoforge", without)
    print(f"   NeoForge client without Deep Time: {r['outcome']}, refused channels {sorted(r['refused_channels'])}, "
          f"disconnect {r['disconnect'][:2]} {r.get('error', '')}", flush=True)
    if r["outcome"] != "refused" or sorted(r["refused_channels"]) != required_ours:
        failures.append(f"refuse_client: a client without Deep Time: {r['outcome']} {sorted(r['refused_channels'])} "
                        f"(expected refused {required_ours}) {r.get('error', '')} events {r['events'][-6:]}")
    r = mcclient.attempt(host, GAME_PORT, "neoforge", channels)
    print(f"   NeoForge client with every channel: {r['outcome']} {r.get('error', '')}", flush=True)
    if r["outcome"] != "accepted":
        failures.append(f"refuse_client: a client with every channel: {r['outcome']} {r['refused_channels']} "
                        f"{r['disconnect'][:2]} {r.get('error', '')}")
    r = mcclient.attempt(host, GAME_PORT, "vanilla")
    print(f"   vanilla client: {r['outcome']}, disconnect {r['disconnect'][:2]} {r.get('error', '')}", flush=True)
    if r["outcome"] != "refused":
        failures.append(f"refuse_client: a vanilla client: {r['outcome']} {r.get('error', '')}")
    print("refuse_client: OK" if not failures else "refuse_client: FAIL", flush=True)
    return failures


SCENARIOS = {"sable_straddle": scenario_sable_straddle, "refuse_client": scenario_refuse_client}


if __name__ == "__main__":
    main()
