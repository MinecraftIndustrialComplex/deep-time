# The converted crust's stock: design notes and reads (the arc-stock lane)

Status: **diagnosis in progress, nothing built** (the coordinator's lane after PR #60, 2026-11-01; branch `arc-stock`).

## The question

The low-continents diagnosis (sea-level.md, "Why base35's continents stand low") and the whole column's buoyancy
(PR #60, docs/design/continental-buoyancy.md "The arm cb-on and the default") leave the drowned continental crust
with crust converted on ocean floor: crust with a continental label over an ocean basement, island-arc crust by
cb_origin's rule the largest part. Under the buoyancy law its mafic rock stands as low as that rock, so base37's
continental crust under the sea is 0.643 against the ≤ 0.56 target. This lane asks where that crust comes from,
whether Earth has as much of it, and which missing law (if any) would change it.

## Reads so far (cached pool runs only)

**Where the island-arc crust stands** (`tools/review/freeboard/arcstock.py`, base36 24 runs, final frames, shares of
the planet): 0.189, of which 0.175 in a continental component holding start-class crust ("taken in") and 0.0098
standing alone (0.0054 at 20 km and more, 22.7 km thick, 8 % dry). Earth's intra-oceanic arcs are 14,500 to 17,000 km
long (Stern & Scholl 2010; Gerya 2011, quoting Leat & Larter 2003) and juvenile arcs "approximately 200-300 km across
and up to 30 km thick" (Stern & Scholl 2010): about 0.006 to 0.010 of the planet (length times width, our
arithmetic). The arcs standing alone are Earth's.

**The arcs' magma** (`tools/review/islfront/cbread.py`, `tools/review/loss-budget.py`, base36): island arcs 1.82
km³/yr (Stern & Scholl 2010's intra-oceanic arcs 1.5), continental arcs 0.48 (1.0); the intra-oceanic front 24,800
km; 0.40 of the island-arc magma made over the run still stands at the end. The additions are Earth's; the stock is
not the arcs' production.

**The start's own crust** (`arcstock.py`, samples with formed_myr 0, which a continental sample keeps through a
resampling; base37, 24 runs): area 0.423 of the planet at -1,000 Myr, 0.261 at -500, 0.156 at the end; volume (km of
crust over the planet's mean) 16.11, 9.81, 5.68; mean thickness 38.1, 37.8, 36.9 km. The basement rule relabels only
0.040 of it as converted. The continental volume stays 16.1 to 16.9 while the island-arc crust's grows from 0 to 4.8.

**Where the start crust's volume goes** (the crust budget's `cb_parts`, one run, supercontinent-1 of base37, 10⁶ km³
over 1,000 Myr, so km³/yr after dividing by 1,000): visible continental crust going under another plate
(`subduction`) -12,434, the hidden stock gaining it +12,470; the hidden stock removed at resamplings
(`resample_blend`) -12,332; the overriding plates' belts thickening (`belts`) +11,034. A collision underthrusts about
12 km³/yr of continental crust and the overriding belts take about 11 back as thickening (Earth's continental
subduction, Clift et al. 2009 as quoted by Stern & Scholl 2010, 0.4 km³/yr, is crust lost, not stacked). Only about
1.3 km³/yr is lost at the resamplings. So the start crust is mostly not destroyed: it is stacked under the
overriding plate. Where the overriding sample is crust converted on ocean floor (an arc), the stack keeps that
sample's label and its formed_myr, so the per-sample traces read the start crust as lost and the arc as gained. The
felsic rock booked: belts +8.04 km³/yr, the resamplings -11.27 (base37 means).

**The explicit loss rows against Stern & Scholl 2010's Table 1** (base36): sediment subduction 2.34 km³/yr (Earth
1.1), subduction erosion 0.31 (1.4), crust-suturing zones 0.60 (0.7), foundering 0.00 ("Unknown" in the table),
resampling drops 0.19. Total about 3.4 against Earth's about 3.2.

**The rock and the lithosphere of each class at the end** (`arcstock.py` on the arm arc-lith: base37-s4's code and
seeds with the crust budget's new `lith/t<Myr>.bin`, a measurement only, its census identical to base37-s4's in every
column; medians of 8 final frames; the L the laws read, `thermal::sample_l_km`):

| class | thickness, km | felsic and metamorphic share | height over the sea, m | L, km | keel, km | heat flow, mW/m² |
|---|---|---|---|---|---|---|
| the start's crust | 37.4 | 0.60 | +49 | 222 | 200 | 41 |
| island-arc crust taken in | 30.3 | 0.49 | -620 | 135 | 0 | 51 |
| collision belts on ocean floor | 32.0 | 0.45 | -255 | 137 | 0 | 51 |
| the rest converted | 25.6 | 0.36 | -1,506 | 136 | 0 | 51 |
| island-arc crust alone | 20.3 | 0.57 | -1,951 | 81 | 0 | 71 |

