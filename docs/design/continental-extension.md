# Continental extension: rifts that stretch before they break up, and the settling's light top

Status: **designed and built, both off** (2026-10-31, the runoff lane's next task, branch `continental-extension` from
`uplift-trace` 9cbdd3c5). `tectonics.rift_stretching` (`plates/extension.rs`, PLT-781 to PLT-784) and
`tectonics.settle_top_density` (`plates/settle.rs`, PLT-780), with a measurement of what the rate cap trims (the land
ledger's `cap_land` and `cap_sea`, PLT-785). The arm waits for base35-s4 ("The arm").

## Why

The uplift trace (docs/design/uplift-trace.md) split the rock the plates lift above the sea by process.
- **No double count.** Its verdict was a missing sink for collisional crust.
- **The export is too large a share of the stacking.** The continents export 0.41 of what their collisions stack, against
  Earth's 0.12 to 0.23 (the agent's arithmetic from Bird 2003 and Clift et al. 2009).
- **Erosion is nearly the only way out.** The other ways thickened crust leaves the land are collapse, flow, founder and
  the numerical rate cap. Rifting thins nothing: under 0.005 km³ a year on every column.

The coordinator's task: read what the model does at rifts and breakups, find what the rate cap is, and design a sourced
extension law. It covers where and when crust thins (post-orogenic collapse of thick, hot crust, rift arms, back-arc
extension) and by how much (β from the sources' physics, not a fitted share). Build it behind a switch, and fix the
settling's density detail the trace found.

## What the model does at rifts and break-ups (the code, read)

- **Rifts** (`rifting.rs`, Cortial et al. 2019 §4.4): a Poisson draw splits a plate into fragments.
  - Membership is only relabelled; nothing about the crust changes.
  - Whether the rift opens is the force balance's answer, and a rift nothing pulls apart heals.
  - Back-arc rifting (PLT-710) splits off the forearc and arc, the samples within 200 km of a qualifying front, as a
    sliver. The slab's suction then pulls the sliver away from its parent.
- **The gap** (`ridges.rs`): at a resampling "a new sample in no footprint (a gap between diverging plates) becomes fresh
  oceanic crust of the nearest plate". A rift between two continental plates is therefore ocean floor at the first
  resampling after it opens, a sample's width or more (about every 17 Myr on the trace's runs: 57 resamplings in
  1,000 Myr).
- **The margins** (`transition.rs`, PLT-261 to PLT-264, PLT-475, PLT-476, PLT-661, PLT-662): every ocean-continent
  boundary of a plate is drawn as a rifted margin after the fact.
  - The crystalline crust tapers from 33 km to 7 km over W = 200 km on the model sphere, a smoothstep in β.
  - It is volume-conserving within catchments about W/2 long.
  - The lithosphere is thinned by the same β (McKenzie 1978).
  - So the model draws the break-up's end state; it has no stretched continent before the break-up, and no rift wider
    than the taper.
- **Collapse** (`collapse.rs`, PLT-254 to PLT-259, PLT-390, PLT-412 to PLT-415) is Rey et al. 2001's fixed-boundary
  collapse. Its two laws:
  - The threshold law: quiet crust thicker than T_c = 50 km thins towards it at τ = 15 Myr.
  - The thin sheet (PLT-414): ε̇ = ΔU / (4 η L) against the collapse's normal column T_col, about 36 to 41 km.

  The crust released fills the plate's nearest room within 5,000 km, up to T_col: thinner continental crust, then the
  plate's own ocean floor. Nothing in it thins crust below T_col, and T_col stands z_0 (525 m) over the sea. So
  collapsed land is still land, and it is exported by erosion. The hot collapse to the normal column (PLT-390) is built
  and off.
- **The thin sheet of the collisions** (PLT-733): the belt's push decays behind the front as e^(−x/L), L = 2W/π
  (England, Houseman & Sonder 1985).

### The rate cap: what it is, and what it stands for

`rate::limit` (PLT-287, PLT-288) is a per-column ceiling.
- **The ceiling.** No visible column ends a step thicker than its start plus 5 mm/yr × δt (10 km in a 2 Myr step), or
  higher than its start plus 3.5 mm/yr × δt.
- **The excess** goes to the nearest room on the same plate within `collapse_reach_km` (5,000 km): continental crust up
  to `crust_max_km` and ocean floor up to the normal column. What finds none goes to the mantle.
