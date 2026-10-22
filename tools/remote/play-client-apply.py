#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Carries the VM's client stage into Ben's Prism instance (run on the laptop).

    tools/remote/play-client-apply.py --hash 8f2b7e79 [--set lite] [--instance <minecraft dir>] [--dry-run] [--devmap] [--extra-mod JAR]...

The stage is /data/ban/play/<hash>-client/minecraft/ on the test VM, built by
tools/remote/vm/play-client-stage.sh: the MIC pack's last commit installed with packwiz-installer
(-s client), without the mods the play server leaves out (Cobblemon and what requires it), and with the
play server's Deep Time and Toroidal World jars. The instance keeps everything of Ben's (options.txt,
saves/, screenshots/, servers.dat, logs/, config files a mod wrote itself, instance.cfg, which is not
in minecraft/ at all). What changes:

  * mods/ becomes exactly the stage's mods (rsync --checksum --delete): the client and the server
    sets match, whatever the instance held before.
  * every other file the pack ships (the instance's packwiz.json "cachedFiles" against the stage's) is
    copied when the pack changed it since the instance's last install, or when it is missing; a file
    the pack did not change is left alone, so local edits to it survive.
  * a file the old install had and the new pack no longer ships is removed.
  * packwiz.json is replaced by the stage's, so the instance's record is the pack commit now installed.

Nothing here is heavy: one rsync of the changed jars. It refuses to run while a Minecraft process
has the instance open.

Opt-in extras, for dev play tests (never the pack's):

  * --devmap also puts the internal developer map (mod id deeptime_devmap) in mods/: the play
    folder's own deeptime_devmap-*.jar (/data/ban/play/<hash>/mods, put there by play-stage.sh
    --devmap), so the client runs the server's build. Not needed when the stage was built with
    play-client-stage.sh --devmap (the mirror brings it then). Without either, the mirror removes
    a dev map jar the instance held: a later plain apply takes it out again.
  * --extra-mod JAR (repeatable) copies a local jar into mods/ after the mirror.

--set lite (Ben, 2026-10-22) carries the lite stage /data/ban/play/<hash>-client-lite/minecraft/ (built by
play-client-stage.sh --set lite: only the mods Deep Time's data names, their required dependencies and the
core in tools/smoke/profiles/mic-lite.core) into a SECOND instance, "MIC Deep Time Lite", next to the full
"MIC Deep Time Test", which it never writes. The lite server it must join is /data/ban/play/<hash>-lite
(play-stage.sh --set lite; the play unit serves one folder at a time, see that script). When the instance does
not exist yet it is created: instance.cfg (the template's Java and memory settings, a new name and uuid, no
play time, and JoinServerOnLaunch=false: no lite play server runs by default), mmc-pack.json (the same NeoForge), the stage's mods and pack files, and from the template instance
Ben's options.txt and config/DistantHorizons.toml. No saves, screenshots or logs. The new folder is built next
to the instances (a hidden folder with no instance.cfg until the end) and renamed into place, so Prism never
sees half an instance; nothing else of Prism's is touched. Run again later to update it like the full one.
"""
import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import uuid

HOST = os.environ.get("DEEPTIME_MAC", "ban@deeptime-test-ban.westus2.cloudapp.azure.com")
DEFAULT_INSTANCE = os.path.expanduser(
    "~/.var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances/MIC Deep Time Test/minecraft")
PRISM_INSTANCES = os.path.expanduser("~/.var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances")
LITE_NAME = "MIC Deep Time Lite"
TEMPLATE_NAME = "MIC Deep Time Test"
# Of the template instance, Ben's own files a new lite instance starts with (relative to its minecraft/).
TEMPLATE_FILES = ("options.txt", "config/DistantHorizons.toml")
# instance.cfg keys a new instance does not inherit (play time and launch history) and its UI section.
CFG_RESET = ("lastLaunchTime", "lastTimePlayed", "totalTimePlayed", "name", "uuid")
# instance.cfg keys a new LITE instance gets instead of the template's (Ben's coordinator, 2026-10-22: no lite
# play server exists, so the full instance's join-on-launch would show a failed join on every launch). Only a
# new instance is seeded with them: a later apply never rewrites instance.cfg, so a change of Ben's stays.
LITE_CFG = {"JoinServerOnLaunch": "false"}
# Files of the instance that the pack also ships as defaults (the pack's options.txt) and the player owns.
KEEP = {"options.txt", "servers.dat", "optionsof.txt", "optionsshaders.txt"}
SSH = "/run/current-system/sw/bin/ssh"
if not os.path.exists(SSH):
    SSH = "ssh"


def rsync(args, dry):
    cmd = ["rsync", "-e", f"{SSH} -o BatchMode=yes", *args]
    if dry:
        cmd.insert(1, "--dry-run")
    print("   $", " ".join(cmd[:1] + [a if " " not in a else repr(a) for a in cmd[1:]]))
    subprocess.run(cmd, check=True)


def new_instance_cfg(template_cfg, name, overrides=None):
    """instance.cfg text for a new instance: the template's [General] keys (Java, memory, join-on-launch), a
    new name and uuid, no launch history, no [UI] section; overrides (key -> value) replace or add keys."""
    out, section = [], None
    for line in open(template_cfg).read().splitlines():
        if line.startswith("["):
            section = line
            if section == "[General]":
                out.append(line)
            continue
        if section != "[General]" or not line.strip():
            continue
        key = line.split("=", 1)[0]
        if key in CFG_RESET or key in (overrides or {}):
            continue
        out.append(line)
    out += [f"{k}={v}" for k, v in (overrides or {}).items()]
    out += [f"name={name}", f"uuid={uuid.uuid4().hex}"]
    return "\n".join(out) + "\n"


def seed_instance(root, template_root, name):
    """The files of a new instance that do not come from the stage: instance.cfg, mmc-pack.json, an empty
    packwiz.json (so every pack file counts as new) and the template's options.txt and DistantHorizons.toml."""
    mc = os.path.join(root, "minecraft")
    os.makedirs(mc)
    with open(os.path.join(root, "instance.cfg"), "w") as f:
        f.write(new_instance_cfg(os.path.join(template_root, "instance.cfg"), name, LITE_CFG))
    shutil.copy2(os.path.join(template_root, "mmc-pack.json"), os.path.join(root, "mmc-pack.json"))
    with open(os.path.join(mc, "packwiz.json"), "w") as f:
        json.dump({"cachedFiles": {}}, f)
    copy_template_files(root, template_root)


