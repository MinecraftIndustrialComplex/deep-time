# The shared test-run pool

Ben, 2026-10-22: "If both deltas and continent spread need to do a world gen to test they should just
merge their test. To my understanding the current #1 bottleneck is the testing speed, not agents
programming."

Every lane that judges a change to the simulation runs the same kind of world-gen ensemble (a preset, a
length, a few starts and seeds) and reads the same kind of numbers off it. The pool makes a run a thing
that is made once: a run is named by what produced it, its outputs are kept, and the next request for the
same run (from another lane, another day) finds it. Lanes that need a world gen together run it once, as
one combined arm, and read each lane's measures off it.

Everything heavy runs on the test VM through its admission tool (`deeptime-admit`): the simulation, the
plate check, the field dump, the cargo build and every measure are admitted one by one. The laptop only
computes a git hash, starts the VM job detached and waits for it.

## Commands (laptop, from any Deep Time checkout)

```
tools/review/pool/pool.sh run <arm> [--key K] [--preset P] [--length Myr] [--starts a,b] [--seeds 1,2,3]
                                    [--threads T] [--set k=v]... [+ENV=v]... [--measures a,b] [--keep-ck]
                                    [--retry] [--rerun]
tools/review/pool/pool.sh table --base <arm> --arm <arm> [--arm <arm> ...] [--measure v1census] [--json FILE]
tools/review/pool/pool.sh measure [--arm <arm>]... [--measures a,b]     a new or changed measure over cached runs
tools/review/pool/pool.sh alias <new key|HEAD> <old key> --check-seed S [--preset P] [--length Myr]
                                    [--start a] [--threads T] [--set k=v]... [--all-arms]
tools/review/pool/pool.sh combine <name> <branch> [<branch> ...]        merge unmerged branches, build, print the key
tools/review/pool/pool.sh forget <name>                                  delete the throwaway branch pool/<name>
tools/review/pool/pool.sh key [--ref REF]                                this checkout's source key
tools/review/pool/pool.sh arms | status | gc [--days N] [--apply] | keep <key> [reason] [--remove]
```