- **Why it exists.** It was introduced against numerical lumps (build log 2026-10-18, "Plate spikes"):
  - a resampling's accretion gave a column up to 566 km of crust in one step;
  - the collision wedge filled thin crust by 35 to 94 km in a step.
- **Its 5 mm/yr** is about three times Garzione et al. 2006's fastest regional thickening (u H = W dH/dt, 1.7 mm/yr for
  the Altiplano), for one column over one step.

So it is two things:
- **A numerical guard** against the model's discrete steps: the resampling's overlap accretion, which applies a
  resampling interval's convergence at once, and the wedge's fill.
- **In effect, a stand-in for the orogen's lateral growth.** The excess moves to room "so the orogen widens instead of
  piling up". The physical counterpart is Garzione's relation: the convergence's thickening spread over the orogen's
  width. But the destination is the collapse's: the nearest room within 5,000 km, ocean floor at the continent's edge
  filled to the normal column, a continent's height over the sea. On the uplift trace, 1.33 of its 2.10 km³ a year
  went onto ocean floor. That crust is land at the coast again and is exported a second time.

It does not stand in for extension: it moves only thickening beyond a step's ceiling, and it never thins a column
below its start. Which processes' thickening it trims is now measured (PLT-785): the land ledger's `cap_land` and
`cap_sea` give the cap's release on land and under the sea by the parts that thickened the column earlier in the step,
read with `tools/review/uplift/ledger.py`. If most of it is the resampling's accretion, the cap is mainly a numerical
repair. If it is the belts, it stands for the wedge's growth, which the two-wedge orogen (PLT-730, PLT-731) should then
take. Either way the follow-up is its destination, not its ceiling, and it waits for the arm's split (below).

## What Earth does (sources read)

A Sonnet research sub-agent read these on 2026-10-31. The quotes are what it printed; the full list is in the
bibliography.

**The modes.** Buck 1991 (JGR 96:20161, abstract only, Crossref):
- "The model predicts three distinct modes of extension depending on the model crustal thickness, heat flow, and strain
  rate. The modes are (1) core complex mode ..., (2) wide rift mode (uniform crustal and mantle lithospheric thinning
  over a width greater than the lithospheric thickness), and (3) narrow rift mode (concentrated crustal and mantle
  lithospheric extension)";
- "The model implies a temporal progression from core complex mode to wide rift mode to narrow rift mode."

Buck, Lavier & Poliakov 1999 (Phil. Trans. A 357:671, author copy, partly read):
- "The wide rift mode occurs for moderate to high extension rates when the continental crust was thicker and hotter than
  normal" (p. 672);
- "narrow means about as wide as the lithosphere is thick" (p. 673);
- wide rifting "requires brittle lithosphere that is thinner than the crust. This only occurs when the regional heat
  flow is greater than normal" (p. 688).

Buck 2007 (Treatise on Geophysics, a publisher proof, pp. 25 to 30):
- narrow rifts (the Rhinegraben, Suez, East Africa, Baikal, the Rio Grande) are "on the order of 100 km wide";
- the Basin and Range's region of normal faulting is "as much as 800 km wide" (p. 25);
- "the wide rift and the 'core complex' modes are only predicted for thick, hot crust" (p. 28);
- his Fig. 30 (from Buck 1991) plots the modes against crustal thickness and heat flow, with the data points the
  sub-agent read off it approximately.

Brun 1999 (Phil. Trans. A 357:695, abstract only): necking "in a stable lithosphere that has a four-layer-type of
strength profile with the greatest strength located in the sub-Moho mantle ... gives birth to narrow rifts"; spreading
"in a thickened lithosphere whose strength profile, after thermal relaxation, exhibits maximum strength at the base of
upper brittle crust ... gives birth to wide rifts, such as the Basin and Range of the western United States or the
Aegean".

**Collapse with a free boundary.** Rey, Vanderhaeghe & Teyssier 2001 (Tectonophysics 342:435, full text, the Wayback
copy of the first author's page):
- "when the surrounding lithosphere is free to move (free-boundary divergent collapse), the thickened crust is
  homogeneously thinned without transfer of gravitational potential energy towards the forelands" (p. 435);
- their mode 2 needs space: "if a free boundary is adjacent to the orogenic domain (e.g.; an active continental margin),
  or alternatively, if a phase of tectonic divergence follows the protracted history, then space is provided for the
  bulk extension of the thickened crust" (p. 445);
- it is expected after "(1) changes in the plates motion; (2) rollback and/or retreat of subducting slab; and (3)
  opening of a back arc basin" (p. 445);
- a weak, flowing lower crust needs "the temperature at the Moho ... sufficiently high (≥700 °C, Sonder et al., 1987;
  Kaufman and Royden, 1994)" (p. 443).

