# Hotspots, plume heads and volcanic rifted margins from sources: design

Status: **built behind switches, off** (2026-10-31, the hotspot lane, branch `hotspot-lip` on
`origin/crust-budget`): `tectonics.hotspots_sourced` (PLT-790 to PLT-794, PLT-801),
`tectonics.plume_heads_sourced` (PLT-795 to PLT-797) and `tectonics.volcanic_margins` (PLT-798,
PLT-799), with the measurement PLT-800. The arm is section 6 below; the readings in plates.md
"Hotspots, plume heads and volcanic margins from sources (2026-10-31)"; the sources and what was
read of them in bibliography.md "The sources of the hotspot lane (2026-10-31)".

## Why

Stern & Scholl 2010's Table 1 books "Additions at rifts, hotspots, and volcanic rifted margins 0.5"
km³ a year to the continents; the model makes 0.05 (base34-s4, `tools/review/loss-budget.py`). The
crust-budget lane (docs/design/crust-budget.md, section 5) found the model's hotspot rules uncited
and nothing added at breakup, and designed volcanic margins without building them: "a 50 % draw
per breakup would be a forcing term, not a law". This design gives the plumes, their heads and the
margins a law from sources, so that what each adds follows from the physics. No knob, cap or
forcing term is chosen to move a V1 number or to reach the 0.5 (Ben: "Results that don't align
with earth but are still perfectly playable and physics backed > Results that are forced to align
with earth by introducing extremely weird forcing functions").

## 1. What the model's rules do and what they rest on

`plates/hotspots.rs`, `plates/init.rs`, `plates/rifting.rs`, the preset defaults (every preset
carries the Rust default):

| rule | value | ledger | rests on |
|---|---|---|---|
| hotspots at the start | 8 | PLT-108 | nothing (`uncited`) |
| new hotspots, Poisson | 3 per 100 Myr | PLT-109 | nothing (`uncited`); King & Adam 2014's catalogues hold 26 to 69 (plates.md, the spot check) |
| a hotspot's life, even draw | 40 to 150 Myr | PLT-110 | nothing (`uncited`) |
| the swell's radius | 150 km | PLT-111 | nothing (`uncited`) |
| uplift at the centre | 0.3 mm/yr × U(0.6, 1.4) | PLT-112 | nothing (`uncited`) |
| the kernel | (1 − (d/r)²)² | PLT-113 | `game-choice` |
| volcanic ceilings | 3 km oceanic, 1.5 km continental | PLT-114 | nothing (`uncited`) |
| basalt added | 3 × the rise | PLT-115 | nothing (`uncited`) |
| hotspot flag | kernel > 0.2 | PLT-116 | `game-choice` |
| a head with the birth | probability 0.5 | PLT-117 | nothing (`uncited`) |
| the LIP | radius 900 km, 1,500 m central pile, rise a third | PLT-118 | nothing (`uncited`) |
| tracks | the plate's speed over the fixed point | PLT-119 | Cortial 2019 §7.5 (`cited-unverified`) |
| initial ages | born up to 50 Myr before the start | PLT-120 | `game-choice` |
| the head's rift boost | λ × 4 for 20 Myr, the rift's cause | PLT-090 | `game-choice` |
| heat at a hotspot | +40 mW/m² | PLT-468 | nothing (`uncited`) |
| the volcano stage's shield strength | uplift over the preset's, 0.6 to 1.4 → 0 to 1 | VOL-008 | `game-choice` |
| breakup | the transition trades the continental side's crust for ocean crust (PLT-294), the gap point is the ocean column (PLT-104) | PLT-294, PLT-104 | Franke 2013 (the majority of margins volcanic); no addition |

What they make (base34, 24 runs, master f80d75a8): 24 to 47 rifts and 6 to 23 LIPs in 1,000 Myr,
the hotspots' basalt 0.026 to 0.059 km³ a year (`hotspot_magma_km3`), about 3 live plumes once the
first 8 have died (plates.md, the spot check), the continents' share of it 0.05 km³ a year.

## 2. The sources (what was read)

One Sonnet research sub-agent read the full texts of Sleep 1990, Courtillot et al. 2003, King &
Adam 2014, White & McKenzie 1989, Campbell & Griffiths 1990, Geoffroy 2005, Franke 2013, Stern &
Scholl 2010, Ernst et al. 2021, Kasbohm et al. 2021, White, Crisp & Spera 2006, and Clift et al.
2009's revised manuscript (partly recovered from a truncated copy); the abstracts of Davies 1988,
van Ark & Lin 2004, Vidal & Bonneville 2004, Coffin & Eldholm 1994, Richards et al. 1989, Menzies
et al. 2002, Courtillot et al. 1999 and Condie 1998; the USGS dataset of Bradley 2008's margins. Not
obtained: Coffin & Eldholm's table, Bryan & Ernst 2008, Crisp 1984, McKenzie & Bickle 1988 (the
copy truncated), Ernst 2014, Condie's plume shares. Everything quoted below is in bibliography.md
with its page.

