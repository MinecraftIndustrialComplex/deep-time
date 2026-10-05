# Playability tools: pick the epoch, continue, stop early, and end-weighted nudges

Status: **design, built on its branch** (2026-10-05). Ben's answers are the last two rows of the
decisions log of 2026-10-05 ("Which playability tools to build?" and the validation policy). In
short: realism comes first and playability second, and playability can be switched off; the
player can bake the world from any keyframe, continue a finished run past its planned length, stop
a run early and play it, and choose end-weighted nudges, which are on by default for the Earth-like
presets.

Statements are marked **measured** (a run made for this note; §8), **read** (from the code) or
**decided here** (an implementation choice of this task, recorded in STATUS.md's build log).

## Summary

- **One mechanism serves all three tools.** A run's *stop* becomes part of its preset
  (`timeline.stop_myr`, default: the planned length). Picking an epoch, stopping early and
  continuing all mean "the same run, with another stop". The run is resumed from a checkpoint taken
  before anything depended on where it ends, and stepped to the new stop. The result is
  bit-identical to a run planned with that stop from the start (tested, §9).
- **What depends on the end** is a short *tail*: the closing plate step, the climate's 3-Myr
  ice-age window (Milankovitch cycles, lagged ice sheets, the present-day adjustment), and, when
  steering is on, the nudges' ramp. Everything before the tail is the same whatever the stop.
- **Epoch checkpoints.** While it runs, a session writes a checkpoint at the last step before the
  tail of every keyframe (`<out>/epochs/`). Picking keyframe *k* resumes there and reruns only its
  tail. Continuing resumes at the start of the old tail. Nothing reruns from the beginning.
- **End-weighted nudges** act on two physical controls, never on outputs: the ocean's water volume
  (which moves sea level, toward a land-share band) and the volcanic outgassing rate (which moves
  CO₂, toward a global-mean-temperature band). Their weight ramps from 0 to 1 over the last part of
  the run (fraction and shape chosen by the player). Inside the band they do nothing. Default on
  for the Earth-like presets; `steering.enabled = false` is the realistic (free) choice and changes
  nothing in a run.
- **Cost.** Nudges are a few array sums per plate step (§8.3). Epoch checkpoints cost disk and a
  few seconds of writing per checkpoint (§8.2). Picking or stopping reruns the tail plus the end
  stages; continuing reruns the old tail plus the new span (§8.4).

## 1. The time axis today, and the stop

**Read.** A planet run starts at world time `t_start = −length_myr · 10⁶` years and ends at 0. The
plates stage takes `round(length / time_step_myr)` steps of `dt = length / steps` (2 Myr on the
built-ins). The climate, surface, volcano, impact and structure stages are coupled to it and step
with it. Absolute world time drives the slow physics: the star brightens with the planet's age
(`present_age_gyr + t / 10⁹`, Gough 1981), and the deposit types' secular clocks (land plants,
oxygenated oceans, the banded-iron acme) are dated the same way.

**Decided here.** The timeline gains an optional `stop_myr`: the Myr after the start at which the
run stops.

- Absent, it equals `length_myr`: the run ends at the present, as today.
- `stop_myr < length_myr` stops the run early. World time ends at `stop − length` (negative): the
  planet is played at that moment of its history, with the star as bright as it was then.
- `stop_myr > length_myr` continues it. World time ends after the present (`+200 Myr` for a
  500-Myr run continued to 700): the star is slightly brighter (about 2 % per 200 Myr) and the
  secular clocks stay where they are. Nothing in the bake assumes the end is at 0: every age the
  bake writes is measured from the run's own end (read: `meta/crust_age_myr`, `volcanoes/active.json`,
  the deposits' survival, the young landscape all use the world's current time).
- The step length stays the planned one, `dt = length / round(length / time_step)`. The stop is
  rounded to whole steps: `steps = max(1, round(stop / dt))`.

The start, and so the planet's age at every step, is fixed by `length_myr`. This is why **a run of
500 Myr continued by 200 is not the same planet as a run planned for 700 Myr**: the 700-Myr run
starts 200 Myr earlier, under a star 2 % fainter, from a starting state drawn for that moment, and
the plate model is chaotic (any difference grows). The continued run *is* bit-identical to a run
planned from the start as `length_myr 500, stop_myr 700` (tested). A player who wants a 700-Myr
planet from the start sets 700 on the setup page.