def copy_template_files(root, template_root):
    for rel in TEMPLATE_FILES:
        src = os.path.join(template_root, "minecraft", rel)
        if os.path.isfile(src):
            os.makedirs(os.path.dirname(os.path.join(root, "minecraft", rel)), exist_ok=True)
            shutil.copy2(src, os.path.join(root, "minecraft", rel))
            print(f"   from the template: {rel}")
        else:
            print(f"   (the template has no {rel})")


def cached(path):
    with open(path) as f:
        return json.load(f).get("cachedFiles", {})


def signature(entry):
    return (entry.get("cachedLocation"), json.dumps(entry.get("hash"), sort_keys=True),
            json.dumps(entry.get("linkedFileHash"), sort_keys=True))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--hash", required=True, help="the play folder's name; the stage is <hash>-client (<hash>-client-lite)")
    ap.add_argument("--set", choices=("full", "lite"), default="full",
                    help="the full stage into the full instance (default), or the lite stage into \"MIC Deep Time Lite\" (created when missing)")
    ap.add_argument("--instance", default=None,
                    help="the instance's minecraft folder (default: the full instance's, or the lite one's with --set lite)")
    ap.add_argument("--template", default=os.path.join(PRISM_INSTANCES, TEMPLATE_NAME),
                    help="--set lite: the instance (its root folder) a new lite instance copies its Java settings, "
                         "mmc-pack.json, options.txt and DistantHorizons.toml from; only read")
    ap.add_argument("--dry-run", action="store_true")
    ap.add_argument("--devmap", action="store_true",
                    help="also install the play folder's deeptime_devmap jar (the internal dev map; opt-in)")
    ap.add_argument("--extra-mod", action="append", default=[], metavar="JAR",
                    help="also copy this local jar into mods/ (repeatable; opt-in)")
    a = ap.parse_args()
    for j in a.extra_mod:
        if not (os.path.isfile(j) and j.endswith(".jar")):
            sys.exit(f"play-client-apply: --extra-mod {j}: not a jar file")
    lite = a.set == "lite"
    suffix = "-lite" if lite else ""
    if a.instance is None:
        a.instance = os.path.join(PRISM_INSTANCES, LITE_NAME, "minecraft") if lite else DEFAULT_INSTANCE
    inst = os.path.abspath(a.instance)
    stage = f"{HOST}:/data/ban/play/{a.hash}-client{suffix}/minecraft/"
    play_folder = f"/data/ban/play/{a.hash}{suffix}"
    root = os.path.dirname(inst)
    if lite and not os.path.exists(root):
        create_lite(a, inst, root, stage, play_folder)
        return
    run_apply(a, inst, stage, play_folder)


