# A plume tail's melt under its lid: design

Status: **built with `tectonics.hotspots_sourced`** (2026-10-31, the hotspot lane, branch `hotspot-lid`
on `hotspot-lip`), which stays off by default; ledger PLT-805 to PLT-808. The arm is in section 5.

## 1. Why

With the sourced tails (PLT-790 to PLT-794) every tail makes Hawaii's magma per unit of buoyancy flux
(Sleep 1990: 4 m³/s at 8.7 Mg/s), whatever lies above it. With the cone budget on (PLT-803) the arm
hs-cum2 still put 0.43 km³ a year of tail magma on continental crust, nearly Stern & Scholl's whole
row for rifts, hotspots and volcanic margins (0.5), and grew the continents (+0.53 km³ a year, the
sea 679 m). The sources say a plume melts by decompression only above the depth where its adiabat
crosses the solidus and only up to the base of the rigid lid, so a thick lid melts less and a
craton's none (section 2). This design gives the tails that law.

## 2. The sources (what was read)

One Sonnet research sub-agent (2026-10-31); everything is in bibliography.md "The sources of the
hotspot lane's lid (2026-10-31)" with what was read of each.

- **The solidus.** Katz, Spiegelman & Langmuir 2003 (full text, Table 2): the anhydrous solidus
  T = A1 + A2 P + A3 P², A1 = 1085.7 °C, A2 = 132.9 °C/GPa, A3 = −5.1 °C/GPa². Korenaga, Kelemen &
  Holbrook 2002 (full text): depth at 30 km per GPa. Fitton et al. 2020 (open access, read through
  the fetch tool's summary): melting with the Katz dry solidus along a 0.3 °C/km adiabat, "no melt
  can form in volatile-free mantle with Tp = 1300 °C when lithosphere thickness exceeds 57 km,
  whereas it would take a lithosphere thickness of ∼120 km to prevent melting of mantle with Tp =
  1500 °C"; "Fmax is the melting reached at the base of the lithosphere"; productivity "1·2 % kbar–1
  (0·4 % km–1)".
- **The plume's temperature.** Herzberg et al. 2007 (full text, para. 68): "Our calculated potential
  temperature below Hawaii is ∼1550°C"; Watson & McKenzie 1991 (abstract): Hawaii's plume "has a
  central potential temperature of 1558 °C and the mechanical boundary layer is 72 km thick", 0.16
  km³ a year of melt.
- **The lid.** White & McKenzie 1989 (full text, Fig. 3 caption): "mechanical boundary layer
  thicknesses of 70, 100, and 130 km correspond to plate thicknesses of 87, 118, and 149 km". Their
  melting stops at the mechanical boundary layer; the plate's base is the thermal lithosphere's.
  Humphreys & Niu 2009 (full text): Hawaii's lavas erupt "on lithosphere of full thickness (>70 Ma,
  and hence ≥90 km thick)". Ribe & Christensen 1999 (abstract): Hawaii's melt, swell height and width
  "can only be matched if zl≤89 km".
- **The law's form.** Niu 2021 (full text, p. 20): "the extent of melting beneath the thin
  lithosphere in Iceland can be up to 8 times greater than that beneath the thick lithosphere in
  Hawaii (linear scaling is assumed here for conceptual clarity)", from the melting interval PO − PF
  (onset less final depth); p. 22: plumes "will not melt beneath thickened cratonic lithosphere lid".
  Sleep 1990 (full text, p. 6719): "Most of the plume material beneath Hawaii remains at lower
  lithospheric depths where extensive melting does not occur". Manglik & Christensen 2006 (through
  the fetch tool): "We do not observe any partial melting beneath the cratonic root because the root
  is thick enough (200 km)". Courtillot et al. 2003 (full text): weak plumes "would not melt beneath
  old lithosphere".
- **A check.** Wang & Li 2021 (full text, Table 2): 3-D plume models with Katz's melting, plume
  excess 300 °C, melt production 0.41, 0.29, 0.13 and 0.049 km³ a year under lithosphere 100, 120,
  140 and 160 km thick.

