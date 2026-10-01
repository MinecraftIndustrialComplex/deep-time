#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""How much of a baked planet's rock is plain stone or deepslate, before and after P1-28.

    deeptime-inspect rock-census --world-data <run>/planet --step 32 > census.json
    tools/review/p1-28-stone-share.py census.json [--before-rev 32ac4b6] [--table]

The census (crates/deeptime-inspect/src/census.rs) counts the planet's dry land per rock class: the top
rock of each column, the rock four blocks under it, the blocks within 64 of the surface and the whole
column. This script maps each class to the block a pure-vanilla pack places for it (Deep Time's own
member tags, else the class's vanilla_fallback) in two revisions of the class data (`--before-rev`,
default the revision P1-28 started from, read with git; and the working tree) and adds up the share
that is `minecraft:stone` or `minecraft:deepslate`. The unclassified crust (columns with no horizon)
counts as stone or deepslate as well. Standard library only.
"""
import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DATA = "neoforge/src/main/resources/data/deeptime"
CRUST = {"minecraft:stone", "minecraft:deepslate"}


def git_show(rev, path):
    return subprocess.run(["git", "show", f"{rev}:{path}"], cwd=ROOT, capture_output=True, text=True, check=True).stdout


def git_ls(rev, path):
    out = subprocess.run(["git", "ls-tree", "--name-only", rev, path + "/"], cwd=ROOT, capture_output=True, text=True, check=True)
    return [Path(line).name for line in out.stdout.split()]


def class_blocks(rev):
    """class -> the block id a pure-vanilla pack places for it (the first member, else the vanilla_fallback)."""
    out = {}
    for name in git_ls(rev, f"{DATA}/deeptime/rock_class") if rev else sorted(p.name for p in (ROOT / DATA / "deeptime/rock_class").glob("*.json")):
        cls = name[:-5]
        if rev:
            doc = json.loads(git_show(rev, f"{DATA}/deeptime/rock_class/{name}"))
            try:
                members = json.loads(git_show(rev, f"{DATA}/tags/block/rock/{name}"))["values"]
            except subprocess.CalledProcessError:
                members = []
        else:
            doc = json.loads((ROOT / DATA / "deeptime/rock_class" / name).read_text())
            tag = ROOT / DATA / "tags/block/rock" / name
            members = json.loads(tag.read_text())["values"] if tag.exists() else []
        out[cls] = (members or [doc["vanilla_fallback"]])
    return out


def share(rows, blocks):
    total = sum(r["count"] for r in rows)
    crust = 0.0
    by_class = {}
    for r in rows:
        bl = blocks.get(r["class"], ["minecraft:stone"])   # the unclassified crust stays stone or deepslate
        frac = sum(b in CRUST for b in bl) / len(bl)
        crust += r["count"] * frac
        by_class[r["class"]] = (r["count"], frac)
    return (crust / total if total else 0.0), by_class


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("census")
    ap.add_argument("--before-rev", default="32ac4b6")
    ap.add_argument("--table", action="store_true", help="print the per-class rows of the surface view")
    a = ap.parse_args()
    census = json.loads(Path(a.census).read_text())
    before, after = class_blocks(a.before_rev), class_blocks(None)
    print(f"{census['land_columns']} land columns of {census['columns_sampled']} sampled "
          f"(step {census['step_blocks']}); stone or deepslate share of the rock a pure-vanilla pack places:")
    print(f"{'view':<30}{'before':>9}{'after':>9}")
    for key, label in (("top_rock", "top block, columns"), ("under_soil_rock", "4 blocks down, columns"),
                       ("top_n_blocks", f"top {census['top_blocks']} blocks"), ("whole_column", "whole column, blocks")):
        rows = census[key]["classes"]
        b, _ = share(rows, before)
        f, by = share(rows, after)
        print(f"{label:<30}{b:>8.1%}{f:>9.1%}")
        if a.table and key == "top_n_blocks":
            print(f"  {'class':<24}{'share':>7}  before -> after")
            total = census[key]["total"]
            for cls, (n, _) in sorted(by.items(), key=lambda kv: -kv[1][0])[:30]:
                bb, aa = before.get(cls, ["-"])[0], after.get(cls, ["-"])[0]
                print(f"  {cls:<24}{n / total:>6.1%}  {bb} -> {aa}")


if __name__ == "__main__":
    sys.exit(main())
