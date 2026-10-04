#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Private server instances, so several Minecraft servers run on the test VM at once.

Ben, 2026-10-03 (decisions log): no fixed slots; jobs start by load (tools/remote/vm/admit.py). For
servers that means each running server needs its own folder and its own ports. A server set
(smoke/<profile>/server, made by tools/smoke/install.sh) is a template here, never a place a server
runs in:

    make            copies the set into a private folder (the job's DEEPTIME_SERVER_DIR from admit.py):
                    `libraries/` is linked (install.sh only ever adds a NeoForge version to it), `mods/`
                    and everything else are copied (servers and install.sh both rewrite files in place,
                    so neither a hard link nor the set itself is safe), worlds and logs are left out;
                    then the job's ports are written in (game, RCON, query, Simple Voice Chat).
                    --world-link NAME links the set's world folder in instead of leaving it out (the
                    smoke runs and the client check keep their world in the set; they hold the set's
                    world lock while they run).
    snapshot-mods   copies a set's mods/ (for the Gradle dev runs, whose run folder links jars from it)
    ports           writes the job's ports into a server folder (server.properties if present, and
                    config/voicechat/voicechat-server.properties)
    wait-jvm        waits until a JVM runs in a folder (a Gradle run task started its game) and prints
                    its pid: the Gradle launchers hold neoforge/build/.run-launch.lock until then

Locks, in smoke/<profile>/ (flock; the files are never removed):
    .set.lock    install.sh holds it exclusively while it changes the set; copies hold it shared.
    .world.lock  whatever runs a server on the set's own world holds it exclusively (run.py,
                 dh-client-check.sh); install.sh takes it too, since it can rewrite the world.

Ports come from the environment admit.py sets for a server job (DEEPTIME_PORT_GAME, _RCON, _QUERY,
_VOICE); without them (outside admit.py) the set's own server.properties values stay. Standard
library only.
"""

import argparse
import contextlib
import fcntl
import os
import re
import shutil
import sys
import time

# Never copied into an instance: worlds, logs and reports a server or a harness writes, and the jars
# install.sh set aside.
SKIP = {"logs", "crash-reports", "mods-excluded", "console.log", "debug", ".fabric", "session.lock"}


def ports_from_env(env=None):
    env = os.environ if env is None else env
    if not env.get("DEEPTIME_PORT_GAME"):
        return None
    return {k: int(env[f"DEEPTIME_PORT_{k.upper()}"]) for k in ("game", "rcon", "query", "voice")}


@contextlib.contextmanager
def flocked(path, exclusive, what, say=None):
    """flock on path; says once (on stderr, or through say) when it has to wait."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    fd = os.open(path, os.O_RDWR | os.O_CREAT, 0o664)
    try:
        mode = fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH
        try:
            fcntl.flock(fd, mode | fcntl.LOCK_NB)
        except BlockingIOError:
            msg = f"waiting for {what} ({path})"
            (say or (lambda m: print(f"instance: {m}", file=sys.stderr, flush=True)))(msg)
            fcntl.flock(fd, mode)
        yield
    finally:
        fcntl.flock(fd, fcntl.LOCK_UN)
        os.close(fd)


def set_lock(set_server, exclusive=False, say=None):
    return flocked(os.path.join(os.path.dirname(os.path.abspath(set_server)), ".set.lock"), exclusive,
                   "the server set (an install is changing it)" if not exclusive else "copies of the server set", say)


def world_lock(set_server, say=None):
    return flocked(os.path.join(os.path.dirname(os.path.abspath(set_server)), ".world.lock"), True,
                   "the set's world (another run of this set uses it)", say)


def level_name(server_dir):
    try:
        m = re.search(r"(?m)^level-name=(.*)$", open(os.path.join(server_dir, "server.properties")).read())
        return m.group(1).strip() if m else "world"
    except OSError:
        return "world"


def copy_mods(src, dst):
    os.makedirs(dst, exist_ok=True)
    for name in os.listdir(src):
        s = os.path.join(src, name)
        if name.startswith(".") and name.endswith(".part"):
            continue   # jarstate.refresh's file in flight
        if os.path.isdir(s) and not os.path.islink(s):
            shutil.copytree(s, os.path.join(dst, name), symlinks=True)
        else:
            shutil.copy2(s, os.path.join(dst, name))


