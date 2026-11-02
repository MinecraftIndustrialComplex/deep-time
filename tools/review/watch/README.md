# Watching a run's behaviour (measurement only)

Ben, 2026-11-01: "are you actually analyzing the behaviour of runs while they are going to see if
there is anything that looks off?" Two views of a run beside its end numbers: pictures of every
snapshot, and the continental crust's shape at every step with the resamplings' part split out.
Both read cached pool runs and write nothing back into them. They need numpy (and scipy for the
pictures) and run with `python3 -I`; on the VM, the pool's
`/data/ban/deeptime-runs/pool/.pyenv-numpy/bin/python3`, admitted as a `shell:` job.

| Script | Reads | Prints / writes |
|---|---|---|
| `timelapse.py <run folder> <out dir> [W]` | the crust budget's maps (`maps/t*.bin`, `maps/pools/`, `maps/plate/`; a run made with `+DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps`) | one PNG per snapshot, `t-0950.png` and so on: three equirectangular panels W x W/2 (default 720), the crust's class (ocean floor by depth; the start's crust, island-arc crust, collision belts, the other converted crust and the hyper-extended crust over them, by `../freeboard/arcstock.py`'s classes), the height against that step's sea, and the plates (continental crust brighter). Prints each snapshot's continental and hyper-extended shares and the sea |
| `shape_jumps.py <arm folder>[@N] ...` | each run's `rec.json`, the record's `shape` and `shape_pre` per step (`dynamics::Shape`, runs made since 2026-11-01) | per quantity (continental points, hyper-extended points, margin edges raw and smoothed, continental pieces, enclosed ocean floor): its start and end, the change at the resamplings and between them, the mean change per resampling and its t; medians over runs and the runs with \|t\| ≥ 3 |

The shape is counted on the lattice: each lattice point shows the visible sample nearest it, a
point with none within a spacing is a gap and shows ocean floor, and the lattice's own edges join
the points. A resampling remaps the crust and is no process, so the quantities should not move at
the resamplings on average; one that does (as PLT-664's walking margin did) is a numerical effect
to trace.

The first watch (2026-11-01, arm `watch`, key b84b019b465c) joined the samples by their own links
from the last resampling, kept while their ends were within two spacings. Plates move up to about
14 spacings between resamplings, so every link across a plate boundary broke: before each
resampling the ocean floor counted as one piece per plate (26,000 points enclosed per resampling,
t −27) and the margin lost its edges along plate boundaries (2,600 per resampling, t +20), and
each resampling joined them again. Those columns measured the lattice's age, not the crust; the
continental and hyper-extended counts (+26 and +25 points per resampling, t 3.6 and 6.8) did not
depend on the links but did count visible samples, which gaps and overlaps between resamplings
change. The record's `pieces` (`StepRec::pieces`, the largest continental pieces) still joins by
the samples' links and splits between resamplings the same way.

Closer snapshots for a transient: `+DEEPTIME_SNAPSHOT_MYR=<every>:<from>:<to>` (Myr, the model's
clock, for example `2:-1000:-880`) adds snapshots in that window to the 50-Myr ones, for the gap
budget, the crust budget's maps and the front trace (`plates::SNAPSHOT_ENV`, ledger PLT-283).