def create_lite(a, inst, root, stage, play_folder):
    template = os.path.abspath(a.template)
    for f in ("instance.cfg", "mmc-pack.json"):
        if not os.path.isfile(os.path.join(template, f)):
            sys.exit(f"play-client-apply: --template {template} has no {f} (the full instance's root folder?)")
    if os.path.basename(inst) != "minecraft":
        sys.exit(f"play-client-apply: {inst}: a new instance's minecraft folder must be called minecraft")
    name = os.path.basename(root)
    parent = os.path.dirname(root)
    if not os.path.isdir(parent):
        sys.exit(f"play-client-apply: {parent} does not exist (Prism's instances folder?)")
    print(f"creating the instance {name!r} at {root}" + (" (dry run: in a scratch folder, nothing is created)" if a.dry_run else ""))
    # A hidden folder beside the instances, without instance.cfg until the end, renamed in place when complete
    # (one atomic rename on one filesystem): Prism lists a folder only if it holds instance.cfg.
    build = tempfile.mkdtemp(prefix="dt-lite-new-") if a.dry_run else os.path.join(parent, f".{name}.deeptime-new")
    try:
        if a.dry_run:
            seed_root = os.path.join(build, "instance")
        else:
            if os.path.exists(build):
                sys.exit(f"play-client-apply: {build} is left over from an earlier run (check it is not in use, then remove it)")
            seed_root = build
        seed_instance(seed_root, template, name)
        run_apply(a, os.path.join(seed_root, "minecraft"), stage, play_folder)
        # The pack may ship its own DistantHorizons.toml: Ben's wins.
        copy_template_files(seed_root, template)
        if a.dry_run:
            print(f"dry run: would create {root}")
        else:
            os.rename(seed_root, root)
            print(f"created {root}")
    finally:
        if a.dry_run:
            shutil.rmtree(build, ignore_errors=True)
        elif os.path.isdir(build):
            shutil.rmtree(build, ignore_errors=True)


def run_apply(a, inst, stage, play_folder):
    if not os.path.isfile(os.path.join(inst, "packwiz.json")):
        sys.exit(f"play-client-apply: {inst} has no packwiz.json (not the instance's minecraft folder?)")
    ps = subprocess.run(["ps", "-eo", "args"], capture_output=True, text=True).stdout
    if any(inst in line and "java" in line for line in ps.splitlines()):
        sys.exit("play-client-apply: a Minecraft process has this instance open; close the game first")

    with tempfile.TemporaryDirectory(prefix="dt-play-apply-") as tmp:
        subprocess.run(["rsync", "-e", f"{SSH} -o BatchMode=yes", stage + "packwiz.json", tmp + "/"], check=True)
        old = cached(os.path.join(inst, "packwiz.json"))
        new = cached(os.path.join(tmp, "packwiz.json"))
        copy, remove = [], []
        new_locations = {e["cachedLocation"] for e in new.values() if e.get("cachedLocation")}
        for key, e in new.items():
            loc = e.get("cachedLocation") or key
            if loc.startswith("mods/"):
                continue  # mods/ is mirrored below
            if loc in KEEP:
                continue  # Ben's own settings, never replaced by the pack's defaults
            if key not in old or signature(old[key]) != signature(e) or not os.path.exists(os.path.join(inst, loc)):
                copy.append(loc)
        for key, e in old.items():
            loc = e.get("cachedLocation") or key
            if loc.startswith("mods/") or loc in new_locations or loc in KEEP:
                continue
            if os.path.exists(os.path.join(inst, loc)):
                remove.append(loc)
        print(f"pack files: {len(copy)} to copy (changed or missing), {len(remove)} to remove (the pack dropped them)")
        for loc in sorted(copy)[:40]:
            print("   copy  ", loc)
        for loc in sorted(remove):
            print("   remove", loc)

        print("mods/: mirror the stage (checksums; extra jars are deleted)")
        os.makedirs(os.path.join(inst, "mods"), exist_ok=True)
        rsync(["-rl", "--checksum", "--delete", "--itemize-changes", stage + "mods/", os.path.join(inst, "mods") + "/"], a.dry_run)
        if a.devmap:
            print("mods/: + the play folder's dev map jar (--devmap)")
            src = f"{HOST}:{play_folder}/mods/deeptime_devmap-*.jar"
            try:
                rsync(["--checksum", "--itemize-changes", src, os.path.join(inst, "mods") + "/"], a.dry_run)
            except subprocess.CalledProcessError:
                sys.exit(f"play-client-apply: --devmap: no deeptime_devmap-*.jar in {play_folder}/mods "
                         "(tools/remote/vm/play-stage.sh --mods-only --hash <hash> --devmap first)")
        for j in a.extra_mod:
            print(f"mods/: + {j} (--extra-mod)")
            if not a.dry_run:
                shutil.copy2(j, os.path.join(inst, "mods", os.path.basename(j)))

        if copy:
            lst = os.path.join(tmp, "files.txt")
            with open(lst, "w") as f:
                f.write("\n".join(copy) + "\n")
            # A file the stage lacks (an excluded mod's data, say) is skipped, not an error.
            r = subprocess.run(["rsync", "-e", f"{SSH} -o BatchMode=yes", *(["--dry-run"] if a.dry_run else []),
                                "-rl", "--checksum", "--ignore-missing-args", "--itemize-changes", "--files-from", lst, stage, inst + "/"])
            if r.returncode not in (0, 23):
                sys.exit(f"play-client-apply: rsync of the pack files failed ({r.returncode})")
        if not a.dry_run:
            for loc in remove:
                os.remove(os.path.join(inst, loc))
            subprocess.run(["rsync", "-e", f"{SSH} -o BatchMode=yes", stage + "packwiz.json", inst + "/"], check=True)
    print("done" if not a.dry_run else "dry run: nothing changed")


if __name__ == "__main__":
    main()
