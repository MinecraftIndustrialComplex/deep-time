#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The Phase 1 exit (plan §10.3, P1-19): Phase 0's suite again plus items 1 to 8 of the "Phase 1 exit test",
one PASS/FAIL line each with its evidence paths, and the draft of the private page "Deep Time: Phase 1 (MVP)".

    tools/review/phase1.sh [--only <step>[,<step>...]] [--skip-phase0] [--publish] [--list]

Steps (ids), in the order they run; each item of the plan is one or more rows:

  phase0          Phase 0's suite re-run (tools/review/phase0.py --no-publish): 11 checks
  build           (no row) cargo build --release of deeptime and deeptime-inspect on the VM
  bench           item 1: the shipped builds (gate-bins.sh), x86-64-v3 and baseline side by side under bench-quiet.py, each on 8 physical cores: v3 quick 16k <= 300 s, full 16k <= 900 s, peak memory <= the estimate; the baseline reported
  smoke-mic       item 2: MIC smoke from an empty world with earthlike_quick_16k: first start, boot, every mixin, probes
  geology         items 3 and 4: porphyry (the essentials-reach clause is gone with the essentials-based spawn), `probe site`, copper ore, gossan, a placer whose
                  why-chain ends in an eroded porphyry (Ben, 2026-10-05),
                  BlueMap; COE and CDG over sites and over nothing  (tools/review/phase1_vm.py: pick, serve, geology, compat)
  gametest-mic    item 5: `climateConsistent` (MIC)
  gametest-tb     item 5: `regionsFollowProvinces` (TerraBlender)
  viewer          item 6: a real run's viewer bundle: timelapse, event ticker, layers, ore provinces, three "Why here?" chains
  scale           item 7: BlueMap renders at 1:100, 1:300 and Earth proportions (tools/review/phase1_scale.py)
  page            item 8: the page drafted from the rows (published only with --publish, and only when everything passed)

Everything heavy runs on the test VM through tools/remote/mac.sh (each command admitted by the VM's admission
tool, shared with other agents; a wait is the system working). Each step's output is kept in
review/out/phase1-src/logs/<step>.log, its rows in review/out/phase1-src/rows.json (so --only re-runs one step
and rebuilds the page from the rest), the page in review/out/phase1-src/index.html, the exit table in
review/out/phase1-src/exit-table.md. Evidence files (pictures, JSON, reports) land in review/out/phase1/.

A row is PASS, FAIL (the check ran and did not hold, or its precondition, such as a porphyry site or a
copper essential, does not exist yet), ERROR (the step could not run) or PENDING (the page: drafted, not
published). Standard library only.
"""

import argparse
import datetime
import html
import json
import os
import re
import shutil
import subprocess
import sys
import threading
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(REPO, "review", "out", "phase1-src")
LOGS = os.path.join(SRC, "logs")
EV = os.path.join(REPO, "review", "out", "phase1")
MAC = os.path.join(REPO, "tools", "remote", "mac.sh")
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# The gates of plan §10.3 item 1 (seconds at 8 threads) and what each preset is run with.
QUICK_GATE_S, FULL_GATE_S = 300.0, 900.0
QUICK_MYR, FULL_MYR = 500, 1000
START = "scattered_continents"

NOISE = re.compile(r"^(copying|building|fetching|these \d|  /nix|warning: Git|trace:|evaluation warning|mac\.sh: waiting|admit: .* waiting)")


class Row:
    """One line of the exit table; `kids` are the sub-checks it summarises."""

    def __init__(self, sid, item, title, status="ERROR", evidence=None, kids=None, secs=0.0):
        self.sid, self.item, self.title, self.status = sid, item, title, status
        self.evidence, self.kids, self.secs = evidence or [], kids or [], secs

    def to_json(self):
        return self.__dict__

    @staticmethod
    def from_json(d):
        r = Row(d["sid"], d["item"], d["title"], d["status"], d["evidence"], d["kids"], d["secs"])
        return r


KINDS = {"build": "cargo:build", "gatebins": "cargo:build", "bench": "sim:earthlike_quick_16k:8t"}


def vm(sid, cmd, flags=(), pulls=(), kind=None):
    """Runs `cmd` on the VM detached (tools/review/phase1_detach.py: a dropped ssh session does not lose the job),
    admitted by the VM's admission tool; returns (output without noise, exit code, seconds). Kept in logs/<sid>.log."""
    import phase1_detach
    os.makedirs(LOGS, exist_ok=True)
    out, rc, secs = phase1_detach.run(sid, cmd, flags=flags, pulls=pulls, kind=kind or KINDS.get(sid, "auto"))
    out = "\n".join(l for l in out.splitlines() if not NOISE.match(l))
    with open(os.path.join(LOGS, f"{sid}.log"), "a") as f:
        f.write(f"$ {cmd}\n{out}\n[exit {rc}, {secs:.0f} s]\n\n")
    return out, rc, secs


def checks(out):
    """The `@check PASS|FAIL title :: evidence` lines of phase1_vm.py and the viewer check, as kids."""
    return [(m.group(1) == "PASS", m.group(2), m.group(3)) for m in re.finditer(r"^@check (PASS|FAIL) (.*?) :: (.*)$", out, re.M)]


def rows_from_checks(sid, item, title, out, need=None, secs=0.0):
    kids = checks(out)
    if not kids:
        return Row(sid, item, title, "ERROR", ["no @check lines: " + (out.strip().splitlines() or ["(no output)"])[-1][:200]], secs=secs)
    ok = all(k[0] for k in kids) and (need is None or len(kids) >= need)
    return Row(sid, item, title, "PASS" if ok else "FAIL", [], [("PASS" if k[0] else "FAIL", k[1], [k[2]]) for k in kids], secs)