The numbers the design uses:

- **Plume tails.** Sleep 1990, Table 1: 37 hotspots, 0.3 to 8.7 Mg/s, "The total buoyancy flux is
  54.9 Mg s-1"; Hawaii "the flux Qv of magmas implaced in the crust or erupted is about 4 m3 s-1"
  at 8.7 Mg/s, Iceland 8 m³/s at 1.4 (on a ridge: "the ratio of plume material to melt varies by a
  factor of 10 between Hawaii and Iceland"). Courtillot et al. 2003, p. 303: "The tails (stems) of
  primary plumes tend to last on the order of 130 Ma ... all plumes born as traps in the last 100
  Ma ... are still quite active, whereas those born between 100 and 140 Ma may be failing ... and
  those older than 150 Ma do not in general have an active trace"; "Only seven ('primary') out of
  49 hotspots" (on the order of 10). White et al. 2006, Table 3: the Hawaiian-Emperor chain's
  intrusive to extrusive ratio 6:1. White & McKenzie 1989: the plume's centre "narrow (150-200 km
  wide)"; Campbell & Griffiths 1990: a chain "~200 km wide".
- **Heads.** Ernst et al. 2021: "Continental LIPs have occurred on average (but irregularly) every
  30 myr ... The oceanic LIP record is similar to that of continental LIPs for the past 200 myr";
  White & McKenzie: "on average a major igneous province is generated once every 30 m.y.". Sleep:
  "The area strongly affected by modern plume heads is typically 1000 km in radius"; White &
  McKenzie: "A 1000-km-radius circle, which is the typical extent of the thermal anomaly created by
  a plume"; Campbell & Griffiths: the head "forms a disk ~2000 km in diameter", its activity decays
  "over a total time of order 20 Ma", the melt of a starting plume "~10⁷ km³". Clift et al. 2009,
  Table 7: nine continental LIPs' "Estimated total volume with intrusives" (four times their
  volcanic volume, after "a value of 1:3"), 30,463,152 km³, "0.12 km3/a" over 250 Myr; Ontong Java
  44.4 × 10⁶ km³ (Coffin & Eldholm 1994 through Clift), Wallaby 4.2 × 10⁶ (Schubert & Sandwell
  1989 through Clift), the Caribbean 4.5 × 10⁶ (Kasbohm et al. 2021).
- **Volcanic margins.** White & McKenzie 1989: "Mantle plumes generate regions beneath the
  lithosphere typically 2000 km in diameter with temperatures raised 100-200 °C above normal"
  (abstract), "of the order of 100-150 °C above normal" (conclusion); Fig. 3 at β = 50 (full
  breakup): about 7 km of melt at 1280 °C, 17 at 1400, 26 at 1480 (the sub-agent's reading of the
  figure, ±1 km); "New continental splits do not always occur above thermal anomalies in the mantle
  caused by plumes, but when they do, huge quantities of igneous material are added to the
  continental crust"; their Table 2: the oceans' first spreading inside their flood basalts' age
  spans (North Atlantic 58 to 56 Ma after 63 to 52; Deccan 64 within 67 to 60; Paraná 130 to 120
  against 130 to 117); the central Atlantic and US East Coast "did not develop into full-blooded
  volcanic margins ... There was no mantle plume immediately beneath the site of the oceanic rift".
  Franke 2013, quoting White: "an anomalously hot mantle (150-200° above normal) must be present
  under the rift shortly before continental breakup"; but also "some volcanic-rifted margins cannot
  be directly linked to hot-spots, as e.g. the NW Australian margin and the U.S. east coast margin".
  Kelemen & Holbrook 1995 (abstract): igneous crust "up to 25 km" thick "only for about 80–100 km
  seaward". Courtillot et al. 1999 (abstract): "In all cases, an active component (a plume and
  resulting flood basalt) is a pre-requisite for the breakup of a major oceanic basin. But rifting
  must be allowed by plate-boundary forces."
