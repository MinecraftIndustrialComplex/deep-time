#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""PROTOTYPE, dev-only (docs/design/seamless-wrapping.md): a Deep Time world in the Guyou aspect of
the Peirce quincuncial projection, its seams joined by Immersive Portals – CE global portals.

Runs on the test VM, holding the server lock:
  tools/remote/mac.sh --server -- 'python3 tools/proto/quincuncial/proto.py all'

Phases (each also alone): setup, boot, portals, terrain, entities, client, physics, stop; `all` runs
them in order. Results land in $RUNS/out/ (report.json, *.png, *.tsv, logs).

What it needs on the VM (nothing here is redistributed or committed):
  - $RUNS/guyou-world: `deeptime-quincuncial resample` output (the world data);
  - $RUNS/deeptime-quincuncial: the prototype tool (for the portal table);
  - the Deep Time jar built from this checkout (neoforge/build/libs), patched by notw_jar.py;
  - Immersive Portals – CE 6.0.10 and Cloth Config, the copies the 2026-10-02 prototype downloaded
    (/data/ban/mic/mods/ip-proto), and that prototype's NeoForge server libraries and portablemc
    data (copied, not changed);
  - the Mesa/Xvfb/ffmpeg/xdotool store paths in ip-proto/scripts/env.sh.
