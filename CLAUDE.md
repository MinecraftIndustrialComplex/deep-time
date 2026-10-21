# Deep Time (`deeptime`)

A NeoForge 1.21.1 mod with a Rust simulation core: it simulates a planet's geologic history, shows it
as a timelapse on the Create World screen, and plays the planet as a wrapped world with
geology-driven terrain, strata and ore. LGPL-3.0-or-later. Private until the MVP (end of Phase 1):
MinecraftIndustrialComplex/deep-time.

## Start of every session
1. Read STATUS.md: the current task, the build log, pending questions, known issues.
2. Read, in this order: docs/deeptime-design-decisions.md **including its log at the end** (newest
   entries win); the plan's §1 and §2; the phase you are on (§10) and your task's work item plus the
   sections it cites; every §14 row whose "Needed by" names your task (ask those first).
3. Check for peers before touching anything: `git status`, `git worktree list`,
   `ps -ef | grep -i claude`. Say what you found. Never `git stash`.
4. Set your task to `doing` in STATUS.md with the date and your session id (the UUID in your
   scratchpad path); commit and push that line.

## Authority and where things are (all in this repo)
1. docs/deeptime-design-decisions.md — Ben's decisions. Its last section, "Decided during
   implementation (append-only log)", holds every answer given after the Q&A; a newer entry wins.
2. docs/deeptime-implementation-plan.md — the plan (§1.3 is the reading list; §10 the tasks).
3. docs/research/geologic-history-worldgen-report.md — the research report (§1.2 of the plan lists
   the positions it held that were overturned).