- **What counts as continental.** Stern & Scholl 2010 give no derivation of the 0.5 (p. 12: "at
  all others sites at approximately 0.5 AU"); p. 13: "Additions to the continental crust are also
  made at volcanic rifted margins that form when continents rift apart", hotspot magmas "added to
  the crust by underplating and intrusion", and plateaus accreting "as recently occurred where the
  Ontong-Java Plateau collided with the Solomon arc"; Hawkesworth & Kemp's "8% hotspot and 92% arc"
  of new crust. White & McKenzie's own budget (p. 7721): 10 million km³ per split every 30 Myr,
  "an average of around 0.3 km³/yr", and with the Basin and Range, the Columbia River, Kenya and
  the non-volcanic margins "0.4 km³/yr" of continental growth above rifted regions (Geoffroy 2005
  quotes the 0.4). Clift et al. 2009: continental flood basalts 0.12 km³ a year with their roots,
  plateaus 1.1 if every one accreted; hotspot magma "that is not erupted in a major plateau has
  little chance of being preserved in the continental crust".

## 3. The design

### 3.1 Plume tails (`hotspots_sourced`, PLT-790 to PLT-794)

- **How many and how long.** A planet holds Sleep's 37 tails: 37 at the start, each part way
  through its life (an age drawn evenly over it, the steady state of a constant birth rate), and
  new ones born at 37 over the mean life, 0.296 a Myr (29.6 per 100 Myr; Little's law, the
  agent's arithmetic). A tail lives an even draw from 100 to 150 Myr (Courtillot et al. 2003's
  primary tails; the only lifetime the sources give, applied to every tail). Births stay uniform
  on the sphere (PLT-109's placement; Earth's cluster over the LLSVPs' edges is not modelled).
- **How strong.** A tail's buoyancy flux is one of Sleep's 37, drawn evenly (PLT-791), so a
  planet's tails add to about Sleep's 54.9 Mg/s.
- **How much magma.** A tail makes Hawaii's magma per unit of buoyancy flux, 4 m³/s at 8.7 Mg/s
  (0.0145 km³ a year per Mg/s, PLT-793): about 0.80 km³ a year over the 37, 0.0215 for the mean
  tail. This applies Hawaii's melt share to every tail. The sources say it varies (Iceland's, on a
  ridge, ten times; Courtillot et al.: deep plumes under 1 Mg/s "should have cooled so much that
  they would not melt beneath old lithosphere"), but none gives a law the model can apply without a
  melting model of its own, so the thick lid's lower melting is not modelled (section 5).
- **Where it goes.** Over the existing 150-km kernel (PLT-111, whose full width at half its peak,
  162 km, is White & McKenzie's 150 to 200 km plume and Campbell & Griffiths' 200 km chain), a
  seventh of it lavas over intrusions (White et al. 2006's 6:1 for the Hawaiian chain, PLT-794),
  less the volcano stage's cones over it (PLT-286). The column rises by the model's Airy isostasy of
  what it gained (`pools::isostatic_change_m`, water-loaded under the sea), in place of PLT-115's
  three-to-one and PLT-114's ceilings. Seamounts on ocean floor stay oceanic and go down with their
  slab or accrete by the model's own rules; on continental crust the magma is continental crust.
- **The volcano stage** keeps its shields' spread: a tail's strength (VOL-008) is the preset's
  reference uplift times 0.6 to 1.4 by its flux's rank in Sleep's table (PLT-801, a game-choice).

### 3.2 Plume heads (`plume_heads_sourced`, PLT-795 to PLT-797)

- **Which tails rise with a head.** The heads' rate is one every 15 Myr (Ernst et al. 2021's
  continental LIPs every 30 Myr and an oceanic record "similar", the agent's reading), so the share
  of births with a head is that rate over the births', 0.225 with the sourced births (Courtillot et
  al. 2003: seven primary of 49, "maybe on the order of 10"), in place of PLT-117's 0.5. With the
  preset's births it is the same rate over theirs, at most every birth.
- **How big.** A head spreads 1,000 km under the lithosphere (PLT-797; Sleep, White & McKenzie,
  Campbell & Griffiths). Its province is one of the record's for the crust it rises under, drawn
  evenly (PLT-796): on continental crust Clift et al.'s nine with their roots (0.14 to 12.2 × 10⁶
  km³, mean 3.4 × 10⁶), on ocean floor the three plateaus read (4.2 to 44.4 × 10⁶, mean 17.7 ×
  10⁶). The record stands in for the lid's control on a head's melt (White & McKenzie's Fig. 3:
  thin lithosphere melts more), which the model does not compute. A quarter is lavas (Clift et
  al.'s 1:3), the rest intrusions.
- **The world holds the record's volume.** The volcano stage lays a flood basalt's plateau, 1 −
  smoothstep(0.6, 1, r/R) (VOL-035), over the plate model's dome (PLT-113's kernel) and adds the
  rock between them, so the world's province is the central pile times the plateau's area (0.648
  πR²), not the dome's (πR²/3): the central pile is the record's volume over the plateau's area,
  and the plate model's dome holds 0.514 of it, the volcano stage the rest (the off rule's events
  report the dome alone). The event carries the head's radius (`radius_km`) for the volcano stage.
  The climate's CO₂ (CLM-057) reads the event's whole volume.
