#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Tests of find_places.py: a volcano's block comes from the Rust evaluator (a cube world has no closed
form from latitude and longitude). Standard library only: `python3 tools/workbench/test_find_places.py`."""

import contextlib
import io
import json
import os
import stat
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import find_places  # noqa: E402


class Volcanoes(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="test-find-places-")
        self.saved = {k: os.environ.get(k) for k in ("DEEPTIME_BIN",)}
        self.real_get = find_places.get
        find_places.get = lambda url: {"events": [
            {"kind": "deeptime:volcano_stratovolcano", "place": {"type": "point", "lat": 12.5, "lon": -40.0},
             "params": {"height_m": 3100}, "t_start": 5},
            {"kind": "deeptime:volcano_shield", "place": {"type": "point", "lat": -3.0, "lon": 80.0},
             "params": {"height_m": 900, "standing": 1}, "t_start": 6},
            {"kind": "deeptime:volcano_shield", "place": {"type": "point", "lat": 1.0, "lon": 1.0},
             "params": {"height_m": 50, "standing": 0}, "t_start": 7},   # extinct: not listed
            {"kind": "deeptime:rift", "place": {"type": "point", "lat": 9.0, "lon": 9.0}, "params": {}},
        ]}

    def tearDown(self):
        find_places.get = self.real_get
        for k, v in self.saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v

    def fake_deeptime(self, lines, code=0):
        p = os.path.join(self.tmp, "deeptime")
        calls, stdin = os.path.join(self.tmp, "calls"), os.path.join(self.tmp, "stdin")
        body = "\n".join(json.dumps(c) for c in lines)
        with open(p, "w") as f:
            f.write(f"#!/bin/sh\necho \"$@\" > {calls}\ncat > {stdin}\ncat <<'EOF'\n{body}\nEOF\nexit {code}\n")
        os.chmod(p, os.stat(p).st_mode | stat.S_IXUSR)
        os.environ["DEEPTIME_BIN"] = p
        return calls, stdin

    def test_volcanoes_are_placed_by_the_evaluator_tallest_first(self):
        calls, stdin = self.fake_deeptime([{"x": 100, "z": -200}, {"x": -7, "z": 33}])
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            find_places.volcanoes("http://x/api/runs/r", "/world")
        lines = out.getvalue().splitlines()
        self.assertEqual(len(lines), 2, out.getvalue())
        self.assertIn("stratovolcano", lines[0])
        self.assertIn("elev/100/-200/1", lines[0])
        self.assertIn("elev/-7/33/1", lines[1])
        with open(stdin) as f:
            self.assertEqual(f.read(), "12.5 -40.0\n-3.0 80.0\n")
        with open(calls) as f:
            self.assertEqual(f.read().split(), ["inspect-column", "--world-data", "/world", "--points", "-", "--latlon"])

    def test_a_failing_evaluator_stops_with_its_message(self):
        self.fake_deeptime([], code=1)
        with self.assertRaises(SystemExit) as cm:
            find_places.blocks_of("/world", [(1.0, 2.0)])
        self.assertIn("inspect-column", str(cm.exception))

    def test_no_volcano_says_so(self):
        find_places.get = lambda url: {"events": []}
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            find_places.volcanoes("http://x/api/runs/r", "/world")
        self.assertIn("no standing volcano", out.getvalue())


if __name__ == "__main__":
    unittest.main()
