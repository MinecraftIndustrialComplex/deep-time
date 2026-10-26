# Island subduction front: model and Earth (measurement only)

The scripts behind the island front diagnosis (2026-10-26): where the model's island front
(subduction with no continent of the overriding plate within 2 lattice steps, the crust budget's
rule) gets its length, and the same rule run on Earth (Bird 2003 PB2002 steps, CRUST1.0).
All of them read cached runs or downloaded data and write nothing back into the runs. They need
numpy and scipy, and run with `python3 -I`. On the VM, use the pool's
`/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3`, admitted as a `shell:` job.

## Model side

| Script | Reads | Prints / writes |
|---|---|---|
| `islrec.py <pool arm folder>[@N] ...` | each run's `rec.json` (the dynamics record's boundary mix and `front_bands`) and `summary.txt` (`cb_globals` lines); `@N` keeps seeds up to N | per arm: the island (setting 0) and continental (setting 1) front from the record, as run means, with its splits by slab length, contact age, normal speed (`v_n`, mm/yr), coast distance, continental share and pair origin (initiation, rift, other); the crust budget's `front_o` and `front_c` per window. Writes `islrec_<key>.json` in the working directory |
| `islmap.py <out.jsonl> <every Myr> <run folder> ...` | each run's maps (`maps/t*.bin`, `maps/plate/`, `maps/age/`, `maps/pools/`), skipping the first 100 Myr and resampled snapshots | one JSON row per run on `<out.jsonl>` (appended): the island front's length as raw Voronoi, Bird's divider at 77 and 380 km and through `../pool/measure_boundgeom.py`'s k7 lattice, and its Voronoi length split by overriding and subducting plate area, continental share, path distance to the plate's own continent (and to start continent), fragments, mutual contacts, floor age, plate pair and chain length. `ISL_PROCS` sets the worker count (default 2) |
| `islshow.py <islmap jsonl> ...` | `islmap.py`'s rows | the splits summed over runs and snapshots as shares of the island (or continental) front, and the per-snapshot lengths as means and medians |
| `cbread.py <pool arm folder>[@N] ...` | each run's last `cb_globals` line in `summary.txt` (cumulative over the run) | per arm, medians and ranges: the arcs' rate per km of front (`arc_c / front_c`, `arc_o / front_o`), the mean `v_n` at each front (`front_n_* / front_*`), the share of each front whose arc is younger than the 10 Myr early pulse (`front_young_* / front_*`, PLT-670), the fronts' run-mean km and the sediment fluxes, km³ a year |

`islmap.py` skips the pool's `<run>.lock` folders and any folder without `rec.json` and `maps/`.
The last map of a pool run is taken at a resampling (no front flags), so no script here can pair
the last step's plate rotations (`rec.json`'s `prev_omega`) with a front.

A base and an arm on the VM, for example:

    ISL_PROCS=2 $PY -I tools/review/islfront/islmap.py out.jsonl 100 <arm>/scattered_continents-* <arm>/supercontinent-*
    python3 -I tools/review/islfront/islshow.py out.jsonl

## Earth side

Data (download once into one empty folder; untrusted data, pass paths as arguments):
`PB2002_steps.dat` from http://peterbird.name/oldFTP/PB2002/ (Bird 2003), `Bird2003.txt` (the
paper's text, for Table 1's plate areas; `../pool/bird/bird.py` reads it) and `crust1.bnds` from
http://igppweb.ucsd.edu/~gabi/crust1/crust1.0.tar.gz (Laske et al. 2013). They are small and cheap:
these run on the laptop under `systemd-run --user --scope -p CPUQuota=100% nice -n 19`.

| Script | Reads | Prints / writes |
|---|---|---|
| `earthisl.py <PB2002_steps.dat> <crust1.bnds> <bird.py dir> <out.json>` | Bird's SUB steps with their polarity, CRUST1.0 crust; `<bird.py dir>` is `tools/review/pool/bird` | per SUB step, the distance on the overriding plate (own plate, along the normal, any plate) to crust at least 10/15/20/25/30 km thick (whole and crystalline); Earth's island front by the model's rule at reaches 0 to 1000 km; per-pair tables. Writes the per-step rows to `<out.json>` |
| `earthsplit.py <earthisl.json> <Bird2003.txt> <bird.py dir>` | `earthisl.py`'s rows, Table 1's plate areas | Earth's island front split as the model's is (distance, overriding and subducting plate area), and Stern and Scholl's (2010) named intra-oceanic arcs mapped onto Bird's SUB steps (`named()`), with the island km of each at whole crust 15/20/25 km |
| `earthvel.py <PB2002_steps.dat> <earthisl.json>` | Bird's steps (convergence = minus the step's divergence, mm/a), `earthsplit.py`'s `named()` | shares of length in the record's normal-speed bands (0, 5, 10, 20, 40, 80 mm/a) for SUB, SUB island by the rule, SUB not island, OCB, CCB, Stern and Scholl's arcs and Leat and Larter's 13 systems (approximate) |