def kid(ok, title, *evidence):
    return ("PASS" if ok else "FAIL", title, list(evidence))


def status_of(kids, need=1):
    return "PASS" if len(kids) >= need and all(k[0] == "PASS" for k in kids) else "FAIL"


# ---------------------------------------------------------------------------------------------
# steps
# ---------------------------------------------------------------------------------------------
def step_phase0(a):
    t0 = time.time()
    p0 = os.path.join(REPO, "tools", "review", "phase0.py")
    out_path = os.path.join(LOGS, "phase0.log")
    os.makedirs(LOGS, exist_ok=True)
    # The coordinator runs GameTests per merge batch, not on branches: --skip-gametest re-runs Phase 0's other steps and
    # leaves the four GameTest-based rows as they were (marked PENDING below, with the result they last had).
    gt_steps = ("gametest", "gametest-mic", "gametest-tb", "planet")
    p0_args = ["--no-publish"] + (["--only", "rust,junit,inspect,smoke-standalone,smoke-mic-32k,smoke-mic-16k"] if a.skip_gametest else [])
    rows_path = os.path.join(REPO, "review", "out", "phase0-src", "rows.json")
    before = os.path.getmtime(rows_path) if os.path.exists(rows_path) else None
    if not a.skip_phase0:
        with open(out_path, "w") as f:
            subprocess.run([sys.executable, p0] + p0_args, cwd=REPO, stdout=f, stderr=subprocess.STDOUT, env={**os.environ, "PHASE0_DETACH": "1"})
        # phase0.py writes rows.json only when it finishes: a crash leaves the previous run's rows, which must not be read as this run's.
        if os.path.exists(rows_path) and os.path.getmtime(rows_path) == before:
            return [Row("phase0", 0, "Phase 0's suite re-run", "ERROR", ["phase0.py did not finish (rows.json is the previous run's): see review/out/phase1-src/logs/phase0.log"])]
    if not os.path.exists(rows_path):
        return [Row("phase0", 0, "Phase 0's suite re-run", "ERROR", ["no review/out/phase0-src/rows.json: phase0.py did not finish (see logs/phase0.log)"])]
    rows = json.load(open(rows_path))
    kids = []
    for r in rows:
        if a.skip_gametest and r[0] in gt_steps:
            kids.append(("PENDING", r[1], [f"not re-run on this branch (the coordinator runs GameTests per merge batch); the last run, on master 6489d4a5: {'PASS' if r[2] else 'FAIL'}"]))
        else:
            kids.append(("PASS" if r[2] else "FAIL", r[1], r[3] + [f"log: review/out/phase0-src/logs/{r[0]}.log"]))
    note = " (reused the last run: --skip-phase0)" if a.skip_phase0 else ""
    npass, npend = sum(1 for k in kids if k[0] == "PASS"), sum(1 for k in kids if k[0] == "PENDING")
    if npend and not any(k[0] == "FAIL" for k in kids):
        status = "PENDING"   # everything that ran passed; the rest waits for the coordinator's GameTest batch
    else:
        status = status_of(kids)
    pend = f", {npend} GameTest checks pending the coordinator's batch" if npend else ""
    return [Row("phase0", 0, f"Phase 0's suite re-run: {npass} of {len(kids)} checks pass{pend}{note}", status, [], kids, time.time() - t0)]


def step_build(a):
    out, rc, secs = vm("build", "cargo build --release -q -p deeptime-cli -p deeptime-inspect")
    if rc:
        print("build failed:\n" + out[-800:])
        a.build_failed = True
    return []


def bench_json(path):
    """(seconds at 8 threads, peak bytes at 8 threads, estimate bytes or None, stages) from a bench.json."""
    try:
        d = json.load(open(path))
    except (OSError, ValueError):
        return None
    run = next((r for r in d.get("runs", []) if r.get("threads") == 8), None)
    if run is None:
        return None
    secs = run.get("total_seconds", (run.get("total_millis") or 0) / 1000.0)
    peak = run.get("peak_native_bytes", run.get("peak_bytes"))
    est = None
    for src in (d, run):
        for k in ("estimate_peak_bytes", "estimated_peak_bytes", "estimate_bytes"):
            if src.get(k):
                est = src[k]
        e = src.get("estimate")
        if isinstance(e, dict) and (e.get("shown_peak_bytes") or e.get("peak_bytes")):
            est = e.get("shown_peak_bytes") or e["peak_bytes"]   # what the editor shows the player: the stages' peak plus the server overhead
    return secs, peak, est, run.get("stages", []), d


def stage_lines(stages, n=6):
    top = sorted(stages, key=lambda s: -s.get("millis", 0))[:n]
    return ", ".join(f"{s['name']} {s.get('millis', 0) / 1000:.0f} s" for s in top)


# The gate's procedure (docs/design/p1-17-benchmark-gate.md, "Running the gate"; decisions log 2026-10-18 and 2026-10-19): the
# shipped builds (tools/natives/gate-bins.sh: the `ship` profile for x86-64-v3 and for the baseline), side by side under
# tools/review/bench-quiet.py, each held to 8 separate physical cores, v3 on set A and the baseline on set B.
GATE_BINS = "target/gate-bins"
GATE_SETS = (("v3", "deeptime-x86-64-v3", "0,16,2,18,4,20,6,22"), ("baseline", "deeptime-baseline", "8,24,10,26,12,28,14,30"))


