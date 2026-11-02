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

The shape's area is counted on the samples (the visible continental samples and the hyper-extended
among them: between resamplings each sample is one lattice cell of area) and its outline on the
lattice (each lattice point shows the visible sample nearest it, a point with none within a
spacing is a gap and shows ocean floor, and the lattice's own edges join the points). A resampling
remaps the crust and is no process, so the outline should not move at the resamplings on average;
one that does (as PLT-664's walking margin did) is a numerical effect to trace. The area's change
at a resampling also holds what the resampling's step does beside the remap (the covered crust
accreted or consumed, the burial, the rate limit, and the rest of that step after it).

How the footing was found (2026-11-01, both arms measurement only, their census equal to
base38-s4's on every column):

- Arm `watch` (key b84b019b465c) joined the samples by their own links from the last resampling,
  kept while their ends were within two spacings. Plates move up to about 14 spacings between
  resamplings, so every link across a plate boundary broke: before each resampling the ocean floor
  counted one piece per plate (26,000 enclosed per resampling, t −27) and the margin lost its edges
  along plate boundaries (2,600 per resampling, t +20). Its areas, on the samples, stand: the
  continental samples +25.6 per resampling (t 3.6, 6 of 8 runs at |t| ≥ 3), the hyper-extended
  +24.5 (t 6.8, 8 of 8), over a run +1,458 and +1,396 of the planet's 100,000 (medians).
- Arm `watch2` (key 4a9ec135e1b9) counted everything on the lattice. Its outline stands: the
  smoothed margin −37 edges per resampling (t −1.8, 0 of 8), the enclosed floor −162 (t −1.5, 0 of
  8), the pieces +1.9 (t 2.2, 2 of 8); the raw edges −161 (t −5.3, 8 of 8) hold the gaps' rims
  below. Its areas do not: a gap point within a spacing of a continental sample shows it, so the
  points showing continental crust grow by the gaps' rims between resamplings (+20,640 over a run)
  and lose them at each (−15,718), about 400 points a 20-Myr cycle.

So the walk PLT-852 fixed stays fixed (the smoothed margin and the enclosed floor do not move at
the resamplings), and what still moves at them is the area of continental and hyper-extended
crust, by about 0.06 % of the continental crust and 0.4 % of the run's final hyper-extended crust
per resampling. Which part of the resampling's step adds
it is a read for the crust budget's parts (`tools/review/continent-budget.py`). The record's
`pieces` (`StepRec::pieces`, the largest continental pieces) still joins by the samples' links and
splits between resamplings as arm `watch` did.

Closer snapshots for a transient: `+DEEPTIME_SNAPSHOT_MYR=<every>:<from>:<to>` (Myr, the model's
clock, for example `2:-1000:-880`; several windows apart by commas) adds snapshots in that window to the 50-Myr ones, for the gap
budget, the crust budget's maps and the front trace (`plates::SNAPSHOT_ENV`, ledger PLT-283).
