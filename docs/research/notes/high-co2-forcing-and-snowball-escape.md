# CO₂ forcing at 0.05–1 bar and the snowball's escape

Date: 2026-10-03. Research task for P1-26 T6, after Ben chose "Research, then fix (Recommended)"
(decisions log 2026-10-02). Ledger rows concerned: CLM-069 to CLM-073, CLM-166. No code was changed.
Sources were read in the project's source library (one folder per key, named below); the numbers
marked "computed" come from scripts kept in the library folder `HEXTOR_software/analysis/`.

## 1. Recommendation in brief

1. **Forcing law.** Replace the Myhre/Byrne–Goldblatt blend (CLM-069 to CLM-071) at every
   concentration with a temperature-dependent forcing taken from a published radiative-convective
   (RC) calculation that covers 10⁻⁴ to 10 bar of CO₂:

   ```
   F(C, T) = m · [ OLR_RC(T, 280 ppm) − OLR_RC(T, C) ]          m = 0.903
   OLR = A + B·T − F(C, T) − c_snow·frozen        (A = 203.3 W/m², B = 2.09 W/m²/K, as today)
   ```

   - OLR_RC is the cloud-free OLR table that ships with HEXTOR (Haqq-Misra & Hayworth 2022, MIT
     licence), computed with the Kopparapu et al. 2013 RC model for 1 bar of N₂ plus CO₂.
   - It is interpolated linearly in T and in log₁₀ pCO₂, at pCO₂ = C·10⁻⁶ bar.
   - T is each tile's own surface temperature.
   - m scales the clear-sky table so that doubling CO₂ at 288.15 K gives Myhre's all-sky
     3.71 W/m².
   - F is zero at 280 ppm, so Earth's calibration is unchanged.
2. **Clouds.** Do not add a blanket cloud term by default (c_snow = 0). A + B·T already carries
   about 20 W/m² of longwave cloud greenhouse at snowball temperatures (computed in §4). That is
   the size the cloudy GCMs give a snowball's clouds, and more than FOAM's, the cloud-free model
   that would not deglaciate. Keep c_snow as a setting (0–20 W/m², on frozen surfaces) for
   sensitivity runs. Choosing a non-zero default is a game choice for Ben (§4.3).
3. **Expected escape at today's Sun (1365.2 W/m², ice albedo 0.62).** These come from a 1-D
   stand-in for the clock (§5.3), which reproduces today's escape to within about 30 %.
   - Today's law: about 3.3·10⁵ ppm.
   - Recommended law: about 1.9·10⁵ ppm.
   - Recommended law with c_snow = 10 W/m²: about 1.1·10⁵ ppm.
   - With c_snow = 20 W/m²: about 6·10⁴ ppm.

   The snowball scenario's Sun (0.938 S₀) gives 2.7·10⁵, 1.8·10⁵ and 1.1·10⁵ ppm with c_snow = 0,
   10 and 20 W/m². Today's model gives 5.5·10⁵ ppm there.
4. **How to read Hoffman et al.'s 10⁴–10⁵ ppm.** It is the geochemical (proxy) range for
   terminations. The physical models put CO₂-only escape for a Cryogenian Sun higher:
   - radiative-convective models: 0.21 bar;
   - diffusive EBMs: 0.29–0.9 bar inventory;
   - GCMs: 0.38 bar to beyond 1 bar.

   The cloudy GCMs bring escape down by a factor of 10–100 relative to FOAM, and so does dust,
   which lowers the tropical ice albedo. After the fix, an escape of 1–3·10⁵ ppm agrees with the
   physical models. Getting inside 10⁴–10⁵ needs an extra 20 W/m² or an ice albedo near 0.55, as
   a dusty sublimation zone would have (§5).

## 2. What "ppm" means in the model, and the conversion used

- **The model's unit.** In `carbon.rs`, ppm = 280 × atm/ATM_REF_MOL, with ATM_REF_MOL = 280·10⁻⁶
  × 1.773·10²⁰ mol of air. So C ppm means n_CO₂ = C·10⁻⁶ × n_air, an inventory ratio. It is not
  a mole fraction.
- **Pierrehumbert et al. 2011's conventions** (Table 1, read):
  - pI,CO₂ is the inventory as a surface pressure.
  - pCO₂ is the partial pressure in the mixture.
  - The molar concentration is (pI,CO₂/m_CO₂)/(pI,CO₂/m_CO₂ + pI,a/m_a).
  - Example: an inventory of 10⁵ Pa gives 39.7 % molar, pCO₂ = 7.95·10⁴ Pa and p_tot = 2.00·10⁵ Pa.
- **Mapping to the model.** The model's C maps to these as follows (computed; checked against
  Table 1's 10⁵ Pa row):

  ```
  C' = C·10⁻⁶;  mole fraction χ = C'/(1+C');  pI,CO₂ ≈ 1.519·C' bar;  p_tot ≈ 1 + 1.519·C' bar
  ```

- **The table's convention.** It holds pN₂ = 1 bar and adds CO₂, so P = 1 + pCO₂ and
  f = pCO₂/P. Looking it up at pCO₂ = C' gives the same mole fraction as the model's atmosphere.
  The table's total pressure, 1 + C', is lower than the true 1 + 1.52·C', so the lookup slightly
  underestimates pressure broadening. That is the conservative side.