def step_bench(a):
    kids = []
    t0 = time.time()
    out, rc, _ = vm("gatebins", f"tools/natives/gate-bins.sh {GATE_BINS}")
    if rc:
        kids.append(kid(False, "bench: the shipped builds (tools/natives/gate-bins.sh)", f"exit {rc}: {out.strip()[-300:]}"))
        return [Row("bench", 1, "Bench gate: quick 16k in 5 min and full 16k in 15 min at 8 threads, peak memory within the estimate", status_of(kids), [], kids, time.time() - t0)]
    # The 1/2/4-thread scaling run of a missed gate: the quick preset by default (about 40 minutes); the full one takes hours at 1 thread
    # (8,312 s on 2026-10-05), so it is opt-in: PHASE1_BENCH_SCALING=quick16k,full16k.
    scaling_keys = set(os.environ.get("PHASE1_BENCH_SCALING", "quick16k").split(","))
    for key, preset, myr, gate in (("quick16k", "earthlike_quick_16k", a.quick_myr, QUICK_GATE_S), ("full16k", "earthlike_full_16k", a.full_myr, FULL_GATE_S)):
        d = f"review/out/phase1/bench/{key}"

        def one(label, binary, cpus, res):
            res[label] = vm(f"bench-{label}", f"tools/review/bench-quiet.py --cpus {cpus} --out {d}/{label} {GATE_BINS}/{binary} {preset} {START} {myr} 8",
                            pulls=[f"{d}/{label}/bench.json", f"{d}/{label}/summary.txt", f"{d}/{label}/load.tsv"], kind=f"sim:{preset}:8t")
        res = {}
        shutil.rmtree(os.path.join(REPO, d), ignore_errors=True)   # no stale bench.json from an earlier run
        pair = [threading.Thread(target=one, args=(*g, res)) for g in GATE_SETS]
        for i, t in enumerate(pair):
            if i:
                time.sleep(60)   # spaced so the two starts' syncs of the checkout do not collide
            t.start()
        for t in pair:
            t.join()
        got = {label: bench_json(os.path.join(REPO, d, label, "bench.json")) for label, _, _ in GATE_SETS}
        if got["v3"] is None:
            o, r, _ = res.get("v3", ("", -1, 0))
            kids.append(kid(False, f"bench {preset}: within {gate:.0f} s at 8 threads (x86-64-v3)", f"no 8-thread run in {d}/v3/bench.json (exit {r}): {o.strip()[-200:]}"))
            kids.append(kid(False, f"bench {preset}: peak memory within the estimate", "no bench.json"))
            continue
        s, peak, est, stages, doc = got["v3"]
        ev = [f"{d}/v3/bench.json", f"{d}/v3/summary.txt",
              f"{preset} {myr} Myr from {START}, 8 threads held to 8 physical cores on {doc.get('host')}: x86-64-v3 {s:.0f} s (gate {gate:.0f} s)"]
        if got["baseline"]:
            ev.append(f"the baseline build beside it (the floor for CPUs older than x86-64-v3; reported, the gate is the v3 build a mid-range gaming PC loads, "
                      f"decisions log 2026-10-19): {got['baseline'][0]:.0f} s, {d}/baseline/summary.txt")
        else:
            ev.append(f"the baseline build: no 8-thread run in {d}/baseline/bench.json (exit {res.get('baseline', ('', -1))[1]})")
        others = re.search(r"^others' busy CPUs: .*$", res.get("v3", ("",))[0], re.M)
        ev.append("the VM's load from other jobs during the v3 run (the bench shares the box; a miss under load is a reason to re-run): "
                  + (others.group(0) if others else "not recorded"))
        ev.append("stages: " + stage_lines(stages))
        if s > gate and key not in scaling_keys:
            ev.append("scaling over 1/2/4 threads: not re-measured in this run (PHASE1_BENCH_SCALING=" + ",".join(sorted(scaling_keys)) + "); the full-length one takes hours at 1 thread")
        if s > gate and key in scaling_keys:
            # The plan's alternative: "a per-stage report for Ben's decision" with the scaling across thread counts.
            sd = f"review/out/phase1/bench/{key}-scaling"
            out2, rc2, _ = vm("bench", f"{GATE_BINS}/deeptime-x86-64-v3 bench --preset {preset} --start {START} --length-myr {myr} --threads 1,2,4 "
                              f"--out {sd}.json 2>&1 | grep -E 'bench: threads=[0-9]+ total_ms'", pulls=[f"{sd}.json"])
            ev.append("scaling (the gate is missed, so the thread counts too): " + " | ".join(re.findall(r"threads=(\d+) total_ms=([\d.]+)", out2) and
                      [f"{t} threads {float(ms) / 1000:.0f} s" for t, ms in re.findall(r"threads=(\d+) total_ms=([\d.]+)", out2)] or ["not measured"]))
        kids.append(kid(s <= gate, f"bench {preset}: within {gate:.0f} s at 8 threads (x86-64-v3)", *ev))
        if est is None:
            kids.append(kid(False, f"bench {preset}: peak memory within the estimate",
                            f"peak {peak / 2**20:.0f} MiB measured; bench.json carries no estimate (`deeptime bench` writes `estimate.shown_peak_bytes`)"))
        else:
            kids.append(kid(peak <= est, f"bench {preset}: peak memory within the estimate", f"peak {peak / 2**20:.0f} MiB against an estimate of {est / 2**20:.0f} MiB"))
    return [Row("bench", 1, "Bench gate: quick 16k in 5 min and full 16k in 15 min at 8 threads, peak memory within the estimate", status_of(kids), [], kids, time.time() - t0)]