Defaults: `--preset earthlike_quick_16k --length 1000 --starts supercontinent,scattered_continents --seeds
1,2,3 --threads 2`, measure `v1census`. `--set k=v` is the CLI's own `--set`; `seed` is the pool's axis
(`--seeds`), not a `--set`. `+NAME=value` is an environment variable for the runs (`@RUN@` is replaced by the
run's work folder). Arguments are plain words: mac.sh joins them into one line, so no spaces or shell
characters (pool.sh refuses them).

`run` without `--key` uses this checkout's HEAD. The files the key covers (below) must be committed:
the key is a hash of what is in git, and the command refuses on a dirty tree rather than name a run after
code that is not the code that ran. The checkout is synced to the VM by mac.sh as usual; the binary is
built there once per key.

An arm is a name for a set of runs with one key, preset, length and argument set; `run` records it under
`pool/arms/<name>.json` on the VM, and a second `run` of the same arm name with more seeds adds to it.
`table` reads arms by name, so a lane can tabulate its arm against `base` made by someone else.

Run it twice at once, from two shells, or from two lanes: the second request waits on the first run's lock
and reuses it (`reused after waiting 412 s for another request`). A run that exists is never run again; a
run that fails leaves `<start>-<seed>.FAILED` (the tail of the summary in it), not a half folder, and `--retry`
runs it again. `--rerun` deletes the cached runs asked for and runs them again.

## The source key

`srckey.py key` prints 12 hex digits: a hash of the git blob of every file the simulation reads, plus the
text of `sources.list` (comments stripped). It is computed from git (`ls-tree` on the laptop, `ls-files -s` on
the VM's synced copy) and the two agree. What it covers is `sources.list`: the five crates the CLI links
(cli, core, format, gpu, noddy), the root Cargo files and toolchain pin, `.cargo`, the viewer's `ramps.json`,
and the four data folders compiled into the binary (scatter, essentials, deposit_type, rock_class). `*.md`
inside those is excluded. A Java-only, pack-only, docs-only or tools-only commit keeps the key, so
the runs under it are found again.

The key fails closed. `members` makes every crate and every neoforge data folder name itself an include
or an ignore, so a new crate or data folder fails the key until someone says which it is; and every
`include_str!` or `include_bytes!` literal path in an included `.rs` file must resolve inside the covered set.
If the key fails, fix `sources.list` (a line, with a reason), and nothing else in the pool changes.

`pool.sh alias <new> <old> --check-seed S` is for the opposite error, a commit that changes a covered file
without changing what a run does (a comment, a rename, a refactor). It runs one seed under the new key,
takes the old key's same run (cached, or run now), and compares the two final checkpoints bit for bit
(sha256 of the `.ck`). If identical, it records `new -> old` in `aliases.json`, and the new key's arms
resolve to the old key's folders from then on, for the verified arm (preset, length, argument set) only.
`--all-arms` extends it to every arm, for a change you know cannot reach the arm you did not test. If the states
differ the alias is not recorded, which is the right answer for a real change.

## Layout on the VM

```
/data/ban/deeptime-runs/pool/
  <srckey>/deeptime.bin                           the CLI built from that key (once, under a flock)
  <srckey>/meta.json                              built when, from which checkout, branch, commit, note
  <srckey>/used                                   touched on every use (gc reads it)
  <srckey>/<preset>-<length>/<argkey>/args.json   the sorted --set, +ENV list, threads of that arm
  <srckey>/<preset>-<length>/<argkey>/<start>-<seed>/
      meta.json          what ran, when, how long, ck_sha256, fields_sha256, measures and their versions
      summary.txt        the stage's own summary
      plate.json, plate-table.txt, rec.json     plate-check's output and the dynamics record
      fields/            the dump of the final checkpoint (index.json, globals.json, <field>.f32)
      measures/<name>.json                      one file per measure and version
  <srckey>/<preset>-<length>/<argkey>/<start>-<seed>.lock / .FAILED
  arms/<name>.json   aliases.json   KEEP   .pyenv-numpy
  BASES.md           the lanes' ledger of their earlier ensembles (one line each); the pool never touches it
```

`argkey` is 8 hex digits of the sorted `--set` list, the sorted `+ENV` list and the thread count. A run
is therefore named by (source key, preset, length, start, seed, sets, envs, threads), which is everything
that can change its bytes.

Each run's final checkpoint (`final.ck`, about 0.8 GB) is deleted after the measures unless `--keep-ck`.
What survives is a few MB: the dump of the fields below, the plate-check output and the measures. A measure
that needs a field outside the dump needs `--keep-ck` (or `--rerun` once, with the field added to its
`fields` list in the registry).

The dumped fields are the registry's `dump_fields` (elevation, the surface and plate elevations, crust
thickness and type, the orogeny type and age, ice thickness, ELA, warm and mean temperature, Koppen,
regime) plus any `fields` a measure lists.

## Measures

A measure is a name, a version, and a command that reads one run folder and prints one JSON object of
numbers on its last line. The pool caches it as `measures/<name>.json` with the version beside it. The
registry is `measures.json`; `v1census` ships in it (`measure_v1census.py`, built on
`tools/review/continents_census.py`): land, largest landmass, continental crust under the sea, land above 2 km,
coast with high ground, land under ice, collisions per 100 Myr, the continental crust's net change over the
run, and the sea over the datum (median after 100 Myr and at the end). It reproduces the numbers of
`continents_census.py fields` on the high-ground lane's base dumps (land 0.2788, under the sea 0.2477, above
2 km 0.3609, coast with high ground 0.480 on scattered_continents seed 2).

