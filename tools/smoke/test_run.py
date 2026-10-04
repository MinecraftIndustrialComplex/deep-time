#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Tests of the smoke harness's P1-13 checks (tools/smoke/run.py): the generation check records the
native memory peak, and the probe-vs-world-data check passes on agreeing answers and fails on a
disagreeing one. No Minecraft, no Rust: a fake server log and a fake CLI. Standard library only:

    python3 -m unittest discover -s tools/smoke -p 'test_*.py'
"""

import contextlib
import importlib.util
import io
import json
import os
import stat
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
spec = importlib.util.spec_from_file_location("smoke_run", os.path.join(HERE, "run.py"))
run = importlib.util.module_from_spec(spec)
spec.loader.exec_module(run)


class FakeServer:
    def __init__(self, timed, probes=None):
        self.timed = timed
        self.lines = [l for _, l in timed]
        self.done_at = timed[-1][0] + 1
        self.probes = probes or {}


def progress(native, cap=4.0):
    return f"[deeptime] plates 10% (-900 Ma) · total 5% · 5s elapsed · ~1m left · native {native}/{cap} GiB"


class GenerationCheck(unittest.TestCase):
    def test_records_the_peak_native_memory_not_the_last_line(self):
        timed = [(0.0, "[deeptime] generating the world's history on first start: x"),
                 (5.0, progress(0.4)), (10.0, progress(2.9)), (15.0, progress(1.7)),
                 (20.0, "[deeptime] world data written to /x in 20s")]
        steps = run.Steps()
        with contextlib.redirect_stdout(io.StringIO()):
            run.check_generation(steps, FakeServer(timed), {})
        peak = [r for r in steps.rows if r[0] == "peak native memory (GiB)"]
        self.assertEqual(len(peak), 1)
        self.assertEqual(peak[0][2], "2.9 of the 4.0 GiB cap")
        self.assertFalse(steps.failed)

    def test_a_gap_over_five_seconds_fails(self):
        timed = [(0.0, "[deeptime] generating the world's history on first start: x"),
                 (5.0, progress(0.4)), (16.0, progress(0.5)),
                 (20.0, "[deeptime] world data written to /x in 20s")]
        steps = run.Steps()
        with contextlib.redirect_stdout(io.StringIO()):
            run.check_generation(steps, FakeServer(timed), {})
        self.assertEqual([r[0] for r in steps.failed], ["[deeptime] progress lines at most 5 s apart"])


class ProbeConsistency(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.cli = os.path.join(self.tmp, "deeptime")
        os.environ["DEEPTIME_CLI"] = self.cli
        self.addCleanup(os.environ.pop, "DEEPTIME_CLI", None)

    def fake_cli(self, elevation, top_y):
        with open(self.cli, "w") as f:
            f.write("#!/bin/sh\necho '" + json.dumps({"elevation": elevation, "top_y": top_y}) + "'\n")
        os.chmod(self.cli, os.stat(self.cli).st_mode | stat.S_IEXEC)

    def check(self, probe):
        steps = run.Steps()
        orig = run.probe_json
        run.probe_json = lambda server, rcon, command, wait=1.0: (dict(probe), "")
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                run.probe_consistency(steps, None, None, self.tmp, 16384)
        finally:
            run.probe_json = orig
        return steps

    def test_agreeing_probe_passes(self):
        self.fake_cli(12.5, 12)
        steps = self.check({"terrainH": 12.5, "surfaceY": 13})
        self.assertFalse(steps.failed)
        self.assertEqual(steps.rows[0][2], "8 of 8")

    def test_a_height_that_differs_fails(self):
        self.fake_cli(12.5, 12)
        steps = self.check({"terrainH": 40.0, "surfaceY": 13})
        self.assertEqual(len(steps.failed), 1)

    def test_a_generator_surface_far_from_the_data_fails(self):
        self.fake_cli(12.5, 12)
        steps = self.check({"terrainH": 12.5, "surfaceY": 90})
        self.assertEqual(len(steps.failed), 1)

    def test_missing_cli_is_recorded_not_failed(self):
        steps = self.check({"terrainH": 1.0, "surfaceY": 1})
        self.assertFalse(steps.failed)
        self.assertIn("skipped", steps.rows[0][2])


if __name__ == "__main__":
    unittest.main()