## 2. The tail: what depends on where a run ends

**Read.** Stepping is independent of the end except for these:

| What | Where | Span |
|---|---|---|
| The closing step: the plates' final resample, closing every contact and hotspot, the dynamics note; the surface's partial last frame and drainage receivers; the volcano stage's flood-basalt layer; the structure stage's folds and plutons becoming bodies; the climate's last snapshot (monthly fields for the bake) and `present_climate` | every coupled stage's `last` step | the last plate step |
| The ice-age window: Milankovitch orbit, lagged ice, the 2-kyr clock | `climate::clock::in_window`, `orbit_at` | the last `orbital_window_myr` (3 Myr) |
| The nudges' ramp (new) | `steering` | the last `tail_fraction` of the run |

**Decided here.** The window and the orbit are measured from the run's end, not from world time 0
(`in_window(t) = t > end − W`; the orbit series takes `t − end`). With the default stop this is
the code as it was. The tail of a run with stop *S* is

  τ(S) = max(W, f · S) if the climate runs (W = `orbital_window_myr`, f = `steering.tail_fraction`
  when steering is on, else 0), and 0 otherwise,

and its **last free step** k_free(S) is the last plate step k < steps(S) whose end time t_k is
at least τ(S) (plus a year of slack, the window test's own) before the end. The world after step
k_free(S) is the same for every stop whose own tail starts after it. That is the whole of the
determinism argument; the tests check it.

## 3. Epoch checkpoints

**Decided here.** With `SessionOptions::epochs` on (the game, the workbench, `deeptime run
--epochs`), the session writes a checkpoint after the coupled group's step k when k is the last
free step of:

- each keyframe time *t_j* of the run (for picking or stopping at that frame), and
- the run's own stop (for continuing).

Each goes to `<out>/epochs/step-<k>.dtcp` (the existing checkpoint format, written streamed and
atomically), and `<out>/epochs/index.json` lists them: step, world time, the stop and tail they
serve, bytes, and the preset key. The key is a CRC of the canonical preset with
`timeline.stop_myr`, the `steering` section and the `climate.present_*` keys taken out: those
only change the tail, so a checkpoint serves any preset that differs from its run only in them.

A checkpoint is written only at a free step of the run that writes it, so it never holds anything
the end shaped. A resume also takes no checkpoint later than the last free step of the run whose
keyframes the folder holds (the index names that run as soon as it starts): the frames that run
drew in its own tail are not frames of the shared trajectory, and a pick followed by a continue
back to the planned end gives the original run's files exactly (tested; found by the client walk). It serves a new stop *S'* when its step is at most k_free(S') under the new
preset (a stricter steering, a longer tail, falls back to an earlier checkpoint, or to the start).

The epochs folder is not world data: the manifest does not list it, and the staging mover deletes
it before moving a staging folder into a world (Accept ends what a player can still change).

**The recorded history inside each checkpoint.** A checkpoint carries the deposit engine's
recorded history (P1-04's frames, about 4–8 MB each on the quick 16k planet, one per 10 Myr).
Written whole into every epoch checkpoint, the history would make the epochs folder grow with the
square of the run's length. **Decided here: a chain.** The history only grows, so each epoch
checkpoint holds only the frames since the store's previous one (`CheckpointRef::write_from`; its
summary says where its frames start), and the index records each file's frame range
(`frames_from`, `frames_to`). Reading checkpoint *k* reads the body of *k* and the frame sections
of every file of its chain (`Checkpoint::read_chain`); the frames stay in their files and are read
on demand (`Record::File`). Every frame is stored once.

## 4. Pick the epoch

**What a player does.** On the run page, scrub to a keyframe and press **Play this epoch** (shown
"Stop here" while the run is going). The world is baked as it stands at that frame's time.

**How.** The editor starts a new session in the same staging folder with the effective preset and
`timeline.stop_myr` = the frame's time after the start (`Session::resume_epoch`, JNI
`resumeEpoch`). The session

1. reads `<out>/epochs/index.json`, picks the latest checkpoint with a matching key and a step at
   most k_free(S'), or none (then it starts over: the same as a fresh run);
2. resumes from it with the new preset (each stage is built with the new stop's step count; their
   saved counters carry on);
3. keeps the keyframes up to the checkpoint's time and deletes the later ones (files and index);
   it draws the later ones again as it passes them, numbered on from the kept ones;
4. steps the tail, then runs the end stages and the bake exactly as a run of that length does.

**What exists only at the end, and is re-derived for the chosen epoch:**

| Product | Stage | Re-derived how | Cost (quick 16k, 4 threads; §8.4) |
|---|---|---|---|
| The closing of contacts and hotspots, final resample, folds and plutons as bodies, the last climate snapshot with monthly fields, `present_climate` | coupled stages' last step | the tail's last step | one plate step plus the snapshot |
| The ice-age window's ice, sea level and glacial history | climate | the tail | W = 3 Myr: two plate steps at 2 kyr clock steps |
| The nudges' ramp | steering | the tail | f · S |
| Deposits (formation, survival, children) | `deposits` | runs on the recorded history up to the stop | as a run's |
| The young landscape and L3 amplification | `young`, `amplify` | from the stopped surface | as a run's |
| The bake: terrain layers, rivers and lakes, biomes, provinces, spawn, lava, impact lenses, placers | `finalize_write` | from the final world | as a run's |
| The timelapse's closing frames (deposits, young, amplify) and the last frame redrawn with the bake's water | session | drawn again | seconds |

## 5. Continue

**What a player does.** When a run is done, the run page shows **Continue to [N] Myr**. The run goes
on from where it was and stops at N; the timelapse keeps its frames and grows.

**How.** The same as picking an epoch, with S' > S. The checkpoint used is the one at the old
run's own last free step, k_free(S): the old tail (the window, the nudges' ramp, the closing step)
is run again as ordinary steps, without being a tail, which is exactly what a run planned to S'
does. A continue reruns τ(S) of the old run plus S' − S of new time. With steering off this is
3 Myr extra; with steering at f = 0.1 on a 500-Myr run, 50 Myr extra.

The game continues (and picks, and stops) with the run's own preset: only the stop changes. The
core also accepts other steering settings and present-day climate for a resume (they are outside
the preset key; `deeptime resume --set steering.…`), and a stricter steering then needs an earlier
checkpoint: the session takes the latest that serves it, or starts over.

**The workbench.** Its runs write epoch checkpoints every 25 Myr (a frame every step would be a
checkpoint every step). A finished run's card has **Stop / continue**: it asks for a stop and
queues a new run that hard-links the source run's checkpoints and keyframes into its own folder and
resumes from them (`POST /api/runs/<id>/resume {"stop_myr": N}`); the source run stays as it was.

## 6. Stop early

**What a player does.** While the run goes, the scrubber can go back to any keyframe already drawn;
**Stop here** stops the run and plays that frame's world. A player who likes the live frame stops at
it.

**How.** The running session is cancelled (it stops within one step), and an epoch pick at that
frame starts in the same staging folder (the editor's launch gate waits for the cancelled session
to let go of its memory). Its checkpoint was written when the run passed it. The staging folder is
not deleted on this cancel (the timelapse model is told the run was superseded).

**Decided here: stopping is at keyframes.** Stopping at the very step the run is on would need a
checkpoint τ before every step; keyframes are where the player looks, and each has its checkpoint.
The frame interval is the preset's `keyframes.every_myr` (50 Myr on the quick presets).

Stop early runs the tail, so the world played is the world of a run planned to that length,
steered at its end when steering is on (inside the targets, the nudges do nothing, so a planet the
player liked is a planet they get). With steering off it differs from the frame shown only by the
3-Myr window's ice-age climate and the closing step.

## 7. End-weighted nudges (steering)

### 7.1 What they act on

Ben: "physics comes from sources"; the nudges "nudge physical controls, never outputs". Two
controls, both terms the model already has:

- **Ocean water volume → sea level → land share.** The climate stage computes the eustatic sea
  level each plate step (the preset's sea-level curve plus glacio-eustasy; read:
  `climate::SEA_LEVEL_GLOBAL`). Steering adds an ocean-volume offset ΔV, as a change of the ocean's
  water (the deep water cycle: water taken into the mantle at trenches and given back at ridges and
  arcs). It raises the sea against the continents by ΔV / A_ocean · (1 − ρ_w / ρ_m): the ocean floor
  sinks under the added water (Airy hydro-isostasy), so the sea gains 0.69 of the water's
  thickness (ρ_w 1030, ρ_m 3300 kg/m³). The steered quantity is this sea-level offset; its volume is
  reported beside it.
- **Volcanic outgassing → CO₂ → temperature.** The carbon cycle's degassing (ridges, arcs, large
  igneous provinces; `carbon::degassing`) is multiplied by a factor e^x. The silicate-weathering
  feedback then moves CO₂ and the climate follows within about a million years (the weathering
  e-folding time, read: the clock's 200-kyr steps settle a step's imbalance within a few steps).
  Earth's outgassing rate is uncertain by about a factor of two over the Phanerozoic, which bounds
  how far the factor may go.