- **Literature values in the model's ppm:**
  - Hu et al.'s 0.21 bar (RC, 1 bar + pCO₂) is about 2.1·10⁵ ppm.
  - Pierrehumbert's inventories of 0.29, 0.38, 0.9 and 1.0 bar are about 1.9, 2.5, 5.9 and
    6.6·10⁵ ppm.
  - Hoffman's 10⁵ ppm by volume is about 1.1·10⁵ ppm.
  - Today's escape at 5.5·10⁵ ppm is a mole fraction of 35 %, pCO₂ ≈ 0.65 bar and an inventory
    of about 0.84 bar.

## 3. The forcing law

### 3.1 Candidates checked

| Candidate | Range | Verdict |
|---|---|---|
| Byrne & Goldblatt 2014 (today's law above 2,000 ppm) | Fit over 200–10,000 ppmv, computed to 50,000 ppmv; all-sky, instantaneous, at the tropopause, for today's atmosphere profiles. | Good inside its range and independent of the table: at 288 K, 10⁴ ppm gives 24.1 against the table's 22.7 (× m), and 5·10⁴ gives 38.1 against 40.4. Beyond 5·10⁴ ppm it is an extrapolation with no temperature dependence. |
| **HEXTOR OLR table** (Kopparapu et al. 2013 RC model, as tabulated in HEXTOR; Haqq-Misra & Hayworth 2022) | T 190–370 K (19 levels, 10 K apart); pCO₂ from 1.0·10⁻⁴ to 10 bar (91 distinct levels); 1 bar N₂ background; cloud-free; moist adiabat, saturated with H₂O; CO₂ collision-induced absorption included (Gruszka & Borysow 1997, Baranov et al. 2004, Halevy et al. 2009, per Kopparapu et al. 2013). | **Recommended.** It covers the model's whole range, comes from a published model, is MIT-licensed, has 1,748 numbers, and gives OLR as a function of both T and pCO₂. |
| Haqq-Misra et al. 2016 polynomial (eq. A3, log₁₀ T and log₁₀ pCO₂), as transcribed in HEXTOR `driver.f` (radparam 1) | Stated as 10⁻⁵–35 bar, 150–350 K, error ≤ 2 % against their RC model. | Rejected. HEXTOR's transcription gives OLR = 347 W/m² at 288 K and 280 ppm, where the table gives 260 (computed). That is a forcing of 78 W/m² for 2×CO₂. Haqq-Misra & Hayworth 2022 themselves note "numerically unstable values of FIR at low pCO₂" for this fit. The paper's coefficients are only in its online supplement, which was not read. |
| Williams & Kasting 1997 polynomial (as in HEXTOR `driver.f`, radparam 0) | "190 < T < 370 K and 10⁻⁵ < PCO2 < 10 bars with an rms error of 4.59 Wm⁻²" (code comment). | Rejected. It gives 1.7 W/m² for 2×CO₂ at 288 K (computed). That is too weak at low CO₂; it is the older Kasting 1991 radiation. |
| Kadoya & Tajika 2019 piecewise 6th×4th-order fit to the same RC model (ApJ 875: 7) | 10⁻⁵–10 bar, 150–350 K, error ≤ 3.3 W/m² (as quoted by Graham 2021). | Not used. The primary paper could not be read (IOP's bot captcha; the copy is bronze open access). The coefficients were seen only as quoted by Graham 2021, without the definition of the constant I₀. The fit is discontinuous at 1 bar (Graham 2021). |
| Pierrehumbert et al. 2011's NCAR CCM OLR fit | Inventories up to about 1 bar with pressure broadening ("remains accurate at least up to inventories of one bar"). | The fit itself is in a Python supplement on the Annual Reviews site, which was not fetched. Its thresholds are used in §5 as a cross-check. |
| Le Hir et al. 2009/2010, Wordsworth & Pierrehumbert 2013, Wolf & Toon, Halevy & Cohen, Ramirez | — | Not used. Le Hir et al. 2010 has no open copy (HAL holds only the record). The others are not snowball or 1-bar-CO₂ OLR fits for an Earth EBM. Ramirez 2024's PlaHab also uses RC lookup tables (library, read): the same approach. |

### 3.2 The recommended law

```
F(C, T) = m · [ OLR_RC(T, 280 ppm) − OLR_RC(T, C) ]
OLR_RC(T, C): bilinear in (T, log₁₀ pCO₂) on the HEXTOR Sun table, pCO₂ = C·10⁻⁶ bar
m = 3.708 / 4.107 = 0.903
```

**The m factor.** The table's clear-sky 2×CO₂ forcing at 288.15 K is 4.107 W/m², and Myhre's
5.35 ln 2 is 3.708 W/m². This is the clear-sky to all-sky masking (computed).

**Table data.**
- The library holds the table as `HEXTOR_software/olr_sun_1bar_N2.csv`, exported from
  `model/radiation/radiation_N2_CO2_Sun.h5`, HEXTOR commit f800e25f.
- Its columns are P (bar), f_CO₂, T (K), pCO₂ = P·f and OLR (W/m²). OLR is stored negated in
  mW/m², per `radiation.f90`.
- One CO₂ level, 2.4989·10⁻⁴, appears twice. HEXTOR keeps the first copy, and so should we.
- Ship the OLR values as a const array (19 × 91), credited in NOTICE.md and in the file header.
- Interpolate both terms of F with the same routine, so that F(280 ppm) = 0 exactly. Interpolation
  is linear in T and in log₁₀ pCO₂, which is what HEXTOR's `getOLR` does.

**Temperature dependence** (computed, F in W/m², × m):

| C (ppm) | 230 K | 250 K | 260 K | 270 K | 288.15 K | 300 K | today's law (any T) |
|---|---|---|---|---|---|---|---|
| 100 | −2.7 | −3.8 | −4.2 | −4.7 | −4.7 | −5.2 | −5.5 |
| 560 | 1.9 | 2.6 | 3.0 | 3.3 | 3.7 | 3.8 | 3.7 |
| 1,000 | 3.5 | 4.8 | 5.6 | 6.0 | 7.0 | 6.9 | 6.8 |
| 10,000 | 10.4 | 14.5 | 16.6 | 18.8 | 22.7 | 23.9 | 24.0 |
| 50,000 | 19.1 | 26.8 | 30.8 | 34.4 | 40.4 | 42.5 | 38.1 |
| 100,000 | 25.3 | 35.5 | 40.8 | 45.6 | 52.8 | 54.7 | 44.7 |
| 300,000 | 37.2 | 54.0 | 62.7 | 70.6 | 82.2 | 85.0 | 56.1 |
| 500,000 | 43.5 | 63.7 | 74.1 | 84.1 | 99.7 | 104.7 | 61.7 |
| 1,000,000 | 51.5 | 76.3 | 89.2 | 101.9 | 123.4 | 133.0 | 69.6 |

These values double as T6's unit-test anchors. The same interpolation reproduces them to ±0.1.

What the table shows:
- **At snowball temperatures (230–260 K)** the RC forcing is weaker than today's law below about
  10⁵–3·10⁵ ppm. Pierrehumbert et al. 2011 give the reason: weak vertical temperature contrast,
  absorption weaker in the cold.
- **At 1 bar** the forcing is 10 % stronger than today's law at 250 K and 77 % stronger at
  288 K, but 26 % weaker at 230 K.
- **At 1 bar the OLR hardly depends on T.** The table's own slope dOLR/dT is 1.9–2.3 W/m²/K at
  280 ppm (230–290 K), close to B = 2.09, but only 0.6–1.25 W/m²/K at 1 bar.
- **This makes escape easier.** Under F(C, T) the effective slope is B − m·∂F/∂T, about 0.8–1.0
  W/m²/K at 0.5–1 bar near 270 K. As the equator warms toward melting, the greenhouse strengthens.
  This is the main reason escape drops (§5.3).
- **It also makes hothouses warmer.** At 10⁵ ppm and 288 K the forcing is 52.8 W/m² against
  today's 44.7: about 4 K more before the lower effective slope, so the 36.6 °C post-snowball
  hothouse will warm. T6 should report its peak.

**Valid range.**
- The table spans T 190–370 K and pCO₂ 1.0·10⁻⁴ to 10 bar. That is about 100 to 10⁷ ppm in the
  model's units, and the model uses up to 10⁶ ppm.
- It assumes 1 bar of N₂, no clouds and a saturated troposphere.

**Continuing beyond it.**
- **T outside 190–370 K:** clamp T for F only. At 190 K, F is −1.1, 4.5, 9.0 and 60.7 W/m² at
  10², 10⁴, 10⁵ and 10⁶ ppm (computed). The 1-bar value is larger than at 230 K (51.5), which
  probably reflects the table's CO₂-condensation corner. Clamping is harmless because only polar
  night gets that cold.
- **C below 100 ppm:** F(C, T) = F(100, T) + k(T)·ln(C/100), where k(T) is the table's own slope
  between 100 and 200 ppm. That is 2.6, 3.65, 4.56 and 4.79 W/m² per e-fold at 230, 250, 270
  and 288 K, already × m. A first version of the clock overshot to 33 ppm after an escape (build
  log 2026-09-29), so low values do occur.
- **Above 10 bar:** clamp. The model never gets there.
- **CO₂ condensation** is not modelled. Frost forms at 31–98 mbar at 160–170 K (Pierrehumbert
  et al. 2011, p. 450), which the winter poles of a snowball can reach. Leave it out and note it.

**The source's own error.**
- Haqq-Misra & Hayworth 2022 do not state the error of the table as such. The values are the RC
  model's own output, not a fit.
- Kopparapu et al. 2013 (p. 6): their method "tends to overestimate the greenhouse effect of
  dense CO₂ atmospheres (Goldblatt & Zahnle 2011)", while neglecting CO₂ clouds "may cause us to
  underestimate" it.
- The saturated troposphere overstates the water-vapour greenhouse when warm. At 288 K the table's
  clear-sky OLR is 260 W/m²; Earth's observed clear-sky OLR is higher (from memory, about
  265–270 W/m²; not checked).
- Spread between models:
  - Abbot et al. 2012 found "differences of only a few W m⁻²" in clear-sky OLR among six GCMs
    run through one offline scheme, against "≈40 W m⁻² associated with increasing CO₂ from 10⁻⁴
    to 0.1". The table gives 37–43.5 W/m² for the same step at 240–250 K (computed): consistent.
  - Against Byrne & Goldblatt's line-by-line all-sky forcing at 288 K, the scaled table agrees
    within 6 % at 10⁴–5·10⁴ ppm (computed).
  - Hu et al. 2011 (Kasting's RC model, a relative of the table's) found that pressure broadening
    and CIA together warm the surface by about 10, 18 and 59 K at mole fractions 0.1, 0.2 and 0.4.
    The table includes both effects. The GCM radiation codes FOAM and CAM3 lack CIA.
- In sum: roughly ±10 % in forcing at 0.1–1 bar from the radiation, and more from the
  assumptions (relative humidity, clouds).

**A compact fit**, if T6 prefers a formula to a table (computed; not a published fit):

```
F_fit = m · Σ_{i=1..5} Σ_{j=0..3} a_ij x^i τ^j,  with x = ln(C/280), τ = (T − 270)/50
```

Over 200–320 K and 100–10⁶ ppm the fit's rms error is 0.44 W/m² and its largest 2.5 W/m². It is
worst at low C: 4.40 against 4.11 for 2×CO₂ at 288 K, both before × m. Over 190–350 K the largest error is
34 W/m². The table is therefore preferred. The coefficients a_ij (rows i = 1..5, columns
j = 0..3, before × m) are:

```
 5.498512e+00  2.099082e+00 -1.346661e+00 -6.325668e-01
 6.008402e-01  4.645256e-01 -4.047608e-01 -3.842731e-01
-5.149487e-01 -6.745147e-02  5.401279e-01  2.820126e-01
 1.279577e-01 -9.449778e-04 -1.337649e-01 -6.044433e-02
-7.139833e-03  1.693596e-03  8.916350e-03  3.664252e-03
```

### 3.3 Knock-on changes for T6

- **Rows replaced.** CLM-069, CLM-070 and CLM-071 are replaced by the new law. CLM-072 (forcing
  subtracted from A) becomes "subtracted from each tile's OLR at its own T".
- **CLM-073.** The implicit carbon step's guess λ = 5.35/B becomes m·∂F/∂lnC / (B − m·∂F/∂T),
  evaluated at the global mean T. Alternatively keep it as a guess, since it only starts the
  bisection.
- **Present-day calibration.** At 280 ppm nothing changes. The 2×CO₂ forcing now falls to
  2.6–3.3 W/m² at 250–270 K, so polar amplification of CO₂ changes weakens slightly. T9 should
  re-measure the Earth-like runs' CO₂ range and temperature swings.
- **Onset.** Onset happens near 300 ppm, where the forcing barely changes, so the onset threshold
  hardly moves.
- **Shortwave.** Rayleigh scattering by a dense CO₂ atmosphere is not included, and that is
  acceptable for a snowball. Pierrehumbert et al. 2011 (p. 427): "for a Snowball state with a
  highly reflective surface, the resulting increase in planetary albedo is not significant".
  Over a dark post-snowball ocean it would cool the hothouse by a few W/m², which is not
  quantified here.

## 4. Clouds on a snowball

### 4.1 What the sources say

- **Abbot et al. 2012** (GRL 39: L20711, read in full):
  - Six GCMs were run with surface albedo 0.6 and S = 1285 W/m². The cloud radiative forcing
    (CRF) "varies widely among the models".
  - FOAM's condensate is "an order of magnitude lower" than the others'.
  - In the cloudy models the CRF "is near zero in the winter hemisphere, where there is a strong
    inversion, but rises to ≈40 W m⁻² in the summer hemisphere", "similar to the modern longwave
    CRF over ocean".
  - Clouds "could warm a Snowball enough to reduce the CO₂ required for deglaciation by a factor
    of 10–100". This is relative to FOAM, which needs CO₂ ≈10–100 times higher "to be as warm as
    the other models".
  - Climate sensitivity in a snowball is 1.6–2.0 K per doubling.
- **Hoffman et al. 2017** (p. 10 of 43, read):
  - "FOAM is essentially a cloud-free model under Snowball conditions."
  - "a cloud-resolving model … produces ≥10 W m⁻² of cloud radiative forcing under a wide range
    of microphysical parameters (82)". Ref. 82 is Abbot 2014, *J. Climate* 27: 4391, which was
    not obtained: AMS, closed.
  - FOAM "remains far below the deglaciation point at any geologically feasible CO₂ level, even
    with prescribed albedos of 0.5 and 0.75 (broadband average) for ablative and snow-covered ice
    surfaces".
- **Hu et al. 2011** (Clim. Past 7: 17, read in full). Longwave CRF is a January zonal-mean
  maximum, not a global mean:
  - FOAM: about 6 W/m² at 100 ppmv and 10.5 W/m² at 0.2 bar.
  - CAM3: up to 26 W/m² at 100 ppmv and 32 W/m² at 0.2 bar.
  - LMDz: 50 W/m² at about 330 ppmv.
- **Pierrehumbert et al. 2011** (read):
  - Their EBMs subtract a fixed cloud term from clear-sky OLR. They use c_ld = 20 W/m² in the
    0-D model and 10–30 W/m² in the 1-D model's Table 2.
  - Ice clouds holding "a mere 1 % of the local saturation water vapor content … reduces the OLR
    by 17 W m⁻² at a surface temperature of 260 K and 30 W m⁻² at 270 K" (p. 453).

### 4.2 What the model already has

A + B·T (North, Cahalan & Coakley 1981) is a fit to observed all-sky OLR, so modern clouds are
inside it. Against the clear-sky table at 280 ppm, OLR_RC(T) − (A + B·T) is (computed):

| T | 220 K | 230 K | 240 K | 250 K | 260 K | 270 K | 280 K | 288 K | 300 K |
|---|---|---|---|---|---|---|---|---|---|
| implicit LW cloud term (W/m²) | 23.3 | 21.2 | 20.1 | 19.8 | 20.3 | 21.6 | 23.6 | 25.7 | 27.7 |

So the model already applies a cloud greenhouse of about 20 W/m² everywhere and at all times in a
snowball:
- That is Pierrehumbert's c_ld = 20 W/m².
- It is roughly an annual mean of Abbot et al.'s "≈0 in winter, ≈40 in summer".
- It is about twice FOAM's.

The shortwave side of the clouds is inside the planetary ice albedo 0.62 (CLM-009). That albedo
is Budyko's figure for today's ice-covered zones, clouds included. The GCMs above use a surface
albedo of 0.6, so the two are close.

The premise of the question ("≥10 W/m² of cloud forcing the model lacks") is therefore not quite
right. What the model lacks is the cloud's state dependence: the seasonal and hemispheric
structure Abbot et al. describe. It does not lack the cloud's magnitude.

### 4.3 Recommended cloud term

- **Default c_snow = 0 W/m²** (no extra term). Ledger the implicit 20 W/m² so that it is visible
  (draft CLM-171).
- **A setting `climate.snowball_cloud_w_m2`**, range 0–20 W/m², added to the greenhouse of the
  frozen share of each tile: OLR −= c_snow × frozen.
  - Frozen share: sea-ice cover on the sea tile; on land, the larger of snow and ice sheets.
  - The range comes from the sources above: Hoffman's ≥10 W/m², and Pierrehumbert's 10–30 W/m²
    total, less the 20 W/m² already implicit, gives −10 to +10 W/m². The cloudiest models
    (LMDz, about 50 W/m²) allow up to +20.
- **A state-dependent version, if wanted later.** Weight c_snow by the season's insolation, which
  mimics "≈0 in winter, ≈40 in summer". That needs a decision and is not proposed for T6.
- **For Ben.** If the escape should sit inside Hoffman's proxy range at the Cryogenian Sun, the
  sourced levers are an extra 20 W/m² of cloud (2·10⁵ → 1.1·10⁵ ppm) or a dusty sublimation-zone
  albedo near 0.55 (§5.3). The forcing fix alone gives about 2.7·10⁵ ppm at 0.938 S₀.

## 5. The escape threshold

### 5.1 Literature, by solar constant and albedo (all read)

| Source | Model | S | Ice albedo | Clouds | Escape CO₂ (model ppm, §2) |
|---|---|---|---|---|---|
| Hu et al. 2011 | Kasting RC, 80 % RH, pressure broadening + CIA, global mean, equator 10 K warmer | 0.94 S₀ | surface 0.663 | none (RC) | "about 0.21 bars" ≈ 2.1·10⁵ |
| Hu et al. 2011 | CAM3 | 0.94 S₀ | snow 0.9/0.6 (visible/near-IR), sea ice 0.5 | LW CRF up to 32 | "about 1.0 bar" without broadening/CIA; "0.38 bars" with both ≈ 3.8·10⁵ |
| Hu et al. 2011, citing Le Hir et al. 2007 | LMDz | 0.94 S₀ | — | strong | "0.45 bars" |
| Pierrehumbert et al. 2011, Table 2 | 1-D diffusive EBM, α_i 0.6, α_O 0.2 | 1285 W/m² | 0.6 | c_ld 10/20/30 | D = 0.5: 9.0/6.0/3.8·10⁴ Pa inventory ≈ 5.9/3.9/2.5·10⁵; D = 0.25: 7.4/5.0/2.9·10⁴ Pa ≈ 4.9/3.3/1.9·10⁵ |
| Pierrehumbert et al. 2011, 0-D | α_i 0.55–0.65, T_i 260, T_O 290 | 1285 W/m² | 0.6 | c_ld 20 / 0 | 3.8·10⁴ Pa ≈ 2.5·10⁵ / 10⁵ Pa ≈ 6.6·10⁵ |
| Pierrehumbert et al. 2011, Fig. 11 (FOAM) | GCM | 0.94 S₀ | default | nearly none | equator 240 → 243.6 K from 2,000 ppmv to 10 %: no escape |
| Hu et al. 2011, citing Pierrehumbert 2005 | EBMs (Caldeira & Kasting 1992, Hyde et al. 2000, Tajika 2003) | — | — | — | "would converge to about 0.2∼0.3 bars" |
| Hu et al. 2011, citing Abbot & Pierrehumbert 2010 and Le Hir et al. 2010 | dust layer on the tropical ice | — | lowered | — | "0.01∼0.1 bar" |
| Abbot et al. 2012 | proxies and models | — | — | — | "maximum CO₂ was 0.01–0.1 by volume" |
| Hoffman et al. 2017 | proxies | — | — | — | 10⁴ to 10⁵ ppm by volume |

Albedo dominates. Pierrehumbert et al. 2011 (p. 431): "A small change of albedo … requires a very
large change of CO₂ to make up for it", because OLR is near-logarithmic in CO₂. The response to
the solar constant is about a factor 1.3–1.5 per 6 % of S, from this note's runs below; no source
gives an escape-against-S curve for an Earth EBM that was read.

### 5.2 Model ppm against the literature

Today's 5.5·10⁵ ppm at 0.938 S₀, ice albedo 0.62:
- lies inside the diffusive-EBM range at the same Sun (1.9–5.9·10⁵);
- lies at the GCMs' upper end;
- lies 5–50 times above the proxy range.

### 5.3 This note's check: a 1-D stand-in for the clock (computed)

**The stand-in.**
- It is an annual-mean diffusive EBM: 90 bands, D = 0.649, insolation 1 − 0.482 P₂ (North
  1975, not re-read), OLR = A + B·T − F.
- Its albedo is the model's: a₀ = 0.33, a₂ = 0.25, a_i = 0.62, smoothstep ±3 K.
- The albedo switches at the melting point. The sea darkens only when its ice melts.
- Escape is found by raising CO₂ by 2 % per step from a snowball until the state jumps.
- Scripts: `HEXTOR_software/analysis/ebm4.py`; a variant with an annual-mean equatorial melt
  criterion is `ebm2.py`.

**Does it stand in for the model?** Today's law at the scenario's Sun (0.938 S₀) gives 7.1·10⁵
ppm, against the model's 5.5·10⁵. The model's seasonal cycle melts summer surfaces first, so T6's
2-D model should come in roughly 20–30 % below these numbers.

Escape CO₂ (model ppm) by forcing law:

| S/S₀ | today's law | recommended | recommended + c_snow 10 | recommended + c_snow 20 |
|---|---|---|---|---|
| 0.75 | > 3·10⁶ | 7.6·10⁵ | 5.1·10⁵ | 3.5·10⁵ |
| 0.80 | > 3·10⁶ | 5.8·10⁵ | 3.9·10⁵ | 2.6·10⁵ |
| 0.85 | 2.0·10⁶ | 4.4·10⁵ | 3.0·10⁵ | 1.9·10⁵ |
| 0.90 | 1.1·10⁶ | 3.4·10⁵ | 2.2·10⁵ | 1.4·10⁵ |
| 0.938 (scenario) | 7.1·10⁵ | 2.7·10⁵ | 1.8·10⁵ | 1.1·10⁵ |
| 1.00 | 3.3·10⁵ | 1.9·10⁵ | 1.1·10⁵ | 6.4·10⁴ |
| 1.05 | 1.7·10⁵ | 1.3·10⁵ | 8.0·10⁴ | 3.8·10⁴ |

**Sensitivity to the ice albedo** (recommended law, S₀):

| a_i | 0.55 | 0.60 | 0.62 | 0.65 |
|---|---|---|---|---|
| Escape CO₂ (ppm) | 4.3·10⁴ | 1.3·10⁵ | 1.9·10⁵ | 3.0·10⁵ |

**Sensitivity to D** (annual-mean equatorial melt criterion, S₀): 2.0·10⁵ ppm at D = 0.4 and
2.4·10⁵ ppm at D = 0.9.

**What "today's law" means here.** Its numbers grow quickly at a faint Sun: beyond 10⁶ ppm the
Byrne–Goldblatt quadratic is far below the RC forcing (69.6 against 89–102 W/m² at 260–270 K).
The recommended law degrades smoothly.

**Cross-check against Pierrehumbert et al.'s Table 2 set-up** (S = 1285, α_i 0.6, clear-sky table
OLR − c_ld, equatorial annual mean at 273.15 K):
- D = 0.5: 2.6, 1.8 and 1.2·10⁵ ppm for c_ld = 10, 20, 30.
- D = 0.25: 2.2, 1.4 and 0.9·10⁵ ppm.

