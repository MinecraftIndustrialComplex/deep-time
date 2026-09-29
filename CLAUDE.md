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
4. docs/research/notes/*.md — the research notes.
5. docs/plan-reviews/ — the cold-read reviews of the plan and their resolutions.
6. /home/greencheetah/Projects/mic/Docs/datapacks/deeptime-mic-rock-tags/README.md — rock vocabulary
   (MIC's datapack, outside this repo; MIC ships it as pack/kubejs/data/deeptime/).

## Layout
crates/deeptime-format   world-data format (the contract with Java)
crates/deeptime-core     simulation (no Minecraft, no JNI)
crates/deeptime-jni      cdylib; the only crate using `jni`
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
./gametest.sh [-PwithMic|-PwithTerraBlender|-PwithEv]   # GameTest server (+ the test dimensions)
./gametest-planet.sh [-PwithMic]                        # dev server with a real Deep Time + TW overworld
tools/smoke/install.sh <standalone|mic|terrablender> [--size N] [--synth K] && python3 tools/smoke/run.py --profile <p>
tools/client/mac-screenshots.sh [--name N] [--no-guidance] [--no-create]   # real client on the Mac's desktop → review/out/N/
tools/review/publish.sh <name> <folder>                 # private spacesheep page (plan §2.6)
tools/viewer/mac-watch.sh [--preset P --start S --length-myr L] [--tv] | stop | status   # workbench on the Mac → http://m6-mac-mini:8642/ (tailnet only)

## Rules
- NixOS: nothing global. Run builds, tests and every `cargo` through `nix develop -c` (the profile's
  `cargo` is a bare rustup shim); one-off tools via `nix shell nixpkgs#<pkg> -c <cmd>`. Stage new files
  with `git add` before Nix reads them (flakes see only tracked files). Never `nix eval` a whole
  configuration set.
- The shell's `ls` is an alias that can hang in tool calls; use `command ls`.
- **Heavy jobs run on Ben's M6 Mac mini, not the laptop** (his rule, 2026-09-28): every cargo and
  Gradle build, GameTest, smoke/planet server and BlueMap render goes through
  `tools/remote/mac.sh [--pack] [--pull <path>]... -- '<command>'`, which rsyncs this checkout to the
  Mac's external SSD (`/Volumes/Graphide/ban/mic/…`; /nix, Gradle and cargo caches are there too)
  and runs the command in a login shell there. The laptop edits, commits and pushes. Builds from
  several agents may overlap; anything that boots Minecraft (GameTests, planet/smoke servers,
  BlueMap) must pass `--server`, which holds the Mac's single server lock (fixed ports, 16 GB shared).
- **Real-client runs** (booting a Minecraft client for screenshots) go to the Mac's logged-in desktop
  session too, never the laptop's Xvfb (Ben, 2026-09-28). That desktop is shared with graphide's
  agents: take a "drive" lease with graphide's `utilities/scripts/mactest/m6lock.sh` first.
  `tools/client/mac-screenshots.sh` does all of it from the laptop (sync and build, Deep Time's server
  lock, the drive lease, the launch in the desktop session, the PNGs back in review/out/<name>/);
  the ScreenshotDriver's walk is the model for any other client check. The old laptop route
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
- Decisions: ask Ben (AskUserQuestion, at most four questions per call, recommended option first)
  before anything that outlives the session, unless he said to work autonomously. While a question
  is open, list it under STATUS.md "Pending questions". When he answers, append an entry to the
  decisions file's log (date, question, his answer verbatim), update any table row above it that the
  answer changes with "(see log YYYY-MM-DD)", point the plan's §14 row at the log, and remove it from
  STATUS. Never rewrite or delete earlier log entries or other parts of the decisions file.
- Implementation details inside a task (class names, file splits, calibrated tolerances) are yours to
  decide; record them in STATUS.md's build log.
- Sub-agents: pass `model` explicitly. Sonnet for lookups (jars, javap, versions, licences); Opus for
  code and design. No Fable.
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
