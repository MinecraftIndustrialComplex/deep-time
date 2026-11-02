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

**The arm msc-off** (2026-11-01; master 2f6cd34c with `tectonics.margin_sediment_continental=false`, PLT-661's margins
off, 8 runs against base37-s4; a measurement, the switch stays on: it is Ben's decision of PR #49): land 0.202 against
0.171 (p 0.13), continental crust under the sea 0.385 against 0.686 (p < 0.001), land above 2 km 0.055, coast with
high ground 0.180 (p 0.038), collisions per 100 Myr 3.45 against 5.15 (p 0.001), the continental crust's net change
over the run -0.136 against +0.115 (p < 0.001), the criteria a median of 5.5 against 5 (mean 5.38 against 4.62, p
0.046), the sea at the end -224 against 124 m; the guard clean (11,016 against 10,210 m, p 0.23). The census counts
the continental label (RLC-030: Earth 0.29 to 0.31 by Marcilly et al.'s continental polygons, which take the margins
to the continent-ocean boundary), so the margins belong in it on both sides; by our arithmetic (land over one less
the drowned share) the model's continental crust covers about 0.545 of the planet with PLT-661 on, 0.33 with it off,
against Earth's 0.41 (210.49 x 10⁶ km² of 510). So PLT-661's margins are most of the drowned share: on, the
continents' area is too large and thin; off, the continents lose 14 % of their crust over the run.

**Why PLT-661's margins cover so much** (Ben, 2026-11-01: "Keep on, find why"; `arcstock.py` on arc-lith, the
switch on, and msc-off, off; final frames, medians of 8):

| class | on: area, thickness, height over the sea | off |
|---|---|---|
| the start's crust | 0.111, 37.4 km, +49 m | 0.067, 37.7 km, +257 m |
| island-arc crust taken in | 0.153, 30.3 km, -620 m | 0.108, 33.4 km, +67 m |
| belts on ocean floor | 0.101, 32.0 km, -255 m | 0.044, 36.4 km, +119 m |
| the rest converted | 0.159, 25.6 km, -1,506 m | 0.044, 34.0 km, +116 m |
| hyper-extended (under 10 km of its own crystalline crust) | 0.102, 17.7 km, -3,102 m | 0.005, 13.0 km, -2,623 m |
| all continental crust | 0.537, 30.4 km, -327 m | 0.286, 34.2 km, +180 m |

- The rule's reach is Earth's: the hyper-extended crust lies a median 70 km (p90 104 km) from the continental crust
  that is not hyper-extended, a ring of 0.8 lattice rows (the ocean side of the 200 km taper, PLT-262; Brune et al.
  2014's thinned domains 70 to 200 km wide).
- What it runs along is long: the non-hyper continental crust's edge, measured as its samples within 1.6 spacings of
  ocean floor or hyper-extended crust times the spacing, is about 896,000 km with the switch on and 587,000 off (a band
  on the lattice, so over the true perimeter by perhaps two to three times). The ring is 19 % of the continental
  area.
- The rule links the crust rather than breaking it up: 13.5 parts over 10⁵ km² with the largest seven holding 0.99 of
  the area, against 33 parts and 0.93 off.
- So the margins' area is one Earth-wide ring along every edge of continental crust, the converted crust's included,
  wherever the floor beside it holds continental rock. Earth's thinned domains are the product of rifting (Brune et al.
  2014 on rifted margins); PLT-661 stands in for rifting by the sediment on the floor, which comes down from any land
  beside any margin (sheared, transform, old active ones); a converted ring turns back only outside the zone's ocean
  side (PLT-667).

**Earth's margins** (sources read by a Sonnet lookup, 2026-11-01, texts extracted from the PDFs):
- Bradley 2008 (Earth-Sci. Rev. 91, 1-26): "The present-day passive margins have an aggregate length of 105,000 km";
  lengths "taken as the great-circle distance between endpoints"; sheared margins and back-arc basins' continental
  sides included.
- Mercier de Lepinay et al. 2016 (Tectonophysics 693, 96-115): "With an aggregated length of 25500 km, transform
  margins represents 16% of the length of continental margins, and 31% of that of non-convergent continental
  margins." Our back-calculation: about 159,000 km of continental margins in all.
- Chenin et al. 2017 as Manatschal & Chenin 2023 (Geol. Field Trips and Maps 15) quote it: necking domains "between
  ca. 10 and 100 km with an average of 55 km; hyperextended domains between ca. 20 and 70 km with an average of 50
  km", the hyper-extended domain being where "the crust is thinned from ca. 10 km to 0 km".
