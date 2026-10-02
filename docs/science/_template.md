# <Feature name>

<!--
Template for a feature page in docs/science/. Copy it, rename it, fill every section, delete this
comment. Rules for the writer:

- Plain declarative sentences. Define a term the first time it is used. Numbers carry units.
- Never state as established anything the ledger (claims.md) marks as unverified. Write
  "the code uses X (ledger ABC-012, not yet checked against its source)" instead.
- A value an agent could not check against its source is marked `from-memory` in the ledger and
  said to be so here. It is never cited as if the source had been read.
- Every number in the "Equations and constants" table comes from the code, by ledger id. If the
  code and a document disagree, the code is what the page reports, and the disagreement goes under
  "Open questions".
- Measured numbers name the run they came from (preset, start, length, seed, date, command).
- No sales language. Say what is simplified and what is a game choice as plainly as what works.
-->

Status of this page: `stub` | `stage 1 (written from the ledger, nothing verified)` |
`stage 2 (constants checked against sources on <date>)`.

Ledger ids on this page: `ABC-001` to `ABC-0NN` (see [claims.md](claims.md)).

## What you see in the world

What a player meets in the game because of this feature: shapes, sizes in blocks and in metres,
where it occurs, what rock and ore go with it. Two or three short paragraphs. No model detail.

## The model in plain words

How the simulation produces it, step by step, in the order the code runs. Name the stage of the
pipeline it runs in and what it reads from and gives to the other stages. A reader who is not a
geologist should be able to follow it; a geologist should be able to tell exactly what was done.

## Equations and constants

| Ledger id | Quantity | As coded | Units | Source | Status |
|---|---|---|---|---|---|
| ABC-001 | ... | ... | ... | ... | `cited-unverified` |

Write each equation once, in the form the code computes it, with every symbol defined beneath it.

## Earth against the mod

| Quantity | Earth | Source of the Earth figure | The mod | Run |
|---|---|---|---|---|
| ... | ... | ... (and how it was read) | ... | preset, start, length, seed, date |

Only measured numbers. If the build log holds no measurement, say so; do not estimate.

## Simplifications and limits

What the model leaves out or flattens, and what that does to the worlds it makes. One bullet each.

## Game choices

Values that are deliberately not physical, with the reason and who decided (the date in the
decisions log, `docs/deeptime-design-decisions.md`, or the build-log entry of the agent that chose).

## Open questions

Values not yet checked, disagreements between code and documents, things a geologist should look at.

## Sources

The entries of [bibliography.md](bibliography.md) this page relies on, by their keys, each with
how it has been read so far (in full, abstract only, secondary source, from memory, not fetched).
