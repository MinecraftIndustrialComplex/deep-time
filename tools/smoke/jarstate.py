#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Which Deep Time jar a server set carries, what it was built from, and whether it is stale.

A server set (smoke/<profile>/server/) holds a copy of the mod jar, taken when
tools/smoke/install.sh ran. Sources move on and the copy does not, so a preview or a smoke run
can boot code from before its checkout's last change (2026-09-29: every workbench preview died
on `$.deposits_synthetic: unknown key` because smoke/mic still carried a jar from before the
synthetic deposit preset existed). The file name never changes (deeptime-1.21.1-0.0.1.jar), so
the name says nothing.

The authority on "what would this checkout build" is Gradle: its `jar` task already tracks the
Java sources, the resources, the embedded presets and, through hostNatives, the Rust natives the
jar packs, and it is a one-second no-op when nothing changed. So staleness is decided by content:

    build the jar (Gradle) -> compare its sha256 with the installed jar's -> replace if different

and this module holds the parts that need no Gradle: the comparison and the replace
(`refresh`), the built jar's location (`built_jar`), and the record kept in install.json
(`jar_fields`, `record`): the jar's sha256, the source revision it was built from and a hash of
the source trees it is built from. `status` compares that record with the tree as it is now
without building anything, for tools that can only warn (tools/smoke/run.py).

Used by tools/smoke/install.sh (`record`) and tools/workbench/preview.py (`refresh` and friends).
Python 3.9 and the standard library only: the Mac's /usr/bin/python3 runs preview.py.

    jarstate.py info    [--repo DIR]                       source revision and hash, as JSON
    jarstate.py record  INSTALL_JSON JAR [--repo DIR]      merge the jar's fields into install.json
    jarstate.py status  INSTALL_JSON [--repo DIR]          is the recorded build still this tree? (exit 1: no)