Neither acts on land share, temperature, ice or CO₂ directly; the world's state after a nudge is
one the physics reached.

### 7.2 The weight

Over the last `tail_fraction` f of the run (default 0.1) the weight w rises from 0 to 1:
x = (t − t_r) / (t_end − t_r), t_r = t_end − f · S, and

- `linear`: w = x;
- `smoothstep` (default): w = x²(3 − 2x), no kink at either end;
- `late`: w = x², most of the push near the end.

w = 0 before t_r, so the run before its tail is the unsteered run.

### 7.3 The controllers

Each plate step in the tail, after the climate's clock (the measured state is the step's end):

  sea: e_L = (L_lo − L) if L < L_lo; (L_hi − L) if L > L_hi; else 0 (L = land share, area-weighted)
       Δh = −w · r_sea · dt · clamp(e_L / 0.05, −1, 1)       (5 points of land share = full rate)
       h_steer ← clamp(h_steer + Δh, −h_max, +h_max)

  CO₂: e_T = (T_lo − T) if T < T_lo; (T_hi − T) if T > T_hi; else 0 (T = global mean, °C)
       Δx = w · r_co2 · dt · clamp(e_T / 5 K, −1, 1)
       x ← clamp(x + Δx, −ln F, +ln F)

Both are integrating controllers with a dead band (the target band) and a saturating rate: no
jump larger than r · dt per step, and no action inside the band, so a world that sits inside stays
untouched. Outside the band near the end the weight is largest. The sea offset is held, not
undone, when the land share comes back inside (water does not go back into the mantle when the
target is reached; nothing undoes it either).