def step_smoke_mic(a):
    # A failed install or a run that never reached its verdict prints the tail of its output (a filtered failure hides why).
    cmd = (f"mkdir -p /tmp/p1-smoke && tools/smoke/install.sh mic --preset earthlike_quick_16k --start {START} --length-myr {a.quick_myr} >/tmp/p1-smoke/install.txt 2>&1 "
           "|| { echo 'install failed:'; tail -n 25 /tmp/p1-smoke/install.txt; }; "
           "python3 tools/smoke/run.py --profile mic --fresh --restart --timeout 3000 >/tmp/p1-smoke/run.txt 2>&1; "
           "grep -E '\\[(PASS|FAIL| -- )\\]|^(PASS|FAIL)|probe (tw|column)' /tmp/p1-smoke/run.txt; "
           "grep -q -E '^(PASS|FAIL) --' /tmp/p1-smoke/run.txt || { echo 'run.py never reached its verdict:'; tail -n 25 /tmp/p1-smoke/run.txt; }")
    out, rc, secs = vm("smoke-mic", cmd, flags=["--server", "--pack"], pulls=["smoke/mic/report.md", "smoke/mic/rcon.jsonl"])
    lines = [l.strip() for l in out.splitlines()]
    kids = []
    for title, pat in SMOKE_CATS:
        sel = [l for l in lines if re.match(r"\[(PASS|FAIL)\]", l) and pat.search(l)]
        ok = bool(sel) and all(l.startswith("[PASS]") for l in sel)
        kids.append(kid(ok, title, *(sel[:6] or ["no such check in the run's output"])))
    rec = [l for l in lines if l.startswith("[ -- ]") and re.search(r"boot:|Done|Generation|RSS", l)]
    other_fail = [l for l in lines if l.startswith("[FAIL]") and not any(p.search(l) for _, p in SMOKE_CATS)]
    kids.append(kid(not other_fail, "every other check of the smoke run", *(other_fail[:6] or ["none failed"])))
    final = next((l for l in lines if re.match(r"^(PASS|FAIL) --", l)), "no final line (the run did not finish)")
    ev = [final, "report: review/out/phase1/ -> smoke/mic/report.md (pulled to smoke/mic/report.md)"] + rec[:4]
    return [Row("smoke-mic", 2, "MIC smoke from an empty world, earthlike_quick_16k: first-start generation, boot, every mixin, probes", status_of(kids) if final.startswith("PASS") else "FAIL", ev, kids, secs)]