- **The rift boost** (PLT-090, a game-choice) stays as it is for every head.

### 3.3 Volcanic rifted margins (`volcanic_margins`, PLT-798, PLT-799)

White & McKenzie's condition is hot mantle under the rift when it breaks up. The model's hot
mantle is its plume heads: a head is hot 150 K above the normal mantle (the one value inside White
& McKenzie's 100 to 200 and 100 to 150 °C and Franke's quotation of 150 to 200) over its 1,000 km
for 20 Myr after it rises (Campbell & Griffiths' "of order 20 Ma"; White & McKenzie's Table 2 has
the first spreading within that), PLT-798.

At a resampling, a gap point that goes to a continent's plate, whose cell reaches into the 90 km
seaward of that continent's outline (Kelemen & Holbrook's "80–100 km seaward", PLT-802), formed
over a head's hot mantle, is the igneous crust that mantle melts to at full breakup over its
cell's share of that strip (`ridges::margin_share`: the point holds the strip from d − s to d
seaward of the outline, d its distance from the edge sample, s the spacing; the shares of a
straight margin's rows add to the 90 km at any spacing) and the ocean column over the rest:
20.4 km at 150 K (White & McKenzie's Fig. 3 at β = 50, interpolated between 1400 and 1480 °C), a
quarter lavas, as continental crust over its 7 km ocean basement (PLT-331), so up to 13.4 km of it
counts (PLT-799). When it formed is the pair's parting: the
gap point's age (α × the time since the last resampling) but not before the pair's rift onset (the
gap beside a continent opened after the last resampling, which filled any older one); where it
formed is its position turned back by its plate's motion over that time. No draw decides it: a
breakup over a head's hot mantle makes a volcanic margin, any other breakup does not.

On the 16k quick planet the lattice spacing is 71 km, so the margin is the first row of gap points
whole and the second at a quarter: 90 km × 13.4 km, about 1,200 km³ per km of margin of
continental crust beyond the basement (Kelemen & Holbrook's whole igneous crust is about 1,350 km³
per km, Eldholm & Grue's North Atlantic about 2,500 with the onshore basalts). The new column's
height starts at the gap's plus the Airy rise of its extra crust; `settle.rs` takes it from there.
The first row is not hyper-extended (13.4 km of its own crust) and no sediment margin, so the
transition (PLT-661, PLT-667) leaves it continental; the second (3.6 km of its own) is the
taper's ocean side, as a hyper-extended margin is.

What the rule does not do: a margin that forms without a head (the US East Coast, NW Australia,
Franke 2013) stays the ocean column; the thick ocean crust of a ridge over a head's or a tail's
hot mantle beyond the margin (Iceland, the Greenland-Iceland-Faroe ridge, up to 40 km, Geoffroy
2005) is not built; rifts that do not break up make no magma (White & McKenzie: β > 2 and hot
mantle).

## 4. What it adds, before the arm (the agent's estimates)

- Tails: about 0.80 km³ a year on the planet, of it on continental crust the continents' share of
  the births, about 0.24 at 0.3 of the surface (Earth's catalogue has 8 % of its flux under
  continents: Afar, Hoggar, Yellowstone, East Australia).
- Heads: one per 15 Myr, about 0.07 km³ a year on continental crust at the model's continental
  share (Clift et al. 0.12), 0.8 on ocean floor.
- Margins: White & McKenzie's 0.3 needs a split over a head every 30 Myr. The model's rifts are a
  Poisson hazard (PLT-082) with the head's boost (PLT-090) on the plate above, its rift lines
  Voronoi cuts through the plate, so a rift meets a head's 1,000 km only by the boost's chance: of
  order 0.01 to 0.02 km³ a year (section 5).

## 5. What is left

- **Rifts at heads.** Courtillot et al. 1999: "an active component (a plume and resulting flood
  basalt) is a pre-requisite for the breakup of a major oceanic basin", with "rift propagation
  (towards the flood basalt area)". The model's rift line ignores the head that boosted its plate
  (PLT-090's 4 for 20 Myr is uncited). A sourced rule would cut the boosted plate through the head
  and drop the boost's factor; it changes every planet's rifting, so it is a design of its own.
- **The lid.** A tail's melt under thick lithosphere (Courtillot et al. 2003, White & McKenzie's
  Fig. 3); it needs the solidus's depth at the plume's temperature (McKenzie & Bickle 1988, not
  read).
- **Thick ocean crust over hot mantle** (Iceland), the plateau's own accretion, and rift magmatism
  short of breakup.

## 6. The arm hs-cum (2026-10-31)

One cumulative 8-run arm on top of the crust-budget lane's three switches (`pool.sh run hs-cum
--seeds 1,2,3,4` with the six switches and the crust-budget environment; key f1352cd8eca1, argkey
e501232a), read against base34-s4 and cb-cum2; the tables are in plates.md "Hotspots, plume heads
and volcanic margins from sources (2026-10-31)". Switches off, the branch is base34 bit for bit
(arm hs-off, one run).

- The rows did what section 4 estimated for the heads (6.74 per 100 Myr, the LIPs 0.26 km³ a year
  whole on continental crust) and the margins (0.04, 670 gap points a run: the estimate's order).
  The tails' row is 0.61 on continental crust, not 0.24: the model's continental crust covers 0.52
  to 0.69 of the planet, not 0.3, and the volcano stage's cones inside the kernels add to it.
- The net change of the continental volume is +3.11 km³ a year (cb-cum2 −0.37), the continental
  crust 0.69 of the planet at the end, the sea 1,321 m over the datum, the criteria 4 of 7 (6 on
  cb-cum2); the guard is clean.
- The cause is outside this design: the volcano stage builds a shield or a silicic centre per
  `hotspot_spacing_km` of each tail's track at the game's size (VOL-007, VOL-021, VOL-022), hands
  its rock to the pools (PLT-286) and only the part inside the 150 km kernel counts against the
  tail. With Sleep's 37 tails the cones are 5.0 km³ a year against 0.8 on cb-cum2, about four times
  the tails' own magma. A tails-only arm (hs-tails, 4 runs) carries 0.9 of the net's rise.
- The next step: a hotspot's cones and its tail as one budget, counted by the cone's hotspot
  (`Volcano::hotspot`) and capped at the tail's lavas (PLT-793, PLT-794), the rest of the cone relief
  over rock the plate model already holds; then this arm again. Built (docs/design/hotspot-cone-budget.md,
  `tectonics.hotspot_cone_budget`, PLT-803); hs-cum2 with it: the net +0.53 km³ a year, the sea
  679 m, criteria 4.5; the tails' 0.43 on continental crust leave the lid as the next step. `plume_heads_sourced` and
  `volcanic_margins` are correct as built, but were run only beside the sourced tails (alone, the
  heads' share caps at the preset's 3 births per 100 Myr).