Snowballs: a snowball's global mean is near −40 °C, far below T_lo; outgassing rises at the full
rate and CO₂ builds up faster, so the escape comes sooner. The escape threshold itself is the
climate's (decisions log 2026-10-02).

### 7.4 Targets and presets

`steering.target` names a band pair; `custom` takes the numbers from the section:

| target | land share | global mean | for |
|---|---|---|---|
| `temperate` (default) | 0.25–0.35 | 10–20 °C | Ben's example "temperate, ~25–35 % land"; Earth's preindustrial mean is 14.1 °C (decisions log 2026-10-02) |
| `warm` | 0.25–0.35 | 18–26 °C | a greenhouse world with no hothouse |
| `cool` | 0.25–0.35 | 6–13 °C | an icehouse world with no snowball |
| `custom` | `land_min`–`land_max` | `temp_min_c`–`temp_max_c` | |

All the bands are game choices (Ben's example, then this task's), not Earth claims. Earth's land
share is 0.29 and its Phanerozoic mean temperatures roughly 10–30 °C (ledger rows for both are
references, not targets).

### 7.5 Settings (`steering`, every built-in preset, editor fields, lang keys)

| key | default (Earth-like presets) | range | meaning |
|---|---|---|---|
| `enabled` | true | | off: the run is the unsteered run, bit for bit |
| `target` | `temperate` | temperate, warm, cool, custom | the bands |
| `tail_fraction` | 0.1 | 0.02–0.5 | f |
| `shape` | `smoothstep` | linear, smoothstep, late | the weight |
| `land_min`, `land_max` | 0.25, 0.35 | 0–1 | custom land band |
| `temp_min_c`, `temp_max_c` | 10, 20 | −50–60 | custom temperature band |
| `sea_rate_m_myr` | see §8.5 | 0–20 | r_sea at full weight |
| `sea_max_m` | see §8.5 | 0–1000 | h_max |
| `co2_rate_per_myr` | see §8.5 | 0–1 | r_co2 (ln of the outgassing factor per Myr) |
| `co2_factor_max` | see §8.5 | 1–10 | F |