"""

import argparse
import datetime
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# The trees and files the jar is built from: the mod, the Rust crates the natives are built from
# (jni -> core -> format), the two assets Gradle packs from the viewer crate, and the workspace
# manifests. The hash is a label for reports; Gradle, not this list, decides what to rebuild.
SOURCE_TREES = ("neoforge", "crates/deeptime-jni", "crates/deeptime-core", "crates/deeptime-format",
                "crates/deeptime-viewer/assets")
SOURCE_FILES = ("Cargo.toml", "Cargo.lock")
SKIP_DIRS = frozenset(("build", ".gradle", "bin", "target", "__pycache__", ".git", ".idea", ".eclipse",
                       ".settings"))
# tools/remote/mac.sh writes this beside the tree it syncs: the Mac's copy has no .git.
SYNC_REV = ".sync-rev"


def now():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds")


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


# ---------------------------------------------------------------------------------------------
# What the jar is built from.
# ---------------------------------------------------------------------------------------------

def source_files(repo):
    """Repo-relative paths (forward slashes, sorted) of the files the jar is built from."""
    out = [f for f in SOURCE_FILES if os.path.isfile(os.path.join(repo, f))]
    for tree in SOURCE_TREES:
        for dirpath, dirnames, filenames in os.walk(os.path.join(repo, tree)):
            dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
            for name in filenames:
                full = os.path.join(dirpath, name)
                if os.path.isfile(full):
                    out.append(os.path.relpath(full, repo).replace(os.sep, "/"))
    return sorted(out)


def source_hash(repo):
    """A short hash of the contents (never the mtimes) of source_files: the same bytes give the
    same hash on the laptop and on the Mac."""
    h = hashlib.sha256()
    for rel in source_files(repo):
        h.update(rel.encode() + b"\0" + sha256_file(os.path.join(repo, rel)).encode() + b"\n")
    return h.hexdigest()[:16]


def _git(repo, *args):
    # Never on macOS: /usr/bin/git there is Apple's stub, which pops an install dialog on the
    # shared screen. The Mac gets its revision from .sync-rev.
    if sys.platform == "darwin" or not os.path.exists(os.path.join(repo, ".git")):
        return None
    try:
        r = subprocess.run(["git", "-C", repo] + list(args), capture_output=True, text=True, timeout=20)
    except (OSError, subprocess.SubprocessError):
        return None
    return r.stdout.strip() if r.returncode == 0 and r.stdout.strip() else None


def source_rev(repo):
    """{"describe", "commit"} of the checkout: from .sync-rev when mac.sh synced it (the laptop's
    `git describe --always --dirty` and HEAD at that moment), else from git; "unknown" if neither."""
    try:
        with open(os.path.join(repo, SYNC_REV)) as f:
            d = json.load(f)
        return {"describe": str(d.get("describe") or "unknown"), "commit": str(d.get("commit") or "unknown")}
    except (OSError, ValueError):
        pass
    return {"describe": _git(repo, "describe", "--always", "--dirty", "--abbrev=8") or "unknown",
            "commit": _git(repo, "rev-parse", "HEAD") or "unknown"}


def mod_version(repo):
    with open(os.path.join(repo, "neoforge", "gradle.properties")) as f:
        m = re.search(r"(?m)^mod_version\s*=\s*(\S+)", f.read())
    if not m:
        raise RuntimeError("no mod_version in neoforge/gradle.properties")
    return m.group(1)


def built_jar(repo):
    """The jar Gradle's `jar` task writes: neoforge/build/libs/deeptime-1.21.1-<mod_version>.jar.
    (Named by version, never `the last one sorted`: an older version's jar can linger in build/libs.)"""
    libs = os.path.join(repo, "neoforge", "build", "libs")
    path = os.path.join(libs, f"deeptime-1.21.1-{mod_version(repo)}.jar")
    if not os.path.isfile(path):
        raise RuntimeError(f"Gradle's jar task left no {path}")
    return path


# ---------------------------------------------------------------------------------------------
# What the server set carries.
# ---------------------------------------------------------------------------------------------

def installed_jars(mods_dir):
    """Every Deep Time jar in a mods folder (the same match install.sh deletes by)."""
    try:
        names = os.listdir(mods_dir)
    except OSError:
        return []
    return sorted(os.path.join(mods_dir, n) for n in names
                  if n.lower().startswith("deeptime-") and n.lower().endswith(".jar")
                  and not n.lower().endswith("-sources.jar"))


def refresh(built, mods_dir):
    """Makes `mods_dir` carry `built` as its one Deep Time jar. Unchanged if the installed jar
    has the same name and the same bytes; otherwise the new jar is written beside it as a file the
    mod loader ignores and renamed into place (atomic: a server that already opened the old jar
    keeps its inode), and any other Deep Time jar is removed.

    Returns {"changed": bool, "name", "sha256" (of `built`), "was": [{"name", "sha256"}, …]}."""
    name = os.path.basename(built)
    want = sha256_file(built)
    have = installed_jars(mods_dir)
    was = [{"name": os.path.basename(p), "sha256": sha256_file(p)} for p in have]
    result = {"changed": False, "name": name, "sha256": want, "was": was}
    if len(was) == 1 and was[0]["name"] == name and was[0]["sha256"] == want:
        return result
    os.makedirs(mods_dir, exist_ok=True)
    part = os.path.join(mods_dir, "." + name + ".part")
    shutil.copyfile(built, part)
    os.replace(part, os.path.join(mods_dir, name))
    for p in have:
        if os.path.basename(p) != name:
            os.remove(p)
    result["changed"] = True
    return result


# ---------------------------------------------------------------------------------------------
# The record in install.json.
# ---------------------------------------------------------------------------------------------

def jar_fields(repo, jar):
    """What install.json keeps about the jar a set carries."""
    rev = source_rev(repo)
    return {"deeptime_jar": os.path.basename(jar),
            "deeptime_jar_sha256": sha256_file(jar),
            "deeptime_src_rev": rev["describe"],
            "deeptime_src_commit": rev["commit"],
            "deeptime_src_hash": source_hash(repo),
            "deeptime_jar_recorded": now()}


def record(install_json, fields):
    """Merges `fields` into install.json (atomic; other keys stay)."""
    with open(install_json) as f:
        data = json.load(f)
    data.update(fields)
    tmp = install_json + ".tmp"
    with open(tmp, "w") as f:
        json.dump(data, f, indent=1)
        f.write("\n")
    os.replace(tmp, install_json)
    return data


def status(repo, install):
    """Compares install.json's record with the tree as it is now, without building: returns
    (ok, reason). Not ok: no record (installed before the record existed), or the source trees
    changed since the jar was installed."""
    if not install.get("deeptime_src_hash"):
        return False, "install.json records no source hash (installed before jarstate.py): the jar may predate the sources"
    now_hash = source_hash(repo)
    if now_hash != install["deeptime_src_hash"]:
        return False, (f"the sources changed since this jar was installed (built from {install.get('deeptime_src_rev', '?')} "
                       f"/ source {install['deeptime_src_hash']}, the tree is now {source_rev(repo)['describe']} / source {now_hash})")
    return True, f"the jar was built from these sources ({install.get('deeptime_src_rev', '?')}, source {now_hash})"


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    i = sub.add_parser("info")
    r = sub.add_parser("record")
    r.add_argument("install_json")
    r.add_argument("jar")
    s = sub.add_parser("status")
    s.add_argument("install_json")
    for p in (i, r, s):
        p.add_argument("--repo", default=REPO)
    a = ap.parse_args()
    repo = os.path.abspath(a.repo)
    if a.cmd == "info":
        print(json.dumps(dict(source_rev(repo), source_hash=source_hash(repo)), indent=1))
    elif a.cmd == "record":
        fields = jar_fields(repo, a.jar)
        record(a.install_json, fields)
        print(f"   install.json: Deep Time jar sha256 {fields['deeptime_jar_sha256'][:12]}, "
              f"source {fields['deeptime_src_rev']} / {fields['deeptime_src_hash']}")
    else:
        with open(a.install_json) as f:
            ok, why = status(repo, json.load(f))
        print(("current: " if ok else "STALE: ") + why)
        sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