That is about 2 times below their NCAR-radiation values (in model ppm: 5.9, 3.9 and 2.5·10⁵ for
D = 0.5; 4.9, 3.3 and 1.9·10⁵ for D = 0.25). The difference is the Kasting-type RC
model's stronger dense-CO₂ greenhouse (see Kopparapu's caveat in §3.2), together with this solver's
different ice-edge criterion. It is the same order and has the same trends.

**What T6 should report.** The 2-D and clock escape CO₂ at the scenario's Sun beside three
ranges:
- the physical-model range for the same Sun: 1.9–6.6·10⁵ ppm without dust;
- this note's stand-in: 2.7·10⁵ with c_snow = 0;
- the proxy range: 10⁴–10⁵.

## 6. Draft ledger rows

The ids follow CLM-166, the highest on master at e6d1609f. CLM-147 to CLM-162 are held by the ice
branch. Statuses are honest: `verified` only where the value was read.

| id | feature | what the code does | where | value or equation as coded | units | source cited | citation written in | status |
|---|---|---|---|---|---|---|---|---|
| CLM-167 | carbon cycle | Planned CO₂ forcing at every concentration, by tile temperature: the clear-sky RC OLR at 280 ppm minus that at C, from HEXTOR's Sun table, scaled by m (CLM-168); replaces CLM-069 to CLM-071. | P1-26 T6 (planned): climate/carbon.rs forcing | F(C,T) = m [OLR_RC(T, 280 ppm) − OLR_RC(T, C)]; bilinear in T and log10 pCO₂; pCO₂ = C·1e-6 bar | W/m² | Haqq-Misra & Hayworth 2022, PSJ (HEXTOR, MIT; table radiation_N2_CO2_Sun.h5, commit f800e25f); Kopparapu et al. 2013, ApJ 765: 131 (the RC model) | research note high-co2-forcing-and-snowball-escape | `verified`, `not-yet-coded` (2026-10-03: the table was read from the file (1,748 rows: 19 T, 190–370 K × 92 CO₂ rows (91 distinct), 1.0e-4 to 10 bar; HaqqMisraHayworth2022.txt p. 3: "1,748 values of FIR … radiative-convective climate model of Kopparapu et al. (2013)", Appendix A: "background N2 pressure of 1 bar"); spot values in the note §3.2, e.g. F(560 ppm, 288.15 K) = 3.71, F(1e5, 250 K) = 35.5, F(5e5, 270 K) = 84.1 W/m². Kopparapu2013.txt pp. 5–8: cloud-free, moist adiabat, updated CO₂ CIA. Error not stated by the source; Kopparapu et al.: may "overestimate the greenhouse effect of dense CO₂ atmospheres".) |
| CLM-168 | carbon cycle | Planned scale from the clear-sky table to all-sky forcing. | P1-26 T6 (planned) | m = 0.903 = 3.708 / 4.107 | dimensionless | Myhre et al. 1998 via IPCC 2001 Table 6.2 (CLM-069) | research note high-co2-forcing-and-snowball-escape | `calibrated`, `not-yet-coded` (target: 5.35 ln 2 = 3.708 W/m² for 2×CO₂ (CLM-069, verified there) at 288.15 K; the table gives 4.107 (computed). With Earth's 14.1 °C target, m moves by < 0.3 %.) |
| CLM-169 | carbon cycle | Planned mapping of the model's ppm (n_CO₂ / n_air × 1e6) to the table's pCO₂ (1 bar N₂ + CO₂): the same CO₂ mole fraction. | P1-26 T6 (planned) | pCO₂_table = C·1e-6 bar; true p_tot = 1 + 1.519 C·1e-6 bar | bar | Pierrehumbert et al. 2011, Annu. Rev. Earth Planet. Sci. 39: 417, Table 1 (conversions) | research note high-co2-forcing-and-snowball-escape | `game-choice`, `not-yet-coded` (agent, this note: the table's total pressure (1 + C') is below the true one, so pressure broadening is slightly underestimated (conservative). The conversion was checked against Table 1's row "10^5 Pa → 39.7 % molar, p_tot 2.00·10^5 Pa" (Pierrehumbert2011.txt p. 427, read).) |
| CLM-170 | carbon cycle | Planned continuation outside the table: T clamped to 190–370 K for F; below 100 ppm F continues logarithmically with the table's 100–200 ppm slope; above 10 bar clamped. | P1-26 T6 (planned) | F(C<100, T) = F(100, T) + k(T) ln(C/100); k = 2.6/3.65/4.56/4.79 W/m² per e-fold at 230/250/270/288 K | W/m² | none | research note high-co2-forcing-and-snowball-escape | `game-choice`, `not-yet-coded` (agent, this note: numerics at the edges; no source covers < 1e-4 bar.) |
| CLM-171 | energy balance | The linear OLR (CLM-003, CLM-004) already holds a longwave cloud greenhouse relative to clear sky. | climate/clock.rs and the per-cell design (A + B T) | OLR_RC(T, 280 ppm) − (A + B T) = 21.2, 19.8, 21.6, 25.7 W/m² at 230, 250, 270, 288 K | W/m² | North, Cahalan & Coakley 1981 (A, B); the CLM-167 table | research note high-co2-forcing-and-snowball-escape | `verified` (computed 2026-10-03 from two read values: CLM-003/004 and the CLM-167 table. It is the size of Pierrehumbert et al. 2011's c_ld = 20 W/m² and of the cloudy GCMs' snowball CRF (Abbot et al. 2012: "≈40 W m-2 in the summer hemisphere", "near zero in the winter hemisphere").) |
| CLM-172 | energy balance | Planned optional extra cloud greenhouse on frozen surfaces in a snowball. | P1-26 T6 (planned): climate.snowball_cloud_w_m2 | OLR −= c_snow × frozen share; default 0; range 0–20 | W/m² | Hoffman et al. 2017, Sci. Adv. 3: e1600983, p. 10; Abbot et al. 2012, GRL 39: L20711; Hu et al. 2011, Clim. Past 7: 17; Pierrehumbert et al. 2011, Table 2 | research note high-co2-forcing-and-snowball-escape | `game-choice`, `not-yet-coded` (default for Ben to confirm. Read: Hoffman p. 10 "≥10 W m-2 of cloud radiative forcing" (CRM; their ref. 82, Abbot 2014, not obtained); Abbot et al. 2012 p. 2 CRF ≈40 W/m² summer, ≈0 winter, CO₂ factor 10–100 relative to FOAM; Hu et al. 2011 pp. 19–20 January maximum LW CRF FOAM 6–10.5, CAM3 26–32, LMDz 50 W/m²; Pierrehumbert et al. 2011 Table 2 c_ld 10–30. The range is the literature's total less the 20 W/m² implicit (CLM-171).) |
| CLM-173 | deep-time clock | Literature escape thresholds the 2-D and clock escapes are reported against (extends CLM-166). | P1-26 T6 (planned): run notes | RC 0.21 bar (0.94 S₀, albedo 0.663); CAM3 0.38 bar (with broadening and CIA), ~1.0 bar without; LMDz 0.45 bar; diffusive EBM 2.9–9.0·10^4 Pa inventory (1285 W/m², α_i 0.6, c_ld 30–10); 0-D 3.8·10^4 Pa (c_ld 20) and 10^5 Pa (c_ld 0); dust 0.01–0.1 bar; proxies 0.01–0.1 by volume | bar, Pa | Hu et al. 2011; Pierrehumbert et al. 2011 (Table 2, §3.2); Abbot et al. 2012; Hoffman et al. 2017 | research note high-co2-forcing-and-snowball-escape | `verified`, `not-yet-coded` (2026-10-03, full texts in the library: Hu2011.txt abstract and pp. 17–22; Pierrehumbert2011.txt pp. 430–431 and Table 2; Abbot2012.txt p. 1. Le Hir et al. 2007 and Abbot & Pierrehumbert 2010 are known only as cited by Hu et al. 2011.) |
| CLM-174 | deep-time clock | Planned expected escape CO₂ of the clock under CLM-167, from a 1-D annual-mean stand-in. | P1-26 T6 (planned): test or run-notes comparison | 0.938 S₀: 2.7·10^5 (c_snow 0), 1.8·10^5 (10), 1.1·10^5 (20) ppm; S₀: 1.9·10^5, 1.1·10^5, 6.4·10^4 ppm; today's law 7.1·10^5 (0.938 S₀) and 3.3·10^5 (S₀) | ppm | none (computed) | research note high-co2-forcing-and-snowball-escape | `uncited`, `not-yet-coded` (a computed expectation, not a literature value: the scripts are in the source library at HEXTOR_software/analysis/ebm4.py. The stand-in gives 7.1·10^5 where today's model gives 5.5·10^5, so expect the seasonal 2-D model about 20–30 % lower.) |

Changes to existing rows when T6 lands:
- CLM-069, CLM-070 and CLM-071 lose `pending-rewrite` as replaced by CLM-167.
- CLM-072's "subtracted from A" becomes "subtracted from each tile's OLR at its T".
- CLM-073's guess changes (§3.3).
- CLM-166's "expected escape" gains CLM-173 and CLM-174.

New bibliography keys: HaqqMisraHayworth2022 (+ HEXTOR_software), Kopparapu2013, HaqqMisra2016,
Hu2011, Abbot2012, Pierrehumbert2011, Graham2021. NOTICE.md needs HEXTOR's MIT credit when the
table is shipped.

## 7. Sources and how each was read

All are in the source library under the key named here. "Read" means the text extraction (poppler
`pdftotext -layout`) was read at the passages cited.

| Key | Work | Copy | Read |
|---|---|---|---|
| HaqqMisraHayworth2022 | Haqq-Misra & Hayworth 2022, "An Energy Balance Model for Rapidly and Synchronously Rotating Terrestrial Planets", PSJ | arXiv 2201.02685 | §2 (radiative transfer, lookup table), Appendix A, software statement |
| HEXTOR_software | HEXTOR code and tables, MIT, commit f800e25f | GitHub BlueMarbleSpace/hextor (sparse clone of model/) | `radiation.f90` header and v1 reader; `driver.f` OLR parameterizations; the Sun table read with h5py and exported to CSV |
| Kopparapu2013 | Kopparapu et al. 2013, "Habitable Zones Around Main-Sequence Stars: New Estimates", ApJ 765: 131 | arXiv 1301.6674 | §2 model description, caveats on dense CO₂ |
| HaqqMisra2016 | Haqq-Misra et al. 2016, "Limit cycles can reduce the width of the habitable zone", ApJ 827: 120 | arXiv 1605.07130 | §2 and Appendix A (fit form, range, ≤ 2 % error). The coefficient tables are in an online supplement, not read. |
| Byrne2014 | Byrne & Goldblatt 2014, GRL 41: 152 | already in the library | §2.1–2.4 (SMART line-by-line, combined solar and thermal, all-sky, instantaneous forcing, profiles) |
| Hu2011 | Hu, Yang, Ding & Peltier 2011, "Model-dependence of the CO₂ threshold for melting the hard Snowball Earth", Clim. Past 7: 17–25 | publisher, CC BY | full |
| Abbot2012 | Abbot et al. 2012, "Clouds and Snowball Earth deglaciation", GRL 39: L20711 | MPG.PuRe file of the published version (Max Planck institutional repository) | full (4 pages) |
| Pierrehumbert2011 | Pierrehumbert, Abbot, Voigt & Koll 2011, "Climate of the Neoproterozoic", Annu. Rev. Earth Planet. Sci. 39: 417–460 | University of Chicago host (author institution) | §3.1–3.3, Table 1, Table 2, §6 |
| Hoffman2017 | Hoffman et al. 2017, Sci. Adv. 3: e1600983 | already in the library | pp. 3 and 10 (clouds, FOAM, Fig. 8 caption) |
| Graham2021 | Graham 2021, "High pCO₂ reduces sensitivity to CO₂ perturbations…" | arXiv 2104.01224 | §2 (only for its quotation of Kadoya & Tajika 2019) |
| Ramirez2024 | Ramirez 2024, PSJ 5: 2 | already in the library | grep only: PlaHab uses RC lookup tables |

**Not obtained** (Wiley and AMS Cloudflare blocks, IOP captcha, or closed). Never cited here as
read:
- Pierrehumbert 2004 (Nature) and 2005 (JGR)
- Abbot & Pierrehumbert 2010 (Mudball, JGR)
- Abbot, Voigt & Koll 2011 (Jormungand, JGR)
- Abbot 2014 (Resolved Snowball Earth clouds, J. Clim.)
- Le Hir et al. 2007 and 2010
- Kadoya & Tajika 2019 (ApJ 875: 7)
- Caldeira & Kasting 1992 (Nature)
- Halevy, Pierrehumbert & Schrag 2009 (JGR)
- Goldblatt & Zahnle 2011
- North 1975 (only its 0.482 P₂ insolation form was used, from memory)
- Abbot & Halevy 2010 (J. Clim., bronze open access at AMS; not fetched)

## 8. Open points for Ben (via the coordinator)

1. Default snowball cloud term: 0 (recommended; the linear OLR already holds about 20 W/m²), or
   +10 W/m² (Hoffman's lower bound on top), or +20 W/m² (escape inside the proxy range at the
   Cryogenian Sun).
2. A dusty or bare-ice albedo for a snowball's tropical sublimation zone. The sources give 0.5
   ablative against 0.75 snow (Hoffman et al. 2017, p. 10) and the Mudball mechanism. This is the
   other sourced route to 10⁴–10⁵ ppm, and it is outside T6's scope.
