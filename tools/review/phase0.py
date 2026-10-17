#!/usr/bin/env python3
# SPDX-License-Identifier: LGPL-3.0-or-later
"""The Phase 0 exit (plan §10.2, P0-15): run every Phase 0 check, build one review page, publish it.

    python3 tools/review/phase0.py [--only <step>[,<step>...]] [--no-publish]

Runs from this laptop and drives Ben's Mac through tools/remote/mac.sh, one step at a time: every step
that boots Minecraft takes the Mac's server lock for itself, so other agents' servers can run between
steps. Each step's full output is kept in review/out/phase0/logs/<step>.log; the page shows a
pass/fail table with the evidence lines, then links to the other review pages and the reports.
Standard library only.
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
import time

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
OUT = os.path.join(REPO, "review", "out", "phase0-src")
LOGS = os.path.join(OUT, "logs")
MAC = os.path.join(REPO, "tools", "remote", "mac.sh")

# The seam tests of P0-06, as their agent ran them (one at a time; positions next to the seam). The
# pole prototype (prototypepushback) became P1-18's cap: its tests load their own polar chunks.
PLANET_RUNS = ("--run columnscontinuous,entitycrossesseam,structurestraddlesseam,treestraddlesseam,"
               "shaftlineacrossseam@8185,250,0 --run jetcrossesseam@8185,250,64 "
               "--run entitypushedback,hardstopat80@0,300,64 --scenario sable_straddle")


def step_defs():
    """(id, row title, mac.sh flags, command, [(evidence regex, …)], pass test)."""
    return [
        ("rust", "Rust: fmt, clippy, tests, licences", [],
         "cargo fmt --all --check >/dev/null 2>&1 && echo 'fmt ok' || echo 'fmt FAILED'; "
         "cargo clippy --workspace --all-targets -q -- -D warnings >/dev/null 2>&1 && echo 'clippy ok' || echo 'clippy FAILED'; "
         "cargo test --workspace -q 2>&1 | grep -E 'test result' ; cargo deny check licenses bans 2>&1 | tail -1",
         [r"fmt ok|fmt FAILED", r"clippy ok|clippy FAILED", r"test result: \w+\. \d+ passed; \d+ failed", r"bans ok, licenses ok"],
         # fmt, clippy, at least one passing test binary, no failing one, licenses: a run where cargo never started must not pass (2026-10-16)
         lambda out, rc: ("fmt ok" in out and "clippy ok" in out and "licenses ok" in out and re.search(r"test result: ok\. [1-9]\d* passed", out) is not None
                          and not re.search(r"test result: FAILED|[1-9]\d* failed", out))),
        ("junit", "Java: build and JUnit", [],
         "cd neoforge && ./gradlew build test --console=plain -q; cd .. && python3 tools/ci/junit-summary.py neoforge/build/test-results/test",
         [r"tests=\d+ failures=\d+ errors=\d+ skipped=\d+"],
         # tests > 0 and no failed build: a Gradle build that failed before testing left "tests=0 failures=0 errors=0" and passed (2026-10-16).
         lambda out, rc: re.search(r"tests=[1-9]\d* failures=0 errors=0", out) is not None and "BUILD FAILED" not in out and "FAILURE:" not in out),
        ("gametest", "GameTest server (Deep Time + Toroidal World)", ["--server"],
         "./gametest.sh 2>&1 | grep -E 'required tests|\\[gametest\\]' | sed 's/^.*\\[gametest\\]/[gametest]/;s/^.*GameTestServer\\]: //'",
         [r"All \d+ required tests passed", r"\[gametest\] (heightMatches\w+|strataClassesResolve|lakeHolds|native|spawnOnLand|oreVeinsDisabled) .*"],
         lambda out, rc: re.search(r"All \d+ required tests passed", out) is not None),
        ("gametest-mic", "GameTest server with the current MIC pack", ["--server", "--pack"],
         "tools/smoke/install.sh mic >/dev/null 2>&1; ./gametest.sh -PwithMic 2>&1 | grep -E 'required tests|\\[gametest\\] (terralith|surfaceRules|lakeHolds) ' | sed 's/^.*\\[gametest\\]/[gametest]/;s/^.*GameTestServer\\]: //'; python3 -c \"import json;d=json.load(open('smoke/mic/install.json'));print('pack', d['pack'], 'stale hashes fixed:', d.get('pack_stale_hashes') or 'none')\"",
         [r"All \d+ required tests passed", r"\[gametest\] terralithBiomesOnLand terralith=\d+ of \d+", r"\[gametest\] surfaceRulesApplied .*", r"pack .*"],
         lambda out, rc: re.search(r"All \d+ required tests passed", out) is not None),
        ("gametest-tb", "GameTest server with TerraBlender + Biomes O' Plenty", ["--server"],
         "tools/smoke/install.sh terrablender >/dev/null 2>&1; ./gametest.sh -PwithTerraBlender 2>&1 | grep -E 'required tests|\\[gametest\\] bopBiomesPresent' | sed 's/^.*\\[gametest\\]/[gametest]/;s/^.*GameTestServer\\]: //'",
         [r"All \d+ required tests passed", r"\[gametest\] bopBiomesPresent biomesoplenty=\d+"],
         lambda out, rc: re.search(r"All \d+ required tests passed", out) is not None and re.search(r"biomesoplenty=[1-9]", out) is not None),
        ("planet", "Planet server: seam, Sable, structures, pole prototype (C = 16,384)", ["--server"],
         "tools/smoke/install.sh standalone >/dev/null 2>&1; ./gametest-planet.sh " + PLANET_RUNS + " 2>&1 | grep -E 'probe tw|gametest-summary|sable_straddle|^PASS|^FAIL'",
         [r"probe tw: .*", r"\[gametest-summary\] \w+: .*", r"sable_straddle: .*", r"^(PASS|FAIL).*"],
         lambda out, rc: re.search(r"^PASS", out, re.M) is not None),
        ("smoke-standalone", "Smoke: standalone server on synthetic world data, seam pregenerated", ["--server"],
         "tools/smoke/install.sh standalone --synth wrap-test --size 16384 >/dev/null 2>&1; python3 tools/smoke/run.py --profile standalone --fresh --chunky 128 --chunky-seam 2>&1 | grep -E '\\[(PASS|FAIL)\\]|tw\\.|^(PASS|FAIL)'",
         [r"\[(PASS|FAIL)\] .*", r"tw\.\w+: .*", r"^(PASS|FAIL).*"],
         lambda out, rc: re.search(r"^PASS", out, re.M) is not None),
        ("inspect", "deeptime-inspect: generated surface vs world data, seam strip", [],
         "cargo run -q --release -p deeptime-inspect -- compare --world smoke/standalone/server/planet --world-data smoke/standalone/server/planet/deeptime --out review/out/phase0-inspect | python3 -c \"import json,sys; d=json.load(sys.stdin); print('p99_abs_diff_blocks', d.get('p99_abs_diff_blocks'), 'compared', d.get('compared', d.get('columns')))\"; cargo run -q --release -p deeptime-inspect -- seam --world smoke/standalone/server/planet --out review/out/phase0-inspect | python3 -c \"import json,sys; d=json.load(sys.stdin); print('seam', {k: v for k, v in d.items() if not isinstance(v, (list, dict))})\"",
         [r"p99_abs_diff_blocks \S+ .*", r"seam .*"],
         lambda out, rc: re.search(r"p99_abs_diff_blocks (0|1)(\.0)? ", out) is not None),
        ("smoke-mic-32k", "Smoke: current MIC pack at C = 32,768 (test terrain, seam pregenerated)", ["--server", "--pack"],
         "tools/smoke/install.sh mic --size 32768 --test-terrain wrap-test >/dev/null 2>&1; python3 tools/smoke/run.py --profile mic --fresh --chunky 128 --chunky-center seam 2>&1 | grep -E '\\[(PASS|FAIL)\\]|tw\\.|peak RSS|^(PASS|FAIL)'",
         [r"\[(PASS|FAIL)\] .*", r"tw\.\w+: .*", r"^(PASS|FAIL).*"],
         lambda out, rc: re.search(r"^PASS", out, re.M) is not None),
        ("smoke-mic-16k", "Smoke: current MIC pack at C = 16,384 on synthetic world data", ["--server", "--pack"],
         "tools/smoke/install.sh mic --synth wrap-test --size 16384 >/dev/null 2>&1; python3 tools/smoke/run.py --profile mic --fresh --chunky 128 --chunky-center seam 2>&1 | grep -E '\\[(PASS|FAIL)\\]|tw\\.|peak RSS|^(PASS|FAIL)'",
         [r"\[(PASS|FAIL)\] .*", r"tw\.\w+: .*", r"^(PASS|FAIL).*"],
         lambda out, rc: re.search(r"^PASS", out, re.M) is not None),
    ]


def run_step(sid, flags, cmd):
    os.makedirs(LOGS, exist_ok=True)
    t0 = time.time()
    if os.environ.get("PHASE0_DETACH"):
        # Phase 1's exit re-runs this suite with each step detached on the VM (a dropped ssh session loses a wait, not a step).
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        import phase1_detach
        out, rc, _ = phase1_detach.run("p0-" + sid, cmd, flags=flags)
        r = subprocess.CompletedProcess([], rc, "", "")
    else:
        r = subprocess.run([MAC, *flags, "--", cmd], cwd=REPO, capture_output=True, text=True, errors="replace")
        out = r.stdout + r.stderr
    noise = re.compile(r"^(copying|building|fetching|these \d|  /nix|warning: Git|trace:|evaluation warning|mac\.sh: waiting)")
    out = "\n".join(l for l in out.splitlines() if not noise.match(l))
    open(os.path.join(LOGS, f"{sid}.log"), "w").write(out + "\n")
    return out, r.returncode, time.time() - t0


def natives_row():
    args = ["run", "list", "-R", "MinecraftIndustrialComplex/deep-time", "-w", "natives.yml", "-L", "1",
            "--json", "databaseId,conclusion,headSha,url,createdAt"]
    # NixOS keeps nothing installed globally: gh comes from nixpkgs when it is not on the PATH.
    cmd = ["gh", *args] if shutil.which("gh") else ["nix", "run", "nixpkgs#gh", "--", *args]
    try:
        r = subprocess.run(cmd, capture_output=True, text=True)
    except OSError as e:
        r = subprocess.CompletedProcess(cmd, 127, "", str(e))
    try:
        run = json.loads(r.stdout)[0]
        ok = run["conclusion"] == "success"
        ev = [f"natives.yml run {run['databaseId']} ({run['createdAt'][:10]}, {run['headSha'][:7]}): {run['conclusion']}: "
              "8 cross-builds, loaded by NativeSmoke on Windows x64/arm64, macOS arm64/x64, Linux gnu x64/arm64, musl x64/arm64",
              run["url"]]
    except Exception:
        ok, ev = False, [f"could not read the natives.yml run: {r.stderr.strip()[:200]}"]
    return ("natives", "Natives: eight builds, loaded on real OS runners", ok, ev, 0.0)


def build_page(rows, started):
    spaces = json.load(open(os.path.join(REPO, "tools", "review", "spaces.json")))
    passed = sum(1 for r in rows if r[2])
    esc = html.escape
    trs = []
    for sid, title, ok, ev, secs in rows:
        evh = "<br>".join(f'<a href="{esc(e)}">{esc(e)}</a>' if e.startswith("http") else f"<code>{esc(e)}</code>" for e in ev) or "n/a"
        log = f' · <a href="logs/{sid}.log">log</a>' if os.path.exists(os.path.join(LOGS, f"{sid}.log")) else ""
        trs.append(f'<tr class="{"ok" if ok else "bad"}"><td>{"PASS" if ok else "FAIL"}</td><td>{esc(title)}{log}'
                   f'<div class="t">{secs:.0f} s</div></td><td>{evh}</td></tr>')
    links = "".join(f'<li><a href="{esc(v["url"])}">{esc(k)}</a></li>' for k, v in spaces.items() if k != "phase0")
    reports = "".join(f'<li><a href="reports/{esc(f)}">{esc(f)}</a></li>' for f in sorted(os.listdir(os.path.join(OUT, "reports")))) \
        if os.path.isdir(os.path.join(OUT, "reports")) else ""
    commit = subprocess.run(["git", "-C", REPO, "rev-parse", "--short", "HEAD"], capture_output=True, text=True).stdout.strip()
    page = f"""<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Deep Time: Phase 0</title>
