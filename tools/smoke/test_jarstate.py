#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Tests of the Deep Time jar staleness check (tools/smoke/jarstate.py) and of the workbench's use
of it (tools/workbench/preview.py refresh_deeptime_jar), on a throwaway checkout and server set;
no Gradle, no Minecraft. Standard library only (Python 3.9 on the Mac too):

    python3 -m unittest discover -s tools/smoke -p 'test_*.py'
    tools/remote/mac.sh -- 'python3 -m unittest discover -s tools/smoke -p "test_*.py" -v'
"""

import contextlib
import importlib.util
import io
import json
import os
import shutil
import sys
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
sys.path.insert(0, HERE)
import jarstate  # noqa: E402


def load_preview():
    spec = importlib.util.spec_from_file_location("preview", os.path.join(REPO, "tools", "workbench", "preview.py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(text)


def read(path):
    with open(path) as f:
        return f.read()


def read_json(path):
    with open(path) as f:
        return json.load(f)


class FakeCheckout(unittest.TestCase):
    """A tiny checkout: the source trees the jar is built from, a server set, and a 'Gradle' that
    writes a jar whose bytes are a function of those sources (so a revert gives the old bytes)."""

    def setUp(self):
        self.tmp = tempfile.mkdtemp(prefix="jarstate-test-")
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.repo = os.path.join(self.tmp, "repo")
        write(os.path.join(self.repo, "neoforge", "gradle.properties"), "mod_version = 0.0.1\n")
        self.java = os.path.join(self.repo, "neoforge", "src", "main", "java", "Gen.java")
        write(self.java, "class Gen { int v = 1; }\n")
        write(os.path.join(self.repo, "crates", "deeptime-core", "src", "lib.rs"), "pub fn f() {}\n")
        write(os.path.join(self.repo, "crates", "deeptime-jni", "src", "lib.rs"), "pub fn g() {}\n")
        write(os.path.join(self.repo, "Cargo.toml"), "[workspace]\n")
        write(os.path.join(self.repo, "docs", "notes.md"), "not a source of the jar\n")
        self.mods = os.path.join(self.repo, "smoke", "mic", "server", "mods")
        write(os.path.join(self.mods, "create-1.21.1.jar"), "another mod")
        self.install_json = os.path.join(self.repo, "smoke", "mic", "install.json")
        write(self.install_json, json.dumps({"profile": "mic", "deeptime_jar": "deeptime-1.21.1-0.0.1.jar"}))

    def gradle_jar(self, job=None):
        """The fake `gradlew jar`: build/libs/deeptime-1.21.1-0.0.1.jar, bytes from the sources."""
        out = os.path.join(self.repo, "neoforge", "build", "libs", "deeptime-1.21.1-0.0.1.jar")
        write(out, "jar of " + jarstate.source_hash(self.repo))
        return out

    def set_jar(self, name="deeptime-1.21.1-0.0.1.jar", text="old jar"):
        write(os.path.join(self.mods, name), text)
        return os.path.join(self.mods, name)


class SourceHash(FakeCheckout):
    def test_changes_with_content_not_with_mtime_or_build_output(self):
        h0 = jarstate.source_hash(self.repo)
        self.assertEqual(h0, jarstate.source_hash(self.repo))
        os.utime(self.java, (1, 1))                                                    # touch: same bytes
        self.assertEqual(h0, jarstate.source_hash(self.repo))
        write(os.path.join(self.repo, "neoforge", "build", "libs", "x.jar"), "output")   # build output
        write(os.path.join(self.repo, "neoforge", ".gradle", "state"), "cache")
        write(os.path.join(self.repo, "docs", "more.md"), "docs are not sources")
        self.assertEqual(h0, jarstate.source_hash(self.repo))
        write(self.java, "class Gen { int v = 2; }\n")                                 # a real edit
        h1 = jarstate.source_hash(self.repo)
        self.assertNotEqual(h0, h1)
        write(self.java, "class Gen { int v = 1; }\n")                                 # reverted
        self.assertEqual(h0, jarstate.source_hash(self.repo))

    def test_rust_sources_count(self):
        h0 = jarstate.source_hash(self.repo)
        write(os.path.join(self.repo, "crates", "deeptime-core", "src", "lib.rs"), "pub fn f() { 1; }\n")
        self.assertNotEqual(h0, jarstate.source_hash(self.repo))
        h1 = jarstate.source_hash(self.repo)
        write(os.path.join(self.repo, "crates", "deeptime-jni", "src", "new.rs"), "// new file\n")
        self.assertNotEqual(h1, jarstate.source_hash(self.repo))


class Refresh(FakeCheckout):
    def test_installs_when_missing(self):
        built = self.gradle_jar()
        r = jarstate.refresh(built, self.mods)
        self.assertTrue(r["changed"])
        self.assertEqual(r["was"], [])
        self.assertEqual(read(os.path.join(self.mods, "deeptime-1.21.1-0.0.1.jar")), read(built))
        self.assertEqual(sorted(os.listdir(self.mods)), ["create-1.21.1.jar", "deeptime-1.21.1-0.0.1.jar"])   # no .part

    def test_same_bytes_are_left_alone(self):
        built = self.gradle_jar()
        jarstate.refresh(built, self.mods)
        installed = os.path.join(self.mods, "deeptime-1.21.1-0.0.1.jar")
        inode = os.stat(installed).st_ino
        r = jarstate.refresh(built, self.mods)
        self.assertFalse(r["changed"])
        self.assertEqual(inode, os.stat(installed).st_ino)

    def test_same_name_different_bytes_is_replaced_atomically(self):
        installed = self.set_jar(text="jar from before the synthetic preset")
        inode = os.stat(installed).st_ino
        reader = open(installed)                     # a running server holds the old file open
        self.addCleanup(reader.close)
        built = self.gradle_jar()
        r = jarstate.refresh(built, self.mods)
        self.assertTrue(r["changed"])
        self.assertEqual(r["was"][0]["name"], "deeptime-1.21.1-0.0.1.jar")
        self.assertEqual(read(installed), read(built))
        self.assertNotEqual(inode, os.stat(installed).st_ino)                    # renamed into place, not overwritten
        self.assertEqual(reader.read(), "jar from before the synthetic preset")   # the open handle is unharmed

    def test_other_deeptime_jars_go_other_mods_stay(self):
        self.set_jar("deeptime-1.21.1-0.0.0.jar", "an older version")
        self.set_jar("Deeptime-1.21.1-0.0.1-sources.jar", "not a mod jar, not touched")
        built = self.gradle_jar()
        r = jarstate.refresh(built, self.mods)
        self.assertTrue(r["changed"])
        self.assertEqual(sorted(os.listdir(self.mods)),
                         ["Deeptime-1.21.1-0.0.1-sources.jar", "create-1.21.1.jar", "deeptime-1.21.1-0.0.1.jar"])


class BuiltJar(FakeCheckout):
    def test_named_by_mod_version_not_by_sort_order(self):
        libs = os.path.join(self.repo, "neoforge", "build", "libs")
        write(os.path.join(libs, "deeptime-1.21.1-0.0.9.jar"), "a stale jar of a version that no longer exists")
        want = self.gradle_jar()
        self.assertEqual(jarstate.built_jar(self.repo), want)

    def test_missing_is_an_error(self):
        with self.assertRaises(RuntimeError):
            jarstate.built_jar(self.repo)


class RecordAndStatus(FakeCheckout):
    def test_record_merges_and_status_follows_the_sources(self):
        install = read_json(self.install_json)
        ok, why = jarstate.status(self.repo, install)
        self.assertFalse(ok)                          # an install.json from before the record
        self.assertIn("no source hash", why)
        built = self.gradle_jar()
        jarstate.record(self.install_json, jarstate.jar_fields(self.repo, built))
        install = read_json(self.install_json)
        self.assertEqual(install["profile"], "mic")   # other keys stay
        self.assertEqual(install["deeptime_jar_sha256"], jarstate.sha256_file(built))
        self.assertTrue(jarstate.status(self.repo, install)[0])
        write(self.java, "class Gen { int v = 3; }\n")
        ok, why = jarstate.status(self.repo, install)
        self.assertFalse(ok)
        self.assertIn("sources changed", why)

    def test_sync_rev_wins_over_git(self):
        write(os.path.join(self.repo, jarstate.SYNC_REV),
              json.dumps({"describe": "da1827e9-dirty", "commit": "da1827e92589e065b3e4bb0100bb6e8894ae932b"}))
        self.assertEqual(jarstate.source_rev(self.repo)["describe"], "da1827e9-dirty")
        os.remove(os.path.join(self.repo, jarstate.SYNC_REV))
        self.assertEqual(jarstate.source_rev(self.repo), {"describe": "unknown", "commit": "unknown"})   # no .git here


class PreviewRefresh(FakeCheckout):
    """preview.py's refresh_deeptime_jar: build, compare, replace, and say so in the job status."""

    def setUp(self):
        super().setUp()
        self.jobs = os.path.join(self.tmp, "jobs")
        os.environ["DEEPTIME_WORKBENCH_JOBS"] = self.jobs
        self.addCleanup(os.environ.pop, "DEEPTIME_WORKBENCH_JOBS", None)
        quiet = contextlib.redirect_stdout(io.StringIO())   # Job.say prints its progress lines
        quiet.__enter__()
        self.addCleanup(quiet.__exit__, None, None, None)
        self.preview = load_preview()
        self.preview.REPO = self.repo
        self.smoke = os.path.join(self.repo, "smoke", "mic")
        self.n = 0

    def job(self):
        self.n += 1
        j = self.preview.Job(f"t{self.n}", {})
        self.addCleanup(j.log.close)
        return j

    def test_stale_jar_refreshed_then_reused_then_refreshed_by_an_edit_and_a_revert(self):
        self.set_jar(text="jar from 2026-09-28")
        # 1. the first preview after the bug: the set's jar is not what the tree builds
        j = self.job()
        self.preview.refresh_deeptime_jar(j, self.smoke, build=self.gradle_jar)
        self.assertTrue(j.st["jar"]["refreshed"])
        self.assertIn("refreshed the Deep Time jar", j.st["message"])
        self.assertIn("jar_s", j.st["timings"])
        install = read_json(self.install_json)
        self.assertEqual(install["deeptime_jar_sha256"], j.st["jar"]["sha256"])
        self.assertEqual(install["deeptime_src_hash"], jarstate.source_hash(self.repo))
        self.assertTrue(jarstate.status(self.repo, install)[0])
        installed = os.path.join(self.mods, "deeptime-1.21.1-0.0.1.jar")
        first_sha = jarstate.sha256_file(installed)
        inode = os.stat(installed).st_ino
        # 2. nothing changed: reused, not rewritten
        j = self.job()
        self.preview.refresh_deeptime_jar(j, self.smoke, build=self.gradle_jar)
        self.assertFalse(j.st["jar"]["refreshed"])
        self.assertIn("is current", j.st["message"])
        self.assertEqual(inode, os.stat(installed).st_ino)
        # 3. touch (same bytes): still reused. A real edit: refreshed. Reverting it: refreshed back.
        os.utime(self.java, (1, 1))
        j = self.job()
        self.preview.refresh_deeptime_jar(j, self.smoke, build=self.gradle_jar)
        self.assertFalse(j.st["jar"]["refreshed"])
        write(self.java, "class Gen { int v = 2; }\n")
        j = self.job()
        self.preview.refresh_deeptime_jar(j, self.smoke, build=self.gradle_jar)
        self.assertTrue(j.st["jar"]["refreshed"])
        self.assertEqual(j.st["jar"]["was"], [first_sha])
        self.assertNotEqual(jarstate.sha256_file(installed), first_sha)
        write(self.java, "class Gen { int v = 1; }\n")
        j = self.job()
        self.preview.refresh_deeptime_jar(j, self.smoke, build=self.gradle_jar)
        self.assertTrue(j.st["jar"]["refreshed"])
        self.assertEqual(jarstate.sha256_file(installed), first_sha)

    def test_the_status_file_carries_the_jar_record(self):
        self.set_jar()
        j = self.job()
        self.preview.refresh_deeptime_jar(j, self.smoke, build=self.gradle_jar)
        st = read_json(os.path.join(j.dir, "status.json"))
        self.assertTrue(st["jar"]["refreshed"])
        self.assertEqual(st["jar"]["src_hash"], jarstate.source_hash(self.repo))

    def test_a_jar_that_does_not_build_fails_the_job_loudly(self):
        self.set_jar(text="stale")
        j = self.job()

        def broken(job):
            raise RuntimeError("gradlew failed with status 1 (see build.log)")

        with self.assertRaises(RuntimeError):
            self.preview.refresh_deeptime_jar(j, self.smoke, build=broken)
        self.assertEqual(read(os.path.join(self.mods, "deeptime-1.21.1-0.0.1.jar")), "stale")   # nothing swapped


if __name__ == "__main__":
    unittest.main()