- Harris et al. 2014 (Mar. Geol. 352, 4-24): "the Earth's margins are approximately 35% active and 65% passive".
- Marcilly et al. 2022 (Gondwana Res. 110): continental crust 210.5 x 10⁶ km², flooded 61.5 x 10⁶ km², which
  "includes areas of for example stretched continental crust (pre-breakup extension) along passive margins".
- No global area of hyper-extended crust was found. Our arithmetic: 105,000 km x 50 km is about 5.3 x 10⁶ km², 0.010
  of the planet.

**Where the margins' area is** (2026-11-01; `arcstock.py` and `ratchet.py` on arc-lith, the switch on, and msc-off,
off; medians of 8; the lattice is 100,000 samples, a spacing of 71 km; lengths are neighbour pairs across the line
times a cell's edge, km):

| final frame | on | off |
|---|---|---|
| hyper-extended crust (all) | 0.102 of the planet, 83 % sediment margins | 0.005 |
| of it on the coast row / the second / three rows and more inland | 0.062 / 0.020 / 0.020 | 0.0046 / 0.0005 / 0 |
| continental boundary, raw / smoothed over two spacings | 602,000 / 416,000 km | 592,000 / 341,000 km |
| its share with hyper-extended crust on the land side | 0.75 | 0.08 |
| ocean floor enclosed by continental crust | 0.095 (0.013 by crust of 10 km and more of its own) | 0.0095 |

- The ring per km of coast is Earth's within the lattice: one row (71 km, the hyper-extended crust a median 70 km
  from crust of 10 km and more of its own) against Chenin's 50 km average and Brune's 70 to 200 km wide sides.
- No ratchet: the converted crust that is neither island arc nor belt has 0.074 of the planet under 10 km of its own
  crystalline crust, 0.075 at 18 km and more, and 0.0019 between 10 and 14 km, so rings do not grow across the line
  and step outward. It lies within one to three rows of island-arc crust, belts or the start class in both arms
  (p90 3 rows).
- The coast row's ring covers 0.73 of the continental coast row. Earth's share of rifted and sheared margins is about
  0.65 (Harris et al. 2014), so the share is near Earth's.
- The length is not Earth's. The model's continental boundary is 602,000 km (416,000 smoothed), against Earth's
  105,000 km of passive margins (Bradley 2008) and about 159,000 km of all margins. That is 2.6 to 3.8 times
  Earth's, though Bradley's endpoint lengths undercount curved margins. It is near the same with the switch off. So
  the coast row's ring of 0.062, against Earth's 0.010 by our arithmetic, is the length about four times over and
  the lattice's one row against 50 km.
- The length grows early in both arms: 84,000 km at the start, about 205,000 at -950 and about 310,000 at -900. At
  -900 Myr the island-arc crust's share of it is 0.20 (on) and 0.31 (off). One run's record (msc-off, seed 1) has 14
  plates at the start and about 40 by -934 Myr; the continents split across new plates (plate 13's 10,149
  continental samples lie on four plates by -966 Myr). At the end, lattice-scale raggedness (raw over smoothed) is
  31 % of the length on and 42 % off.
- The part three rows and more inland (0.020 of the planet; 88 % sediment margins) is no margin ring. It is columns of
  22.5 km: about 7 km of ocean basement, 5.5 km of crystalline crust of its own and the rest cover. They stand 2.4 km
  under the sea. Half were raised by an orogeny. They are on the plate of the crust beside them (99.8 %). They are
  margins a collision carried in and welded, and they keep their sediment as cover. The taper still counts them as its
  ocean side (any hyper-extended crust is), so it holds them at the margin's profile inland.
- The rings close straits: the ocean floor enclosed by continental crust (an ocean component that is not the world
  ocean, linked within 1.6 spacings) is 0.095 of the planet in 194 pieces. It is old (a median 134 Myr against the
  world ocean's 63) and 88 % of it is on the plate of the continental crust nearest it. Counting hyper-extended crust
  as the ocean side leaves 0.013, as the switch off does. The enclosed floor's rim is 0.75 hyper-extended crust.
  This is a reading of the label (water crosses the drowned rings); it adds to the boundary's length.

So the rule's reach per km is Earth's. Its area is the boundary's length (about four times Earth's, near the same with
the switch off), a second row on part of it, and the welded margins inland. The fixes, in order of size: (1) what
lengthens the continental boundary in the first 100 Myr (the continents split across new plates, the island-arc pieces,
the lattice raggedness); (2) margins a collision welds inland, which Earth's orogens take in (Chenin et al. 2017's title
reads the orogens closed narrow and wide oceans made from hyper-extended margins; not read beyond that) where the model
keeps them as margins at 2.4 km under the sea; (3) the share of edges the rule rings (0.73 against about 0.65).

