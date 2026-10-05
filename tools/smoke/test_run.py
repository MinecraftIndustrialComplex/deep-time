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


class ProcessRss(unittest.TestCase):
    def test_reads_vmrss_from_proc_without_ps(self):
        root = tempfile.mkdtemp()
        os.makedirs(os.path.join(root, "42"))
        with open(os.path.join(root, "42", "status"), "w") as f:
            f.write("Name:\tjava\nVmHWM:\t 9999 kB\nVmRSS:\t  1234 kB\n")
        self.assertEqual(run.process_rss_kb(42, proc_root=root), 1234)

    def test_this_process_has_a_size(self):
        self.assertGreater(run.process_rss_kb(os.getpid()) or 0, 0)


class ProbeConsistency(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp()
        self.cli = os.path.join(self.tmp, "deeptime")
        os.environ["DEEPTIME_CLI"] = self.cli
        self.addCleanup(os.environ.pop, "DEEPTIME_CLI", None)

    def fake_cli(self, elevation, top_y):
        with open(self.cli, "w") as f:
            f.write("#!/bin/sh\necho '" + json.dumps({"surface": elevation, "top_y": top_y}) + "'\n")
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

    def test_a_cave_mouth_below_the_surface_is_tolerated_up_to_a_limit(self):
        self.fake_cli(12.5, 12)
        calls = []

        def probe(server, rcon, command, wait=1.0):
            calls.append(command)
            return ({"terrainH": 12.5, "surfaceY": 12 - 16 if len(calls) <= run.MAX_CARVED_COLUMNS else 13}, "")
        steps = run.Steps()
        orig = run.probe_json
        run.probe_json = probe
        try:
            with contextlib.redirect_stdout(io.StringIO()):
                run.probe_consistency(steps, None, None, self.tmp, 16384)
        finally:
            run.probe_json = orig
        self.assertFalse(steps.failed)
        self.assertIn("cave mouth", str(steps.rows[0]))

    def test_every_column_below_the_surface_fails(self):
        self.fake_cli(12.5, 12)
        steps = self.check({"terrainH": 12.5, "surfaceY": -4})
        self.assertEqual(len(steps.failed), 1)

    def test_missing_cli_is_recorded_not_failed(self):
        steps = self.check({"terrainH": 1.0, "surfaceY": 1})
        self.assertFalse(steps.failed)
        self.assertIn("skipped", steps.rows[0][2])


class CompatMixins(unittest.TestCase):
    def mods(self, *names):
        d = tempfile.mkdtemp()
        for n in names:
            open(os.path.join(d, n), "w").close()
        return d

    def test_each_compat_mixin_is_expected_only_when_its_mod_is_installed(self):
        none = run.expected_compat_mixins(self.mods("deeptime-1.21.1-0.0.1.jar"))
        self.assertEqual(none, [])
        labels = [l for l, *_ in run.expected_compat_mixins(self.mods(
            "createoreexcavation-1.21-1.6.8.jar", "createdieselgenerators-1.21.1-1.3.15.jar"))]
        self.assertEqual(labels, ["Create Ore Excavation (COE)", "Create Diesel Generators (CDG)"])

    def test_the_dh_guard_needs_both_distant_horizons_and_toroidal_world(self):
        only_dh = run.expected_compat_mixins(self.mods("DistantHorizons-3.3.1.jar"))
        both = run.expected_compat_mixins(self.mods("DistantHorizons-3.3.1.jar", "toroidal_world-0.19.3.jar"))
        self.assertEqual(only_dh, [])
        self.assertEqual([c for _, c, *_ in both], ["deeptime.tw_dh.mixins.json"])


if __name__ == "__main__":
    unittest.main()
