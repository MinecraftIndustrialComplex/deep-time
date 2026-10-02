# Too-tall mountains, and why the height limit should be emergent and not a cap

Date: 2026-10-01 (transcript UTC timestamps, 16:01 to 18:53).
Status: the coordinator's assessment, recorded from a working session. Not a literature review.

## Question

Some generated worlds had mountains taller than the game's build limit (the highest block a world
can hold, y 639 in the default world), and the game cut them off flat. Are peaks of 12 to 20 km
realistic for an Earth-like planet? Should the simulation enforce a fixed elevation cap, or should
the height limit come out of the physics of the model (rock strength, gravity, erosion)?

## Assessment

The measurement that started it. On one test planet 2.5 % of land stood above the build limit, with
the highest point at y 969. Two of six test seeds did this, and two of four normal 500-million-year
runs peaked at 12 to 13 km against a limit equivalent to 11.5 km.

Realism. The coordinator treated 12 to 20 km peaks as not realistic for an Earth-like planet. The
reasoning recorded in the decisions log is that Earth's practical ceiling is about 9 km, and that
taller mountains need low gravity and no plate motion, as on Mars. Its first recommendation was a
cap applied inside the surface simulation, scaled with the planet's gravity setting, plus a
game-side safety net (a smooth compression of anything above a knee, and a test that fails if any
terrain exceeds the limit).

The owner then asked whether a hard-coded cap was needed at all, or whether the limit should result
from the simulation. The coordinator agreed in principle and gave caveats:

- No fixed limit exists in nature. Real mountain height is the outcome of rock strength, gravity and
  erosion, so a fixed limit stands in for physics not yet modelled. Gravity comes for free if the
  limit is the weight of rock against its strength.
- The fixed limit would have hidden a bug. The excess came from erosion rebound and sediment being
  stored in a term that nothing bounded, and a clamp would have hidden that.
- It will still contain a tuned number. At the model's resolution (cells about 5 km wide, steps of
  2 Myr) rock failure and flow cannot be simulated. "Crust spreads when it is too thick for its
  strength" is a simplified rule with a strength setting that has to be tuned until Earth-like
  worlds top out near 9 km. The gain is that it responds to gravity, acts smoothly and can make high
  plateaus such as Tibet instead of flat-topped peaks.
- It costs more than a clamp, and the game still needs a hard guarantee, because the world height
  is a real ceiling.
- It admitted it should have said at once that fixing the bug alone might be enough, and it staged
  the work: fix the flaw first, then bring peak heights over several seeds before building the
  spreading mechanism.

Stage one (a sub-agent's findings, relayed). The cause was not what the coordinator had first
said. The plate model's 10 km limit almost never triggered. Three flaws were found: height added by
erosion and sediment was stored against a map position and not with the crust it belonged to, so it
was added again to different crust moving in; the plate model was not given the true height, so
already-high crust kept taking full collision uplift; and volcano cones kept rebuilding height that
erosion had removed. The fix had no clamp. Results: test-planet peaks of 16 to 20 km became 3.0 to
4.5 km, and normal 500 Myr worlds peaked at 9.0 to 11.2 km (highest before: 13.0 km). The agent's
own view was that the 9 to 11 km result was still held by the plate model's taper toward 10 km,
which is a number and not a mechanism, and ignores gravity.

Stage two. The design was that crust too thick for its strength spreads sideways (lower-crustal
flow). The agent predicted collision zones would stop rising at about 5 km and widen into steep-edged
plateaus, with summits of 8 to 9 km. It gave plateau levels of about 5 km at Earth gravity, about
13 km at Mars gravity and about 2.5 km at double gravity, and a cost of a few percent of the erosion
step. As built, with a crust strength setting of 140 MPa (5.1 km at 1 g), Earth-like worlds peaked
at 6.0 to 6.9 km with plateaus at 5 to 5.7 km and little summit relief above them. The plate model's
cap became a 30 km numerical guard.

## What was sourced and what was not

- The size of the problem and the stage-one and stage-two numbers: measurements by sub-agents in
  this repository, on a small number of seeds. The mechanism and constants are written up in the
  `STATUS.md` build log entry "The height limit, stage 2".
- Earth's ceiling of about 9 km, the remark about Mars-like low gravity and Tibet, and the general
  statement that real height comes from strength, gravity and erosion: no source was consulted; this
  is the model's own knowledge and has not been checked.
- The literature behind the spreading mechanism and the plateau levels at other gravities: the
  coordinator reported that the agent cited it from memory and did not re-check it. No paper is
  named in the chat.

## What was decided

All on 2026-10-01, in `docs/deeptime-design-decisions.md`:

1. First, a cap at the source scaled with gravity, with a build-limit safety net.
2. That was superseded: no clamp in the surface stage. Fix the bookkeeping flaw, add the missing
   physics (gravitational collapse or lower-crustal flow of over-thick crust, strength a setting,
   gravity entering through density times gravity times height), and keep the game-side safety net.
3. Stage two was approved with the added statement that raising the world height is acceptable if
   the model really needs it. The plate model's 10 km taper became a far guard. If realistic peaks
   reach the build limit, the world height is raised and the terrain is not compressed.
4. A vertical scale too small for the mountains to fit: compress smoothly so that every slider
   position stays valid.

## Open

Four follow-up questions are listed in the `STATUS.md` pending-questions table, not yet put to the
owner: give the detail pass summit relief over the plateaus, or raise the strength setting (to about
190 MPa); the world bottom (default trenches reach y -412 to -472 against a floor of -352); light
planets, whose fold belts are not taller because erosion and uplift do not use gravity; and whether
to raise the top (y 639) for Mars-like worlds. Earth-like peaks of 6.0 to 6.9 km sit below the
roughly 9 km the coordinator named, so the strength constant is not yet calibrated. The literature
for the mechanism is unchecked.