The paper states no thickness the collapse thins to. Dewey 1988 (Tectonics 7:1123, abstract only): "Where, as in the
disruption of Pangea, boundary forces change rapidly from compressional to tensional, body force collapse is continued
by general extension which may lead to continental splitting".

**The examples.**
- **The Basin and Range.** Long 2019 (GSA Bull. 131:99, pp. 99-101, 109-111, 114-115):
  - "230 ± 42 km of cumulative extension (46% ± 8%), and an average pre-extensional thickness of 54 ± 6 km" (p. 99);
  - "an average modern thickness of 37 ± 1 km" (p. 109);
  - the inception "attributed to reorganization of the Pacific–North American plate boundary in the middle Miocene" and
    the San Andreas transform (p. 101);
  - "geodynamic events that altered boundary conditions ... were necessary to initiate pulses of gravitational
    collapse" (p. 115).

  Wernicke et al. 1987 (p. 216): "the locus of extension is controlled principally by crustal thickness, while its
  timing is governed by the thermal state of the lithosphere at the time of thickening"; "a lower crust at or near its
  minimum-melting temperature (~650-750 °C) is apparently a requirement for it to begin".

  Wernicke et al. 2000 (GSA Today 10(11), pp. 1-5): the Sierra Nevada block moved west "first at rates near 20 mm/yr
  ..., then slowing to the current overall rate near 12 mm/yr", "~250 km" since 16 Ma.
- **The Aegean.** Jolivet & Brun 2010 (author PDF):
  - "a rather flat Moho around 25–26 km deep below most of the Aegean Sea", the Cretan Sea "~15 km", Rhodope "up to
    45 km" (p. 112);
  - "~600–650 km of retreat since 30–35 Ma, thus an average rate of ~2.0 cm/year" (p. 128), while Africa and Eurasia
    converged at "1.3 cm/year in average".
- **Tibet.**
  - Zhang et al. 2004 (Geology 32:809, full text): "eastward stretching of 21.6 ± 2.5 mm/yr", an extensional strain
    rate "2.1 ± 0.3 × 10^-8 yr^-1" (p. 810).
  - Molnar, England & Martinod 1993 (Rev. Geophys. 31:357, full text): "Roughly half is absorbed by crustal thinning
    and half is absorbed by north-south shortening by strike-slip faulting", "the crust has thinned about 2.8 km" since
    8 Ma (p. 374).

**The isostatic response to erosion.** Molnar & England 1990 (Nature 346:29, full text): "the removal of a layer of
crust of thickness ΔT and density ρc, compensated by a crustal root in a mantle of density ρm, results in a lowering of
the surface of ΔT(ρm − ρc)/ρm ≈ ΔT/6" (p. 30).

**Not found** (the sub-agent's list):
- Buck 1991's, Brun 1999's and Dewey 1988's full texts;
- Buck's own Moho-temperature thresholds and β per mode;
- an Aegean β or pre-extension thickness;
- the widths of rifted margins' thinned crust.

## What the sources say the model lacks

- **A continent stretches before it breaks up.** In the model a rift is ocean floor at the first resampling. The margin
  taper draws the end state on every ocean-continent boundary, but no continental crust is ever stretched by a plate
  boundary's opening, and no rift is wider than the taper's 200 km. Buck's wide rifts (the Basin and Range, 800 km) and
  the Aegean back-arc are missing.
- **Thick, hot crust next to an opening boundary is thinned homogeneously, below the lowland's column.** That is Rey et
  al.'s free-boundary collapse, the Aegean's 26 km, and the Basin and Range's 37 km from 54. The model's collapse is
  fixed-boundary only and stops at T_col, a continent's height over the sea.
- **Extension while the plates converge** (Tibet) thins little: 2.8 km in 8 Myr. It is not built.

So the missing law is extension at an opening boundary: a rift arm, a back-arc opening behind a sliver, a plate
boundary turning divergent. Its amount is the plates' own opening, so β is the integral of a strain rate the
kinematics set, not a share.

## The law (`tectonics.rift_stretching`, off; `plates/extension.rs`)

**Where.** A rift is a boundary between two plates where both sides are visible continental crust (no trench) and the
relative motion opens it: positive along the boundary's normal and no transform (PLT-311). Per pair of plates it has a
length W, the boundary's straight length (PLT-530's shares, the mean of the two sides), and a mean opening speed u.
Back-arc slivers on continental crust are rifts of this kind once they move off.