The converted crust is half felsic, so its rock is continental enough; it is thin (30 km against Earth's continental
mean of 36.4 km, sea-level.md) and it stands on a cold mantle lithosphere with no keel, about 135 km deep, where
Earth's hot back-arcs stand on 50 to 60 km (`thermal::L_HOT_KM`, PLT-461, from Hyndman, Currie & Mazzotti). The thermal
state heats a continental column only while it carries an arc, subduction-zone or hotspot flag (`thermal::heating`),
in the arc's band; off it, and after the arc dies, the column cools toward a craton's lithosphere over McKenzie's time
with no keel to buoy it. Candidates (to read against sources before building): how far behind the arc and for how long
Earth's back-arcs stay hot (Hyndman & Currie: 200 to 900 km behind the arc, cooling over about 300 Myr after
subduction stops, to be read again); and what thickens accreted arcs on Earth (collision and magmatic addition) where
the model's stay at 30 km.

**Back-arc heating is not the lever.** The model already heats every overriding continental sample within the
subduction margin's band (800 km on quick_16k, HYP-015) towards L_HOT, and Earth's back-arc heat decays over about 300
Myr after subduction stops (Currie & Hyndman 2006, read 2026-10-23), with regions older than about 300 Ma on 150 to 300
km of lithosphere (Hyndman, Currie & Mazzotti 2005): the converted crust's 135 km is within Earth's range.

**The continental arcs' rate is a mismatch of measures, not of the law.** `cbread.py`'s arc rate per km divides arc
magma by the crust it lands on (`ARC_CONTINENTAL_GLOBAL`: on continental samples) by fronts classed by the crust within
`arc_distance_km` (`ARC_FRONT_CONTINENTAL_GLOBAL`): a continental arc's magma that falls on the ocean fringe in front of
the coast, or follows the island law on crust an intra-oceanic arc made (PLT-760), books as island-arc magma. The two
rows together, 2.30 km³/yr on base36, are Stern & Scholl's 2.5.

**So far, then:** the arcs' magma, the arcs standing alone and the explicit losses are Earth's; the continental volume
is 16.4 km of planet-mean crust against Earth's 14.0 (36.4 km over 0.384 of the planet, sea-level.md), 17 % more, over
about 24 % more area, so 13 % thinner; the thin converted crust (25 to 33 km) on cold, keelless lithosphere stands
under the sea. The first lever is what spreads the crust thin (the collapse to its normal column, PLT-412; the
margin's sediment counted continental, PLT-661, "the rest converted" carries 18 % cover; the transition; erosion and
redeposition), and the second the extra volume; arc foundering (Jagoutz & Kelemen 2015) would remove volume but is
not shown to be missing until the arcs' net rates are read as net or gross.

**Foundering would count twice.** The arcs' rates (PLT-045, PLT-285) are Stern & Scholl 2010's crustal growth rates,
the arc crust standing today over its age (p. 12: "adopted as a useful long-term rate estimate for continental margin
arc productivity"; Izu-Bonin-Mariana's 130 to 135 from its crust's volume): net of whatever foundered. A foundering
law on top of them would remove rock the rates already left out, so it is not a missing law at these rates.

## Earth's side (sources read by Sonnet lookups, 2026-11-01; bibliography entries to add when used)

- Condie & Aster 2010 (Precambrian Res. 180:227-236): "only 14% during the last 400 My" (the body says 600 My).
- Hawkesworth, Cawood & Dhuime 2019 (Geosci. Front. 10:165-173): "65%-70% of the present volume ... generated by 3 Ga".
- Jagoutz & Kelemen 2015 (Annu. Rev. Earth Planet. Sci. 43): about two-thirds of the Kohistan arc's magma removed by
  delamination or subduction erosion; density-unstable cumulates 60 to 70 % (55 to 70 % of the original melt's mass).
- Lee & Anderson 2015 (Sci. Bull. 60:1141): garnet pyroxenites 50 to 70 % of the parental magma; cumulate recycling 1.5
  to 6 km³/yr. Jagoutz & Behn 2013 (abstract only): foundering every 0.5 to 5 Myr.

## Open questions, in order

1. What the stacked columns are: the felsic share and the pools of the "taken in" crust against Earth's continental
   crust's layers (Rudnick & Gao's as Hacker et al. 2015 print them, PLT-834), and their heights under the law.
2. Whether the label (cb_origin's) or the rock is wrong: a column of start crust stacked under an arc is continental
   crust on Earth (an arc-continent collision, the arc accreted to the margin).
3. Answered: the arcs' rates are net (above), so no foundering law at these rates.
4. Subduction erosion at 0.31 against Earth's 1.4 km³/yr.
