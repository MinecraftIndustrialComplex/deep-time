#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Deallocates the Azure test VM `deeptime-test` after a stretch with no work on it.

Ben, 2026-10-03 (decisions log): "Auto-stop when idle". The VM costs about $38 a day running around
the clock; a deallocated VM stops the compute charge (its disks keep billing). The laptop's
tools/remote/vm-access.sh starts it again when an agent needs it.

Runs as root under systemd (deeptime-idle.service, installed by tools/remote/vm/install.sh). Every
INTERVAL seconds it decides "busy" or "idle"; after IDLE_MINUTES of continuous idle it deallocates the
VM through Azure Resource Manager with the VM's own managed identity (custom role "Deep Time Test VM
Self Deallocate", assigned at the scope of this one VM: read + deallocate, nothing else). An OS
shutdown alone would leave the VM "stopped (allocated)" and still billed.

Busy means any of:
  - the override file KEEP_AWAKE exists;
  - the Minecraft server lock (mac.sh --server) exists;
  - the workbench reports a run running or queued, or a preview job not finished;
  - the play server has a player online (RCON `list`; if RCON does not answer, an established TCP
    connection to its game port counts instead);
  - a job process from any user: cargo/rustc and friends, Gradle (the client; idle Gradle and Kotlin
    daemons do not count), any Java other than the idle play server, `deeptime` other than the
    workbench, nix builds, Python, Chromium, rsync, apt/dpkg ...;
  - a detached shell script (the batch scripts and detach.sh jobs are `setsid nohup bash ...`);
  - an ssh session running a command (the laptop's `ssh -N` port forwards have no command and do
    not count; neither does an interactive shell sitting at its prompt);
  - the 1-minute load average above LOAD_BUSY (a backstop for anything the rules above miss).

Every decision goes to the journal (stdout) and to LOG; STATE holds the current one and LAST_BUSY the
last busy reasons, for agents asking "why is it still up?" or "when will it stop?".

  idle-watcher.py               the service loop
  idle-watcher.py --once        print the current decision and exit (any user; root sees more cwds)
  idle-watcher.py --services    exit 0 when neither the workbench nor the play server is in use (a run,
                                a preview, a player): install.sh --switch asks this before a restart
  idle-watcher.py --check-azure fetch a token and read the VM's power state through the identity (the
                                dry run of the deallocate: same token, same role assignment), then exit