A preset without a `steering` section runs unsteered (the core's default is off; tests and the
plates and pipeline presets). The four built-in Earth-like presets carry it on.

### 7.6 Later: scenario forcing (not built)

Ben floated forcing toward a chosen state: a snowball, a supercontinent. The same machinery covers
them, as targets of the same controllers plus one more control:

- **snowball / hothouse / ice age**: a target band of the temperature controller (T < −20 °C for a
  snowball), reached through outgassing alone; `present_climate` keeps its role for the last 100 kyr.
- **supercontinent**: a third controller on a weak bias in the plates' driving forces (the force
  balance of plate tasks 7–8, `plates::forces`): a small extra torque on each plate toward the
  continents' centroid, with the same weight and a dead band on the continents' dispersal (the
  mean great-circle distance of continental crust from its centroid). A bias on the forces, not on
  positions, keeps every collision and suture the plate physics'. It waits for the force balance
  to drive the plates (it is recorded, not applied, today).
- **Measured, reported, never gated**: whatever the target, the run reports the steered and the
  unsteered state (§7.7), so a forced world is labelled as one.

### 7.7 What a run reports

The climate note gains `steering`: the settings, the weight's start, the final sea-level offset
(m and km³, and as a share of the ocean), the final outgassing factor, the land share and
temperature at the start of the tail and at the end, whether each target was met, the artefact
counters (largest step of each control, reversals of direction) and one row per steered plate step
(`t_myr, weight, land_frac, t_global_c, sea_offset_m, ln_outgassing`). The climate's own series is
left as it was, so an unsteered run's notes are unchanged. The steering is part of a world's
provenance: the manifest's notes say a world was steered and how hard.

## 8. Measurements

### 8.1 Method

**Measured** on the test VM, 2026-10-05: `deeptime run` (release, this branch before the merge of
master 6489d4a5), quick 16k preset, scattered start, 500 Myr, 4 threads, seeds 1 and 2;
`tools/review/steering-measure.sh` runs steering off, at its defaults and strong (tail 0.2, 10
m/Myr up to 600 m, outgassing 0.5/Myr up to ×4), each with `--epochs`, then picks the 250-Myr
epoch, stops at 120 Myr and continues to 700 Myr from copies of the seed-1 default run;
`tools/review/steering-report.py` summarises. Master's binary (e102343b) ran the same two seeds for
the identity check. Scratch under `/Volumes/Graphide/ban/deeptime-runs/steering/`, deleted after.

### 8.2 Epoch checkpoints (size, time)

| run | checkpoints | store on disk | writing them | the run |
|---|---|---|---|---|
| seed 1, steering off | 10 | 2.85 GiB | 12.5 s | 662 s |
| seed 1, default steering | 10 | 2.64 GiB | 12.4 s | 655 s |
| seed 2, default steering | 10 | 2.65 GiB | 11.2 s | 649 s |

