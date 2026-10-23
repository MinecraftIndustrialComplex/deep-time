#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Checks the pool's pure-Python Mann-Whitney U against scipy (needs scipy: run it with the pool's numpy
environment, `tools/remote/mac.sh -- <pool>/.pyenv-numpy/bin/python3 -I tools/review/pool/selftest.py`)
and renders a table from made-up runs to look at its shape."""
import random
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import pooltable  # noqa: E402

try:
    from scipy.stats import mannwhitneyu
except ImportError:
    print("scipy is not importable here; run with the pool's numpy environment", file=sys.stderr)
    sys.exit(2)


def main():
    rng = random.Random(7)
    worst, cases = 0.0, 0
    for n1, n2 in [(3, 3), (3, 4), (4, 4), (6, 6), (8, 8), (5, 9), (9, 9), (12, 12), (12, 11), (24, 24), (2, 12), (8, 13), (9, 13), (7, 20)]:
        for tied in (False, True):
            for shift in (0.0, 0.5, 2.0):
                for _ in range(40):
                    if tied:
                        x = [rng.randint(0, 6) + shift for _ in range(n1)]
                        y = [float(rng.randint(0, 6)) for _ in range(n2)]
                    else:
                        x = [rng.gauss(shift, 1) for _ in range(n1)]
                        y = [rng.gauss(0, 1) for _ in range(n2)]
                    ours = pooltable.mwu(x, y)
                    ref = float(mannwhitneyu(x, y, alternative="two-sided").pvalue)
                    worst = max(worst, abs(ours - ref))
                    cases += 1
                    if abs(ours - ref) > 1e-9:
                        print(f"MISMATCH n={n1},{n2} tied={tied} shift={shift}: ours {ours} scipy {ref}\n  x={x}\n  y={y}")
                        return 1
    print(f"mwu: {cases} cases agree with scipy.stats.mannwhitneyu (largest difference {worst:.2e})")
    rng = random.Random(1)
    spec = {"criteria": ["land"], "columns": [
        {"key": "land", "label": "land", "target": [0.26, 0.32], "digits": 3},
        {"key": "net", "label": "net", "target": None, "digits": 3, "signed": True},
        {"special": "criteria", "label": "criteria met per run: median, mean"},
        {"key": "a", "key2": "b", "label": "pair", "digits": 0, "pair": True}]}

    def runs(shift):
        return [{"land": rng.gauss(0.28 + shift, 0.03), "net": rng.gauss(-0.08, 0.03), "a": rng.gauss(300, 50), "b": rng.gauss(450, 90)}
                for _ in range(12)]

    text, _ = pooltable.render(spec, ("base", runs(0)), [("arm", runs(0.03)), ("same", runs(0))])
    print(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
