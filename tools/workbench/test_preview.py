#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Tests of preview.py's workbench routes: which worlds a run's preview and inspect use.
Standard library only: `python3 tools/workbench/test_preview.py`."""

import json
import os
import stat
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import preview  # noqa: E402


def write_manifest(runs, run, **kw):
    """A run's world/manifest.json with what run_bake reads."""
    d = os.path.join(runs, run, "world")
    os.makedirs(d, exist_ok=True)
    m = {"format_version": "0.1", "source": "run:earthlike_quick_16k", "scale": {"circumference_blocks": 16384},
         "layers": [{"name": "elevation"}, {"name": "strata/horizons"}],
         "notes": {"bake": {"baked": True}, "run": {}}}
    m.update(kw)
    with open(os.path.join(d, "manifest.json"), "w") as f:
        json.dump(m, f)
    return d


class Runs(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="test-preview-")
        self.runs = os.path.join(self.tmp, "runs")
        os.makedirs(self.runs)
        self.saved = {k: os.environ.get(k) for k in (preview.RUNS_ENV, preview.DEEPTIME_ENV, "DEEPTIME_WORKBENCH_JOBS")}
        os.environ[preview.RUNS_ENV] = self.runs
        os.environ["DEEPTIME_WORKBENCH_JOBS"] = os.path.join(self.tmp, "jobs")
        self.submitted = []
        self.real_submit = preview.submit
        preview.submit = lambda spec, job=None: (self.submitted.append(preview.normalise_spec(spec)) or
                                                 {"job": "j1", "state": "queued", "spec": spec,
                                                  "note": spec.get("note"), "note_kind": spec.get("note_kind")})

    def tearDown(self):
        preview.submit = self.real_submit
        for k, v in self.saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v

    def test_a_baked_run_is_found(self):
        d = write_manifest(self.runs, "r1")
        bake, why = preview.run_bake("r1")
        self.assertEqual((bake["dir"], bake["c"], why), (d, 16384, ""))

    def test_no_bake_for_old_running_placeholder_or_odd_runs(self):
        write_manifest(self.runs, "old", format_version="0.0", notes={})
        write_manifest(self.runs, "ph", notes={"bake": {"baked": True}, "run": {"placeholder": True}})
        write_manifest(self.runs, "nobake", notes={})
        write_manifest(self.runs, "flat", layers=[{"name": "elevation"}])
        write_manifest(self.runs, "odd", scale={"circumference_blocks": 5000})
        os.makedirs(os.path.join(self.runs, "running", "world", "keyframes"))
        for run in ("old", "ph", "nobake", "flat", "odd", "running", "missing", "../x", ""):
            bake, why = preview.run_bake(run)
            self.assertIsNone(bake, run)
            self.assertTrue(why, run)
        preview_root = os.environ.pop(preview.RUNS_ENV)
        self.assertIsNone(preview.run_bake("r1")[0])
        os.environ[preview.RUNS_ENV] = preview_root

    def test_runs_folder_is_an_allowed_world_data_root(self):
        d = write_manifest(self.runs, "r1")
        self.assertEqual(preview.check_path(d, "world_data"), os.path.realpath(d))

    def test_preview_of_a_baked_run_uses_its_world_data(self):
        d = write_manifest(self.runs, "r1", scale={"circumference_blocks": 32768})
        code, st = preview.workbench_preview(
            {"run": "r1", "frame": 3, "box": {"x0": 0, "z0": 0, "x1": 1, "z1": 1},
             "lat_lon": {"north": 30.5, "south": 30.0, "west": 100.0, "east": 100.5}})
        self.assertEqual(code, 202)
        spec = self.submitted[0]
        self.assertEqual(spec["world"]["world_data"], os.path.realpath(d))
        self.assertNotIn("preset", spec["world"])
        self.assertEqual(spec["size"], 32768)
        self.assertEqual(st["note_kind"], "run")
        self.assertIn("own terrain", st["note"])
        self.assertNotIn("Not run", st["note"])
        # the box is on the run's planet (C = 32,768): 100° east of the seam is x = 32768·100/360 − 16384
        x0, z0, x1, z1 = spec["boxes"][0]
        self.assertAlmostEqual((x0 + x1) / 2, 32768 * 100.25 / 360 - 16384, delta=2)
        self.assertAlmostEqual((z0 + z1) / 2, -32768 * 30.25 / 360, delta=2)
        self.assertLessEqual(max(x1 - x0, z1 - z0), preview.MAX_SIDE)

    def test_preview_of_a_run_without_a_bake_keeps_the_test_world(self):
        write_manifest(self.runs, "old", format_version="0.0", notes={})
        for run in ("old", "unknown"):
            code, st = preview.workbench_preview({"run": run, "box": {"x0": -5700, "z0": -1100, "x1": -5500, "z1": -900}})
            self.assertEqual(code, 202)
            spec = self.submitted[-1]
            self.assertEqual(spec["world"]["preset"], preview.PREVIEW_PRESET)
            self.assertEqual(spec["size"], preview.PREVIEW_C)
            self.assertEqual(st["note_kind"], "test-world")
            self.assertTrue(st["note"].startswith(f"Not run {run}'s terrain"), st["note"])

    def fake_deeptime(self, body, code=0):
        p = os.path.join(self.tmp, "deeptime")
        with open(p, "w") as f:
            f.write("#!/bin/sh\necho \"$@\" > " + os.path.join(self.tmp, "args") + "\n" +
                    f"cat <<'EOF'\n{body}\nEOF\nexit {code}\n")
        os.chmod(p, os.stat(p).st_mode | stat.S_IXUSR)
        os.environ[preview.DEEPTIME_ENV] = p

    def test_inspect_of_a_baked_run_asks_inspect_column_and_answers_at_once(self):
        d = write_manifest(self.runs, "r1")
        col = {"x": -5563, "z": -1002, "surface": 812.34, "top_y": 812, "sites": [], "strata": []}
        self.fake_deeptime(json.dumps(col))
        code, out = preview.workbench_inspect({"run": "r1", "frame": "3", "x": "-5563", "z": "-1002",
                                                "lat": "21.9", "lon": "57.6"})
        self.assertEqual(code, 200)
        self.assertEqual(out["source"], "bake")
        self.assertEqual(out["terrain_height"], 812.3)
        self.assertEqual(out["inspect"], col)
        self.assertEqual(out["note_kind"], "run")
        self.assertEqual(self.submitted, [], "no preview job starts")
        with open(os.path.join(self.tmp, "args")) as f:
            args = f.read().split()
        self.assertEqual(args, ["inspect-column", "--world-data", os.path.realpath(d), "--why", "-5563", "-1002"])

    def test_inspect_folds_x_and_reports_a_failing_evaluator(self):
        write_manifest(self.runs, "r1")
        self.fake_deeptime(json.dumps({"surface": 1.0, "top_y": 0}))
        _, out = preview.workbench_inspect({"run": "r1", "x": str(16384 + 5), "z": "0"})
        self.assertEqual(out["column"], {"x": 5, "z": 0})
        self.fake_deeptime("", code=1)
        with self.assertRaises(ValueError):
            preview.workbench_inspect({"run": "r1", "x": "0", "z": "0"})

    def test_inspect_without_a_bake_starts_the_small_test_world_preview(self):
        code, out = preview.workbench_inspect({"run": "old", "x": "-5563", "z": "-1002"})
        self.assertEqual(code, 202)
        spec = self.submitted[0]
        self.assertEqual(spec["world"]["preset"], preview.PREVIEW_PRESET)
        self.assertEqual(spec["pick"], [-5563, -1002])
        self.assertEqual(out["note_kind"], "test-world")



