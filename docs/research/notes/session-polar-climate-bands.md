# Polar climate bands, and computing climate per cell and not per latitude band

Date: 2026-10-01 (transcript UTC timestamps, 01:21 to 07:00). The decisions log dates the same
exchange 2026-09-30, which is the local date.
Status: the coordinator's assessment, recorded from a working session. Not a literature review.

## Question

On the satellite-style map of a simulated planet, a north polar continent appeared as a full-width
band of ice-free tundra, and the southern sea-ice edge was a straight line round the planet near
62 degrees S. Are these bands realistic? And should the climate model compute temperature and rain
for each grid cell, where it now computes them per latitude band?

## Assessment

North. The coordinator said the northern band is a real continent and behaves plausibly. Land runs
from about 60 degrees N to the pole, and any polar continent stretches into a full-width strip on a
flat map, as Antarctica does. It carries no ice sheet because the climate classes most of it as
tundra (warmest month 0 to 10 degrees C), so summers melt the winter snow. The ice rule grows ice
only where summers stay below about -2 degrees C, and colder for a dry interior. Northern Siberia
and Arctic Canada, at 70 to 80 degrees N, are ice-free for the same reason. The white necklace along
the coast is a volcanic arc whose cones, about 60 km apart, are the only places tall enough to keep
ice. A supercontinent ringed by subduction zones is realistic, it said, as Pangaea was. The beige
colour is a palette choice, dry tundra painted as cold desert.

South. The sea-ice extent (edge near 62 degrees S) is plausible, since today's winter sea ice
reaches about 55 to 60 degrees S. The straight edge is a simplification: the model computes sea ice
as one value per latitude band all the way round the planet, so no current, wind or coastline can
bend it. That hemisphere is open ocean, where the real ice edge is also roughly circular, so it
looks acceptable there. It would look wrong in an ocean bordered by continents, where it cannot show
an ice-free coast warmed by a current like the Gulf Stream or a cold current carrying ice toward the
equator. The model already computed each sea cell's monthly temperature, including coastal current
offsets, so sea ice could be taken from those.

Per cell in general. The coordinator agreed that climate should be per cell. It checked the code and
reported:

- Already per cell: temperature drop with elevation; continentality (harsher seasons far from the
  sea); coastal warm and cold currents; rain shadows and rain forced out over mountains.
- Still banded and setting the pattern: the temperature baseline (an energy balance computed once
  per latitude band) and the rain baseline (fixed wind belts: wet equator, dry subtropics, wet
  westerlies). The per-cell terms only nudge those baselines, and climate-class boundaries then cut
  along the band lines, which gives the horizontal stripes of desert and savanna on a large
  continent.

A fully per-cell model has two parts. First, temperature from an energy balance solved on a coarse
grid of 2 to 4 degrees per cell, with heat storage (land or ocean), albedo (ice, snow, desert,
forest), sea ice, sideways heat spread in two dimensions and ocean heat carried along basin edges.
Sea ice then follows from the temperature. It said the physics is published (North, Short and
Mengel 1983) and cheap, since it runs only at each 10-million-year climate snapshot. Second, rain
from a moisture budget (evaporation driven by sea temperature, winds carrying moisture, summer land
heating pulling in monsoons, sinking air and distance from the sea making deserts). It called this
the harder half. The baked climate fields would not change, so the world-data format would stay,
but everything downstream would need recalibration.

A sub-agent then wrote a design (`docs/design/p1-26-per-cell-climate.md`), and the coordinator
relayed these measurements: latitude explains 99.6 % of the variance of land temperature in the
simulation against 96.8 % on Earth; a prototype on Earth's relief brings winter temperature
variation along a latitude circle to 5.4 K (Earth 6.0 K, current model 3.3 K) and cuts winter
temperature error from 7.2 K to 4.9 K; rain from a moisture budget gets deserts right but is still
less accurate overall than the current belts; the cost is 15 to 60 s extra per 500-million-year
quick run. The banded climate was also linked to a later problem: flowing ice sheets covered 47 to
52 % of land on it, so the ice branch was held until the per-cell work.

## What was sourced and what was not

- What is per cell and what is banded: the coordinator read the climate module overview and the
  glacier code in this repository (`crates/deeptime-core/src/climate/`) before answering.
- Statements about real places and values (Siberia and Arctic Canada ice-free, 55 to 60 degrees S
  winter sea ice, Pangaea ringed by subduction, Antarctica on a flat map): no source was consulted;
  this is the model's own knowledge and has not been checked.
- North, Short and Mengel 1983: named from memory in the chat. The design document lists the paper
  as North, Mengel and Short 1983 (J. Geophys. Res. 88, 6576) with equations only, and marks some
  of its other citations as cited from memory.
- The 99.6 %, 96.8 %, 5.4 K, 6.0 K, 3.3 K and 7.2 to 4.9 K figures: a sub-agent's measurements in
  the design document. The Earth values in it were not re-derived here.

## What was decided

Dates are from the decisions log, `docs/deeptime-design-decisions.md`. First, per-cell sea ice was
chosen and queued as P1-26. Then the owner asked for climate in general to be per cell, and the
choice was "design doc first": a design replacing the sea-ice item, with P1-26 renamed "Per-cell
climate". On 2026-10-01 the design was approved as recommended: a 2.8 degree grid (4 degrees as
fallback), an overturning current that sinks in the narrowest polar-connected basin, rain staged
(belts on the new temperatures first, the moisture budget as default once it beats them on Earth's
relief), the old banded model kept as a setting until the Phase 1 exit, a wind layer in the
workbench, and no seasonal sea ice baked. The build waits on the ice-sheet branch and on the decision
whether ice sheets are on by default (held on 2026-10-01).

## Open

- The build had not started when the transcript ends (it is the longest remaining item in Phase 1).
- The moisture budget was less accurate overall than the belts in the prototype, and its default
  status depends on beating them on Earth's relief.
- The real-world statements above are unchecked, and some design citations are marked from memory.
- Whether the 2.8 degree grid is fine enough for the sea-ice edge to bend around real coastlines is
  not shown.