def step_geology(a):
    t0 = time.time()
    plan_path = os.path.join(EV, "plan.json")
    world = "smoke/mic/server/planet"
    # PHASE1_GEOLOGY_ANALYSE_ONLY=1: re-run the checks over the world, plan and RCON answers the last full run left on the VM (the
    # boot, Chunky and BlueMap take an hour on a busy box; a changed check does not need them). The two kids that need them keep their last result.
    only_ana = bool(os.environ.get("PHASE1_GEOLOGY_ANALYSE_ONLY"))
    prev = load_rows().get("geology")
    prev_kid = {k[1]: k for k in (prev.kids if prev else [])}
    out, rc = ("", 0) if only_ana else vm("geology", f"python3 tools/review/phase1_vm.py pick --world {world} --out review/out/phase1/plan.json",
                                           pulls=["review/out/phase1/plan.json"])[:2]
    if rc or not os.path.exists(plan_path):
        e = Row("geology", 3, "Geology in the world: porphyry, gossan, placer", "ERROR", ["pick failed (is smoke/mic's world there? run the smoke-mic step first): " + out.strip()[-300:]])
        e2 = Row("compat", 4, "COE and CDG", "ERROR", ["no plan (see geology)"])
        return [e, e2]
    plan = json.load(open(plan_path))
    # The plan is on the VM already (pick wrote it there); the second boot reads it from review/out/phase1/plan.json.
    srv, rc = ("", 0) if only_ana else vm("geology", "python3 tools/review/phase1_vm.py serve --profile mic --plan review/out/phase1/plan.json 2>&1 | grep -E '\\[(PASS|FAIL)\\]|^(PASS|FAIL)' | cut -c1-240",
                                           flags=["--server"], pulls=["smoke/mic/report.md", "smoke/mic/rcon.jsonl"])[:2]
    boot_ok = bool(re.search(r"^PASS --", srv, re.M))
    ana = ("python3 tools/review/phase1_vm.py geology --world smoke/mic/server/planet --plan review/out/phase1/plan.json --rcon smoke/mic/rcon.jsonl "
           "--out review/out/phase1/geology; python3 tools/review/phase1_vm.py compat --plan review/out/phase1/plan.json --rcon smoke/mic/rcon.jsonl")
    out, rc, _ = vm("geology", ana, flags=[] if only_ana else ["--no-sync"], pulls=["review/out/phase1/geology"])
    allk = checks(out)
    compat_titles = ("COE", "CDG")
    geo = [k for k in allk if not k[1].startswith(compat_titles)]
    com = [k for k in allk if k[1].startswith(compat_titles)]
    # BlueMap over the porphyry and its range (item 3's last clause).
    bm_kid = kid(False, "BlueMap shows the arc range and the deposit", "not rendered")
    if only_ana and "BlueMap shows the arc range and the deposit" in prev_kid:
        bm_kid = prev_kid["BlueMap shows the arc range and the deposit"]
    elif plan["porphyry"]:
        cx, cy, cz = (int(v) for v in plan["porphyry"]["center"])
        r = plan["chunky"][0][2]
        env = f"BLUEMAP_BOX='{cx - r} {cx + r - 1} {cz - r} {cz + r - 1}' BLUEMAP_ONLY_PRESENT=1 BLUEMAP_MAP_LABEL='Deep Time phase 1 geology'"
        bm, rc, _ = vm("geology", f"{env} tools/review/render-bluemap.sh {world} smoke/mic/server/mods review/out/phase1/bluemap-geology 6 2>&1 | "
                       f"grep -E -i 'render-bluemap|error|exception' | grep -v -E 'Missing.*(sound|lang)' | tail -8; "
                       f"nix shell nixpkgs#chromium -c python3 tools/review/phase1_shots.py --web review/out/phase1/bluemap-geology/web --map planet "
                       f"--out review/out/phase1/geology/bluemap --view 'tilt:{cx}:{cy}:{cz}:{2 * r}:1.0' --view 'flat:{cx}:{cy}:{cz}:{r}:0:flat' --wait 90",
                       flags=["--server"], pulls=["review/out/phase1/geology"])
        shots = [f for f in ("bluemap-tilt.png", "bluemap-flat.png") if os.path.exists(os.path.join(EV, "geology", f))]
        bm_kid = kid(len(shots) == 2, "BlueMap shows the arc range and the deposit",
                     f"{2 * r}-block square around the porphyry (site {plan['porphyry']['site']}) at {cx},{cz}; pictures: " + (", ".join(f"review/out/phase1/geology/{s}" for s in shots) or "none"),
                     "the BlueMap app: review/out/phase1/bluemap-geology/web on the VM (not pulled)")
    gk = [("PASS" if k[0] else "FAIL", k[1], [k[2]]) for k in geo] + [bm_kid]
    ck = [("PASS" if k[0] else "FAIL", k[1], [k[2]]) for k in com]
    boot = kid(boot_ok, "the second boot (Chunky over the sites, the probes) finished without a failed check", srv.strip().splitlines()[-1] if srv.strip() else "no output")
    if only_ana and "the second boot (Chunky over the sites, the probes) finished without a failed check" in prev_kid:
        boot = prev_kid["the second boot (Chunky over the sites, the probes) finished without a failed check"]
    secs = time.time() - t0
    return [Row("geology", 3, "Geology in the world: a porphyry, `probe site`, copper ore, gossan, a placer whose why-chain ends in an eroded porphyry, BlueMap",
                status_of(gk + [boot], need=6), [f"plan: review/out/phase1/plan.json", "pictures and JSON: review/out/phase1/geology/"], gk + [boot], secs),
            Row("compat", 4, "COE reports the copper vein over the porphyry and nothing elsewhere, yield follows tonnage; CDG reports oil over a trap and none elsewhere",
                status_of(ck, need=5), ["probes: smoke/mic/rcon.jsonl"], ck, 0.0)]


def gametest_row(sid, item, title, flags, install, name, pattern, a=None):
    if a is not None and a.skip_gametest:
        # The coordinator runs the GameTests per merge batch, not branches: the row waits (PENDING) instead of booting a server.
        return Row(sid, item, title, "PENDING", ["not run on this branch (--skip-gametest): the coordinator runs GameTests per merge batch; run `tools/review/phase1.sh --only " + sid + "` after the merge"])
    cmd = (f"{install}./gametest.sh {flags} -PgametestOnly={name} 2>&1 | grep -E 'required tests|\\[gametest\\] {name}' | "
           "sed 's/^.*\\[gametest\\]/[gametest]/;s/^.*GameTestServer\\]: //'")
    out, rc, secs = vm(sid, cmd, flags=["--server", "--pack"] if "Mic" in flags else ["--server"])
    passed = re.search(r"All \d+ required tests passed", out) is not None and re.search(pattern, out) is not None
    ev = [l.strip() for l in out.splitlines() if l.strip()][:6] or ["no output"]
    return Row(sid, item, title, "PASS" if passed else "FAIL", ev, [], secs)


def step_gametest_mic(a):
    return [gametest_row("gametest-mic", 5, "`climateConsistent` passes (GameTest server with the current MIC pack)", "-PwithMic",
                         "tools/smoke/install.sh mic >/dev/null 2>&1; ", "climateConsistent", r"\[gametest\] climateConsistent", a)]


def step_gametest_tb(a):
    return [gametest_row("gametest-tb", 5, "`regionsFollowProvinces` passes (GameTest server with TerraBlender + Biomes O' Plenty)", "-PwithTerraBlender",
                         "tools/smoke/install.sh terrablender >/dev/null 2>&1; ", "regionsFollowProvinces", r"\[gametest\] regionsFollowProvinces", a)]


