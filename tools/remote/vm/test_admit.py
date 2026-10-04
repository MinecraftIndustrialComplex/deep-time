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
        # 60 - 15 = 45 GiB; 45 * 1.25 = 56.25.
        d, _ = admit.plan([waiting("a", 2.0, 56.0, 0)], [], CAP, 10)
        self.assertIs(d["a"], True)
        d, _ = admit.plan([waiting("a", 2.0, 57.0, 0)], [], CAP, 10)
        self.assertIn("GiB", d["a"])

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
                      admit.RESERVE_CORES, admit.RESERVE_MEM_GIB)
        admit.MIN_RECORD_S, admit.SAMPLE_S, admit.POLL_S = 1.0, 0.5, 0.2
        admit.LEGACY_LOCK = os.path.join(self.tmp.name, "no-lock")
        admit.RESERVE_CORES, admit.RESERVE_MEM_GIB = 0.0, 0.0
        # A roomy box whatever the test machine is doing (its real load would make these wait).
        self.saved_capacity = admit.capacity
        admit.capacity = lambda samples, now: {"cpu_total": 32.0, "load1": 0.0, "busy30": 0.0, "free_cpu": 32.0,
                                               "mem_total_gib": 125.0, "mem_avail_gib": 100.0}

    def tearDown(self):
        (admit.MIN_RECORD_S, admit.SAMPLE_S, admit.POLL_S, admit.LEGACY_LOCK,
         admit.RESERVE_CORES, admit.RESERVE_MEM_GIB) = self.saved
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


if __name__ == "__main__":
    unittest.main(verbosity=2)
