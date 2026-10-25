#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""ledger_ids.py [PREFIX ...]: the highest ledger ids of each prefix on every local and origin branch, so
that a new row can leave a gap above what other lanes are adding (default prefixes PLT ISO FRC). Run from
the repository; reads `git show <ref>:docs/science/ledger/<PREFIX>.md` and the code comments' ids by
`git grep` on each ref."""
import re
import subprocess
import sys


def git(*args):
    return subprocess.run(["git", *args], capture_output=True, text=True).stdout


def main():
    prefixes = sys.argv[1:] or ["PLT", "ISO", "FRC"]
    refs = git("for-each-ref", "--format=%(refname:short)", "refs/heads", "refs/remotes/origin").split()
    for p in prefixes:
        best = []
        for ref in refs:
            text = git("show", f"{ref}:docs/science/ledger/{p}.md")
            rows = [int(m) for m in re.findall(rf"^\| {p}-(\d+) ", text, flags=re.M)]
            code = git("grep", "-h", "-o", "-E", rf"ledger: [^\n]*{p}-[0-9]+", ref, "--", "crates")
            refs_in_code = [int(m) for m in re.findall(rf"{p}-(\d+)", code)]
            top = max(rows + refs_in_code, default=0)
            best.append((top, max(rows, default=0), ref))
        best.sort(reverse=True)
        print(p, "highest id (rows or code), highest row, branch:")
        for top, row, ref in best[:6]:
            print(f"  {top:5d} {row:5d}  {ref}")


if __name__ == "__main__":
    main()
