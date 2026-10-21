# Playability steering and the epoch tools

Status of this page: `stage 1 (written with the feature, 2026-10-05; the ocean volume read from a source, the rest from-memory or game choices)`.

Ledger ids on this page: `STE-001` to `STE-015` (see [claims.md](claims.md) and [ledger/STE.md](ledger/STE.md)). Design: [docs/design/playability-steering.md](../design/playability-steering.md).

## What you see in the world

On the Create World screen's timelapse a player can stop a run at any keyframe it has drawn
(**Stop here**), bake a finished run's world as it stood at an earlier keyframe (**Play this
epoch**), or let a finished run go on past its end (**Continue to N Myr**). The world they get is
the world a run planned to that length from the start would have given.

With the Earth-like presets the last tenth of a history is **steered**: if the planet is heading
for too little land or a frozen or overheated end, its ocean slowly loses (or gains) water and its
volcanoes give off a little more (or less) CO₂, so it ends inside a playable band (by default
25–35 % land and a global mean of 10–20 °C). A planet already inside the band is not touched.
Turning steering off is the realistic choice: no steering at all, so the present-day climate
adjustment (`climate.present_climate`) is off too, and the planet ends however its physics ends it
(Ben, decisions log 2026-10-06).

## The model in plain words

**Where a run ends.** A run's preset now says where it stops (`timeline.stop_myr`, Myr after its
start; the planned length by default). World time stays absolute: a run planned for 500 Myr runs
from 500 Myr before the present to the present; stopped at 120 Myr it ends 380 Myr before the
present, under the dimmer star of that time; continued to 700 Myr it ends 200 Myr after the
present, under a slightly brighter one (STE-012). The deposit types' secular clocks keep their
dates the same way.

**The tail.** Almost everything a run does is independent of where it will end. The exceptions
are its last plate step (the plate model closes its open contacts and hotspots, the climate takes
its monthly snapshot for the bake and the present-day climate adjustment), the climate's 3-Myr
ice-age window with its Milankovitch orbit (now measured from the run's end, not from the
present), and the steering's ramp. Together they are the run's tail (STE-011). A run writes a
checkpoint of its world before the tail of each keyframe; stopping, picking an epoch or continuing
starts again from the latest checkpoint before the new stop's tail. The result is bit for bit the
run planned to that stop (tested).

**The nudges** (STE-002 to STE-010). Over the steered tail a weight rises smoothly from 0 to 1.
Each plate step two controllers compare the planet with the target band:

- *Land share → ocean water volume.* Too little land lowers the sea level a little (water drawn
  into the mantle, as Earth's deep water cycle does at subduction zones); too much raises it. The
  step is the weight × the rate (2 m/Myr by default) × the error (full rate from 5 points of land
  share outside the band), at most 200 m in all. Water added to an ocean raises it against the
  continents by only 0.688 of its depth, because the ocean floor sinks under the extra load (Airy
  hydro-isostasy, STE-008); the run reports the volume the offset stands for (STE-009).
- *Temperature → volcanic outgassing.* Too cold raises the carbon cycle's degassing by a factor;
  too warm lowers it (by default at most ×2 or ÷2, at 0.2 of ln(factor) per Myr at full weight
  from 5 K outside the band). The silicate-weathering feedback then moves CO₂, and the climate
  follows within a few hundred thousand years.

Neither controller sets land, temperature, CO₂ or ice: the end state is one the model's physics
reached. Inside the band both stand still, so a planet that ends well on its own is the unsteered
planet. The run's notes (`notes.climate.steering`) record how much each control moved, where the
tail began, whether the targets were met, and two artefact counters (the largest single step and
the number of times either control reversed).

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| STE-001 | steering on for the Earth-like presets | on | | Ben, decisions log 2026-10-05 | `game-choice` |
| STE-002 | steered share of the run | 0.1 | share | | `game-choice` |
| STE-003 | weight | w = x²(3 − 2x) | | | `game-choice` |
| STE-004 | land band (named targets) | 0.25–0.35 | share | Ben's example | `game-choice` |
| STE-005 | temperature bands | 10–20, 18–26, 6–13 | °C | | `game-choice` |
| STE-006 | sea rate, largest offset | 2; 200 | m/Myr; m | | `game-choice` |
| STE-007 | outgassing rate, largest factor | 0.2; 2 | ln/Myr; × | | `game-choice` |
| STE-008 | hydro-isostatic share | 1 − 1030/3300 = 0.688 | | Airy isostasy | `from-memory` |
| STE-009 | Earth's ocean volume | 1.332 × 10⁹ | km³ | Charette & Smith 2010 | `verified` (Table 2, 2026-10-21) |
| STE-010 | errors at full rate | 0.05; 5 | share; K | | `game-choice` |

Sea controller, each plate step of length Δt (Myr) in the tail, land share L, band [L_lo, L_hi]:

  e = L_lo − L if L < L_lo; L_hi − L if L > L_hi; else 0
  Δh = −w · r_sea · Δt · clamp(e / 0.05, −1, 1);  h ← clamp(h + Δh, −h_max, h_max)

and the eustatic sea level becomes the ocean's water over its basins (less the land ice's, [sea-level.md](sea-level.md); until 2026-10-21 the curve plus glacio-eustasy) + h. The CO₂ controller is the same on
x = ln(degassing factor) with the global mean temperature, 5 K and r_co2, x within ±ln F.