**What lengthens the boundary: the nearest label's walk** (2026-11-01, branch `margin-level-set`, PR #63). A resampled
sample took its kind and class from its nearest old corner (PLT-664), the rule PLT-536's level set had replaced for plate
outlines. The rule keeps a class's area on average but moves its edge by up to half a spacing at random at every resampling.
The M5 lane measured it in 2026-10-25 ("Which old sample a new one follows", plates.md): 143 flecks and 138 holes on a
supercontinent after 70 resamplings. A new test, `how_resamplings_roughen_a_margin`, puts continental crust and its own
plate's ocean floor either side of the equator. Over 40 resamplings the margin's spread grows from 0.44 to 1.64 spacings
and its edges per spacing from 2.26 to 4.56. A run resamples about 50 times (every 20 Myr), so this alone about doubles a
margin's raw length. It matches the raw over smoothed lengths above (1.04 at the start, 1.45 on and 1.74 off at the end).
The fix is a numerical one (PLT-852, plates.md "The kind and class on level sets, each piece's area kept"): the crust's kind
and class on level sets, each piece keeping its count of samples. The arm `kls-on` reads it on the pool.

**The sediment the margins hold** (`seased.py`, the same arms; GlobSed, Straume et al. 2019, Table 3 as bibliography.md
quotes it: about 3.37 x 10⁸ km³ over about 3.63 x 10⁸ km², on the margins 1.43 x 10⁸ km³ over 4.69 x 10⁷ km², beyond
200 km of the continent-ocean boundary 1.13 x 10⁸ km³ over 2.79 x 10⁸ km²):

| final frame | on | off |
|---|---|---|
| sediment and sedimentary rock on the ocean floor | 3.78 x 10⁸ km³ (a mean of 1,554 m) | 5.26 x 10⁸ km³ (1,446 m) |
| of it within 200 km of continental crust | 2.93 x 10⁸ km³ over 8.5 x 10⁷ km² (3,397 m) | 2.86 x 10⁸ km³ (3,378 m) |
| beyond 200 km | 8.5 x 10⁷ km³ (559 m) | 2.45 x 10⁸ km³ (862 m) |
| the cover of continental crust on an ocean basement | 6.68 x 10⁸ km³ | 1.84 x 10⁸ km³ |

- With the switch on, the ocean floor's sediment is near GlobSed's, in total and near the margins (3.4 against 3.05
  km). The converted margins hold another 6.7 x 10⁸ km³ of cover, twice GlobSed's whole ocean, and nearly all of it
  under the sea. GlobSed's area is the ocean's, shelves and slopes included. So the sediment under the sea is about
  three times Earth's, and the margins hold two thirds of it. With the switch off it lies on the far ocean floor (862
  m against 405).