**How much.** The opening's area grows at u W, and the two sides' continental lithosphere takes it in pure shear, the
columns' area growing as their crust thins (McKenzie 1978's uniform stretching):

  Σ_i ε̇_i A = u W,   ε̇_i = u W w_i / (A Σ_j w_j),

A a sample's area, w_i a column's weight by its side's mode. A column keeps e^(−ε̇ δt) of its crust in its own mix, so
its β is the integral of ε̇ the plates' motion sets. It never keeps less than 10 km of crystalline crust (Brune et al.
2014's hyper-extended limit, PLT-661). Its lithosphere and keel thin by the same factor (`thermal::stretch`, PLT-476),
and its surface goes down by the Airy change of what it lost. A column at the limit takes no share. What the opening
asks beyond the columns that can still thin is break-up: new sea floor.

**Narrow or wide (PLT-782).** The test is the Moho's temperature on the plate model's own geotherm
(`thermal::geotherm_share`, T_m at L). A side whose edge columns' Moho stands at 700 °C or more is wide, else narrow:
Rey et al.'s "≥700 °C" (p. 443), the middle of Wernicke et al.'s "~650-750 °C" (p. 216). On the model's geotherm, with
the lithosphere from the heat flow by the model's own law (q = 26 + 3,375 / L), it puts each of Buck's Fig. 30 rifts in
its mode:

| rift (Buck's Fig. 30, as read) | crust, km | heat flow, mW/m² | Moho on the model's geotherm, °C | mode |
|---|---|---|---|---|
| northern Basin and Range | 33 | 90 | 878 | wide |
| Altiplano | 48 | 90 | 1,237 | wide (core complex conditions) |
| Tibet | 64 | 87 | 1,350 | wide (core complex conditions) |
| Rhinegraben | 36 | 68 | 655 | narrow |
| East Africa, Baikal | 40 | 51 | 466 | narrow |
| northern Red Sea | 30 | 45 | 303 | narrow |
| southern Rio Grande | 26 | 87 | 679 | narrow |

The Moho temperatures are the agent's arithmetic on the model's geotherm, and the test
`the_moho_test_puts_bucks_rifts_in_their_modes` repeats it. The mode then sets which columns take part:
- **A narrow side** takes its share evenly over the columns within L_lith / 2 of the rift, L_lith its edge columns' mean
  lithosphere: "about as wide as the lithosphere is thick" (Buck et al. 1999).
- **A wide side** spreads it over its hot columns, each with its own Moho at 700 °C or more, as e^(−x/L) with
  L = 2W/π: the thin sheet's decay for a boundary W long (England, Houseman & Sonder 1985), as for the collision's push
  (PLT-733).
- **As a rift thins its crust,** its Moho rises into colder rock and the rift narrows: Buck's progression from wide to
  narrow. A weak, hot side takes more of the opening than a cold one.

**Where the crust goes (PLT-784).** The opening is no sample until a resampling, so the crust the columns give waits
on the pair (`PlateState::rift_fill`). At the next resampling the gap samples between the two plates take it evenly,
as a column of its mix: the rift's floor of stretched continental crust, its lithosphere the thinned one it came from,
at its Airy height under the sea.
- **Break-up.** If that column would hold less than 10 km of crystalline crust, the rift has broken up. The gap samples
  nearest the plates' edges take 10 km columns of it, nearest first, until it is used, and the rest is new sea floor,
  the hyper-extended distal margin.
- **No gap.** A fill whose pair has no gap at a resampling waits while both plates live.
- **A dead plate.** When a plate has died, its fill goes to the continental samples nearest where it came from
  (counted as `plates/rift_fallback_km3`).

**Bookkeeping.** The globals `plates/rift_stretched_km3`, `plates/rift_floor_km3`, `plates/rift_fallback_km3` and
`plates/rift_steps`, and the budgets' new part `extension` (last, so the older parts keep their numbers).

**What it is not.**
- **A strain solve.** The mode and the decay distribute the opening's area; the area is the plates' motion.
- **The collapse.** A rift needs an opening boundary; the fixed-boundary collapse needs none.
- **Extension while the plates converge.**
- **Magmatism at rifts** (volcanic margins stay the transition's PLT-294).

The weight function's form (uniform over half the lithosphere when narrow, the sheet's decay over hot columns when wide)
is the agent's reading of the modes, a game-choice.

**Expected, before the arm.** The opening's area is the plates' own, so the law moves crust only where continental
plates diverge. On the trace's supercontinent run there were 37 rifts and 61 back-arc rifts in 1,000 Myr.
- **It thins land without erosion where a rift crosses thick or hot crust.** That is a wide side, or a sliver off a
  continental arc. The crust goes into rift floors under the sea instead of being filled to T_col at the coast.
- **Its share of the export is not known before the arm.** If most opening boundaries cross normal crust near the sea,
  the export moves little and the trace's lead is the rate cap's destination instead.

## The settling's light top (`tectonics.settle_top_density`, off; PLT-780)

**The finding** (the uplift trace). The settling (ISO-078) snaps every continental column to its whole-column Airy height
z_A, at the crust's mean density, 2,830 kg/m³ (PLT-281, 7.0 m of crust per m of height). PLT-281 itself keeps "the upper
crust's 2,700 (ISO-008, 5.5)" for "erosion from the top of a column", and the surface's own rebound is 0.8 (ISO-001). So
after the surface erodes a column the settling lifts it further, and the response to erosion comes to 6/7 of the rock
taken against the upper crust's 0.82.

**The source.** Molnar & England 1990 (p. 30): the surface lowers by ΔT (ρm − ρc)/ρm for crust of density ρc removed,
"≈ ΔT/6", 5/6 coming back, with one crustal density. Their 5/6 is between the model's two: 0.818 with 2,700 and 0.857
with 2,830 (the agent's arithmetic).

**The size.** The difference is 0.04 of the eroded thickness in the response, against Molnar & England's round value in
between. It sets heights, not the long-run export (the uplift trace's mass balance), so it is no plain error to be
corrected without an arm. It is a switch, off.

**The rule.** With the switch, each column carries a deficit D (`PlateState::top_deficit_m`, beside the samples, since
a `Sample` cannot grow).
- D grows by (ρ̄_c − ρ_u)/ρ_m = (2,830 − 2,700)/3,300 = 0.039 m per metre the surface takes off its top, and falls as
  much per metre it lays down.
- D is blended with the sample's weights at a resampling.
- The column settles at z_A − D. A metre eroded then lowers the column by 1 − 2,700/3,300 = 0.18 m, so 0.82 comes back
  (the test `rock_off_the_top_lowers_the_column_by_the_upper_crusts_airy_share`).
- Simplifications: the collapse's pure shear, the belts' stacking and the founder leave D as it is. The rift's
  stretching thins it with the crust.

## Tests

- `extension.rs`:
  - the opening's area shared by the weights;
  - pure shear down to the 10 km limit;
  - the Moho test against Buck's rifts;
  - the wide decay length;
  - a rift between two continental plates that stretches both sides and keeps every metre for its floor (nothing when
    the plates converge or the switch is off);
  - the weaker side taking more;
  - the resampling's floor, break-up, wait and fallback.
- `settle.rs`: the upper crust's Airy share with the deficit.
- `gap_budget.rs`: the rate cap's release by the parts that thickened the column.
- `plates/tests.rs`: `the_rifts_stretching_keeps_its_crust_and_off_changes_nothing` (a 200 Myr supercontinent run: the
  stretched crust is all laid, by the fallback or still waiting; with both switches off the run is master's).

## The arm

After base35-s4 exists: merge origin/master into this branch, then
`tools/review/pool/pool.sh alias HEAD <base35's key> --check-seed 1`. The switches off must be base35's physics. Then
one cumulative arm:

    tools/review/pool/pool.sh run ext-cum --seeds 1,2,3,4 --set tectonics.rift_stretching=true --set tectonics.settle_top_density=true +DEEPTIME_CRUST_BUDGET=1 +DEEPTIME_CRUST_BUDGET_MAPS=@RUN@/maps +DEEPTIME_GAP_BUDGET=1
    tools/review/pool/pool.sh table --base base35-s4 --arm ext-cum

The gap budget is measurement only (the uplift trace showed its worlds bit for bit), and gives the land ledger and the
rate cap's split on the arm.

**What to read:**
- the guard and `v1census`;
- the crust budget's export (`tools/review/loss-budget.py`) against base35-s4's;
- the land ledger's split (`tools/review/uplift/ledger.py`): the `extension` part and the cap's `cap_land` by part;
- the rift globals: stretched, laid as floor, fallback;
- the high ground through time (`tools/review/plateau/wear.sh`);
- coastset and collcoast.

If the arm moves something it should not, per-switch arms find which. No V1 number is a gate (decisions log
2026-10-23).
