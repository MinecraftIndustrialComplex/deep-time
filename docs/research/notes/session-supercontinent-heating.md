# Does supercontinent heating exist?

Date: 2026-10-02 (transcript UTC timestamp, 01:38). The decisions log dates the entry 2026-10-02.
Status: the coordinator's assessment, recorded from a working session. Not a literature review.

## Question

In the simulation, one plate sometimes grows to cover more than half the planet after plates weld
together. A design for fixing plate collisions offered a fallback: a rift term that grows under a
long-assembled supercontinent, "supercontinent heating", built only if the mega-plate target was
missed. The question was whether such heating exists in nature, meaning whether the mantle under a
long-lived supercontinent heats or upwells and helps push it apart. It was asked at the same time as
the question of which plate-motion design is most realistic in the long term.

## Assessment

The coordinator's conclusion was that the effect is real and the mechanism is debated.

What is well supported, in its words:

- Supercontinents break up, on a cycle of roughly 300 to 500 million years.
- Breakup comes with large mantle upwellings.
- Pangaea's breakup began with one of the largest flood basalt provinces known, and Pangaea sat over
  a broad region of hot, upwelling mantle.

Two explanations:

- The classic one is thermal blanketing (thermal insulation). A large continent insulates the mantle
  beneath it, which heats, domes the crust and rifts it.
- The challenge is that later mantle models find insulation alone is too weak. They argue the
  upwelling forms because subduction rings the supercontinent and the return flow rises under it.

It concluded that "the mantle under a long-lived supercontinent pushes it apart" is real, and
"because the continent acts as a blanket" is one of two explanations. It then drew a consequence
for the design choice. In a force-balance model of plate motion (plate speeds set by slab pull,
ridge push, basal drag and collision resistance), subduction around a supercontinent's rim already
pulls it apart, so a separate heating term may never be needed. For that reason it changed its
recommendation to the full force-balance design, and it re-offered the mega-plate question as
"decide when measured". It also said that a force-balance design costs about two to three extra
days of agent work over a simple collision-resistance patch (an estimate), and that the patch adds
one more hand-placed rule of the kind the owner had objected to with the mountain cap.

## What was sourced and what was not

- The coordinator said so itself: its answers were "from my own knowledge rather than a fresh
  literature check". No web page, paper or sub-agent report was used for the heating statements.
  No source was consulted; this is the model's own knowledge and has not been checked. The 300 to
  500 million year cycle, the flood basalt claim and the two competing explanations are all
  unverified.
- No paper is named in the chat answer.
- The repository's design documents contain citations on the same question, written by a design
  sub-agent and not by the coordinator's chat answer: `docs/design/plate-collisions-and-welding.md`
  lists Gurnis 1988 and Coltice et al. 2007 (heating up to about 100 degrees C) for insulation, and
  Heron and Lowman 2011 (about 50 degrees C) and Lenardic et al. 2011 against it. The
  `docs/design/plate-force-balance.md` document cites Zhang et al. 2018 and Huang et al. 2019 on
  upwelling under supercontinents. The coordinator's chat answer cites none of them, and this note has
  not checked them.
- The two-to-three-day estimate is a guess, not a measurement.

## What was decided

On 2026-10-02, per `docs/deeptime-design-decisions.md`, Ben chose full force-balance plate motion
(slab pull, ridge push, basal drag and collision resistance set plate speeds), with the design
brought to him before anything is built. For the mega-plate he chose "decide when measured":
nothing is built for a heating term now, and the coordinator brings numbers and options if the
target is missed. The collision design lists "supercontinent heating" as a fallback task, to be
done only if the share of steps with one plate above 50 % stays over 10 %. The log row summarises
the coordinator's answer as: upwelling under supercontinents is well supported, thermal blanketing
is one of two explanations. The force-balance design is `docs/design/plate-force-balance.md`.

## Open

- Whether any heating term is needed at all depends on the mega-plate measurement after the
  force-balance model exists.
- Everything above about the real cause of supercontinent breakup is unchecked against the
  literature, and the debate between blanketing and subduction-driven return flow is not resolved
  by anything in this session.
- The citations in the design documents have not been read here.
- The owner asked for the mod to be backed by science as far as possible; a science page for plate
  motion would need these claims verified before it states them as fact.