def step_viewer(a):
    t0 = time.time()
    run = (f"target/release/deeptime run --preset earthlike_quick_16k --start {START} --length-myr {a.quick_myr} --threads 8 "
           "--out review/out/phase1/viewer-run/planet/deeptime --viewer review/out/phase1/viewer 2>&1 | tail -3")
    # PHASE1_VIEWER_CHECK_ONLY=1 re-checks the bundle already on the VM (the bake is the 5 minutes; the check is seconds).
    out1, rc1 = ("(bundle reused)", 0) if os.environ.get("PHASE1_VIEWER_CHECK_ONLY") else vm("viewer", run)[:2]
    out, rc, secs = vm("viewer", "nix shell nixpkgs#chromium -c python3 tools/viewer/phase1_viewer_check.py --bundle review/out/phase1/viewer "
                       "--out review/out/phase1/viewer-shots", pulls=["review/out/phase1/viewer-shots"])
    kids = checks(out)
    if not kids:
        return [Row("viewer", 6, "The viewer shows the timelapse, event ticker, layers, ore provinces and three \"Why here?\" chains", "ERROR",
                    [f"run exit {rc1}: {out1.strip()[-200:]}", f"check exit {rc}: {out.strip()[-300:]}"], secs=time.time() - t0)]
    ks = [("PASS" if k[0] else "FAIL", k[1], [k[2]]) for k in kids]
    return [Row("viewer", 6, "The viewer shows the timelapse, event ticker, layers, ore provinces and three \"Why here?\" chains",
                status_of(ks, need=5), ["bundle: review/out/phase1/viewer on the VM (deeptime run --viewer)", "pictures: review/out/phase1/viewer-shots/"], ks, time.time() - t0)]


def step_scale(a):
    t0 = time.time()
    log = os.path.join(LOGS, "scale.log")
    os.makedirs(LOGS, exist_ok=True)
    with open(log, "w") as f:
        rc = subprocess.run([sys.executable, os.path.join(REPO, "tools", "review", "phase1_scale.py")] + (["--only", os.environ["PHASE1_SCALE_ONLY"]] if os.environ.get("PHASE1_SCALE_ONLY") else []) + (["--scales", os.environ["PHASE1_SCALE_SCALES"]] if os.environ.get("PHASE1_SCALE_SCALES") else []), cwd=REPO, stdout=f, stderr=subprocess.STDOUT).returncode
    sc = os.path.join(REPO, "review", "out", "phase1", "scale")
    kids = []
    stats = {}
    try:
        stats = {s["label"]: s for s in json.load(open(os.path.join(sc, "scale-stats.json")))}
    except (OSError, ValueError):
        pass
    for lab, m in (("1:100", 100), ("1:300", 300), ("Earth-proportioned", 1221)):
        have = [f for f in (f"h{m}-tilt.png", f"h{m}-flat.png") if os.path.exists(os.path.join(sc, f))]
        s = stats.get(lab) or stats.get("earth" if m == 1221 else lab) or {}
        kids.append(kid(len(have) == 2, f"BlueMap render at {lab} ({m} m per block)",
                        *[f"review/out/phase1/scale/{f}" for f in have] or ["not rendered"],
                        *([f"radius {s['radius_km']:,} km, land share of the window {100 * s['land_share']:.0f} %, median land slope {s['median_slope'] if s['median_slope'] is not None else float('nan'):.2f}, land steeper than 45 deg {100 * s['steep_share']:.1f} %"] if s else [])))
    # Each scale's window is chosen so the three show a like share of land (decisions log 2026-10-17: the exit run's windows had 0.73, 0.84 and 0.56).
    try:
        ag = json.load(open(os.path.join(sc, "scale-agreement.json")))
        kids.append(kid(ag["within"], f"land shares of the window agree within {ag['tolerance']:g} across the scales",
                        "land share " + ", ".join(f"{s['label']} {s['land_share']:.3f}" for s in stats.values()) if stats else "no statistics",
                        f"spread {ag['spread']:.3f} (target share {ag['target']:g})"))
    except (OSError, ValueError, KeyError):
        kids.append(kid(False, "land shares of the window agree within 0.05 across the scales", "scale-agreement.json missing"))
    # A world that does not fit the build height has no terrain to compare (the Earth-proportioned bake of 2026-10-18 diverged numerically).
    bad = [s for s in stats.values() if not s.get("bake_fits_build_height", True)]
    kids.append(kid(bool(stats) and not bad, "each bake fits the build height (no cell compressed under the ceiling or below the floor)",
                    *([f"{s['label']}: {s['build_limit_cells'].get('cells_compressed')} cells compressed, {s['build_limit_cells'].get('cells_below_min_y')} below the floor, "
                      f"highest surface {s.get('surface_max_elevation_m')} m" for s in bad] or ["all bakes fit"])))
    kids.append(kid(os.path.exists(os.path.join(sc, "index.html")), "side-by-side page section built", "review/out/phase1/scale/index.html"))
    return [Row("scale", 7, "Scale comparison: a window of each planet at 1:100, 1:300 and Earth proportions, land shares matched, vertical scale unchanged", status_of(kids) if rc == 0 else "FAIL",
                [f"log: review/out/phase1-src/logs/scale.log (exit {rc})"], kids, time.time() - t0)]


# ---------------------------------------------------------------------------------------------
# the page
# ---------------------------------------------------------------------------------------------
def load_rows():
    p = os.path.join(SRC, "rows.json")
    return {r["sid"]: Row.from_json(r) for r in json.load(open(p))} if os.path.exists(p) else {}


def exit_table(rows):
    lines = ["| Item | Check | Status |", "|---|---|---|"]
    for r in rows:
        lines.append(f"| {r.item or '0'} | {r.title} | **{r.status}** |")
        for k in r.kids:
            lines.append(f"|  | {'&nbsp;' * 3}{k[1]} | {k[0]} |")
    return "\n".join(lines)


