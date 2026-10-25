#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""recpeek.py <run folder> [substring ...]: the keys of a run's rec.json and of its last step, and the
last step's values of the keys that contain any substring given (plain words only, for mac.sh)."""
import json
import sys
from pathlib import Path

r = json.loads((Path(sys.argv[1]) / "rec.json").read_text())
print("top keys", list(r))
s = r["steps"][-1]
print("step keys", len(s), sorted(s)[:400])
for k in sorted(s):
    if any(w in k for w in sys.argv[2:]):
        v = s[k]
        print(" ", k, json.dumps(v)[:600])
