#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Carries the VM's client stage into Ben's Prism instance (run on the laptop).

    tools/remote/play-client-apply.py --hash 8f2b7e79 [--instance <minecraft dir>] [--dry-run]

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
"""
import argparse
import json
import os
import subprocess
import sys
import tempfile

HOST = os.environ.get("DEEPTIME_MAC", "ban@deeptime-test-ban.westus2.cloudapp.azure.com")
DEFAULT_INSTANCE = os.path.expanduser(
    "~/.var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances/MIC Deep Time Test/minecraft")
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


def cached(path):
    with open(path) as f:
        return json.load(f).get("cachedFiles", {})


def signature(entry):
    return (entry.get("cachedLocation"), json.dumps(entry.get("hash"), sort_keys=True),
            json.dumps(entry.get("linkedFileHash"), sort_keys=True))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--hash", required=True, help="the play folder's name; the stage is <hash>-client")
    ap.add_argument("--instance", default=DEFAULT_INSTANCE, help="the instance's minecraft folder")
    ap.add_argument("--dry-run", action="store_true")
    a = ap.parse_args()
    inst = os.path.abspath(a.instance)
    if not os.path.isfile(os.path.join(inst, "packwiz.json")):
        sys.exit(f"play-client-apply: {inst} has no packwiz.json (not the instance's minecraft folder?)")
    ps = subprocess.run(["ps", "-eo", "args"], capture_output=True, text=True).stdout
    if any(inst in line and "java" in line for line in ps.splitlines()):
        sys.exit("play-client-apply: a Minecraft process has this instance open; close the game first")
    stage = f"{HOST}:/data/ban/play/{a.hash}-client/minecraft/"

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