Ports: game 25985, RCON 25995 (never the play server's 25765).
"""
import json, os, re, shutil, socket, struct, subprocess, sys, time

REPO = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
RUNS = os.environ.get("RUNS", "/data/ban/deeptime-runs/quincuncial")
MC = os.path.join(RUNS, "mc")
SRV = os.path.join(MC, "server")
CLI = os.path.join(MC, "client")
OUT = os.path.join(RUNS, "out")
IPP = "/data/ban/mic/mods/ip-proto"
GAME_PORT, RCON_PORT = 25985, 25995
B = 8192  # hemisphere square side: the world is 2B x B  (ledger: SCL-076, game-choice, prototype)
LON = 31.0  # the western hemisphere centre; puts the singular points at sea (ledger: SCL-076)
PLAYER = "IPTester"


def env():
    out = subprocess.run(["bash", "-c", f"source {IPP}/scripts/env.sh; env -0"], capture_output=True, check=True).stdout
    e = dict(kv.split("=", 1) for kv in out.decode().split("\0") if "=" in kv)
    e["DISPLAY"] = ":95"
    return e


ENV = env()


def log(*a):
    print(time.strftime("%H:%M:%S"), *a, flush=True)


def report(key, value):
    path = os.path.join(OUT, "report.json")
    data = json.load(open(path)) if os.path.exists(path) else {}
    data[key] = value
    json.dump(data, open(path, "w"), indent=1)


# ---------------------------------------------------------------- RCON

class Rcon:
    def __init__(self):
        pw = open(os.path.join(SRV, "rcon.pw")).read().strip()
        self.s = socket.create_connection(("127.0.0.1", RCON_PORT), timeout=120)
        self.id = 0
        self.cmd(pw, kind=3)

    def cmd(self, text, kind=2):
        self.id += 1
        body = text.encode() + b"\0\0"
        self.s.sendall(struct.pack("<iii", len(body) + 8, self.id, kind) + body)
        n = struct.unpack("<i", self._read(4))[0]
        return self._read(n)[8:-2].decode(errors="replace")

    def _read(self, n):
        b = b""
        while len(b) < n:
            c = self.s.recv(n - len(b))
            if not c:
                raise EOFError("rcon closed")
            b += c
        return b


_r = None


def rc(c, show=False):
    global _r
    for attempt in range(3):
        try:
            if _r is None:
                _r = Rcon()
            o = _r.cmd(c)
            if show:
                log(f"> {c[:150]}\n    {o[:300]}")
            return o
        except (OSError, EOFError):
            _r = None
            time.sleep(2)
    raise RuntimeError(f"rcon failed: {c}")


NUM = re.compile(r"(-?\d+\.?\d*(?:[eE]-?\d+)?)[dfb]?")


def entity_vec(sel, key):
    o = rc(f"data get entity {sel} {key}")
    if "No entity" in o or "found" in o and ":" not in o:
        return None
    return [float(v) for v in NUM.findall(o.split(":", 1)[-1])]


# ---------------------------------------------------------------- the world's seams

def seams():
    out = subprocess.run([os.path.join(RUNS, "deeptime-quincuncial"), "portals", "--layout", "guyou", "--size", str(B),
                          "--lon", str(LON), "--y", "64", "--height", "1200"], capture_output=True, check=True).stdout
    return json.loads(out)


def glue_point(p, x, z):
    """The glued image of a point (x, z) of the plane, by portal p (from `seams()`)."""
    ox, _, oz = p["origin"]
    dx, _, dz = p["destination"]
    if p["kind"] == "half_turn":
        return 2 * ox - x, 2 * oz - z
    return x + dx - ox, z + dz - oz


def inward(p):
    wx, _, wz = p["axis_w"]
    return -wz, wx


# ---------------------------------------------------------------- phases

def setup():
    os.makedirs(OUT, exist_ok=True)
    jars = sorted(f for f in os.listdir(os.path.join(REPO, "neoforge/build/libs")) if f.startswith("deeptime-1.21.1") and "sources" not in f)
    assert jars, "build the Deep Time jar first (cd neoforge && ./gradlew jar)"
    notw = os.path.join(MC, "deeptime-noTW.jar")
    os.makedirs(MC, exist_ok=True)
    subprocess.run([sys.executable, os.path.join(REPO, "tools/proto/quincuncial/notw_jar.py"),
                    os.path.join(REPO, "neoforge/build/libs", jars[-1]), notw], check=True)
    if not os.path.isdir(SRV):
        subprocess.run(["rsync", "-a", "--exclude=/world", "--exclude=/logs", "--exclude=/console.log", "--exclude=/mods",
                        "--exclude=/server.pid", f"{IPP}/vp2-server/", SRV + "/"], check=True)
    mods = os.path.join(SRV, "mods")
    shutil.rmtree(mods, ignore_errors=True)
    os.makedirs(mods)
    for j in (f"{IPP}/dl/immersive_portals-6.0.10.jar", f"{IPP}/vp2-server/mods/cloth-config-15.0.140-neoforge.jar", notw):
        shutil.copy(j, mods)
    pw = open(os.path.join(SRV, "rcon.pw")).read().strip()
    open(os.path.join(SRV, "server.properties"), "w").write(f"""# deep-time tools/proto/quincuncial/proto.py (dev-only prototype)
server-port={GAME_PORT}
query.port={GAME_PORT}
enable-rcon=true
rcon.port={RCON_PORT}
rcon.password={pw}
online-mode=false
level-type=deeptime\\:planet_32k
level-name=planet
gamemode=creative
force-gamemode=true
difficulty=peaceful
allow-flight=true
spawn-protection=0
max-tick-time=-1
view-distance=10
simulation-distance=6
enable-command-block=true
sync-chunk-writes=false
spawn-monsters=false
motd=Deep Time quincuncial prototype
""")
    open(os.path.join(SRV, "user_jvm_args.txt"), "w").write("-Xmx8G\n-XX:+UseG1GC\n-XX:ActiveProcessorCount=6\n")
    world = os.path.join(SRV, "planet")
    shutil.rmtree(world, ignore_errors=True)
    os.makedirs(world)
    subprocess.run(["cp", "-a", os.path.join(RUNS, "guyou-world"), os.path.join(world, "deeptime")], check=True)
    # Client: its own game dir; the portablemc data copied once.
    game = os.path.join(CLI, "game")
    os.makedirs(CLI, exist_ok=True)
    if not os.path.isdir(game):
        subprocess.run(["rsync", "-a", "--exclude=/logs", "--exclude=/saves", "--exclude=/mods", f"{IPP}/vp-client/game/", game + "/"], check=True)
    cm = os.path.join(game, "mods")
    shutil.rmtree(cm, ignore_errors=True)
    os.makedirs(cm)
    for j in os.listdir(mods):
        shutil.copy(os.path.join(mods, j), cm)
    if not os.path.isdir(os.path.join(MC, "pmc")):
        subprocess.run(["cp", "-a", f"{IPP}/pmc", os.path.join(MC, "pmc")], check=True)
    log("setup done:", os.listdir(mods))


def server_up():
    try:
        pid = int(open(os.path.join(SRV, "server.pid")).read())
        os.kill(pid, 0)
        return pid
    except (OSError, ValueError):
        return None


def boot(timeout=1800):
    if server_up():
        log("server already up")
        return
    cons = open(os.path.join(SRV, "console.log"), "a")
    cons.write(f"\n===== proto.py boot {time.strftime('%H:%M:%S')}\n")
    cons.flush()
    start_at = os.path.getsize(os.path.join(SRV, "console.log"))
    p = subprocess.Popen(["nice", "-n", "5", "taskset", "-c", "16-23", ENV["JAVA"], "@user_jvm_args.txt",
                          "@libraries/net/neoforged/neoforge/21.1.249/unix_args.txt", "nogui"],
                         cwd=SRV, stdout=cons, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL, start_new_session=True)
    open(os.path.join(SRV, "server.pid"), "w").write(str(p.pid))
    t0 = time.time()
    while time.time() - t0 < timeout:
        with open(os.path.join(SRV, "console.log"), errors="replace") as f:
            f.seek(start_at)
            text = f.read()
        if re.search(r"Done \(", text):
            log("server up in", round(time.time() - t0), "s")
            global _r
            _r = None
            return
        if p.poll() is not None:
            raise RuntimeError("server died:\n" + text[-3000:])
        time.sleep(5)
    raise RuntimeError("server boot timed out")


def stop():
    global _r
    pid = server_up()
    if not pid:
        return
    try:
        rc("stop")
    except Exception:
        pass
    _r = None
    # The server saves and stops within seconds; with Immersive Portals the JVM can linger after
    # "All dimensions are saved" (a non-daemon thread), so TERM and then KILL it.
    for sig, wait in ((0, 60), (15, 30), (9, 10)):
        if sig:
            log(f"server JVM still alive, signal {sig}")
            try:
                os.kill(pid, sig)
            except OSError:
                pass
        for _ in range(wait):
            try:
                os.kill(pid, 0)
                os.waitpid(pid, os.WNOHANG)
            except ChildProcessError:
                pass
            except OSError:
                log("server stopped")
                os.remove(os.path.join(SRV, "server.pid"))
                return
            time.sleep(1)


def wait_loaded(x, z, secs=120):
    for _ in range(secs):
        if "passed" in rc(f"execute if loaded {int(x)} 0 {int(z)}"):
            return True
        time.sleep(1)
    return False


def portals():
    t = seams()
    report("portals", t["portals"])
    report("summon", t["summon"])
    tps0 = rc("neoforge tps")
    for p, cmd in zip(t["portals"], t["summon"]):
        ox, _, oz = p["origin"]
        rc(f"forceload add {int(ox) - 1} {int(oz) - 1} {int(ox) + 1} {int(oz) + 1}")
        assert wait_loaded(ox, oz), f"chunk at {ox},{oz} never loaded"
        time.sleep(1)
        rc(cmd, show=True)
        time.sleep(1)
        rc("execute as @e[type=immersive_portals:portal] run portal global convert_normal_portal_to_global_portal", show=True)
        rc(f"forceload remove {int(ox) - 1} {int(oz) - 1} {int(ox) + 1} {int(oz) + 1}")
    left = rc("execute if entity @e[type=immersive_portals:portal]")
    report("portal_entities_left_after_convert", left)
    report("tps_before_portals", tps0)
    # A fresh global portal moves entities only near its centre until a restart (2026-10-02 note §1).
    stop()
    time.sleep(5)
    boot()


def probe(x, z):
    o = rc(f"deeptime probe column {int(x)} {int(z)}")
    m = re.search(r"\{.*\}", o, re.S)
    return json.loads(m.group(0)) if m else {"raw": o}


def terrain():
    """The data-level and generator-level seam check: across each seam, the folded outside equals the
    glued inside, and the step from a block to its glued neighbour is like the step between ordinary
    neighbours."""
    t = seams()
    res = {}
    for p in t["portals"]:
        name, length = p["name"], p["width"]
        ix, iz = inward(p)
        ox, _, oz = p["origin"]
        along = (p["axis_w"][0], p["axis_w"][2])
        same, cross, interior, rows = [], [], [], []
        n = 48
        for k in range(n):
            s = -0.5 * length + (k + 0.5) * length / n
            ex, ez = ox + along[0] * s, oz + along[1] * s
            ex, ez = round(ex), round(ez)
            a = (ex + ix * 1, ez + iz * 1)        # one block inside
            b = (ex + ix * 2, ez + iz * 2)        # two blocks inside (ordinary neighbour of a)
            o = (ex - ix * 1, ez - iz * 1)        # one block outside (hidden, folded)
            g = glue_point(p, *o)                 # its glued image, inside on the far side
            ha, hb, ho, hg = (probe(*q).get("terrainH") for q in (a, b, o, g))
            if None in (ha, hb, ho, hg):
                continue
            same.append(abs(ho - hg))
            cross.append(abs(ha - hg) / 2.0)      # a and g are 2 blocks apart across the seam
            interior.append(abs(ha - hb))
            rows.append({"edge": [ex, ez], "a": a, "glued": g, "H_a": round(ha, 2), "H_outside": round(ho, 2), "H_glued": round(hg, 2)})
        def st(v):
            v = sorted(v)
            return {"n": len(v), "median": round(v[len(v) // 2], 3), "p90": round(v[int(0.9 * (len(v) - 1))], 3), "max": round(v[-1], 3)}
        res[name] = {"kind": p["kind"], "outside_minus_glued": st(same), "per_block_step_across_seam": st(cross),
                     "per_block_step_interior": st(interior), "samples": rows[:6]}
        log(name, json.dumps({k: v for k, v in res[name].items() if k != "samples"}))
    report("terrain_seams", res)


def entities():
    """Items and armour stands sent across each seam; where they land against the glued image."""
    t = seams()
    out = []
    run_id = time.strftime("%H%M%S")
    rc("kill @e[tag=qx]")
    for p in t["portals"]:
        ix, iz = inward(p)
        ox, _, oz = p["origin"]
        ax = (p["axis_w"][0], p["axis_w"][2])
        s = 1200.5 if p["kind"] == "half_turn" else 700.5
        ex, ez = ox + ax[0] * s, oz + ax[1] * s
        start = (ex + ix * 3, ez + iz * 3)
        far = glue_point(p, ex - ix * 3, ez - iz * 3)
        for q in (start, far):
            rc(f"forceload add {int(q[0]) - 16} {int(q[1]) - 16} {int(q[0]) + 16} {int(q[1]) + 16}")
            wait_loaded(*q)
        y = max(probe(*start).get("surfaceY", 100), probe(*far).get("surfaceY", 100), 63) + 20
        mx, mz = -ix * 0.8, -iz * 0.8
        for kind, nbt in (("item", 'Item:{id:"minecraft:diamond",count:1},PickupDelay:32767,NoGravity:1b'),
                          ("minecart", "NoGravity:1b"), ("arrow", "NoGravity:1b,pickup:0b")):
            tag = f"qx_{run_id}_{p['name']}_{kind}"
            rc(f"summon {kind} {start[0] + 0.5:.2f} {y:.1f} {start[1] + 0.5:.2f} {{Tags:[\"qx\",\"{tag}\"],Motion:[{mx}d,0.0d,{mz}d],{nbt}}}")
        time.sleep(6)
        for kind in ("item", "minecart", "arrow"):
            tag = f"qx_{run_id}_{p['name']}_{kind}"
            pos = entity_vec(f"@e[tag={tag},limit=1]", "Pos")
            want = glue_point(p, start[0] + 0.5 - ix * 6, start[1] + 0.5 - iz * 6)
            row = {"seam": p["name"], "entity": kind, "start": [start[0] + 0.5, y, start[1] + 0.5], "pos": pos,
                   "expected_near": [round(want[0], 1), round(want[1], 1)]}
            if pos:
                d0 = ((pos[0] - start[0]) ** 2 + (pos[2] - start[1]) ** 2) ** 0.5
                gx, gz = glue_point(p, pos[0], pos[2])
                row["crossed"] = d0 > 100
                # Distance of the landing point from the seam line on the far side, and how far along.
                row["landing_offset_from_glued_track"] = round(abs((pos[0] - want[0]) * ax[0] + (pos[2] - want[1]) * ax[1]), 2)
            out.append(row)
            log(json.dumps(row))
        for q in (start, far):
            rc(f"forceload remove {int(q[0]) - 16} {int(q[1]) - 16} {int(q[0]) + 16} {int(q[1]) + 16}")
    report("entities", out)


# ---------------------------------------------------------------- the real client

def xd(*a):
    subprocess.run([ENV["XDOTOOL"], *a], env=ENV, check=False)


def shot(name):
    path = os.path.join(OUT, name + ".png")
    subprocess.run([ENV["FFMPEG"], "-nostdin", "-loglevel", "error", "-y", "-f", "x11grab", "-draw_mouse", "0",
                    "-video_size", "1280x720", "-i", ENV["DISPLAY"], "-frames:v", "1", path], env=ENV, check=False)
    return path


def client_start():
    if not os.path.exists("/tmp/.X11-unix/X95"):
        subprocess.Popen([ENV["XVFB"], ":95", "-screen", "0", "1280x720x24", "-nolisten", "tcp", "+extension", "GLX"],
                         env=ENV, stdout=open(os.path.join(MC, "xvfb.log"), "w"), stderr=subprocess.STDOUT, start_new_session=True)
        time.sleep(3)
    pmc = subprocess.run(["nix", "build", "--no-link", "--print-out-paths", "nixpkgs#portablemc"], capture_output=True, check=True).stdout.decode().split()[0]
    game = os.path.join(CLI, "game")
    opts = os.path.join(game, "options.txt")
    if os.path.exists(opts):
        lines = [ln for ln in open(opts).read().splitlines()
                 if not ln.startswith(("renderDistance:", "pauseOnLostFocus:", "simulationDistance:"))]
        lines += ["renderDistance:8", "simulationDistance:6", "pauseOnLostFocus:false"]
        open(opts, "w").write("\n".join(lines) + "\n")
    lp = os.path.join(game, "logs", "latest.log")
    if os.path.exists(lp):
        os.remove(lp)
    p = subprocess.Popen(["taskset", "-c", "24-31", pmc + "/bin/portablemc", "--main-dir", os.path.join(MC, "pmc"), "start",
                          "neoforge::21.1.249", "--mc-dir", game, "--jvm", ENV["JAVA"], "--jvm-arg=-Xmx6G", "--jvm-arg=-Xms2G",
                          "--jvm-arg=-XX:+UseG1GC", "--jvm-arg=-XX:ActiveProcessorCount=8", "-u", PLAYER, "--resolution", "1280x720",
                          "--join-server", "127.0.0.1", "--join-server-port", str(GAME_PORT)],
                         env=ENV, stdout=open(os.path.join(CLI, "client.log"), "w"), stderr=subprocess.STDOUT, start_new_session=True)
    open(os.path.join(CLI, "client.pid"), "w").write(str(p.pid))
    cons_at = os.path.getsize(os.path.join(SRV, "console.log"))
    t0 = time.time()
    while time.time() - t0 < 900:
        if os.path.exists(lp) and "Level init received" in open(lp, errors="replace").read():
            break
        with open(os.path.join(SRV, "console.log"), errors="replace") as f:
            f.seek(cons_at)
            if f"{PLAYER} joined" in f.read():
                break
        if p.poll() is not None:
            raise RuntimeError("client died: " + open(os.path.join(CLI, "client.log"), errors="replace").read()[-2000:])
        time.sleep(5)
    else:
        raise RuntimeError("client never joined")
    log("client joined after", round(time.time() - t0), "s")
    time.sleep(20)


def procs():
    """(pid, ppid, cmdline) of every process, from /proc (the dev shell's procps tools do not run
    on the VM: their libc does not match)."""
    out = []
    for d in os.listdir("/proc"):
        if not d.isdigit():
            continue
        try:
            stat = open(f"/proc/{d}/stat").read()
            ppid = int(stat[stat.rindex(")") + 2:].split()[1])
            cmd = open(f"/proc/{d}/cmdline", "rb").read().replace(b"\0", b" ").decode(errors="replace")
            out.append((int(d), ppid, cmd))
        except (OSError, ValueError):
            pass
    return out


def kill_tree(pid):
    kids = [p for p, pp, _ in procs() if pp == pid]
    for k in kids:
        kill_tree(k)
    try:
        os.kill(pid, 15)
    except OSError:
        pass


def client_stop():
    try:
        kill_tree(int(open(os.path.join(CLI, "client.pid")).read()))
    except (OSError, ValueError):
        pass
    for pid, _, cmd in procs():
        if "Xvfb :95 " in cmd + " ":
            try:
                os.kill(pid, 15)
            except OSError:
                pass


def player_state():
    p = entity_vec(PLAYER, "Pos")
    r = entity_vec(PLAYER, "Rotation")
    return p, r


def fly_mode():
    rc(f"gamemode creative {PLAYER}")
    rc(f"effect give {PLAYER} minecraft:night_vision infinite 0 true")
    # Creative flight: double-tap space, unless already flying.
    if "1b" not in rc(f"data get entity {PLAYER} abilities.flying"):
        xd("key", "--delay", "120", "space", "space")
        time.sleep(1)


def look_and_walk(name, start, yaw, pitch, seconds, portal=None, keys=("w",), settle=12, fly=True):
    """Teleports the player, takes a 'look' screenshot, holds the keys and logs the position every
    0.25 s; returns the jump (if any) and the end."""
    x, y, z = start
    rc(f"tp {PLAYER} {x:.2f} {y:.2f} {z:.2f} {yaw} {pitch}")
    time.sleep(settle)
    if fly:
        fly_mode()
        rc(f"tp {PLAYER} {x:.2f} {y:.2f} {z:.2f} {yaw} {pitch}")
        time.sleep(3)
    shot(f"{name}-look")
    for k in keys:
        xd("keydown", k)
    t0 = time.time()
    track = []
    try:
        while time.time() - t0 < seconds:
            p, r = player_state()
            if p:
                track.append([round(time.time() - t0, 2), *[round(v, 3) for v in p[:3]], *(round(v, 1) for v in (r or [0, 0])[:2])])
            time.sleep(0.25)
    finally:
        for k in keys:
            xd("keyup", k)
    time.sleep(2)
    shot(f"{name}-after")
    with open(os.path.join(OUT, f"{name}.tsv"), "w") as f:
        f.write("t\tx\ty\tz\tyaw\tpitch\n")
        for row in track:
            f.write("\t".join(str(v) for v in row) + "\n")
    jumps = []
    for a, b in zip(track, track[1:]):
        if abs(b[1] - a[1]) > 50 or abs(b[3] - a[3]) > 50:
            j = {"t": b[0], "from": a[1:4], "to": b[1:4], "yaw_from": a[4], "yaw_to": b[4]}
            if portal:
                gx, gz = glue_point(portal, a[1], a[3])
                ax = (portal["axis_w"][0], portal["axis_w"][2])
                j["seam"] = portal["name"]
                j["glued_from"] = [round(gx, 2), round(gz, 2)]
                # Along the seam the landing must match the glued point exactly; across it, it
                # differs by the distance flown between two samples (about 3 blocks).
                j["error_along_seam_blocks"] = round(abs((b[1] - gx) * ax[0] + (b[3] - gz) * ax[1]), 2)
                j["error_across_seam_blocks"] = round(abs((b[1] - gx) * ax[1] - (b[3] - gz) * ax[0]), 2)
                j["yaw_turn"] = round(((b[4] - a[4]) + 540) % 360 - 180, 1)
            jumps.append(j)
    res = {"name": name, "start": start, "yaw": yaw, "samples": len(track), "jumps": jumps,
           "end": track[-1][1:4] if track else None}
    log(json.dumps(res))
    return res


def glue_cell(p, bx, bz):
    """The glued image of block (cell) (bx, bz): a half-turn about the integer point (cx, cz) maps
    the cell to (2cx - bx - 1, 2cz - bz - 1); a slide moves it."""
    ox, _, oz = p["origin"]
    dx, _, dz = p["destination"]
    if p["kind"] == "half_turn":
        return int(2 * ox - bx - 1), int(2 * oz - bz - 1)
    return int(bx + dx - ox), int(bz + dz - oz)


def preload(points, r=48):
    for (x, z) in points:
        rc(f"forceload add {int(x) - r} {int(z) - r} {int(x) + r} {int(z) + r}")
    for (x, z) in points:
        for dx in (-r, 0, r):
            for dz in (-r, 0, r):
                wait_loaded(x + dx, z + dz, 300)


def unload(points, r=48):
    for (x, z) in points:
        rc(f"forceload remove {int(x) - r} {int(z) - r} {int(x) + r} {int(z) + r}")


def seam_walk(p, s, name):
    """A glass bridge 3 wide across seam p at edge parameter s: 30 blocks inside, and the glued image
    of 40 blocks outside on the far side; the player sprints along it across the seam."""
    ix, iz = (int(v) for v in inward(p))
    ox, _, oz = p["origin"]
    ax = (int(p["axis_w"][0]), int(p["axis_w"][2]))
    ex, ez = int(ox + ax[0] * s), int(oz + ax[1] * s)       # a point of the edge (integer)
    plane = ex if ix else ez                                  # the edge's coordinate
    lat = ez if ix else ex                                    # the path's lateral coordinate
    sign = ix + iz                                            # +1: the inside is the + side
    ins = (plane, plane + 29) if sign > 0 else (plane - 30, plane - 1)
    outs = (plane - 40, plane - 1) if sign > 0 else (plane, plane + 39)

    def cell(n, l):
        return (n, l) if ix else (l, n)

    start_n = plane + 24.5 if sign > 0 else plane - 24.5
    sx, sz = cell(start_n, lat + 0.5)
    g0 = glue_cell(p, *cell(outs[0], lat - 1))
    g1 = glue_cell(p, *cell(outs[1], lat + 1))
    far = ((g0[0] + g1[0]) / 2, (g0[1] + g1[1]) / 2)
    preload([(sx, sz), far])
    hs = [probe(*q).get("surfaceY", 80) for q in ((sx, sz), far)]
    yb = max(hs + [63]) + 4
    a0, a1 = cell(ins[0], lat - 1), cell(ins[1], lat + 1)
    rc(f"fill {a0[0]} {yb} {a0[1]} {a1[0]} {yb} {a1[1]} minecraft:glass", show=True)
    rc(f"fill {g0[0]} {yb} {g0[1]} {g1[0]} {yb} {g1[1]} minecraft:glass", show=True)
    # Air above both bridges, so terrain never blocks the walk.
    rc(f"fill {a0[0]} {yb + 1} {a0[1]} {a1[0]} {yb + 3} {a1[1]} minecraft:air")
    rc(f"fill {g0[0]} {yb + 1} {g0[1]} {g1[0]} {yb + 3} {g1[1]} minecraft:air")
    yaw = {(0, -1): 180, (0, 1): 0, (1, 0): -90, (-1, 0): 90}[(-ix, -iz)]
    res = look_and_walk(name, (sx, yb + 1, sz), yaw, 15, 9, portal=p, keys=("Control_L", "w"), settle=45, fly=False)
    res["bridge_y"] = yb
    unload([(sx, sz), far])
    return res


def land_sites(per_seam=1, step=32):
    """Points along each seam where land crosses it: both the block 2 inside and the glued image of
    the block 2 outside stand above sea level; the highest such point per seam."""
    sites = []
    for p in seams()["portals"]:
        ix, iz = inward(p)
        ox, _, oz = p["origin"]
        ax = (p["axis_w"][0], p["axis_w"][2])
        best = []
        n = int(p["width"] // step)
        for k in range(1, n):
            s = -0.5 * p["width"] + k * step
            ex, ez = round(ox + ax[0] * s), round(oz + ax[1] * s)
            a = (ex + ix * 2, ez + iz * 2)
            g = glue_point(p, ex - ix * 2, ez - iz * 2)
            ha, hg = probe(*a).get("terrainH", -999), probe(*g).get("terrainH", -999)
            if min(ha, hg) > 68:
                best.append((min(ha, hg), s))
        best.sort(reverse=True)
        sites += [{"seam": p["name"], "s": s, "min_height": round(h, 1)} for h, s in best[:per_seam]]
    report("land_sites", sites)
    log("land sites:", sites)
    return sites


def land():
    """Views and ground walks where land crosses a seam (no bridge: the terrain itself), and the
    pole walk again with the hidden outside cleared."""
    by = {p["name"]: p for p in seams()["portals"]}
    sites = land_sites()
    client_start()
    rc(f"op {PLAYER}")
    rc(f"gamemode creative {PLAYER}")
    rc(f"effect give {PLAYER} minecraft:night_vision infinite 0 true")
    rc("time set noon")
    out = []
    for site in sites:
        p = by[site["seam"]]
        ix, iz = (int(v) for v in inward(p))
        ox, _, oz = p["origin"]
        ax = (int(p["axis_w"][0]), int(p["axis_w"][2]))
        ex, ez = int(ox + ax[0] * site["s"]), int(oz + ax[1] * site["s"])
        yaw = {(0, -1): 180, (0, 1): 0, (1, 0): -90, (-1, 0): 90}[(-ix, -iz)]
        g = glue_point(p, ex, ez)
        preload([(ex, ez), g], 64)
        # A view across the seam from 40 blocks inside, 20 above the ground.
        vx, vz = ex + ix * 40 + 0.5 * abs(ax[0]), ez + iz * 40 + 0.5 * abs(ax[1])
        hv = probe(vx, vz).get("surfaceY", 80)
        rc(f"tp {PLAYER} {vx:.1f} {max(hv, 63) + 20} {vz:.1f} {yaw} 20")
        time.sleep(50)
        shot(f"land-view-{site['seam']}")
        # The same view from the far side, looking back across the seam.
        fx, fz = glue_point(p, ex - ix * 40, ez - iz * 40)
        hf = probe(fx, fz).get("surfaceY", 80)
        # On the far side of a half-turn seam the seam lies the same way (yaw unchanged); across a
        # slide it is the opposite edge.
        back = yaw if p["kind"] == "half_turn" else {180: 0, 0: 180, -90: 90, 90: -90}[yaw]
        rc(f"tp {PLAYER} {fx + 0.5 * abs(ax[0]):.1f} {max(hf, 63) + 20} {fz + 0.5 * abs(ax[1]):.1f} {back} 20")
        time.sleep(50)
        shot(f"land-view-{site['seam']}-back")
        # A walk on the ground, 12 blocks inside, straight at the seam.
        sx, sz = ex + ix * 12 + 0.5 * abs(ax[0]) + 0.5 * abs(ix), ez + iz * 12 + 0.5 * abs(ax[1]) + 0.5 * abs(iz)
        hs = probe(sx, sz).get("surfaceY", 80)
        res = look_and_walk(f"land-walk-{site['seam']}", (sx, hs + 1, sz), yaw, 10, 8, portal=p,
                            keys=("Control_L", "w"), settle=30, fly=False)
        res["site"] = site
        out.append(res)
        unload([(ex, ez), g], 64)
        report("land_walks", out)
    # The pole again, with the hidden outside beyond the north seam cleared above the bridge.
    p = by["north"]
    yb = 67
    rc("forceload add -4110 -4140 -4080 -4090")
    wait_loaded(-4096, -4120)
    rc(f"fill -4097 {yb + 1} -4136 -4095 {yb + 3} -4097 minecraft:air", show=True)
    res = seam_walk(p, -B // 2, "walk-north-pole-cleared")
    rc("forceload remove -4110 -4140 -4080 -4090")
    report("pole_walk_cleared", res)


def client():
    t = seams()
    by = {p["name"]: p for p in t["portals"]}
    rc("difficulty peaceful")
    rc("gamerule doMobSpawning false")
    rc("gamerule doInsomnia false")
    rc("time set noon")
    rc("gamerule doDaylightCycle false")
    rc("weather clear")
    client_start()
    rc(f"op {PLAYER}")
    rc(f"gamemode creative {PLAYER}")
    rc(f"effect give {PLAYER} minecraft:night_vision infinite 0 true")
    shot("00-joined")
    walks = []
    # One crossing per seam, away from the singular points; then over the north pole (the north
    # edge's quarter point x = -B/2) and along the equator through the east slide.
    for name, (seam, s) in {"walk-north": ("north", -2000), "walk-south": ("south", 3000),
                            "walk-east": ("east", 1500), "walk-west": ("west", -1200),
                            "walk-north-pole": ("north", -B // 2), "walk-equator-east": ("east", 0)}.items():
        walks.append(seam_walk(by[seam], s, name))
        report("walks", walks)
    # Views across seams from inside, and at the singular point (0, -B/2) on the north seam.
    views = {}
    for name, (x, z, yaw) in {"view-north-seam": (-2000.5, -B / 2 + 50.5, 180), "view-south-seam": (-3000.5, B / 2 - 50.5, 0),
                              "view-east-seam": (B - 50.5, 1500.5, -90), "view-singular-north": (0.5, -B / 2 + 60.5, 180),
                              "view-north-pole": (-B / 2 + 0.5, -B / 2 + 60.5, 180)}.items():
        h = probe(x, z).get("surfaceY", 80)
        preload([(x, z)], 64)
        rc(f"tp {PLAYER} {x} {max(h, 63) + 30} {z} {yaw} 25")
        time.sleep(45)
        views[name] = shot(name)
        unload([(x, z)], 64)
    report("views", views)
    # Chunk loading through a seam: with the player at the north seam, is the glued side loaded?
    p = by["north"]
    rc(f"tp {PLAYER} -2000.5 120 {-B / 2 + 8.5} 180 20")
    time.sleep(30)
    g = glue_point(p, -2000, -B / 2 - 8)
    report("chunk_loading", {"player_at": [-2000.5, -B / 2 + 8.5], "glued_side": g,
                             "glued_side_loaded": "passed" in rc(f"execute if loaded {int(g[0])} 64 {int(g[1])}"),
                             "hidden_outside_loaded": "passed" in rc(f"execute if loaded -2000 64 {-B // 2 - 8}"),
                             "tps": rc("neoforge tps")})


def physics():
    """Fluids and redstone at the north seam, on a platform built across it."""
    p = [q for q in seams()["portals"] if q["name"] == "north"][0]
    x, z = -3000, -B // 2
    for q in ((x, z), glue_point(p, x, z)):
        rc(f"forceload add {int(q[0]) - 16} {int(q[1]) - 16} {int(q[0]) + 16} {int(q[1]) + 16}")
        wait_loaded(*q)
    y = 200
    gx, gz = glue_point(p, x, z)
    gx, gz = int(gx), int(gz)
    rc(f"fill {x - 4} {y} {z - 4} {x + 4} {y} {z + 3} minecraft:stone")          # inside z >= -B/2 ... and outside
    rc(f"fill {gx - 5} {y} {gz - 3} {gx + 4} {y} {gz + 4} minecraft:stone")
    # Water source one block inside the seam (cell z = -B/2 is the first inside row).
    rc(f"setblock {x} {y + 1} {z} minecraft:water")
    # Redstone: a powered line up to the seam; the glued neighbour cell holds a lamp.
    rc(f"setblock {x + 2} {y + 1} {z + 2} minecraft:redstone_block")
    rc(f"setblock {x + 2} {y + 1} {z + 1} minecraft:redstone_wire")
    rc(f"setblock {x + 2} {y + 1} {z} minecraft:redstone_wire")
    # The cell glued to (x+2, z-1) (outside) is the half-turn image of that cell.
    lx, lz = 2 * int(p["origin"][0]) - (x + 2) - 1, 2 * int(p["origin"][2]) - (z - 1) - 1
    rc(f"setblock {lx} {y + 1} {lz} minecraft:redstone_lamp")
    time.sleep(8)
    wcx, wcz = 2 * int(p["origin"][0]) - x - 1, 2 * int(p["origin"][2]) - (z - 1) - 1
    res = {
        "water_source_kept": "passed" in rc(f"execute if block {x} {y + 1} {z} minecraft:water"),
        "water_spread_inside": "passed" in rc(f"execute if block {x} {y + 1} {z + 1} minecraft:water"),
        "wire_at_seam_power_14": "passed" in rc(f"execute if block {x + 2} {y + 1} {z} minecraft:redstone_wire[power=14]"),
        "water_in_hidden_outside": "passed" in rc(f"execute if block {x} {y + 1} {z - 1} minecraft:water"),
        "water_in_glued_cell": "passed" in rc(f"execute if block {wcx} {y + 1} {wcz} minecraft:water"),
        "lamp_lit": "passed" in rc(f"execute if block {lx} {y + 1} {lz} minecraft:redstone_lamp[lit=true]"),
        "lamp_at": [lx, y + 1, lz],
    }
    log(json.dumps(res))
    report("physics", res)


def all_phases():
    setup()
    boot()
    portals()
    terrain()
    entities()
    physics()
    try:
        client()
    finally:
        client_stop()
        report("tps_end", rc("neoforge tps"))
        stop()


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    phases = sys.argv[1:] or ["all"]
    table = {"setup": setup, "boot": boot, "portals": portals, "terrain": terrain, "entities": entities,
             "physics": physics, "client": client, "land": land, "stop": stop, "all": all_phases}
    try:
        for ph in phases:
            log("=== phase", ph)
            table[ph]()
    finally:
        # The server lock is held only while this runs: never leave a server or client behind.
        client_stop()
        stop()