class LevelTypeSpecs(unittest.TestCase):
    """A comparison world in the pack's own generator (vanilla or Terralith, no Deep Time)."""

    def test_a_level_type_world_with_located_boxes(self):
        s = preview.normalise_spec({"profile": "mic", "world": {
            "level_type": "minecraft:normal", "seed": 5,
            "locate": ["terralith:emerald_peaks", "minecraft:jagged_peaks@1000,-2000"], "locate_side": 320,
            "jvm_props": ["deeptime.steepScale=0"]}}, max_side=512)
        self.assertEqual(s["boxes"], [])
        self.assertEqual(s["y"], list(preview.VANILLA_Y))
        self.assertEqual(s["world"]["locate_side"], 320)
        self.assertEqual(s["max_side"], 512)

    def test_bad_level_type_specs_are_refused(self):
        for world in ({"level_type": "normal"},                       # not a resource location
                      {"synth": "wrap-test", "locate": ["minecraft:plains"]},
                      {"level_type": "minecraft:normal", "locate": ["minecraft:plains@x,1"]},
                      {"level_type": "minecraft:normal", "locate": ["minecraft:plains"], "jvm_props": ["java.home=/"]},
                      {"level_type": "minecraft:normal", "synth": "wrap-test", "box": [0, 0, 16, 16]}):
            with self.assertRaises(ValueError, msg=str(world)):
                spec = {"world": world}
                if "box" in world:
                    spec["box"] = world.pop("box")
                preview.normalise_spec(spec)


if __name__ == "__main__":
    unittest.main()