def build_page(rows, started, published):
    esc = html.escape
    try:
        import phase1_scale
        scale = phase1_scale.section_html("scale/")
        scale_css = phase1_scale.SECTION_CSS
    except Exception as e:   # the page is a draft: say so instead of failing
        scale, scale_css = f'<p class="bad">scale section unavailable: {esc(str(e))}</p>', ""
    spaces = json.load(open(os.path.join(REPO, "tools", "review", "spaces.json")))
    counts = {s: sum(1 for r in rows if r.status == s) for s in ("PASS", "FAIL", "ERROR", "PENDING")}
    cls = {"PASS": "ok", "FAIL": "bad", "ERROR": "bad", "PENDING": "pend"}

    def ev(items):
        out = []
        for e in items:
            if e.startswith("http"):
                out.append(f'<a href="{esc(e)}">{esc(e)}</a>')
            elif e.startswith("review/out/phase1/") and re.search(r"\.(png|json|md|html)$", e):
                out.append(f'<a href="{esc(e[len("review/out/phase1/"):])}"><code>{esc(e)}</code></a>')
            else:
                out.append(f"<code>{esc(e)}</code>")
        return "<br>".join(out) or "n/a"

    trs = []
    for r in rows:
        log = f' · <a href="logs/{r.sid}.log">log</a>' if os.path.exists(os.path.join(LOGS, f"{r.sid}.log")) else ""
        trs.append(f'<tr class="{cls[r.status]}"><td>{r.status}</td><td>{"item " + str(r.item) if r.item else "Phase 0"}</td><td><b>{esc(r.title)}</b>{log}'
                   f'<div class="t">{r.secs:.0f} s</div></td><td>{ev(r.evidence)}</td></tr>')
        for st, title, evid in r.kids:
            trs.append(f'<tr class="kid {cls[st]}"><td>{st}</td><td></td><td>{esc(title)}</td><td>{ev(evid)}</td></tr>')
    links = "".join(f'<li><a href="{esc(v["url"])}">{esc(k)}</a></li>' for k, v in sorted(spaces.items()) if k not in ("phase1",) and v.get("url"))
    commit = subprocess.run(["git", "-C", REPO, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    verdict = ("Every check passes." if all(r.status == "PASS" for r in rows) else
               f"{counts['PASS']} of {len(rows)} rows pass; {counts['FAIL'] + counts['ERROR']} do not; {counts['PENDING']} pending.")
    page = f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Deep Time: Phase 1 (MVP)</title>
<meta name="description" content="Phase 1 exit of Deep Time on {started[:10]}: Phase 0's suite again and the eight Phase 1 checks, with the scale comparison (1:100, 1:300, Earth-proportioned).">
<style>
:root{{--bg:#fbfaf7;--fg:#1d1d1b;--mut:#6b6a66;--ok:#1f7a3d;--bad:#b3261e;--pend:#9a6a00;--line:#e3e0d8;--card:#fff}}
@media (prefers-color-scheme: dark){{:root:not([data-theme=light]){{--bg:#161614;--fg:#ecebe6;--mut:#a3a19a;--ok:#5cc987;--bad:#ff8a80;--pend:#f0b94a;--line:#34332f;--card:#1f1e1c}}}}
:root[data-theme=dark]{{--bg:#161614;--fg:#ecebe6;--mut:#a3a19a;--ok:#5cc987;--bad:#ff8a80;--pend:#f0b94a;--line:#34332f;--card:#1f1e1c}}
body{{background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,-apple-system,Segoe UI,sans-serif;margin:0;padding:24px 16px}}
main{{max-width:1200px;margin:0 auto}} h1{{margin:0 0 4px}} .sub{{color:var(--mut);margin-bottom:20px}}
.sum{{display:inline-block;padding:6px 12px;border-radius:8px;background:var(--card);border:1px solid var(--line);font-weight:600}}
table.exit{{width:100%;border-collapse:collapse;margin:16px 0;background:var(--card);border:1px solid var(--line)}}
table.exit td{{border-top:1px solid var(--line);padding:7px 10px;vertical-align:top}} table.exit td:first-child{{font-weight:700;width:70px}}
tr.ok td:first-child{{color:var(--ok)}} tr.bad td:first-child{{color:var(--bad)}} tr.pend td:first-child{{color:var(--pend)}}
tr.kid td{{font-size:13.5px;background:color-mix(in srgb,var(--card) 92%,var(--fg) 8%)}} tr.kid td:nth-child(3){{padding-left:28px}}
.t{{color:var(--mut);font-size:12px}} code{{font-size:12.5px;overflow-wrap:anywhere}} a{{color:inherit}} ul{{padding-left:20px}} .bad{{color:var(--bad)}}
img{{max-width:100%}}{scale_css}
@media (max-width:700px){{table.exit td{{display:block;border-top:none}} table.exit tr{{display:block;border-top:1px solid var(--line)}}}}
</style></head><body><main>
<h1>Deep Time: Phase 1 (MVP)</h1>
<div class="sub">A simulated planet, generated on first start, played as a wrapped world with geology-driven terrain and ore. Re-run end to end on {esc(started)} at commit {esc(commit)} on the Azure test VM (builds, simulation, servers, renders, screenshots).</div>
<div class="sum">{esc(verdict)}</div>
{"" if published else '<p class="t">Draft. Not published: the coordinator publishes it when the exit has passed.</p>'}
<table class="exit">{''.join(trs)}</table>
<h2>Scale: one planet at three horizontal scales</h2>
<p>The decisions file asks Ben to playtest about 1:300 against about 1:100 and Earth proportions (2π × 6,370 km ÷ 32,768 blocks = 1,221 m per block). Same window, vertical scale unchanged:</p>
{scale}
<h2>Other review pages</h2><ul>{links}</ul>
<h2>Not verifiable here</h2><ul>
<li>How the seam, the pole storm and the Create World screens look and feel on a real client (the P1-11 screenshots are on their own page).</li>
<li>Multiplayer entity tracking across the seam; contraption rendering at the seam.</li>
<li>Whether the repository goes public: that is Ben's decision (the decisions file keeps it private until the MVP).</li></ul>
</main></body></html>"""
    os.makedirs(SRC, exist_ok=True)
    open(os.path.join(SRC, "index.html"), "w").write(page)
    # The page's relative links: evidence beside it.
    for sub in ("scale", "geology", "viewer-shots", "bench"):
        s, d = os.path.join(EV, sub), os.path.join(SRC, sub if sub != "scale" else "scale")
        if os.path.isdir(s):
            shutil.copytree(s, d, dirs_exist_ok=True)
    return counts


def step_page(a):
    started = datetime.datetime.now().strftime("%Y-%m-%d %H:%M")
    rows = [r for r in order_rows(load_rows()) if r.sid != "page"]
    counts_ok = all(r.status == "PASS" for r in rows)
    published = False
    if a.publish:
        if not counts_ok:
            print("page: not publishing: not every row passes (the coordinator publishes after the exit passed)")
        else:
            subprocess.run([os.path.join(REPO, "tools", "review", "publish.sh"), "phase1", SRC, "-m", f"Phase 1 exit {started[:10]}", "--emoji", "🌍"], cwd=REPO)
            published = True
    build_page(rows + [Row("page", 8, "", "PENDING")], started, published)
    ev = [f"draft: review/out/phase1-src/index.html ({sum(1 for r in rows)} rows)"]
    if published:
        ev.append(json.load(open(os.path.join(REPO, "tools", "review", "spaces.json"))).get("phase1", {}).get("url", ""))
    status = "PASS" if published else "PENDING"
    title = '"Deep Time: Phase 1 (MVP)" published privately' if published else '"Deep Time: Phase 1 (MVP)": drafted from the rows, not published (waits for the coordinator)'
    return [Row("page", 8, title, status, ev)]


STEPS = [("phase0", step_phase0), ("build", step_build), ("bench", step_bench), ("smoke-mic", step_smoke_mic), ("geology", step_geology),
         ("gametest-mic", step_gametest_mic), ("gametest-tb", step_gametest_tb), ("viewer", step_viewer), ("scale", step_scale), ("page", step_page)]
ORDER = ["phase0", "bench", "smoke-mic", "geology", "compat", "gametest-mic", "gametest-tb", "viewer", "scale", "page"]


def order_rows(d):
    return [d[k] for k in ORDER if k in d] + [v for k, v in d.items() if k not in ORDER]


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--only", help="comma-separated steps (compat is part of geology)")
    ap.add_argument("--skip-phase0", action="store_true", help="reuse the last phase0 rows instead of re-running the suite")
    ap.add_argument("--publish", action="store_true", help="publish the page, only if every row passes (the coordinator's call)")
    ap.add_argument("--skip-gametest", action="store_true", help="leave the two GameTest rows PENDING (the coordinator runs GameTests per merge batch, not on branches)")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--quick-myr", type=int, default=QUICK_MYR)
    ap.add_argument("--full-myr", type=int, default=FULL_MYR)
    a = ap.parse_args()
    a.build_failed = False
    if a.list:
        print("\n".join(s for s, _ in STEPS))
        return
    only = set(a.only.split(",")) if a.only else None
    if only and "compat" in only:
        only = (only - {"compat"}) | {"geology"}
    os.makedirs(SRC, exist_ok=True)
    rows = load_rows()
    started = datetime.datetime.now().strftime("%Y-%m-%d %H:%M")
    for sid, fn in STEPS:
        if only is not None and sid not in only:
            continue
        if sid not in ("phase0", "build", "page") and a.build_failed:
            rows[sid] = Row(sid, 0, f"{sid} (skipped)", "ERROR", ["the build failed (logs/build.log)"])
            continue
        print(f"== {sid}", flush=True)
        t0 = time.time()
        try:
            got = fn(a)
        except Exception as e:   # one broken step must not hide the others
            import traceback
            traceback.print_exc()
            got = [Row(sid, 0, f"{sid}: the step crashed", "ERROR", [f"{type(e).__name__}: {e}"], secs=time.time() - t0)]
        for r in got:
            rows[r.sid] = r
            print(f"   {r.status:7s} item {r.item}: {r.title} ({r.secs:.0f} s)", flush=True)
            for k in r.kids:
                print(f"            {k[0]:5s} {k[1]}", flush=True)
        # Several invocations may run side by side (--only bench / --only smoke-mic,...): merge into the file as it is now.
        merged = load_rows()
        merged.update({r.sid: r for r in got})
        rows.update(merged)
        json.dump([r.to_json() for r in order_rows(rows)], open(os.path.join(SRC, "rows.json"), "w"), indent=1)
    allrows = order_rows(rows)
    if "page" not in rows or (only is not None and "page" not in only):
        build_page([r for r in allrows if r.sid != "page"] + [rows.get("page", Row("page", 8, "", "PENDING"))], started, False)
    table = exit_table(allrows)
    open(os.path.join(SRC, "exit-table.md"), "w").write(f"# Phase 1 exit {started}\n\n{table}\n")
    print("\nPhase 1 exit table\n" + table)
    print(f"\npage: {os.path.join(SRC, 'index.html')}")
    sys.exit(0 if all(r.status in ("PASS", "PENDING") for r in allrows) else 1)


if __name__ == "__main__":
    main()
