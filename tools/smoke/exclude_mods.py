#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Leaves mods out of a smoke server set by mod id, plus everything that requires them.

    exclude_mods.py <server dir> <mod-id list file> <excluded.txt> <excluded-why.txt>

Reads every jar in <server>/mods, and the ones already in <server>/mods-excluded (left there by the
glob rules of profiles/<profile>.excluded), for their mod ids and mandatory dependencies from
META-INF/neoforge.mods.toml (and META-INF/mods.toml). Moves to <server>/mods-excluded every jar that
provides a listed mod id, then every jar with a required dependency on a mod id of an excluded jar,
until nothing more is caught (a closure: an add-on of an add-on goes too).

A dependency counts when its type is "required" (the default in neoforge.mods.toml) or, in the older
form, mandatory = true, and it does not apply to the client only. Optional, discouraged and
incompatible dependencies do not pull a jar out.

Appends each moved jar's name to <excluded.txt> (the file install.sh and run.py already keep) and
"<jar>\\t<reason>" to <excluded-why.txt>, which the smoke report prints. Prints one line per jar.

Why (Ben, 2026-10-02, decisions log): Cobblemon's event list is not thread-safe and is filled during
NeoForge's parallel mod construction; it crashed MIC test servers at start-up three times.
"""
import os
import shutil
import sys
import tomllib
import zipfile


def read_mods(jar):
    """(mod ids, {mod id: [required dependency ids]}) of one jar; empty with a note when unreadable."""
    ids, deps = [], {}
    try:
        with zipfile.ZipFile(jar) as z:
            for name in ("META-INF/neoforge.mods.toml", "META-INF/mods.toml"):
                if name not in z.namelist():
                    continue
                try:
                    data = tomllib.loads(z.read(name).decode("utf-8", "replace"))
                except tomllib.TOMLDecodeError as e:
                    print(f"   exclude_mods: cannot parse {name} of {os.path.basename(jar)}: {e}", file=sys.stderr)
                    continue
                for m in data.get("mods", []):
                    if "modId" in m:
                        ids.append(m["modId"])
                for owner, lst in (data.get("dependencies") or {}).items():
                    if isinstance(lst, dict):
                        lst = [lst]
                    for d in lst:
                        if str(d.get("side", "BOTH")).upper() == "CLIENT":
                            continue
                        if "type" in d:
                            required = str(d["type"]).lower() == "required"
                        else:
                            required = bool(d.get("mandatory", True))
                        if required and "modId" in d:
                            deps.setdefault(owner, []).append(d["modId"])
    except zipfile.BadZipFile:
        print(f"   exclude_mods: not a zip: {jar}", file=sys.stderr)
    return ids, deps


def main():
    server, listfile, excluded_txt, why_txt = sys.argv[1:5]
    seeds = []
    for line in open(listfile):
        line = line.split("#", 1)[0].strip()
        if line:
            seeds.append(line)
    mods_dir, out_dir = os.path.join(server, "mods"), os.path.join(server, "mods-excluded")
    os.makedirs(out_dir, exist_ok=True)

    info = {}  # jar path -> (mod ids, required dependency ids, already excluded?)
    for d, already in ((mods_dir, False), (out_dir, True)):
        for f in sorted(os.listdir(d)):
            if f.endswith(".jar"):
                ids, deps = read_mods(os.path.join(d, f))
                info[os.path.join(d, f)] = (ids, {x for v in deps.values() for x in v}, already)

    # Mod ids that are out: the listed ones, and every id a jar left out already provides.
    out_ids = set(seeds)
    for ids, _, already in info.values():
        if already:
            out_ids.update(ids)
    reasons = {}
    changed = True
    while changed:
        changed = False
        for path, (ids, needs, already) in sorted(info.items()):
            if already or path in reasons:
                continue
            hit = [i for i in ids if i in seeds]
            if hit:
                reasons[path] = f"listed mod id {', '.join(hit)}"
            else:
                hit = sorted(needs & out_ids)
                if not hit:
                    continue
                reasons[path] = f"requires {', '.join(hit)}"
            out_ids.update(ids)
            changed = True

    with open(excluded_txt, "a") as ex, open(why_txt, "a") as why:
        for path, reason in sorted(reasons.items()):
            name = os.path.basename(path)
            shutil.move(path, os.path.join(out_dir, name))
            ex.write(name + "\n")
            why.write(f"{name}\t{reason}\n")
            print(f"   excluded {name}: {reason}")
    for s in seeds:
        if not any(s in ids for ids, _, _ in info.values()):
            print(f"   exclude_mods: no jar provides the listed mod id '{s}' (fine if the pack dropped it)")
    print(f"   {len(reasons)} jar(s) left out by mod id (listed: {', '.join(seeds) or 'none'})")


if __name__ == "__main__":
    main()