To replace or add a measure, write a registry file with `measures` entries (and `dump_fields` if it needs
more), and pass it with `--registry FILE` or `DEEPTIME_POOL_MEASURES=FILE` (a colon-separated list); an entry
replaces the shipped one of the same name. The high-ground lane's driver goes in this way. Then
`pool.sh measure --arm <arm> --measures <name>` computes it over the cached folders without a simulation.
Raise `version` when a definition changes: the pool recomputes folders that hold an older one and
leaves current ones alone. In the command, `{run}` is the run folder, `{tools}` this folder and `{py}` the
Python of the `python` environment the measure names (`numpy`: numpy and scipy, built once by nix and rooted
under the pool).

`table` renders the measure's columns as plates.md's tables are written: a target row, each arm's median
per column with the two-sided Mann-Whitney U p against the base arm in brackets, the criteria met per run
as `median, mean (p)`, and the pair column `median / median`. `--json FILE` also writes every arm's per-run
values, for anything the table does not show. The p is `scipy.stats.mannwhitneyu`'s (exact when the smaller
sample has 8 runs or fewer and no ties, the corrected normal approximation otherwise; `selftest.py` checks
3,360 cases against scipy).

## Testing two lanes together

Two lanes that each need a world gen (say A and B) do not run two:

1. The base, once: from a checkout at master, `pool.sh run base --seeds 1,2,3,4`. If it exists under master's key
   already, this returns at once.
2. Each lane's own arm, from its own committed checkout: `pool.sh run A ...` and `pool.sh run B ...` with
   the same preset, length, starts and seeds.
3. The combination: `pool.sh combine AB <branch-A> <branch-B>` (merges both onto origin/master in a scratch
   worktree, stops and reports on a conflict, never resolves one; builds on the VM; prints the key), then
   `pool.sh run AB --key <key> ...` with the same arguments, then `pool.sh forget AB`.
4. `pool.sh table --base base --arm A --arm B --arm AB`.

Read each measure's three changes: dA = median(A) - median(base), dB likewise, dAB = median(AB) -
median(base). If dAB is within the seed spread of dA + dB (the spread of the base's seeds in the
`--json` values, or the arms' own min and max), the lanes do not interact: the combined arm is the evidence
for both, and neither lane needs a second run to land. If it is outside, the lanes interact on that
measure, and the combined arm is the result to judge (the later lane to land reruns on the merged
master). 3 or 4 seeds per start is the least that gives the table a p worth reading; the Mann-Whitney U on
3 against 3 cannot go below 0.1, so with 3 seeds judge by the sizes and the spread, not by p alone.

## Housekeeping

- `pool.sh status`: disk free, each key's size, runs, days unused, BUSY when a run holds a lock, aliases.
- `pool.sh gc` is a dry run: it lists the keys unused for 3 days (`--days N`) that it would delete, and why it
  keeps the others (`keep ...`, like mac-gc.sh). `--apply` deletes them, their arms and their aliases.
  `pool.sh keep <key> [reason]` writes the key into `pool/KEEP`, which gc never deletes (also the alias target
  of a kept key); `--remove` unkeeps. A key counts as used when a run, build, table or alias touches it.
- A run needs about 1.2 GiB while it runs and refuses to start with under 6 GiB free on the pool's disk.
- `combine.sh` leaves nothing behind on a conflict (the scratch worktree and branch are removed, exit 3).
  After a successful build the scratch worktree and its VM copy are removed; the branch `pool/<name>` stays
  until `pool.sh forget <name>`. Nothing is pushed and master is untouched.
- The pool's own tests: `selftest.py` (Mann-Whitney against scipy), run with
  `tools/remote/mac.sh -- /data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3 -I tools/review/pool/selftest.py`.

## Files

`pool.sh` (laptop entry), `pool.py` (VM-side orchestrator), `srckey.py` and `sources.list` (the key),
`pooltable.py` (table and Mann-Whitney U), `measures.json` and `measure_v1census.py` (measures), `combine.sh`,
`selftest.py`.