<meta name="description" content="Phase 0 exit of Deep Time: every bridge check re-run on {started[:10]}: natives, generator, Toroidal World wrap, biomes, water, renders.">
<style>
:root{{--bg:#fbfaf7;--fg:#1d1d1b;--mut:#6b6a66;--ok:#1f7a3d;--bad:#b3261e;--line:#e3e0d8;--card:#fff}}
@media (prefers-color-scheme: dark){{:root:not([data-theme=light]){{--bg:#161614;--fg:#ecebe6;--mut:#a3a19a;--ok:#5cc987;--bad:#ff8a80;--line:#34332f;--card:#1f1e1c}}}}
:root[data-theme=dark]{{--bg:#161614;--fg:#ecebe6;--mut:#a3a19a;--ok:#5cc987;--bad:#ff8a80;--line:#34332f;--card:#1f1e1c}}
body{{background:var(--bg);color:var(--fg);font:15px/1.5 system-ui,-apple-system,Segoe UI,sans-serif;margin:0;padding:24px 16px}}
main{{max-width:1100px;margin:0 auto}} h1{{margin:0 0 4px}} .sub{{color:var(--mut);margin-bottom:20px}}
.sum{{display:inline-block;padding:6px 12px;border-radius:8px;background:var(--card);border:1px solid var(--line);font-weight:600}}
table{{width:100%;border-collapse:collapse;margin:16px 0;background:var(--card);border:1px solid var(--line)}}
td{{border-top:1px solid var(--line);padding:8px 10px;vertical-align:top}} td:first-child{{font-weight:700;width:52px}}
tr.ok td:first-child{{color:var(--ok)}} tr.bad td:first-child{{color:var(--bad)}} .t{{color:var(--mut);font-size:12px}}
code{{font-size:12.5px;overflow-wrap:anywhere}} a{{color:inherit}} ul{{padding-left:20px}}
@media (max-width:640px){{td{{display:block;border-top:none}} tr{{display:block;border-top:1px solid var(--line)}}}}
</style></head><body><main>
<h1>Deep Time: Phase 0</h1>
<div class="sub">The bridge from a simulated planet to a Minecraft world. Re-run end to end on {esc(started)} at commit {esc(commit)}, on Ben's M6 Mac mini (builds, GameTests, servers) and GitHub Actions (natives).</div>
<div class="sum">{passed} of {len(rows)} checks pass</div>
<table>{''.join(trs)}</table>
<h2>Other review pages</h2><ul>{links}</ul>
<h2>Reports</h2><ul>{reports}</ul>
<h2>Not verifiable here</h2><ul>
<li>How the seam and the pole storm look and feel on a real client (hitch, pop-in, ship hulls).</li>
<li>Multiplayer entity tracking across the seam; contraption rendering at the seam.</li>
<li>The Create World timelapse screen (Phase 1).</li></ul>
</main></body></html>"""
    open(os.path.join(OUT, "index.html"), "w").write(page)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only")
    ap.add_argument("--no-publish", action="store_true")
    a = ap.parse_args()
    started = datetime.datetime.now().strftime("%Y-%m-%d %H:%M")
    only = set(a.only.split(",")) if a.only else None
    state_path = os.path.join(OUT, "rows.json")
    old = {r[0]: r for r in json.load(open(state_path))} if os.path.exists(state_path) else {}
    os.makedirs(OUT, exist_ok=True)
    rows = []
    for sid, title, flags, cmd, pats, test in step_defs():
        if only is not None and sid not in only:
            if sid in old:
                rows.append(tuple(old[sid]))
            continue
        print(f"== {sid}: {title}", flush=True)
        out, rc, secs = run_step(sid, flags, cmd)
        ok = bool(test(out, rc))
        ev = []
        for p in pats:
            ev += [m.group(0) for m in re.finditer(p, out, re.M)][:8]
        print(f"   {'PASS' if ok else 'FAIL'} ({secs:.0f} s)", flush=True)
        rows.append((sid, title, ok, ev, secs))
    rows.append(natives_row())
    json.dump(rows, open(state_path, "w"), indent=1)
    # Reports the page links (smoke reports pulled from the Mac, P0-06's saved spike reports).
    rep = os.path.join(OUT, "reports")
    os.makedirs(rep, exist_ok=True)
    for prof in ("standalone", "mic"):
        subprocess.run(["rsync", "-a", "-e", "ssh -o BatchMode=yes",
                        f"ban@mac-mini:/Volumes/Graphide/ban/mic/mods/deep-time/smoke/{prof}/report.md",
                        os.path.join(rep, f"smoke-{prof}.md")], capture_output=True)
    for f in ("report-C16384.md", "report-C32768.md"):
        src = os.path.join(REPO, "review", "out", "p0-06", f)
        if os.path.exists(src):
            shutil.copy(src, os.path.join(rep, f"p0-06-{f}"))
    build_page(rows, started)
    print(f"page: {OUT}/index.html ({sum(1 for r in rows if r[2])}/{len(rows)} pass)")
    if not a.no_publish:
        subprocess.run([os.path.join(REPO, "tools", "review", "publish.sh"), "phase0", OUT, "-m", f"Phase 0 exit {started[:10]}",
                        "--emoji", "🌋"], cwd=REPO)


if __name__ == "__main__":
    main()