4. docs/research/notes/*.md — the research notes (index: docs/research/README.md).
   docs/science/ — the science documentation: one page per feature, the claims ledger
   (claims.md: every constant and law in the code, its source and whether it was checked) and the
   bibliography. See the rule "Science documentation is part of done".
5. docs/plan-reviews/ — the cold-read reviews of the plan and their resolutions.
6. /home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/README.md — rock vocabulary
   (MIC's datapack, outside this repo; MIC ships it as pack/kubejs/data/deeptime/).

## Layout
crates/deeptime-format   world-data format (the contract with Java)
crates/deeptime-core     simulation (no Minecraft, no JNI)
crates/deeptime-jni      cdylib; the only crate using `jni`
crates/deeptime-cpuprobe CPU-level probe cdylib (its own JNI class) that picks the baseline or x86-64-v3 native
crates/deeptime-cli      `deeptime` binary: run, stage, synth, bench, inspect-column
crates/deeptime-viewer   HTML viewer bundles
crates/deeptime-inspect  renders Minecraft region files (dev tool)
neoforge/                the mod (Gradle 8.11.1, ModDevGradle 2.0.74, NeoForge 21.1.249, Java 21)
tools/                   smoke/, bluemap/, review/, natives/, ci/, validation/ (GPL oracles, never shipped)

## Commands (from the repo root; everything through `nix develop -c`)
nix develop -c cargo test --workspace
nix develop -c cargo clippy --workspace --all-targets -- -D warnings
nix develop -c cargo deny check licenses bans
cd neoforge && nix develop -c ./gradlew build test && python3 ../tools/ci/junit-summary.py build/test-results/test
./gametest.sh [-PwithMic|-PwithTerraBlender|-PwithEe|-PwithEv]   # GameTest server (+ the test dimensions; Ee: Expanded Ecosphere)
./gametest-planet.sh [-PwithMic]                        # dev server with a real Deep Time + TW overworld
tools/smoke/install.sh <standalone|mic|terrablender|ee> [--size N] [--synth K] && python3 tools/smoke/run.py --profile <p>
tools/client/mac-screenshots.sh [--name N] [--no-guidance] [--no-create]   # real client on the Mac's desktop → review/out/N/
tools/remote/mac.sh -- tools/remote/detach.sh start <job> --kind client:editor tools/client/vm-client.sh --name N [--prop length=300]   # the Create World walk on the test VM (Xvfb, llvmpipe) → review/out/N/ (index.html)
tools/review/publish.sh <name> <folder>                 # private spacesheep page (plan §2.6)
tools/viewer/mac-watch.sh [--preset P --start S --length-myr L] [--tv] | stop | status   # workbench on the Mac → http://mac-mini:8642/ (tailnet only)
tools/remote/mac-gc.sh [--dry-run]                      # after merges: delete merged worktrees' Mac copies
python3 ../../pack/tools/gc/pack-gc.py [report|--dry-run]  # the laptop side: Rust targets and spent agent worktrees of every pack repo (daily timer)
tools/remote/mac.sh --no-sync -- deeptime-admit status  # the VM's admission tool: what runs, what waits and why, the estimates

## Rules
- **The test machine is the Azure VM `deeptime-test`** (Ben, 2026-10-01; D32as_v7, 32 vCPU, 125 GiB,
  x86_64 Ubuntu 24.04, West US 2, resource group `deeptime-test`, pay-as-you-go). Rebuilt from scratch 2026-10-16 after the resource group was deleted (Ben: "feel free to bring
  it back up"); the scripts reach it by the DNS name `deeptime-test-ban.westus2.cloudapp.azure.com`,
  which a rebuild can keep, and the gate batch is `tools/remote/vm/gate-batch.sh`. The `tools/remote/`
  scripts and `tools/viewer/mac-watch.sh` default to it (`DEEPTIME_MAC` overrides); its data disk is
  mounted at `/data` with `/Volumes/Graphide -> /data`, so every `/Volumes/Graphide/ban/...` path in
  this file and in tools/ holds there. Where this file says "the Mac", read this machine.
  **Never run Deep Time jobs on the M6 Mac (mac-mini) again**: our load made graphide's ms-precision
  timing tests flaky. The subscription also holds graphide's production: touch nothing outside the
  `deeptime-test` resource group. The box is ours but shared by every agent, and **every heavy job
  goes through its admission tool** (Ben, 2026-10-03: "No set slots, instead we should have a system
  of: Guess load, run if load is less than what is available + 25%, (measure load while running to
  fine-tune future guesses)."): `tools/remote/vm/admit.py`, installed as
  `/usr/local/lib/deeptime/admit.py` (`deeptime-admit` on PATH). There are no fixed slots, no
  per-agent counts and no fixed job sizes. A job has a kind (`cargo:test`, `gradle:build`,
  `gametest:mic`, `preview:standalone`, `sim:<preset>:<threads>t`, ...); its estimated load (mean
  cores, peak memory: a seed table at first, then a high percentile of the kind's measured runs) must
  fit what is free now or it waits in order and says why. Memory (Ben, 2026-10-03: "Don't kill old
  things, just prevent new ones from launching if their usage isn't within it ... If we are at 3.5Gb
  and a run is predicted to use 200mb it is green to run. If an agent predicts it will use 0.4 [GB]
  then it will be red since that would use up 100% of the machine after margin is applied."): a job
  starts when its memory estimate x 1.25 is under the free memory, where free = MemAvailable minus
  what admitted jobs have reserved and not used yet; the margin is on the job's prediction, there is
  no standing headroom, and running jobs are never stopped (a 4 GiB box with 0.5 GiB free: a 0.2 GiB
  job runs, a 0.4 GiB job waits). CPU is unchanged (estimate fits in 1.25 x the free cores, 4 cores
  kept for the play server, the workbench and ssh, since overcommitting CPU only slows jobs down); a job that has waited 15 minutes stops anything behind it from starting
  first. Its process tree is measured while it runs and refines the next estimate. mac.sh admits
  every command (`--kind` overrides the kind it classifies; quick looks and detached starts run at
  once); `tools/remote/detach.sh start`, `gametest.sh`, `gametest-planet.sh`, `tools/smoke/run.py`,
  `tools/workbench/preview.py` (so the workbench's previews) and `tools/client/dh-client-check.sh`
  admit themselves. A script of your own that starts several heavy steps itself wraps each:
  `deeptime-admit run --kind <kind|auto> -- <command>`. Minecraft servers run side by side: each
  server job gets its own ports (26000 + 10 per slot: game, RCON, query, voice chat), X display and
  folder (`deeptime-cache/servers/<job>`, removed when it ends), and runs in a private copy of its
  server set (`tools/smoke/instance.py`), never in `smoke/<profile>/server` itself; the play server
  (25765) and the workbench (8642) stay outside the pool. `mac.sh --server` is only for scripts that
  boot Minecraft without those launchers. The old `deeptime-cache/server.lock` (agents still on the
  old scripts) counts as one running server job while it exists. It bills by the hour while running
  (about $38 a day), so **it deallocates
  itself after 30 idle minutes** (Ben, 2026-10-03: "Auto-stop when idle"): the watcher
  `deeptime-idle.service` (`tools/remote/vm/idle-watcher.py`) counts as busy any job the admission
  tool has admitted or queued, any cargo/rustc,
  Gradle client, Java (except the idle play server and Gradle daemons), `deeptime` other than the
  workbench, nix/Python/rsync process, any detached shell script (`setsid nohup bash …`, detach.sh
  jobs), any ssh session running a command (not the laptop's `ssh -N` tunnels), the old server lock, a
  workbench run or preview in progress, a player on the play server, or load above 4. Keep a
  long job visible to it: run it as a process or detached script on the VM, never as a laptop-side
  loop that only ssh-polls now and then. `cat /data/ban/deeptime-cache/idle-watcher.state` says
  what it sees (`idle-watcher.last-busy`, `idle-watcher.log`, `sudo python3
  /usr/local/lib/deeptime/idle-watcher.py --once` for the full list); `touch
  /data/ban/deeptime-cache/KEEP_AWAKE` holds it up (remove it when done). **The remote scripts start
  it again by themselves**: `tools/remote/vm-access.sh` (sourced by mac.sh, mac-gc.sh, mic-climate.sh,
  mac-watch.sh) finds it deallocated, runs `az vm start` and waits for SSH (it prints "STARTING the
  test VM"; 2 to 4 minutes). A plain `ssh` does not: use `tools/remote/mac.sh --no-sync -- true`
  first. The workbench (`deeptime-workbench.service`) and the play server (`deeptime-play.service`)
  are systemd units that start at boot (`tools/remote/vm/install.sh` installs all three; restart
  the workbench only with no run in progress, `sudo systemctl restart deeptime-workbench`, or
  `tools/viewer/mac-watch.sh` for a new binary). The laptop's tunnels to them
  (`deeptime-workbench-tunnel`, `deeptime-play-tunnel`) retry every 2 minutes while the VM is down;
  `tools/remote/tunnels.sh` recreates them after a laptop restart. The VM's own managed identity
  holds one custom role, "Deep Time Test VM Self Deallocate" (VM read + deallocate, assigned on this
  VM only).
  Its firewall allows SSH only from addresses this laptop has had; `tools/remote/vm-access.sh` (called
  by the remote scripts) adds the laptop's current address to that one rule when the VM stops
  answering (Ben, 2026-10-01: "Auto-update the rule"). A plain `ssh` that times out means the VM is
  deallocated or the address changed: run any `tools/remote/mac.sh --no-sync -- true` to repair it.
- **No backward compatibility before a public release** (Ben, 2026-10-01: "Until I have told you to
  do a public release, you are to give absoluteley 0 shits about backwards compatability"). Old bakes,
  checkpoints, saved runs, presets, world-data format numbers and older jars need not keep working:
  change formats and behaviour freely, write no migrations or compatibility shims, and do not bring
  compatibility questions to Ben. This holds until he says a public release is being made.
- NixOS: nothing global. Run builds, tests and every `cargo` through `nix develop -c` (the profile's
  `cargo` is a bare rustup shim); one-off tools via `nix shell nixpkgs#<pkg> -c <cmd>`. Stage new files
  with `git add` before Nix reads them (flakes see only tracked files). Never `nix eval` a whole
  configuration set.
- The shell's `ls` is an alias that can hang in tool calls; use `command ls`.
- **Heavy jobs run on Ben's M6 Mac mini, not the laptop** (his rule, 2026-09-28): every cargo and
  Gradle build, GameTest, smoke/planet server and BlueMap render goes through
  `tools/remote/mac.sh [--pack] [--pull <path>]... -- '<command>'`, which rsyncs this checkout to the
  Mac's external SSD (`/Volumes/Graphide/ban/mic/…`; /nix, Gradle and cargo caches are there too)
  and runs the command in a login shell there, through the VM's admission tool (above). The laptop
  edits, commits and pushes. Jobs from several agents run side by side as their load allows;
  anything that boots Minecraft goes through the repo's launchers (gametest.sh, gametest-planet.sh,
  smoke run.py, preview.py, dh-client-check.sh), which take a server slot themselves, or passes
  `--server`.
- **Nothing heavy on the laptop, not even "just a script"** (Ben, 2026-09-29, after agents' local
  headless Chromium screenshot runs, Python sweeps and CFR decompiles drove it to load ~60 and swap):
  screenshots (playwright/Chromium), decompiles (cfr/javap over jars), jar greps, Python sweeps and
  analysis, image processing — all on the Mac via `tools/remote/mac.sh`, results pulled back with
  `--pull`. If something must run locally: one job at a time, wrapped as
  `systemd-run --user --scope -p CPUQuota=100% nice -n 19 <cmd>`. No parallel local batches; no
  `nix develop` builds on the laptop. **One exception: GPU speed benches** (Ben, 2026-10-18,
  decisions log): the VM has no GPU, so real-GPU timings run on the laptop's Radeon 8050S (Strix Halo, RADV; the driver's name for it),
  one bench at a time and only when no other local heavy job runs, with a binary built on the VM
  (never build here), 8 sim threads and
  `systemd-run --user --scope -p CPUQuota=800% nice -n 19 <cmd>`; compare GPU against CPU on the
  laptop in A/B pairs, never against VM numbers. GPU correctness tests stay on the VM (lavapipe).
- **Mac disk hygiene** (2026-09-30: the SSD filled to 100 % with ~60 worktree copies, each with its own
  target/; Ben: "Be more efficient about mac storage usage in the future"): after every merge run
  `tools/remote/mac-gc.sh`, which deletes the Mac copies of merged, idle worktrees and prunes stale
  incremental caches (`--dry-run` to preview). Scratch runs go under
  `/Volumes/Graphide/ban/deeptime-runs/<task>/`, never inside target/; use the smallest planet and
  fewest keyframes that answer the question, and delete them when the task lands. mac.sh builds with
  `CARGO_INCREMENTAL=0` and line-table debuginfo to keep each copy's target/ small; do not override
  that. Keep everything on the SSD, never the Mac's internal disk.
- **Real-client runs** (booting a Minecraft client for screenshots) go to the Mac's logged-in desktop
  session too, never the laptop's Xvfb (Ben, 2026-09-28). That desktop is shared with graphide's
  agents: take a "drive" lease with graphide's `utilities/scripts/mactest/m6lock.sh` first.
  `tools/client/mac-screenshots.sh` does all of it from the laptop (sync and build, a server job of
  the admission tool, the drive lease, the launch in the desktop session, the PNGs back in review/out/<name>/);
  the ScreenshotDriver's walk is the model for any other client check. Any other real-client tool
  (`tools/client/ev-assets.sh`) launches through the same Mac-side library, `tools/client/mac-client-lib.sh`
  (lease, desktop session, cleanup), and stops with exit 4 when nobody is logged in or the screen is
  locked; never type a password. The old laptop route
  (`tools/client/screenshots.sh`, HeadlessMC + Xvfb) refuses to run without DEEPTIME_XVFB_CLIENT=1;
  never set it on the laptop.
- Ben cannot launch the game. Every check must be runnable here and reviewable in a browser.
- LGPL-3.0-or-later. No GPL code, no line-by-line port of GPL code, GPL tools only in tools/validation/.
  Attribute every port in NOTICE.md and the file header (`SPDX-License-Identifier: LGPL-3.0-or-later`).
- The MIC jar set is whatever /home/greencheetah/Projects/mic/pack pins today: re-run
  `tools/smoke/install.sh mic` before any -PwithMic gate. Paths under mods/mic-climate are read-only
  and its smoke/ folder is a stale copy, never a reference.
- Land your work: commit, `git merge --no-ff` finished branches into master, push master. Catch up a
  behind checkout with `git merge --ff-only --autostash origin/master`.
- **Science documentation is part of done** (Ben, 2026-10-02: "as backed by science as possible";
  decisions log 2026-10-02). A feature is not done until its page in `docs/science/`, its rows in
  the ledger `docs/science/claims.md` and `docs/science/bibliography.md` are updated in the same
  branch. Every physical constant, law, threshold, rate or calibration target in code, presets or
  data files carries its ledger id in a comment beside it (`// ledger: IMP-012`), or a
  `game-choice` note saying why it is not physical and who decided. A value you could not check
  against its source is marked `from-memory` in the ledger and in the comment, never cited as if
  the source had been read; `verified` needs the source fetched and the value read, with what was
  read recorded in the ledger. Changing a ledgered value means changing its row. New pages start
  from `docs/science/_template.md`.
- Decisions (Ben's rule of 2026-10-16): take any choice that can be undone with the recommended
  option, log it as the coordinator's call and tell him in the final message. Ask Ben only what is
  his: architecture, public interfaces, data formats, money, anything outward-facing, big scope, and
  what he said he would decide himself (AskUserQuestion, at most four questions per call,
  recommended option first, in one batch, early, with other work still running). While a question
  is open, list it under STATUS.md "Pending questions". When he answers, append an entry to the
  decisions file's log (date, question, his answer verbatim), update any table row above it that the
  answer changes with "(see log YYYY-MM-DD)", point the plan's §14 row at the log, and remove it from
  STATUS. Never rewrite or delete earlier log entries or other parts of the decisions file.
- Implementation details inside a task (class names, file splits, calibrated tolerances) are yours to
  decide; record them in STATUS.md's build log.
- Sub-agents: pass `model` explicitly. Sonnet 5.5 (xhigh, never max) for lookups and for
  well-specified implementation, including follow-ups to finished work; Opus for design, hard
  diagnosis, cross-subsystem changes, data formats and review (Ben, 2026-09-30). A resumed agent
  keeps its model, so give a Sonnet-shaped follow-up to a fresh Sonnet agent, not a resumed Opus
  one. No Fable.
- **GameTests run once per merge batch** (Ben, 2026-09-30; the Mac's single server slot had ~7
  agents queueing two runs each): an agent on a branch runs cargo (test, clippy, fmt, deny) and
  `./gradlew build test`, not `./gametest.sh`. The coordinator merges a batch into master, then runs
  `./gametest.sh` and `./gametest.sh -PwithMic` once and fixes failures on master. Planet probes
  and smoke runs that a task needs for its own measurements run as usual (admitted like any job).
- Never commit smoke/, run/, run-gametest/, run-planet/, libs/, review/out/, rcon.pw or
  .spacesheep.json. Never redistribute Project Atmosphere.
- Licence acceptances Ben made (2026-09-28): Minecraft EULA for headless test servers, BlueMap
  accept-download, Microsoft CRT/SDK via cargo-xwin. Anything else: ask.
- Publishing: private spacesheep pages only (plan §2.6). Never `spacesheep login/connect/logout/update/
  share`, `--org` or `--new`, and never read ~/.config/spacesheep/config.json.

## End of every session
Add a build-log entry to STATUS.md (what changed, what was measured and with which command, anything
that contradicts the plan, the exact next step), update its header (next task, plan revision, pack
commit), commit and push master.

## Reference jars
NeoForge + Minecraft sources: neoforge/build/moddev/artifacts/neoforge-21.1.249-sources.jar (after any
Gradle build; `tools/dev/sources.sh` extracts it into the gitignored libs/neoforge-src/).
Current MIC server jars: smoke/mic/server/mods/ (this repo, gitignored; `tools/smoke/install.sh mic`).