def make(set_server, dest, ports=None, world_link=None, say=None):
    """A private copy of the server set at dest (made empty first); returns dest."""
    set_server = os.path.abspath(set_server)
    if not os.path.isdir(os.path.join(set_server, "mods")):
        raise SystemExit(f"instance: {set_server} is not an installed server set (no mods/)")
    worlds = {level_name(set_server), "world", "planet"}
    if os.path.lexists(dest):
        shutil.rmtree(dest)
    os.makedirs(dest)
    t = time.time()
    with set_lock(set_server, exclusive=False, say=say):
        for name in sorted(os.listdir(set_server)):
            s, d = os.path.join(set_server, name), os.path.join(dest, name)
            if name in SKIP or name.endswith((".workbench-backup", ".smoke-backup")):
                continue
            if name in worlds:
                if name == world_link:
                    os.symlink(s, d)
                continue
            if name == "libraries":
                os.symlink(s, d)
            elif name == "mods":
                copy_mods(s, d)
            elif os.path.isdir(s) and not os.path.islink(s):
                shutil.copytree(s, d, symlinks=True)
            else:
                shutil.copy2(s, d, follow_symlinks=False)
    if world_link and not os.path.lexists(os.path.join(dest, world_link)):
        os.makedirs(os.path.join(set_server, world_link), exist_ok=True)
        os.symlink(os.path.join(set_server, world_link), os.path.join(dest, world_link))
    write_ports(dest, ports if ports is not None else ports_from_env())
    if say:
        say(f"server instance {dest} from {set_server} in {time.time() - t:.1f} s")
    return dest


def snapshot_mods(set_server, dest, say=None):
    if os.path.lexists(dest):
        shutil.rmtree(dest)
    with set_lock(set_server, exclusive=False, say=say):
        copy_mods(os.path.join(set_server, "mods"), dest)
    return dest


def set_prop(text, key, value):
    line = f"{key}={value}"
    if re.search(rf"(?m)^{re.escape(key)}=", text):
        return re.sub(rf"(?m)^{re.escape(key)}=.*$", line, text)
    return text + ("" if text.endswith("\n") or not text else "\n") + line + "\n"


def write_ports(server_dir, ports):
    """The job's ports into server.properties (when the folder has one) and Simple Voice Chat's
    server config (always: a fresh folder would otherwise bind its default 24454)."""
    if not ports:
        return
    props = os.path.join(server_dir, "server.properties")
    if os.path.exists(props):
        text = open(props).read()
        for k, v in (("server-port", ports["game"]), ("rcon.port", ports["rcon"]), ("query.port", ports["query"]),
                     ("enable-query", "false")):
            text = set_prop(text, k, v)
        with open(props, "w") as f:
            f.write(text)
    vc = os.path.join(server_dir, "config", "voicechat", "voicechat-server.properties")
    os.makedirs(os.path.dirname(vc), exist_ok=True)
    text = open(vc).read() if os.path.exists(vc) else ""
    with open(vc, "w") as f:
        f.write(set_prop(text, "port", ports["voice"]))


def find_jvm(run_dir):
    """The pid of a java process whose working directory is run_dir, or None."""
    want = os.path.realpath(run_dir)
    for d in os.listdir("/proc"):
        if not d.isdigit():
            continue
        try:
            if open(f"/proc/{d}/comm").read().strip() != "java":
                continue
            if os.readlink(f"/proc/{d}/cwd") == want:
                return int(d)
        except OSError:
            continue
    return None


def wait_jvm(run_dir, pid=None, timeout=900.0):
    """Waits until a JVM runs in run_dir (a Gradle run task started the game), the launcher pid ends,
    or timeout; returns the JVM's pid or None. Gradle writes a run's argument files into the shared
    build folder just before it starts the JVM, so a launcher holds the checkout's launch lock until
    this returns: two runs of one checkout must not interleave there."""
    end = time.time() + timeout
    while time.time() < end:
        j = find_jvm(run_dir)
        if j:
            return j
        if pid is not None:
            try:
                os.kill(pid, 0)
            except OSError:
                return None
        time.sleep(1.0)
    return None


@contextlib.contextmanager
def launch_lock(repo, say=None):
    """The checkout's Gradle run-launch lock (neoforge/build/.run-launch.lock)."""
    with flocked(os.path.join(repo, "neoforge", "build", ".run-launch.lock"), True,
                 "this checkout's Gradle run launch (another run of it is starting)", say):
        yield


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    m = sub.add_parser("make")
    m.add_argument("--set", required=True, help="the server set, e.g. smoke/mic/server")
    m.add_argument("--dest", required=True)
    m.add_argument("--world-link", help="link the set's world folder of this name into the instance")
    s = sub.add_parser("snapshot-mods")
    s.add_argument("--set", required=True)
    s.add_argument("--dest", required=True)
    p = sub.add_parser("ports")
    p.add_argument("--dir", required=True)
    w = sub.add_parser("wait-jvm")
    w.add_argument("--dir", required=True)
    w.add_argument("--pid", type=int)
    w.add_argument("--timeout", type=float, default=900.0)
    a = ap.parse_args()
    say = lambda msg: print(f"instance: {msg}", file=sys.stderr, flush=True)  # noqa: E731
    if a.cmd == "make":
        print(make(a.set, a.dest, world_link=a.world_link, say=say))
    elif a.cmd == "snapshot-mods":
        print(snapshot_mods(a.set, a.dest, say=say))
    elif a.cmd == "ports":
        write_ports(a.dir, ports_from_env())
    elif a.cmd == "wait-jvm":
        j = wait_jvm(a.dir, a.pid, a.timeout)
        print(j or "")
    return 0


if __name__ == "__main__":
    sys.exit(main())