Configuration (environment, from /etc/default/deeptime-idle): IDLE_MINUTES (30), INTERVAL (60),
CACHE, PLAY_DIR, WORKBENCH_URL, LOAD_BUSY (4.0), DRY_RUN (0: 1 logs the deallocate instead of calling
it).
"""

import json
import os
import re
import socket
import struct
import sys
import time
import urllib.error
import urllib.request

env = os.environ.get
IDLE_MINUTES = float(env("IDLE_MINUTES", "30"))
INTERVAL = float(env("INTERVAL", "60"))
CACHE = env("CACHE", "/data/ban/deeptime-cache")
KEEP_AWAKE = env("KEEP_AWAKE", f"{CACHE}/KEEP_AWAKE")
SERVER_LOCK = env("SERVER_LOCK", f"{CACHE}/server.lock")
PLAY_DIR = env("PLAY_DIR", "/data/ban/mic/mods/play-fa09e8b/smoke/mic/server")
WORKBENCH_URL = env("WORKBENCH_URL", "http://127.0.0.1:8642")
LOAD_BUSY = float(env("LOAD_BUSY", "4.0"))
DRY_RUN = env("DRY_RUN", "0") not in ("", "0", "false", "no")
LOG = env("LOG", f"{CACHE}/idle-watcher.log")
STATE = env("STATE", f"{CACHE}/idle-watcher.state")
LAST_BUSY = env("LAST_BUSY", f"{CACHE}/idle-watcher.last-busy")
# Written just before the deallocate call; vm-access.sh treats a VM carrying it as going down
# (waits for the deallocate to finish, then starts it). Removed when the watcher starts at boot.
GOING_DOWN = env("GOING_DOWN", f"{CACHE}/DEALLOCATING")
LOG_MAX = 5 * 1024 * 1024

IMDS = "http://169.254.169.254/metadata"
ARM = "https://management.azure.com"
ARM_API = "2024-07-01"

# Job processes, by executable basename (argv[0]) or kernel comm name (15 characters).
JOB_NAMES = {
    "cargo", "rustc", "rustdoc", "clippy-driver", "cargo-clippy", "cargo-deny", "cargo-xwin",
    "cargo-nextest", "cc", "gcc", "g++", "clang", "ld", "ld.lld", "lld", "mold", "make", "cmake",
    "gradle", "gradlew", "java", "javac", "nix", "nix-build", "nix-shell", "nix-store",
    "python", "python3", "node", "chromium", "chrome", "headless_shell", "ffmpeg", "magick",
    "convert", "rsync", "scp", "sftp-server", "portablemc", "Xvfb", "deeptime", "deeptime-inspect",
    "deeptime-viewer", "mcrcon", "curl", "wget", "tar", "zstd", "xz", "unzip", "git",
    "apt", "apt-get", "dpkg", "unattended-upgrade",
}
SHELLS = {"bash", "zsh", "sh", "dash", "fish"}
# Long-lived Java processes that are not work by themselves: Gradle keeps its daemon for hours
# after a build (the build's own `gradlew` client process is what shows a build in progress).
IDLE_JAVA = ("org.gradle.launcher.daemon.bootstrap.GradleDaemon", "KotlinCompileDaemon",
             "org.jetbrains.kotlin.daemon")
FINISHED_PREVIEW = {"done", "failed", "unknown", "cancelled"}


def now_iso():
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def log(msg):
    line = f"{now_iso()} {msg}"
    print(msg, flush=True)
    try:
        if os.path.exists(LOG) and os.path.getsize(LOG) > LOG_MAX:
            os.replace(LOG, LOG + ".1")
        with open(LOG, "a") as f:
            f.write(line + "\n")
    except OSError:
        pass


def remove(path):
    try:
        os.remove(path)
    except OSError:
        pass


def write(path, text):
    try:
        tmp = path + ".tmp"
        with open(tmp, "w") as f:
            f.write(text)
        os.chmod(tmp, 0o644)
        os.replace(tmp, path)
    except OSError:
        pass


# ---------------------------------------------------------------------------------- processes

class Proc:
    __slots__ = ("pid", "ppid", "uid", "tty", "comm", "argv", "children")

    def __init__(self, pid):
        self.pid = pid
        with open(f"/proc/{pid}/stat") as f:
            stat = f.read()
        # comm may hold spaces and parentheses: split at the last ')'.
        self.comm = stat[stat.index("(") + 1:stat.rindex(")")]
        rest = stat[stat.rindex(")") + 2:].split()
        self.ppid = int(rest[1])
        self.tty = int(rest[4])
        with open(f"/proc/{pid}/status") as f:
            self.uid = next(int(l.split()[1]) for l in f if l.startswith("Uid:"))
        with open(f"/proc/{pid}/cmdline", "rb") as f:
            self.argv = [a.decode(errors="replace") for a in f.read().split(b"\0") if a]
        self.children = []

    def name(self):
        return os.path.basename(self.argv[0]) if self.argv else self.comm

    def cmd(self, n=110):
        s = " ".join(" ".join(self.argv).split()) if self.argv else f"[{self.comm}]"
        return s if len(s) <= n else s[:n] + "…"

    def cwd(self):
        try:
            return os.readlink(f"/proc/{self.pid}/cwd")
        except OSError:
            return None


def scan():
    procs = {}
    for d in os.listdir("/proc"):
        if d.isdigit():
            try:
                procs[int(d)] = Proc(int(d))
            except (OSError, ValueError, StopIteration, IndexError):
                pass  # gone, or a kernel thread without a status we can read
    for p in procs.values():
        if p.ppid in procs:
            procs[p.ppid].children.append(p)
    return procs


def is_ssh_session(p):
    """The unprivileged per-connection sshd process: `sshd: ban@notty` / `sshd: ban@pts/3`."""
    return p.comm in ("sshd", "sshd-session") and bool(p.argv) and p.argv[0].startswith("sshd") \
        and "@" in p.argv[0]


def descendants(p):
    out, stack = [], list(p.children)
    while stack:
        c = stack.pop()
        out.append(c)
        stack.extend(c.children)
    return out


def under_ssh_session(p, procs):
    seen = 0
    while p.ppid in procs and seen < 64:
        p = procs[p.ppid]
        if is_ssh_session(p):
            return True
        seen += 1
    return False


def is_play_server(p):
    return p.name() == "java" and p.cwd() == PLAY_DIR


def is_workbench(p):
    return p.name() == "deeptime" and len(p.argv) > 1 and p.argv[1] == "watch"


def job_reason(p):
    """Why this process is work, or None."""
    n = p.name()
    if n not in JOB_NAMES and p.comm not in JOB_NAMES:
        return None
    if n == "java" or p.comm == "java":
        joined = " ".join(p.argv)
        if any(k in joined for k in IDLE_JAVA) or is_play_server(p):
            return None
        if "GradleWrapperMain" in joined or "org.gradle" in joined:
            return f"gradle (pid {p.pid})"
        return f"java (pid {p.pid}, {p.cwd() or '?'})"
    if n == "deeptime" and is_workbench(p):
        return None
    if n.startswith("python") or p.comm.startswith("python"):
        # root's own Pythons are system daemons (waagent, networkd-dispatcher, the
        # unattended-upgrades shutdown helper); this watcher is root's too.
        if p.uid == 0 and not any(os.path.basename(a) == "unattended-upgrade" for a in p.argv[1:3]):
            return None
    return f"{n} (pid {p.pid}: {p.cmd(80)})"


def process_reasons(procs):
    reasons, me = [], os.getpid()
    for p in procs.values():
        if p.pid == me or p.pid <= 2:
            continue
        r = job_reason(p)
        if r:
            reasons.append(r)
            continue
        # A detached shell script: setsid nohup bash … (batch.sh, after-previews.sh, detach.sh jobs).
        # Shells inside an ssh session are judged with their session below.
        if (p.name() in SHELLS or p.comm in SHELLS) and p.uid >= 1000 and p.tty == 0 \
                and not under_ssh_session(p, procs):
            reasons.append(f"detached script (pid {p.pid}: {p.cmd(80)})")
    for p in procs.values():
        if not is_ssh_session(p) or p.uid < 1000:
            continue
        kids = p.children
        if not kids:
            continue  # a port forward (ssh -N) or a session between commands
        # An interactive shell waiting at its prompt is not work; anything it runs is.
        if all(c.tty != 0 and (c.name() in SHELLS or c.comm in SHELLS) and not c.children for c in kids):
            continue
        what = next((d for d in descendants(p) if d.name() not in SHELLS), kids[0])
        reasons.append(f"ssh session {p.argv[0].split()[-1]} (pid {p.pid}: {what.cmd(80)})")
    return reasons


# ---------------------------------------------------------------------------- other signals

def http_json(url, timeout=5):
    with urllib.request.urlopen(url, timeout=timeout) as r:
        return json.load(r)


def workbench_reasons():
    out = []
    try:
        runs = http_json(f"{WORKBENCH_URL}/api/runs")
    except (OSError, ValueError):
        return out  # not running: nothing in progress there
    if runs.get("running"):
        out.append(f"workbench run {runs['running']} running")
    if runs.get("queue"):
        out.append(f"workbench queue: {len(runs['queue'])} run(s)")
    try:
        jobs = http_json(f"{WORKBENCH_URL}/api/preview")
        for j in jobs if isinstance(jobs, list) else []:
            if j.get("state") not in FINISHED_PREVIEW:
                out.append(f"workbench preview {j.get('job')} {j.get('state')}")
    except (OSError, ValueError):
        pass
    return out


def server_properties():
    props = {}
    try:
        with open(os.path.join(PLAY_DIR, "server.properties")) as f:
            for line in f:
                if "=" in line and not line.startswith("#"):
                    k, v = line.rstrip("\n").split("=", 1)
                    props[k.strip()] = v.strip()
    except OSError:
        pass
    return props


def rcon(port, password, command, timeout=5):
    """Minecraft RCON (the Source RCON protocol): one command, its reply."""
    with socket.create_connection(("127.0.0.1", port), timeout=timeout) as s:
        def send(rid, kind, body):
            payload = struct.pack("<ii", rid, kind) + body.encode() + b"\0\0"
            s.sendall(struct.pack("<i", len(payload)) + payload)

        def recv_exact(n):
            buf = b""
            while len(buf) < n:
                chunk = s.recv(n - len(buf))
                if not chunk:
                    raise OSError("rcon: connection closed")
                buf += chunk
            return buf

        def recv():
            (length,) = struct.unpack("<i", recv_exact(4))
            data = recv_exact(length)
            rid, _kind = struct.unpack("<ii", data[:8])
            return rid, data[8:-2].decode(errors="replace")

        send(1, 3, password)
        rid, _ = recv()
        if rid == -1:
            raise OSError("rcon: wrong password")
        send(2, 2, command)
        return recv()[1]


def established_to(port):
    n = 0
    for table in ("/proc/net/tcp", "/proc/net/tcp6"):
        try:
            with open(table) as f:
                next(f)
                for line in f:
                    parts = line.split()
                    local, state = parts[1], parts[3]
                    if state == "01" and int(local.rsplit(":", 1)[1], 16) == port:
                        n += 1
        except OSError:
            pass
    return n


def play_reasons(procs):
    if not any(is_play_server(p) for p in procs.values()):
        return []
    props = server_properties()
    try:
        game_port = int(props.get("server-port", "25765"))
    except ValueError:
        game_port = 25765
    if props.get("enable-rcon") == "true" and props.get("rcon.password"):
        try:
            reply = rcon(int(props.get("rcon.port", "25575")), props["rcon.password"], "list")
            m = re.search(r"There are (\d+)", reply)
            if m:
                return [f"play server: {m.group(1)} player(s) online"] if int(m.group(1)) > 0 else []
        except (OSError, ValueError, struct.error):
            pass
    # RCON down (still starting, or broken): a connection to the game port stands in for a player.
    n = established_to(game_port)
    return [f"play server: {n} connection(s) on port {game_port} (rcon did not answer)"] if n else []


def busy_reasons():
    reasons = []
    if os.path.exists(KEEP_AWAKE):
        reasons.append(f"override {KEEP_AWAKE}")
    if os.path.exists(SERVER_LOCK):
        owner = ""
        try:
            with open(os.path.join(SERVER_LOCK, "owner")) as f:
                owner = f" ({f.read().strip()[:60]})"
        except OSError:
            pass
        reasons.append(f"server lock{owner}")
    procs = scan()
    reasons += workbench_reasons()
    reasons += play_reasons(procs)
    reasons += process_reasons(procs)
    try:
        with open("/proc/loadavg") as f:
            load1 = float(f.read().split()[0])
        if load1 > LOAD_BUSY:
            reasons.append(f"load {load1:.1f} > {LOAD_BUSY:g}")
    except (OSError, ValueError):
        pass
    return reasons


# ------------------------------------------------------------------------------------- Azure

def imds(path, query):
    req = urllib.request.Request(f"{IMDS}/{path}?{query}", headers={"Metadata": "true"})
    with urllib.request.urlopen(req, timeout=10) as r:
        return json.load(r)


def vm_id_and_token():
    compute = imds("instance/compute", "api-version=2021-02-01")
    token = imds("identity/oauth2/token",
                 "api-version=2018-02-01&resource=https://management.azure.com/")["access_token"]
    return compute["resourceId"], token


def arm(method, path, token, query=f"api-version={ARM_API}"):
    req = urllib.request.Request(f"{ARM}{path}?{query}", method=method, data=b"" if method == "POST" else None,
                                 headers={"Authorization": f"Bearer {token}"})
    try:
        with urllib.request.urlopen(req, timeout=30) as r:
            body = r.read()
            return r.status, json.loads(body) if body else None
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode(errors="replace")[:400]


def check_azure():
    vm, token = vm_id_and_token()
    print(f"vm: {vm}")
    status, view = arm("GET", f"{vm}/instanceView", token)
    if status == 200:
        print("power state:", [s["code"] for s in view.get("statuses", []) if s["code"].startswith("PowerState/")])
    else:
        print(f"instanceView: HTTP {status}: {view}")
    print("deallocate: not sent (dry run). The identity's one role assignment, \"Deep Time Test VM"
          " Self Deallocate\" on this VM, grants read and deallocate together, so a readable instance"
          " view proves the assignment is in force.")
    return 0 if status == 200 else 1


def deallocate():
    vm, token = vm_id_and_token()
    if DRY_RUN:
        log(f"DRY_RUN: would POST {vm}/deallocate")
        return True
    status, body = arm("POST", f"{vm}/deallocate", token)
    if status in (200, 202):
        log(f"deallocate accepted (HTTP {status}); the VM is going down")
        return True
    log(f"deallocate failed: HTTP {status}: {body}")
    return False


def save_play_world():
    """Flush the play server's world before Azure shuts the guest down (it stops services too, but
    a flushed world costs nothing)."""
    props = server_properties()
    if props.get("enable-rcon") != "true" or not props.get("rcon.password"):
        return
    try:
        rcon(int(props.get("rcon.port", "25575")), props["rcon.password"], "save-all flush", timeout=60)
        log("play server: save-all flush")
    except (OSError, ValueError, struct.error):
        pass


# -------------------------------------------------------------------------------------- loop

def main():
    if "--once" in sys.argv:
        r = busy_reasons()
        print("busy:\n  " + "\n  ".join(r) if r else "idle")
        return 0
    if "--services" in sys.argv:
        r = workbench_reasons() + play_reasons(scan())
        print("in use:\n  " + "\n  ".join(r) if r else "workbench and play server: nothing in progress")
        return 1 if r else 0
    if "--check-azure" in sys.argv:
        return check_azure()
    remove(GOING_DOWN)
    log(f"started: deallocate after {IDLE_MINUTES:g} idle minutes, a check every {INTERVAL:g} s"
        f"{', DRY_RUN' if DRY_RUN else ''}")
    idle_since = None
    retry_at = 0.0
    while True:
        reasons = busy_reasons()
        t = time.monotonic()
        if reasons:
            if idle_since is not None:
                log(f"busy again after {(t - idle_since) / 60:.1f} idle min")
            idle_since = None
            text = "; ".join(reasons)
            log(f"busy: {text}")
            write(LAST_BUSY, f"{now_iso()} {text}\n")
            write(STATE, f"{now_iso()} busy: {text}\n")
        else:
            if idle_since is None:
                idle_since = t
            idle_min = (t - idle_since) / 60
            log(f"idle {idle_min:.1f}/{IDLE_MINUTES:g} min")
            write(STATE, f"{now_iso()} idle {idle_min:.1f} of {IDLE_MINUTES:g} min\n")
            if idle_min >= IDLE_MINUTES and t >= retry_at:
                write(GOING_DOWN, f"{now_iso()} idle {idle_min:.1f} min\n")
                again = busy_reasons()  # a job may have started since this tick's scan
                if again:
                    remove(GOING_DOWN)
                    log(f"not deallocating, busy at the last check: {'; '.join(again)}")
                    idle_since = None
                else:
                    save_play_world()
                    try:
                        ok = deallocate()
                    except (OSError, ValueError, KeyError) as e:
                        ok = False
                        log(f"deallocate failed: {e}")
                    write(STATE, f"{now_iso()} {'deallocating' if ok and not DRY_RUN else 'deallocate failed or dry run'}"
                                 f" after {idle_min:.1f} idle min\n")
                    retry_at = t + 300  # a refused call is retried, an accepted one not repeated
                    if not ok or DRY_RUN:
                        remove(GOING_DOWN)
                        if DRY_RUN:
                            idle_since = None
        time.sleep(INTERVAL)


if __name__ == "__main__":
    sys.exit(main())