## Earth against the mod

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| Long-term sea-level change rate | about 1–2.5 m/Myr over 100–200 Myr | Haq & Schutter 2008; Miller et al. 2005 (from memory, STE-013) | at most 2 m/Myr by default | see the build log 2026-10-05 |
| Outgassing variation | about ×2 over the Phanerozoic | Berner 2004, GEOCARB (from memory, STE-014) | at most ×2 by default | see the build log 2026-10-05 |

Measured (quick 16k, scattered start, 500 Myr, 2026-10-05; design §8): the default nudges
lowered the sea by 41–52 m (about 2 % of an Earth ocean's water) and added 0.7–0.9 points of land
to planets that end at 15–18 % land; strong settings (10 m/Myr, 20 % of the run) added 4–6 points
for 270–470 m of sea level. The outgassing nudge moved a 17.3 °C planet to 18.7 °C (warm target,
×1.36) and to 13.7 °C (cool target, ÷2, its limit). No control reversed direction in any run.

The measured effect of the nudges (land and temperature at the end with and without them, the
artefact counters, the cost) is in the design note's §8 and STATUS.md's build log of 2026-10-05.

## Simplifications and limits

- The sea controller moves the sea level directly and reports the water it stands for; the ocean
  floor itself is not deepened (the plate model's floors do not feel the water load).
- The temperature measured is the energy-balance model's global mean at the end of each plate
  step; the controller acts from the next step (one step of lag, by design).
- The nudges run only where the climate stage runs (a preset without a `climate` section cannot
  turn them on).
- A continued run's star keeps brightening by Gough's law past the present; nothing else about
  the future (the Sun's evolution, the mantle's cooling) is modelled differently from the past.
- Stopping is at keyframes (50 Myr apart on the quick presets), where the epoch checkpoints are.
  The live map between keyframes (a picture of the run as it goes, docs/design/create-world-ux.md)
  is not a stop: Stop here while it shows stops at the newest keyframe.

## Game choices

Every steering number (STE-001 to STE-007, STE-010) is a game choice. Ben chose that steering
exists, that it is end-weighted, player-chosen and on by default for the Earth-like presets
(decisions log 2026-10-05); the bands follow his example ("temperate, ~25–35 % land"); the rates,
limits, shape and tail fraction are this task's (build log 2026-10-05), set from the measurements.

## Open questions

- None open. Ben answered the three of 2026-10-05 on 2026-10-06: the nudges stay gentle (the
  plate work brings land up), stops stay at 50-Myr keyframes, and the realistic choice is no
  steering at all: with `steering.enabled` false, `climate.present_climate` is off too (see the
  decisions log).