Not obtained: McKenzie & Bickle 1988's solidus and tables (their 1990 note gives 8.2 and 27.3 km of
melt at 1280 and 1480 °C for infinite stretching), Watson & McKenzie 1991 and Ribe & Christensen
1999 beyond their abstracts (Ribe & Christensen's scaling law came through Sobolev et al. 2005's
supplement and could not be reproduced), Albers & Christensen 1996, any continental hotspot's melt
flux with its lid.

## 3. The law (PLT-805 to PLT-808)

A tail's plume is Hawaii's temperature, 1550 °C (PLT-806). It begins to melt where its adiabat,
0.3 °C/km, crosses Katz's solidus, at depth z₀ (30 km per GPa; PLT-808): 138.9 km at 1550 °C (the
agent's arithmetic; the same construction gives 56 km at 1300 °C and 120 km at 1500 °C, Fitton et
al.'s 57 and ∼120). It melts up to the base of its mechanical boundary layer, the lid: the model's
thermal lithosphere L (`Sample::lithosphere_km`, the plate's base) less 18 km, White & McKenzie's
offset between plate and mechanical boundary layer (17, 18, 19 km in their caption; PLT-807). Each
parcel that rises to the lid has melted in proportion to the interval it rose through above the
solidus (Fitton's F_max, Niu's linear scaling), so the tail's melt per unit of flux is Hawaii's
times

  f(L) = max(0, z₀ − (L − 18)) / (z₀ − 72),

72 km Hawaii's mechanical boundary layer (Humphreys & Niu's ≥ 90 km plate less 18; Watson &
McKenzie's 72; PLT-807): f = 1 under Hawaii's lid, 0 at a lid of 157 km or more (a craton's keel,
200 km, PLT-463), 1.45 under the 60 km a plume heats a keel-less continent's lithosphere to (PLT-461,
PLT-464), 2.08 with no lid (a ridge, the lid at 0; Iceland's twelve times Hawaii's melt per unit of buoyancy flux,
Sleep 1990, holds the ridge's own upwelling, which is not this law's).

The tail's rock over a step is its magma (PLT-793) times the kernel-weighted mean of f over the
samples under it, laid on each sample in proportion to its kernel weight times its own f (PLT-805).
With no thermal state (`tectonics.thermal_state` off, L = 0) f is 1. The cone budget (PLT-803) then
takes its lavas from that rock.

The check against Wang & Li's models (the agent's): the law's ratios at lithosphere 100, 120, 140 and
160 km, with a plume at their 300 °C excess over a 1300 °C mantle (1600 °C, z₀ = 159 km, the lid L −
18), are 1, 0.74, 0.48 and 0.22 against their 1, 0.71, 0.32 and 0.12: the same fall, a little
slower (their lid is not stated as plate or boundary layer; their ambient temperature was not read).

What the law leaves out: weak plumes' cooling (Courtillot et al.'s 1 Mg/s), the lid's thinning by
the plume itself on the ocean floor (the model heats only continental lithosphere under a plume),
and a ridge's own upwelling over a plume.

## 4. Tests

- `the_lid_constants_are_their_sources` (plates/tests.rs): Katz's solidus crossing at 1300 and 1500
  °C against Fitton et al.'s 57 and ∼120 km; z₀ at 1550 °C; f = 1 at Hawaii's plate (90 km), 0 at a
  craton's 200 km, 1.45 at 60 km.
- `a_tail_melts_less_under_a_thicker_lid` (plates/tests.rs): one Hawaii tail over the planet adds
  Sleep's 4 m³/s under Hawaii's lid, f(L) of it under another, none under a craton.

## 5. The arm

hs-lid against base35-s4 (the next master base) when it exists; until then the design's estimate:
the start's continents carry 200 km keels (PLT-463), so tails under them melt nothing; keel-less
continental crust under a plume heats to 60 km and melts 1.45 times Hawaii; old ocean floor (the
model's L 104 to 110 km at 80 to 90 Myr, PLT-466) about 0.75.
