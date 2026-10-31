# SPDX-License-Identifier: LGPL-3.0-or-later
"""The plateau lane: words of the runs' summaries, per run and the median (VM, cached pool folders, no simulation).

Usage: python3 -I words.py <arm folder> <word> [<word> ...]

Reads <arm folder>/<start>-<seed>/summary.txt and prints, for each `word=value`, each run's value and the median over
the runs. Every occurrence in a summary is summed, so a word printed once per stage adds up. A value may carry a unit
suffix (`3.7s`); the number before it is read.
"""
import re
import statistics
import sys
from pathlib import Path


def main():
    folder, words = Path(sys.argv[1]), sys.argv[2:]
    pats = {w: re.compile(rf"(?<![\w.]){re.escape(w)}=(-?[0-9][0-9.eE+-]*)") for w in words}
    rows = {w: {} for w in words}
    for run in sorted(p for p in folder.iterdir() if p.is_dir()):
        f = run / "summary.txt"
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8", errors="replace")
        for w, pat in pats.items():
            vals = [float(m.group(1).rstrip("eE+-")) for line in text.splitlines() for m in [None] + list(pat.finditer(line)) if m]
            if vals:
                rows[w][run.name] = sum(vals)
    print(f"== {folder}")
    for w in words:
        per = rows[w]
        if not per:
            print(f"  {w:24s} -")
            continue
        runs = ", ".join(f"{k} {v:.4g}" for k, v in per.items())
        print(f"  {w:24s} median {statistics.median(per.values()):.4g} ({len(per)} runs): {runs}")


if __name__ == "__main__":
    main()
