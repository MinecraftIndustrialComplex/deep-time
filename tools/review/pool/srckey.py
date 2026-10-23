#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The source key of a Deep Time checkout: a short hash of everything a simulation run reads.

    srckey.py key [--ref REF] [--root DIR] [--list]      laptop: the key of REF (default HEAD) in the
                                                          checkout at DIR (default: this one); refuses
                                                          when a file the key covers is modified, staged,
                                                          deleted or untracked in the working tree
    srckey.py verify KEY [--root DIR]                    VM: the key of the synced copy's git index
                                                          (tools/remote/mac.sh runs `git add -A` there)
                                                          must be KEY

The key covers the files `sources.list` includes (the CLI's crates, the lockfile, the toolchain, the four
compiled-in data folders), read as git blob hashes: the laptop takes them from `git ls-tree -r REF`, the
VM from `git ls-files -s`, and the same file has the same blob hash on both. A commit that touches none of
those files (Java, the pack, docs, tools, the viewer) keeps the key. The checks that make it fail closed:
a crate or data folder `sources.list` does not name, and an include_str!/include_bytes! in an included
Rust file whose literal path lands on a tracked file the list does not cover.

Exit status: 0 key printed / verified, 2 the checkout is dirty or the key does not match, 3 a closure check
failed. Standard library only.
"""
import argparse
import fnmatch
import hashlib
import os
import posixpath
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
SOURCES = HERE / "sources.list"
VM_GIT = "/data/ban/deeptime-cache/git/bin/git"
INCLUDE_RE = re.compile(r'include_(?:str|bytes)!\s*\(\s*"([^"\\]+)"\s*,?\s*\)')


class KeyError_(Exception):
    def __init__(self, msg, code):
        super().__init__(msg)
        self.code = code


def git_bin():
    return os.environ.get("POOL_GIT") or shutil.which("git") or (VM_GIT if os.path.exists(VM_GIT) else "git")


def git(root, *args, check=True, text=True):
    r = subprocess.run([git_bin(), "-C", str(root), *args], capture_output=True, text=text)
    if check and r.returncode != 0:
        err = r.stderr if text else r.stderr.decode("utf-8", "replace")
        raise KeyError_(f"git {' '.join(args)} failed in {root}: {err.strip()}", 3)
    return r.stdout


class Rules:
    def __init__(self, path=SOURCES):
        self.include, self.ignore, self.exclude, self.members = [], [], [], []
        digest = []
        for raw in Path(path).read_text().splitlines():
            line = raw.split("#", 1)[0].strip()
            if not line:
                continue
            verb, _, arg = line.partition(" ")
            arg = arg.strip().rstrip("/")
            if verb not in ("include", "ignore", "exclude", "members") or not arg:
                raise KeyError_(f"{path}: cannot read the line {raw!r}", 3)
            getattr(self, verb).append(arg)
            digest.append(f"{verb} {arg}")
        self.digest = hashlib.sha256("\n".join(digest).encode()).hexdigest()

    @staticmethod
    def under(path, prefix):
        return path == prefix or path.startswith(prefix + "/")

    def covers(self, path):
        """True when the key covers this tracked path."""
        if not any(self.under(path, p) for p in self.include):
            return False
        return not any(fnmatch.fnmatch(path, g) or fnmatch.fnmatch(posixpath.basename(path), g) for g in self.exclude)

    def classified(self, path):
        return any(self.under(path, p) or self.under(p, path) for p in self.include + self.ignore)


def tracked(root, ref):
    """[(blob sha, path)] of the tracked files: `ls-tree -r ref` for a commit, `ls-files -s` (the index)
    when ref is None."""
    out = []
    if ref is None:
        for rec in git(root, "ls-files", "-s", "-z").split("\0"):
            if not rec:
                continue
            meta, _, path = rec.partition("\t")
            mode, sha, _stage = meta.split()
            if mode != "160000":
                out.append((sha, path))
    else:
        for rec in git(root, "ls-tree", "-r", "-z", "--full-tree", ref).split("\0"):
            if not rec:
                continue
            meta, _, path = rec.partition("\t")
            mode, typ, sha = meta.split()
            if typ == "blob":
                out.append((sha, path))
    return out


def dirty_paths(root, rules):
    """Paths the key covers that differ from HEAD in the working tree (changed, staged, deleted or new)."""
    paths = set(git(root, "diff", "--name-only", "--no-renames", "-z", "HEAD").split("\0"))
    paths |= set(git(root, "ls-files", "--others", "--exclude-standard", "-z").split("\0"))
    paths.discard("")
    return sorted(p for p in paths if rules.covers(p))


def closure_errors(root, files, rules):
    """The reasons the key cannot be trusted: unclassified crates and data folders, and compiled-in
    paths outside the covered set."""
    errors = []
    paths = [p for _, p in files]
    for parent in rules.members:
        seen = set()
        for p in paths:
            if p.startswith(parent + "/"):
                seen.add(p[len(parent) + 1:].split("/", 1)[0])
        for m in sorted(seen):
            full = f"{parent}/{m}"
            if not rules.classified(full):
                errors.append(f"{full} is neither included nor ignored in {SOURCES.name}: say which it is")
    covered = {p for p in paths if rules.covers(p)}
    all_paths = set(paths)
    rs = [(sha, p) for sha, p in files if p in covered and p.endswith(".rs")]
    if rs:
        proc = subprocess.run([git_bin(), "-C", str(root), "cat-file", "--batch"], input="".join(f"{sha}\n" for sha, _ in rs).encode(),
                              capture_output=True)
        buf, pos = proc.stdout, 0
        for _, p in rs:
            nl = buf.index(b"\n", pos)
            head = buf[pos:nl].split()
            if len(head) < 3:  # "<sha> missing": the object is not in this repository
                pos = nl + 1
                continue
            size = int(head[2])
            body = buf[nl + 1:nl + 1 + size].decode("utf-8", "replace")
            pos = nl + 1 + size + 1
            for rel in INCLUDE_RE.findall(body):
                target = posixpath.normpath(posixpath.join(posixpath.dirname(p), rel))
                if target.startswith("..") or target in covered:
                    continue
                if target in all_paths:
                    errors.append(f"{p} compiles in {target}, which {SOURCES.name} does not cover")
    return errors


def compute(root, ref, rules=None, listing=False):
    rules = rules or Rules()
    files = tracked(root, ref)
    errors = closure_errors(root, files, rules)
    if errors:
        raise KeyError_("the source key cannot be trusted:\n  " + "\n  ".join(errors), 3)
    covered = sorted((p, sha) for sha, p in files if rules.covers(p))
    if not covered:
        raise KeyError_(f"no tracked file under {root} is covered by {SOURCES.name}: not a Deep Time checkout?", 3)
    h = hashlib.sha256()
    h.update(rules.digest.encode() + b"\n")
    for p, sha in covered:
        h.update(f"{sha} {p}\n".encode())
    key = h.hexdigest()[:12]
    return (key, covered) if listing else key


def key_of_checkout(root, ref="HEAD", allow_dirty=False):
    """The key of `ref` in the checkout at root, after checking the working tree matches HEAD in the
    covered files (the VM copy is the working tree, so a dirty tree would run code the key does not name)."""
    rules = Rules()
    if not allow_dirty and ref == "HEAD":
        dirty = dirty_paths(root, rules)
        if dirty:
            shown = "\n  ".join(dirty[:15]) + (f"\n  ... and {len(dirty) - 15} more" if len(dirty) > 15 else "")
            raise KeyError_(f"{len(dirty)} file(s) the source key covers are modified or untracked; commit them first:\n  {shown}", 2)
    return compute(root, ref, rules)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    k = sub.add_parser("key")
    k.add_argument("--ref", default="HEAD")
    k.add_argument("--root", default=".")
    k.add_argument("--list", action="store_true", help="also print the covered files")
    k.add_argument("--allow-dirty", action="store_true")
    v = sub.add_parser("verify")
    v.add_argument("key")
    v.add_argument("--root", default=".")
    a = ap.parse_args()
    try:
        if a.cmd == "key":
            key = key_of_checkout(a.root, a.ref, a.allow_dirty)
            if a.list:
                _, files = compute(a.root, a.ref, listing=True)
                for p, sha in files:
                    print(f"{sha} {p}", file=sys.stderr)
            print(key)
        else:
            key = compute(a.root, None)
            if key != a.key:
                raise KeyError_(f"the synced copy at {a.root} has source key {key}, not {a.key}: the sync is stale or the laptop's tree "
                                f"differs from its commit (sync again, commit, and rerun)", 2)
            print(key)
    except KeyError_ as e:
        print(f"srckey: {e}", file=sys.stderr)
        sys.exit(e.code)


if __name__ == "__main__":
    main()