- At the trenches (the ocean floor flagged at a trench at -500 and -100 Myr; the final frame carries no step flags), a
  third of the samples hold 1 km or more (0.27 to 0.34 on, 0.34 to 0.39 off; near continental crust 0.44 to 0.56), a
  median of 100 to 300 m. So PLT-442's sediment gate is not what keeps subduction erosion small (below).

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
4. Subduction erosion at 0.31 against Earth's 1.4 km³/yr. arc-lith's globals (two runs): the erosive fronts (PLT-442)
   average 9,100 km of the fronts' 52,800 (straight, PLT-530), 85 % of them intra-oceanic. At PLT-441's 90 km³/km/Myr
   they ask 0.82 km³/yr. The band gives 0.27; 0.55 (`plates/forearc_short_km3`) is not there, since an intra-oceanic
   forearc holds little crust beyond a normal ocean floor's 7 km. So the gap is the length that erodes (17 % of the
   fronts) and the forearcs' crust, not the rate. Next: which gate stops the rest (the 60 mm/yr normal convergence),
   and Clift & Vannucchi 2004's share of erosive margins by length.
   Earth's side (`tools/review/islfront/subgate.py` on Bird 2003's PB2002_steps.dat, 2026-11-01; it reproduces Table 3:
   1,127 SUB steps, 51,309 km, a length-weighted mean of 53.2 mm/yr across the step): 0.441 of Earth's subduction
   length converges at 60 mm/yr or more (0.537 at 50, 0.299 at 70). base38's fronts converge at a mean of 42.8 mm/yr
   (intra-oceanic) and 55.7 (continental arcs) normal to the front (`cbread.py`), against Bird's 53.2. Our arithmetic:
   if about two thirds of Earth's fast trench also carries under 1 km of sediment, about 0.29 of 51,309 km erodes,
   about 15,000 km, and at PLT-441's 90 km³/km/Myr that is about 1.35 km³/yr, Stern & Scholl's 1.4. The model's
   9,100 km is about 0.6 of that, and its forearcs give a third of what that length asks.
   The model's own speeds (`islrec.py` on base38-s4, the record's front bands, the subducting front's share by its
   normal convergence) are Earth's: at 40 mm/yr or more 0.539 of the intra-oceanic front and 0.661 of the continental
   arcs' against Bird's 0.624; at 80 or more 0.196 and 0.298 against 0.170. So the speed gate is not the term (Ben,
   2026-11-01: "Arc crust and fronts (Recommended)"). What is short is the forearcs' crust: 0.55 of the 0.82 km³/yr the
   erosive length asks finds no crust beyond an ocean floor's 7 km within the arc distance of the front, and 85 % of
   the erosive length is intra-oceanic. Next: what crust stands within 200 km of the model's intra-oceanic fronts
   against Earth's (the forearc and arc crust of Izu-Bonin-Mariana and Tonga in the sources read: Stern & Scholl
   2010's juvenile arcs up to 30 km thick; Bai et al. 2023's Kyushu-Palau remnant arc 8 to 23 km), and the young-floor
   excess the oceanic-fronts lane left (plates.md, "The intra-oceanic fronts' length").
   The crust behind the fronts (2026-11-01; `tools/review/islfront/forecrust.py` on base38's 24 runs, the maps at -950
   to -50 Myr, against `earthisl.py` on Bird's SUB steps and CRUST1.0, whole crust on the overriding plate). The share
   of the subduction front with no crust of T km within 200 km of the trench on its own plate:

   | T | Earth (51,309 km) | base38, all fronts | intra-oceanic (0.39) | continental (0.61) |
   |---|---|---|---|---|
   | 10 km | 0.079 | 0.294 | 0.731 | 0.014 |
   | 15 km | 0.165 | 0.399 | 0.952 | 0.047 |
   | 20 km | 0.195 | 0.476 | 0.986 | 0.150 |
   | 25 km | 0.308 | 0.575 | 0.994 | 0.307 |

   (At 77 km, a model spacing: Earth 0.158 at 10 km, the model 0.617.) The intra-oceanic fronts stand on bare ocean
   floor: the thickest crust within 200 km is 8.2 km at the median (p90 13.1), and within 400 km 0.62 of them have none
   of 15 km, so the arc is not left behind beyond the reach. They are not young: 0.67 of the intra-oceanic front belongs
   to contacts older than 50 Myr, 0.36 to contacts older than 100 Myr (`islrec.py`'s contact-age split, base38-s4's
   record). Their arcs are not there. The arc's band is 140 to 260 km behind the front (PLT-042's 200 km, PLT-043's
   60 km); the most arc magma on any sample in it is 2.0 km at the median (p25 0.7, p90 9.3; 8 runs), where the
   continental fronts carry 19.1. At base38's island-arc rate per km of front (61 km³/km/Myr, `cbread.py`) over the
   band's Gaussian (about 106 km wide), a front that stood T Myr carries about 0.58 T km at the axis (our arithmetic):
   2.0 km is about 3.5 Myr of standing, 9.3 km about 16 Myr. An island arc turns its ocean column continental at 15 km
   (PLT-047), and the front with it reads continental (PLT-370), so the intra-oceanic front is, by its class, the front
   whose arc has not grown; what is wrong is that it does not grow at fronts that have lasted 50 Myr. The forearc floor
   is younger than those contacts too (median 36 Myr, p10 13). Either the front moves across its overriding plate's rock
   (the design says it does not: PLT-710, "Fronts are fixed in the plate that holds them"), or the magma reaches those
   fronts seldom (the map's front flag drops where a sample's convergence falls under the slab's floor, PLT-681). The
   record's turnover says the front changes fast: of the island subducting front (about 22,300 km) 3,246 km is new each
   2-Myr step and 3,201 km gone, the continental front alike. `frontstay.py` counts it on the same rock
   (look2, plates.md "Why the island arcs do not grow: a front's life on its rock"): a front lasts about 20 Myr where
   it stands (0.58 to 0.66 still a front on its rock after 10 Myr, 0.38 to 0.52 after 16), and it ends, it does not
   wander. A third or less of the ending is the plates' motion turning; most of it is the subducting plate's crust
   running out while the plates still converge: its own ridge 200 to 300 km in front of the trench, young floor (18 to
   23 Myr) between, an open gap behind (no floor until the next resampling). The arcs and the young-floor excess are
   the boundaries' shape (the same pair spreading and converging a few hundred km apart, plates.md "Where the pairs'
   relative poles lie"), not the arc law or the front's. Ben's choice of 2026-11-01 left plate-motion design out; the
   next step is his (STATUS, pending questions).
