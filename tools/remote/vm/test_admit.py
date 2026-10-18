#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""Tests for tools/remote/vm/admit.py: the estimator, the admission decision, classification, slots,
the state file under concurrent writers, and a measured end-to-end run. Linux only (/proc).

    python3 tools/remote/vm/test_admit.py
"""

import json
import multiprocessing
import os
import subprocess
import sys
import tempfile
import time
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import admit  # noqa: E402


def run_rec(cpu, mem, end, wall=100.0, peak=None):
    return {"cpu_mean": cpu, "rss_peak_gib": mem, "end": end, "wall_s": wall, "cpu_peak": peak or cpu}


CAP = {"free_cpu": 20.0, "mem_avail_gib": 60.0, "cpu_total": 32.0, "mem_total_gib": 125.0}


def waiting(jid, cpu, mem, enq, kind="k", label="agent"):
    return {"id": jid, "kind": kind, "label": label, "enq": enq, "est": {"cpu": cpu, "mem_gib": mem}}


def admitted(jid, cpu, mem, at, measured=None, **kw):
    e = {"id": jid, "kind": "k", "label": "a", "admitted_at": at, "est": {"cpu": cpu, "mem_gib": mem},
         "measured": measured or {}}
    e.update(kw)
    return e


class Seeds(unittest.TestCase):
    def test_sim_threads_and_size(self):
        cpu, mem, peak, _ = admit.seed("sim:earthlike_quick_16k:2t")
        self.assertAlmostEqual(cpu, 1.8)
        self.assertEqual(mem, 3.0)
        self.assertEqual(peak, 2.0)
        cpu, mem, _, _ = admit.seed("sim:default_32k:alltt".replace("tt", "t"))
        self.assertEqual(cpu, 16.0)
        self.assertEqual(mem, 7.0)

    def test_prefix_fallback_takes_the_heaviest_of_the_family(self):
        self.assertEqual(admit.seed("gametest:newset"), max(
            (v for k, v in admit.SEEDS.items() if k.startswith("gametest:")), key=lambda v: (v[0], v[1])))
        self.assertEqual(admit.seed("server:cold-all"), admit.SEEDS["server"])
        self.assertEqual(admit.seed("nothing-known"), admit.SEEDS["shell"])

    def test_compound_is_its_heaviest_part_run_in_sequence(self):
        c = admit.seed("cargo:clippy+cargo:test")
        self.assertEqual(c[0], max(admit.SEEDS["cargo:clippy"][0], admit.SEEDS["cargo:test"][0]))
        self.assertEqual(c[3], admit.SEEDS["cargo:clippy"][3] + admit.SEEDS["cargo:test"][3])

    def test_server_kinds(self):
        for k in ("gametest:mic", "preview:standalone", "smoke:dh", "client:dh", "planet:mic", "server:x",
                  "gametest:standalone+gametest:mic"):
            self.assertTrue(admit.is_server_kind(k), k)
        for k in ("cargo:test", "sim:a:2t", "gradle:build", "shell"):
            self.assertFalse(admit.is_server_kind(k), k)


class Estimator(unittest.TestCase):
    def test_no_history_is_the_seed(self):
        e = admit.estimate("cargo:test", {})
        self.assertEqual((e["cpu"], e["mem_gib"], e["source"]), (admit.SEEDS["cargo:test"][0], admit.SEEDS["cargo:test"][1], "seed"))

    def test_one_run_is_mixed_with_the_seed(self):
        # One run far below the seed: the 90th percentile of {run, seed} with the seed weighing
        # 0.85 of the run keeps the estimate at the seed (cautious until there is evidence).
        e = admit.estimate("cargo:test", {"cargo:test": [run_rec(2.0, 1.0, 100)]})
        self.assertEqual(e["runs"], 1)
        self.assertIn("seed mixed in", e["source"])
        self.assertEqual(e["cpu"], admit.SEEDS["cargo:test"][0])

    def test_many_runs_follow_the_measurements_not_the_seed(self):
        hist = {"cargo:test": [run_rec(3.0 + 0.1 * i, 2.0, 1000 + i) for i in range(10)]}
        e = admit.estimate("cargo:test", hist)
        self.assertLess(e["cpu"], 5.0)
        self.assertGreaterEqual(e["cpu"], 3.0)
        self.assertNotIn("seed", e["source"])

    def test_high_percentile_covers_most_runs(self):
        hist = {"k": [run_rec(c, 4.0, 1000 + i) for i, c in enumerate([2, 2, 2, 2, 2, 2, 2, 2, 2, 9])]}
        # The 9-core run is the newest: weight 1 of a total ~6, enough to set the 90th percentile.
        self.assertEqual(admit.estimate("k", hist)["cpu"], 9)

    def test_old_outliers_decay(self):
        hist = {"k": [run_rec(9.0, 4.0, 1000)] + [run_rec(2.0, 4.0, 2000 + i) for i in range(15)]}
        self.assertEqual(admit.estimate("k", hist)["cpu"], 2.0)

    def test_memory_is_the_runs_peak(self):
        hist = {"k": [run_rec(2.0, m, 1000 + i) for i, m in enumerate([5, 6, 7, 8, 9])]}
        self.assertGreaterEqual(admit.estimate("k", hist)["mem_gib"], 8)


class Decision(unittest.TestCase):
    def test_fits_with_the_25_percent_margin(self):
        # free 20 - reserve 4 = 16 cores; 16 * 1.25 = 20.
        d, info = admit.plan([waiting("a", 20.0, 4.0, 0)], [], CAP, 10)
        self.assertIs(d["a"], True)
        self.assertEqual(info["avail_cpu"], 16.0)
        d, _ = admit.plan([waiting("a", 20.5, 4.0, 0)], [], CAP, 10)
        self.assertIn("needs 20.5 cores", d["a"])

    def test_memory_must_fit_too(self):
        # 60 GiB free, no headroom: the estimate times 1.25 must be under it (48 * 1.25 = 60 is not).
        d, info = admit.plan([waiting("a", 2.0, 47.9, 0)], [], CAP, 10)
        self.assertIs(d["a"], True)
        self.assertEqual(info["avail_mem"], 60.0)
        d, _ = admit.plan([waiting("a", 2.0, 48.0, 0)], [], CAP, 10)
        self.assertIn("GiB", d["a"])

    # Ben, decisions log 2026-10-03: on a 4 GB box at 3.5 GB used, a run predicted at 200 MB is green
    # and one predicted at 0.4 GB is red (with the margin it would use the whole machine).
    SMALL = {"free_cpu": 8.0, "mem_avail_gib": 0.5, "cpu_total": 8.0, "mem_total_gib": 4.0}

    def test_ben_example_small_job_runs_on_a_nearly_full_box(self):
        d, info = admit.plan([waiting("a", 0.5, 0.2, 0)], [], self.SMALL, 10)      # 0.2 * 1.25 = 0.25 < 0.5
        self.assertIs(d["a"], True)
        self.assertEqual(info["avail_mem"], 0.5)                                    # no standing headroom

    def test_ben_example_job_that_would_fill_the_box_waits(self):
        d, _ = admit.plan([waiting("a", 0.5, 0.4, 0)], [], self.SMALL, 10)         # 0.4 * 1.25 = 0.5, not under 0.5
        self.assertIn("needs 0.40 GiB", d["a"])
        self.assertIn("0.50 free", d["a"])

    def test_the_margin_is_on_the_job_not_on_the_free_memory(self):
        # The old rule (estimate <= 1.25 * free) would admit 0.6 GiB into 0.5 free; the new one must not.
        d, _ = admit.plan([waiting("a", 0.5, 0.6, 0)], [], self.SMALL, 10)
        self.assertNotEqual(d["a"], True)
        # And a job just under free / 1.25 runs.
        d, _ = admit.plan([waiting("a", 0.5, 0.399, 0)], [], self.SMALL, 10)
        self.assertIs(d["a"], True)

    def test_memory_reserved_but_not_yet_used_is_not_free(self):
        # A job admitted with a 3 GiB estimate has used 1 GiB so far: 2 GiB are still promised to it.
        # MemAvailable is 2.7 GiB, so free = 2.7 - 2.0 = 0.7 GiB.
        fresh = admitted("j", 1.0, 3.0, at=100, measured={"cpu": 1.0, "cpu_max": 1.0, "rss_gib": 1.0})
        cap = dict(self.SMALL, mem_avail_gib=2.7)
        d, info = admit.plan([waiting("a", 0.5, 0.5, 0)], [fresh], cap, 110)       # 0.5 * 1.25 = 0.625 < 0.7
        self.assertEqual(info["pending_mem"], 2.0)
        self.assertAlmostEqual(info["avail_mem"], 0.7)
        self.assertIs(d["a"], True)
        d, _ = admit.plan([waiting("a", 0.5, 0.6, 0)], [fresh], cap, 110)          # 0.75 is not under 0.7
        self.assertIn("needs 0.60 GiB", d["a"])
        # Without the reservation the same job runs, so the reservation is what held it back.
        d, _ = admit.plan([waiting("a", 0.5, 0.6, 0)], [], cap, 110)
        self.assertIs(d["a"], True)

    def test_jobs_started_in_one_pass_reserve_memory_for_the_next(self):
        q = [waiting("a", 0.5, 0.2, 0), waiting("b", 0.5, 0.2, 1)]
        d, _ = admit.plan(q, [], self.SMALL, 10)                                    # 0.5 - 0.2 = 0.3; 0.25 < 0.3
        self.assertIs(d["a"], True)
        self.assertIs(d["b"], True)
        d, _ = admit.plan(q + [waiting("c", 0.5, 0.2, 2)], [], self.SMALL, 10)      # 0.1 left; 0.25 is not under it
        self.assertNotEqual(d["c"], True)

    def test_running_jobs_are_never_touched(self):
        # plan() only decides starts: an overcommitted box (nothing free) leaves its jobs alone.
        busy = admitted("j", 1.0, 3.0, at=0, measured={"cpu": 1.0, "rss_gib": 3.0})
        cap = dict(self.SMALL, mem_avail_gib=-0.2)
        d, info = admit.plan([waiting("a", 0.5, 0.1, 0)], [busy], cap, 10_000)
        self.assertNotEqual(d["a"], True)
        self.assertEqual(set(d), {"a"})

    def test_admitted_jobs_reserve_what_they_have_not_used_yet(self):
        fresh = admitted("j", 10.0, 20.0, at=100, measured={"cpu": 2.0, "cpu_max": 2.0, "rss_gib": 5.0})
        # 20 free - 4 headroom - 8 reserved = 8 cores; 8 * 1.25 = 10.
        d, info = admit.plan([waiting("a", 10.0, 4.0, 0)], [fresh], CAP, 110)
        self.assertEqual(info["pending_cpu"], 8.0)
        self.assertEqual(info["pending_mem"], 15.0)
        self.assertIs(d["a"], True)
        d, _ = admit.plan([waiting("a", 10.5, 4.0, 0)], [fresh], CAP, 110)
        self.assertIn("needs 10.5 cores", d["a"])

    def test_cpu_reservation_ends_after_the_ramp_or_once_reached(self):
        j = admitted("j", 10.0, 0.0, at=0, measured={"cpu": 1.0, "cpu_max": 1.0})
        self.assertEqual(admit.pending(j, 10)[0], 9.0)
        self.assertEqual(admit.pending(j, admit.RAMP_S + 1)[0], 0.0)
        reached = admitted("j", 10.0, 0.0, at=0, measured={"cpu": 3.0, "cpu_max": 9.0})
        self.assertEqual(admit.pending(reached, 10)[0], 0.0)

    def test_memory_reservation_lasts_the_whole_job(self):
        j = admitted("j", 1.0, 12.0, at=0, measured={"rss_gib": 4.0})
        self.assertEqual(admit.pending(j, 10_000)[1], 8.0)

    def test_a_slot_guest_counts_as_its_host(self):
        host = admitted("h", 4.0, 10.0, at=0, measured={"rss_gib": 0.5})
        guest = admitted("g", 4.0, 10.0, at=0, measured={"rss_gib": 7.0}, inherited="h")
        ps = admit.pendings([host, guest], 10)
        self.assertEqual(ps[1], (0.0, 0.0))
        self.assertAlmostEqual(ps[0][1], 2.5)

    def test_in_order_with_backfill(self):
        q = [waiting("big", 30.0, 4.0, 0), waiting("small", 2.0, 2.0, 1), waiting("small2", 2.0, 2.0, 2)]
        d, _ = admit.plan(q, [], CAP, 60)
        self.assertNotEqual(d["big"], True)
        self.assertIs(d["small"], True)
        self.assertIs(d["small2"], True)

    def test_earlier_jobs_take_room_first(self):
        q = [waiting("b", 12.0, 2.0, 5), waiting("a", 12.0, 2.0, 1)]
        d, _ = admit.plan(q, [], CAP, 60)
        self.assertIs(d["a"], True)
        self.assertNotEqual(d["b"], True)

    def test_a_starving_job_stops_everything_behind_it(self):
        q = [waiting("big", 30.0, 4.0, 0, kind="gametest:mic"), waiting("small", 2.0, 2.0, 1)]
        now = admit.STARVE_MIN * 60 + 1
        d, _ = admit.plan(q, [], CAP, now)
        self.assertNotEqual(d["big"], True)
        self.assertIn("behind gametest:mic", d["small"])

    def test_quick_kinds_start_at_once_even_on_a_full_box(self):
        cap = dict(CAP, free_cpu=-6.0)
        hist = {"shell:python3": [run_rec(0.3, 0.05, 1000 + i, wall=2.0) for i in range(2)]}
        est = admit.estimate("shell:python3", hist)
        self.assertTrue(admit.quick(est))
        d, _ = admit.plan([dict(waiting("q", 0, 0, 0), est=est)], [], cap, 10)
        self.assertIs(d["q"], True)
        self.assertFalse(admit.quick(admit.estimate("shell:python3", {})))          # unknown: not quick
        slow = {"k": [run_rec(0.3, 0.05, 1000 + i, wall=600.0) for i in range(3)]}
        self.assertFalse(admit.quick(admit.estimate("k", slow)))

    def test_estimates_are_capped_at_what_the_box_has(self):
        cap = dict(CAP, free_cpu=32.0, mem_avail_gib=120.0)
        d, _ = admit.plan([waiting("huge", 100.0, 500.0, 0)], [], cap, 10)
        self.assertIs(d["huge"], True)

    def test_overloaded_box_admits_nothing(self):
        cap = dict(CAP, free_cpu=-6.0)
        d, _ = admit.plan([waiting("a", 0.5, 1.0, 0)], [], cap, 10)
        self.assertNotEqual(d["a"], True)

    def test_legacy_server_lock_is_an_admitted_server_job(self):
        with tempfile.TemporaryDirectory() as t:
            lock = os.path.join(t, "server.lock")
            os.mkdir(lock)
            open(os.path.join(lock, "owner"), "w").write("agent-old\n")
            old = admit.LEGACY_LOCK
            admit.LEGACY_LOCK = lock
            try:
                j = admit.legacy_job(time.time())
                self.assertEqual((j["kind"], j["label"]), ("server:legacy", "agent-old"))
                cpu, mem = admit.pending(j, time.time())
                self.assertGreater(cpu, 0)
                self.assertGreater(mem, 0)
                os.utime(lock, (time.time() - admit.LEGACY_STALE_S - 10,) * 2)
                self.assertIsNone(admit.legacy_job(time.time()))
            finally:
                admit.LEGACY_LOCK = old


class Classify(unittest.TestCase):
    CASES = {
        "cargo test --workspace": "cargo:test",
        "cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace":
            "cargo:other+cargo:clippy+cargo:test",
        "cd neoforge && ./gradlew build test && python3 ../tools/ci/junit-summary.py build/test-results/test": "gradle:build",
        "./gametest.sh -PwithMic": "gametest:mic",
        "./gametest.sh": "gametest:standalone",
        "tools/smoke/install.sh mic && ./gametest.sh -PwithMic": "gametest:mic",
        "./gametest.sh && ./gametest.sh -PwithMic": "gametest:standalone+gametest:mic",
        "python3 tools/smoke/run.py --profile mic --chunky 8": "smoke:mic",
        "python3 tools/workbench/preview.py run --profile standalone --synth wrap-test --box 0,0,64,64": "preview:standalone",
        "target/release/deeptime run --preset earthlike_quick_16k --threads 2 --out /x": "sim:earthlike_quick_16k:2t",
        "cargo run --release -q -p deeptime-cli -- run --preset crates/x/planet_world_4k.json --threads 4 --out y":
            "sim:planet_world_4k:4t",
        "cargo build --release -p deeptime-cli": "cargo:build",
        "true": "light",
        "git status && cat STATUS.md | head": "light",
        "uptime; free -g": "light",
        "grep -E \"a|b\" log | cut -c1-260; awk '{print $1}' x | sed s/a/b/": "light",
        "grep -E \"GAME TESTS|required tests\" neoforge/build/gametest.log | head; grep -c x y | wc -l": "light",
        "tools/remote/detach.sh start run1 target/release/deeptime run --preset x": "light",
        "setsid nohup bash tools/review/sweep.sh > log 2>&1 &": "light",
        "bash tools/review/some-analysis.sh": "shell:some-analysis.sh",
        "python3 -c 'import json; print(1)'": "shell:python3",
        "cd tools && FOO=1 ./thing --x": "shell:thing",
        "tools/client/dh-client-check.sh dh": "client:dh",
    }

    def test_cases(self):
        for cmd, kind in self.CASES.items():
            self.assertEqual(admit.classify(cmd), kind, cmd)


class Slots(unittest.TestCase):
    def test_skips_used_slots_and_taken_ports(self):
        st = {"jobs": {"a": {"slot": 0}, "b": {"slot": 2}}}
        taken = {admit.PORT_BASE + 1 * admit.PORT_STRIDE + 1}   # slot 1's RCON port is in use
        n = admit.allocate_slot(st, check=lambda p, udp=False: p not in taken)
        self.assertEqual(n, 3)

    def test_env(self):
        e = admit.slot_env(3, "/x")
        base = admit.PORT_BASE + 30
        self.assertEqual((e["DEEPTIME_PORT_GAME"], e["DEEPTIME_PORT_RCON"], e["DEEPTIME_PORT_VOICE"]),
                         (str(base), str(base + 1), str(base + 3)))
        self.assertEqual(e["DEEPTIME_DISPLAY"], f":{admit.DISPLAY_BASE + 3}")
        for p in range(base, base + 10):
            self.assertNotIn(p, admit.RESERVED_PORTS)

    def test_the_pool_avoids_the_play_server_and_workbench(self):
        pool = {admit.PORT_BASE + n * admit.PORT_STRIDE + i for n in range(admit.SLOTS) for i in range(10)}
        self.assertFalse(pool & {25765, 24464, 8642, 25665, 25675, 25666, 25676, 24454})


def _bump(d, n):
    for _ in range(n):
        with admit.Store(d) as s:
            s.state.setdefault("count", 0)
            s.state["count"] += 1


class StateFile(unittest.TestCase):
    def test_concurrent_writers_lose_nothing(self):
        with tempfile.TemporaryDirectory() as d:
            ps = [multiprocessing.Process(target=_bump, args=(d, 50)) for _ in range(6)]
            for p in ps:
                p.start()
            for p in ps:
                p.join()
            with admit.Store(d) as s:
                self.assertEqual(s.state["count"], 300)

    def test_dead_entries_are_dropped(self):
        p = subprocess.Popen([sys.executable, "-c", "pass"])
        p.wait()
        st = {"jobs": {"x": {"id": "x", "kind": "k", "pid": p.pid, "start": 1}},
              "queue": {"me": {"id": "me", "kind": "k", "pid": os.getpid(), "start": admit.pid_start(os.getpid())}}}
        gone = admit.cleanup(st)
        self.assertEqual([e["id"] for _, e in gone], ["x"])
        self.assertIn("me", st["queue"])


BURN = "import time\nt=time.time()\nwhile time.time()-t<{s}: pass\n"


class EndToEnd(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.saved = (admit.MIN_RECORD_S, admit.SAMPLE_S, admit.POLL_S, admit.LEGACY_LOCK,
                      admit.RESERVE_CORES)
        admit.MIN_RECORD_S, admit.SAMPLE_S, admit.POLL_S = 1.0, 0.5, 0.2
        admit.LEGACY_LOCK = os.path.join(self.tmp.name, "no-lock")
        admit.RESERVE_CORES = 0.0
        # A roomy box whatever the test machine is doing (its real load would make these wait).
        self.saved_capacity = admit.capacity
        admit.capacity = lambda samples, now: {"cpu_total": 32.0, "load1": 0.0, "busy30": 0.0, "free_cpu": 32.0,
                                               "mem_total_gib": 125.0, "mem_avail_gib": 100.0}

    def tearDown(self):
        (admit.MIN_RECORD_S, admit.SAMPLE_S, admit.POLL_S, admit.LEGACY_LOCK,
         admit.RESERVE_CORES) = self.saved
        admit.capacity = self.saved_capacity
        self.tmp.cleanup()

    def test_a_run_is_measured_and_recorded(self):
        store = admit.Store(self.tmp.name)
        rc = admit.run("test:burn", [sys.executable, "-c", BURN.format(s=3)], cpu=0.1, mem=0.1, store=store, quiet=True)
        self.assertEqual(rc, 0)
        with store as s:
            runs = s.history["test:burn"]
            self.assertEqual(s.state["jobs"], {})
        r = runs[-1]
        self.assertGreater(r["cpu_mean"], 0.6)    # one busy core for 3 s of ~3 s
        self.assertLess(r["cpu_mean"], 1.5)
        self.assertGreater(r["rss_peak_gib"], 0.0)

    def test_children_and_exit_status(self):
        store = admit.Store(self.tmp.name)
        code = f"import subprocess,sys\nps=[subprocess.Popen([sys.executable,'-c',{BURN.format(s=2)!r}]) for _ in range(2)]\n" \
               "[p.wait() for p in ps]\nsys.exit(3)\n"
        rc = admit.run("test:two", [sys.executable, "-c", code], cpu=0.1, mem=0.1, store=store, quiet=True)
        self.assertEqual(rc, 3)
        with store as s:
            r = s.history["test:two"][-1]
        self.assertGreater(r["cpu_peak"], 1.4)    # two children busy at once

    def test_a_reaped_child_is_not_counted_twice(self):
        # A shell runs one busy child for 3 s and waits for it: when the child ends, the shell's
        # cutime jumps by the child's whole 3 s, already counted while the child ran.
        store = admit.Store(self.tmp.name)
        code = f"import subprocess,sys,time\nsubprocess.run([sys.executable,'-c',{BURN.format(s=3)!r}])\ntime.sleep(1.5)\n"
        rc = admit.run("test:reap", [sys.executable, "-c", code], cpu=0.1, mem=0.1, store=store, quiet=True)
        self.assertEqual(rc, 0)
        with store as s:
            r = s.history["test:reap"][-1]
        self.assertLess(r["cpu_peak"], 1.6)
        self.assertLess(r["cpu_mean"] * r["wall_s"], 4.0)    # about 3 core-seconds, not 6

    def test_a_server_job_gets_a_slot_and_a_nested_launcher_takes_it_over(self):
        store = admit.Store(self.tmp.name)
        inner = (f"import json,os,sys\nsys.path.insert(0,{HERE!r})\nimport admit\n"
                 f"admit.MIN_RECORD_S=1.0\nadmit.SAMPLE_S=0.5\n"
                 f"s=admit.Store({self.tmp.name!r})\n"
                 "code='import os,json,time; t=time.time()\\nwhile time.time()-t<2: pass\\n"
                 "print(json.dumps({k:v for k,v in os.environ.items() if k.startswith(\"DEEPTIME_\")}))'\n"
                 "rc=admit.run('gametest:fake',[sys.executable,'-c',code],store=s,quiet=True)\nsys.exit(rc)\n")
        out = os.path.join(self.tmp.name, "out.json")
        outer = f"import subprocess,sys\nr=subprocess.run([sys.executable,'-c',{inner!r}],capture_output=True,text=True)\n" \
                f"open({out!r},'w').write(r.stdout)\nsys.exit(r.returncode)\n"
        rc = admit.run("gametest:outer", [sys.executable, "-c", outer], cpu=0.1, mem=0.1, store=store, quiet=True)
        self.assertEqual(rc, 0)
        env = json.loads(open(out).read())
        self.assertEqual(env["DEEPTIME_ADMIT_KIND"], "gametest:fake")
        self.assertEqual(env["DEEPTIME_SERVER_SLOT"], "0")
        self.assertTrue(env["DEEPTIME_SERVER_DIR"].startswith(os.path.join(self.tmp.name, "servers")))
        self.assertFalse(os.path.exists(env["DEEPTIME_SERVER_DIR"]))   # removed at the end
        with store as s:
            self.assertIn("gametest:fake", s.history)       # the launcher's own record
            self.assertNotIn("gametest:outer", s.history)   # the wrapper's time went to the launcher
            r = s.history["gametest:fake"][-1]
        # It runs inside its host's admit.py: measured on its own processes, not as nothing
        # (2026-10-18: removing the host's whole tree removed the guest too).
        self.assertGreater(r["cpu_mean"], 0.6)
        self.assertGreater(r["rss_peak_gib"], 0.0)

    def test_a_job_inside_another_job_is_measured_on_its_own_processes(self):
        # 2026-10-18: a script run through admission that ran sims through admission (env -u
        # DEEPTIME_ADMIT_JOB, so each is its own job) measured every inner job at 0.0 cores and 0.0 GiB:
        # the outer job's admit.py is the inner's ancestor, and "take off the other jobs' trees" took
        # off the inner job's own tree. The outer job must not count the inner's work either.
        store = admit.Store(self.tmp.name)
        inner = (f"import sys\nsys.path.insert(0,{HERE!r})\nimport admit\n"
                 f"admit.MIN_RECORD_S=1.0\nadmit.SAMPLE_S=0.5\nadmit.RESERVE_CORES=0.0\n"
                 f"admit.LEGACY_LOCK={admit.LEGACY_LOCK!r}\n"
                 "admit.capacity=lambda samples, now: {'cpu_total': 32.0, 'load1': 0.0, 'busy30': 0.0, "
                 "'free_cpu': 32.0, 'mem_total_gib': 125.0, 'mem_avail_gib': 100.0}\n"
                 f"s=admit.Store({self.tmp.name!r})\n"
                 f"sys.exit(admit.run('test:inner',[sys.executable,'-c',{BURN.format(s=3)!r}],cpu=0.1,mem=0.1,store=s,quiet=True))\n")
        outer = ("import os,subprocess,sys,time\n"
                 "env={k: v for k, v in os.environ.items() if k not in ('DEEPTIME_ADMIT_JOB', 'DEEPTIME_ADMIT_PID')}\n"
                 f"r=subprocess.run([sys.executable,'-c',{inner!r}],env=env)\n"
                 "time.sleep(1.5)\nsys.exit(r.returncode)\n")
        rc = admit.run("test:outer", [sys.executable, "-c", outer], cpu=0.1, mem=0.1, store=store, quiet=True)
        self.assertEqual(rc, 0)
        with store as s:
            ri = s.history["test:inner"][-1]
            ro = s.history["test:outer"][-1]
        # The inner job: its one busy core for 3 s, and its interpreter's memory.
        self.assertGreater(ri["cpu_mean"], 0.6)
        self.assertLess(ri["cpu_mean"], 1.5)
        self.assertGreater(ri["rss_peak_gib"], 0.0)
        # The outer job: waiting for it. The inner's 3 core-seconds, which reach the outer's shell as
        # cutime when the inner is reaped, are not the outer's (they were double counted before).
        self.assertLess(ro["cpu_mean"] * ro["wall_s"], 1.5)
        # (What the inner did after the outer's last look at it, one 0.5 s interval of its one core at
        # most, still reaches the outer as a blip; the double count was 3 core-seconds in one sample.)
        self.assertLess(ro["cpu_peak"], 1.5)
        self.assertGreater(ro["rss_peak_gib"], 0.0)

    def test_attribution_skips_outer_jobs_and_takes_off_inner_ones(self):
        outer = subprocess.Popen(["sh", "-c", "sleep 30 & wait"])
        try:
            time.sleep(0.5)
            procs = admit.scan()
            kids = admit.children_map(procs)[outer.pid]
            self.assertEqual(len(kids), 1)
            child = kids[0]
            outer_root = (outer.pid, procs[outer.pid].start)
            # Seen from the inner job (root = the child): the outer's tree contains it, so nothing is taken off.
            inner = admit.Sampler("inner", child)
            self.assertIn(child, inner.attributed(procs, [outer_root]))
            # Seen from the outer job: the inner job's tree is taken off, the outer's own process stays.
            top = admit.Sampler("outer", outer.pid)
            mine = top.attributed(procs, [(child, procs[child].start)])
            self.assertIn(outer.pid, mine)
            self.assertNotIn(child, mine)
            # A pid that belongs to another process now (wrong start time) takes nothing off.
            self.assertIn(child, admit.Sampler("outer", outer.pid).attributed(procs, [(child, procs[child].start + 1)]))
        finally:
            outer.kill()
            outer.wait()
            subprocess.run(["pkill", "-f", "^sleep 30$"], check=False)

    def test_a_measurement_that_cannot_be_right_is_not_recorded_and_the_log_says_so(self):
        store = admit.Store(self.tmp.name)
        said = []
        job = {"id": "j1", "kind": "test:zero", "label": "x", "est": {"cpu": 2.0, "mem_gib": 2.0}}

        def finish(kind, **summ):
            base = {"wall_s": 300.0, "cpu_mean": 0.0, "cpu_p90": 0.0, "cpu_peak": 0.0, "rss_mean_gib": 0.0,
                    "rss_peak_gib": 0.0, "samples": 60}
            base.update(summ)
            with store as s:
                s.state["jobs"]["j1"] = dict(job, kind=kind)
            admit._finish(store, "j1", dict(job, kind=kind), base, 0, said.append)
            with store as s:
                return s.history.get(kind, [])

        # Five minutes at 0.0 cores and 0.0 GiB: not recorded; stderr and the log say why.
        self.assertEqual(finish("test:zero"), [])
        self.assertTrue(any("cannot be right" in m and "not recorded" in m for m in said), said)
        self.assertIn("cannot be right", open(store.log_path).read())
        # A bare interpreter's first-sample memory (0.02 GiB) is still no memory.
        self.assertEqual(finish("test:zero2", rss_peak_gib=0.02), [])
        # Real use in either dimension is recorded; so is a short run (under the minute).
        self.assertEqual(len(finish("test:cpu", cpu_mean=0.5)), 1)
        self.assertEqual(len(finish("test:mem", rss_peak_gib=2.0)), 1)
        self.assertEqual(len(finish("test:short", wall_s=30.0)), 1)
        # A quick shell kind still learns that it is quick (zero cores, zero memory, under a minute).
        self.assertEqual(len(finish("shell:quick.sh", wall_s=5.0, samples=1)), 1)
        self.assertTrue(admit.implausible_measure({"wall_s": 61.0, "cpu_mean": 0.0, "rss_peak_gib": 0.0}))
        self.assertFalse(admit.implausible_measure({"wall_s": 59.0, "cpu_mean": 0.0, "rss_peak_gib": 0.0}))

    def test_an_idle_run_over_the_line_is_not_recorded_end_to_end(self):
        saved = admit.IMPLAUSIBLE_S
        admit.IMPLAUSIBLE_S = 1.0
        try:
            store = admit.Store(self.tmp.name)
            rc = admit.run("test:idle", [sys.executable, "-c", "import time; time.sleep(2.5)"],
                           cpu=0.1, mem=0.1, store=store, quiet=True)
        finally:
            admit.IMPLAUSIBLE_S = saved
        self.assertEqual(rc, 0)
        with store as s:
            self.assertNotIn("test:idle", s.history)
        self.assertIn("cannot be right", open(store.log_path).read())

    def test_a_job_that_does_not_fit_waits_says_why_and_leaves_the_queue_when_stopped(self):
        # Headroom larger than the box: nothing fits.
        env = dict(os.environ, DEEPTIME_ADMIT_RESERVE_CORES="1000", DEEPTIME_ADMIT_POLL_S="0.2",
                   DEEPTIME_ADMIT_LEGACY_LOCK=admit.LEGACY_LOCK)
        code = (f"import sys\nsys.path.insert(0,{HERE!r})\nimport admit\n"
                f"sys.exit(admit.run('test:wait',['true'],store=admit.Store({self.tmp.name!r})))")
        p = subprocess.Popen([sys.executable, "-c", code], env=env, stderr=subprocess.PIPE, text=True)
        time.sleep(2.0)
        with admit.Store(self.tmp.name) as s:
            self.assertEqual([e["kind"] for e in s.state["queue"].values()], ["test:wait"])
        p.terminate()
        err = p.communicate(timeout=10)[1]
        self.assertIn("test:wait waiting", err)
        self.assertIn("needs", err)
        with admit.Store(self.tmp.name) as s:
            self.assertEqual(s.state["queue"], {})


class StoreSharing(unittest.TestCase):
    """One Store shared by a job's main thread, its sampler thread and its signal handlers (the
    2026-10-04 self-deadlock: a second entry overwrote the first's fd and leaked its flock)."""

    def test_two_threads_on_one_store_never_deadlock(self):
        import threading
        with tempfile.TemporaryDirectory() as d:
            store = admit.Store(d)

            def bump(n):
                for _ in range(n):
                    with store as s:
                        s.state.setdefault("n", 0)
                        s.state["n"] += 1
            ts = [threading.Thread(target=bump, args=(200,)) for _ in range(4)]
            for t in ts:
                t.start()
            for t in ts:
                t.join(timeout=30)
            self.assertFalse(any(t.is_alive() for t in ts), "a thread is stuck on the state lock")
            with admit.Store(d) as s:
                self.assertEqual(s.state["n"], 800)

    def test_reentry_on_one_thread_keeps_one_lock(self):
        with tempfile.TemporaryDirectory() as d:
            store = admit.Store(d)
            with store as s:
                s.state["a"] = 1
                with store as s2:  # a signal handler entering while the main thread holds it
                    s2.state["b"] = 2
            with admit.Store(d) as s:
                self.assertEqual((s.state["a"], s.state["b"]), (1, 2))


if __name__ == "__main__":
    unittest.main(verbosity=2)