One checkpoint is about 250 MB of world body (the L1 fields, the plate samples, the surface's
strata log); the recorded history (222 MB at 500 Myr, 50 frames) is stored once across the chain.
Writing costs about 2 % of the run. Without the chain the store would have held the history ten
times (about 1.2 GB more at these sizes, growing with the square of the run's length). The cap of
about ten checkpoints (`EPOCH_CHECKPOINTS`) bounds the store at about ten bodies whatever the
length: on a 1,000-Myr run the grid becomes 100 Myr, so a frame between two checkpoints reruns at
most 100 Myr plus its tail.

### 8.3 What the nudges cost

Nothing measurable: steering off 662 s and 655 s against 655 s and 649 s with the defaults (the
VM's noise is larger). The controllers are two area sums over L1 per plate step.

**Steering off changes nothing.** The seed 1 and 2 runs with `steering.enabled: false` against
master's binary: 24 of the 27 files the manifest lists are byte-identical; the others are `preset.json`
(it names the section) and the event log (`events/1-0.dtev` and its index), which differ by a bug
this task fixed in passing (inferred: it is the only change to how events are written; every
other file being identical shows the worlds are):
the Andean orogeny event's end time was written in Myr instead of years (`plates/subduction.rs`;
on a run ending at the present it read as a few years before 0; a run continued past the present
made it end before it began, and the event validator refused it).

### 8.4 Pick, stop and continue: what reruns

| what (seed 1, default steering, 500-Myr run of 655 s) | resumed from | steps run | keyframes kept | wall time |
|---|---|---|---|---|
| pick the 250-Myr epoch | step 112 (224 Myr) | 13 of 125 | 5 | 119 s |
| stop at 120 Myr | step 45 (90 Myr) | 15 of 60 | 2 | 116 s |
| continue to 700 Myr | step 225 (450 Myr) | 125 of 350 | 10 | 302 s |

About 90 s of each is the end stages and the bake (deposits, young landscape, amplification,
the bake: as in a run of that length). A continue reruns the old tail (50 Myr with the default
10 %) plus the new span; a planned 700-Myr run would take about 900 s.

### 8.5 How hard the nudges can steer, and the defaults

| run | land at the tail's start | land at the end | unsteered end | sea offset | as Earth's ocean | largest step | reversals | mean T at end |
|---|---|---|---|---|---|---|---|---|
| seed 1, default (2 m/Myr, ≤ 200 m, 10 %) | 0.171 | 0.160 | 0.151 | −52 m | 2.4 % | 4 m | 0 | 17.1 °C |
| seed 2, default | 0.216 | 0.183 | 0.176 | −41 m | 1.9 % | 4 m | 0 | 16.4 °C |
| seed 1, strong (10 m/Myr, ≤ 600 m, 20 %) | 0.183 | 0.211 | 0.151 | −473 m | 20.8 % | 18 m | 0 | 14.0 °C |
| seed 2, strong | 0.188 | 0.217 | 0.176 | −269 m | 11.7 % | 13 m | 0 | 15.9 °C |

- **Land.** These planets lose land through their last 50–100 Myr (seed 2: 0.216 → 0.183 even
  steered) and end far below the 25 % the band asks; the plate model's crust budget is the cause
  (decisions log 2026-10-05, plate tasks 7–8). The default nudge adds 0.7–0.9 points of land; the
  strong one 4–6 points, but only by drawing 12–21 % of an Earth ocean's water into the mantle
  (270–470 m of sea level, beyond the ±250 m of Earth's Phanerozoic). Neither reaches the band.
- **Artefacts.** No reversal of either control in any run; the sea moves at most `rate × dt` a
  step (4 m and 13–18 m a 2-Myr step), smoothly ramped.
- **Temperature.** All six ended between 14 and 17 °C, inside the temperate band, so the
  outgassing controller never acted (factor 1.0). The warm and cool targets (seed 1, default
  rates) measure it: unsteered the planet ends at 17.3 °C (708 ppm); **warm** (18–26 °C) raised
  outgassing ×1.36 and ended at 18.7 °C, 1,139 ppm (met); **cool** (6–13 °C) lowered it to its
  limit ÷2 and ended at 13.7 °C, 215 ppm (0.7 K short). Largest step 0.07 of ln(factor) per
  2 Myr, no reversals. The carbon cycle answers within a plate step, so the CO₂ nudge is strong
  where the sea nudge is weak; ×2 is what bounds it.
- **Defaults kept gentle** (2 m/Myr, at most 200 m, outgassing 0.2/Myr up to ×2, 10 % of the run):
  within Earth's long-term eustatic rates and its outgassing range, so a steered world is still a
  world its physics could have made. Reaching the land band on today's planets would take
  unphysical amounts of water; that is a question for Ben (STATUS.md, Pending questions).

## 9. Tests

- Continue is deterministic: a run stopped at S then continued to S' equals a run planned
  `stop_myr = S'` (world, events, sites, history, keyframes), with and without steering.
- An epoch pick (and a stop) equals a run planned to that stop, and keeps the earlier frames.
- Without a `steering` section, and with `enabled: false`, a run equals the run without either
  (and master's; checked on the VM against master's binary).
- The nudges meet their targets within tolerance on a planet that starts outside them, with no
  sea-level step larger than r · dt and no sign flip-flop of the controls.
- Checkpoint selection: keys, stricter steering falling back, none found (starts over).
- Java: the editor's Continue, Stop here and Play this epoch launch a resume in the same staging
  folder; a superseded run's folder survives; the steering fields are editable with their defaults.

## 10. Questions for Ben

Listed in STATUS.md "Pending questions", recommendation first.
