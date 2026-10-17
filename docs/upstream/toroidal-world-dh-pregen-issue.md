# Draft issue: `/dh pregen` stalls when its square crosses the world's edge

**Status: DRAFT, updated for Exotic Worlds 1.0.0 on 2026-10-17. Not posted.** Nobody has opened, commented or
written to anyone about this. The stall was measured again on Exotic Worlds 1.0.0 (the current release) with
Distant Horizons 3.3.1 and 3.3.3 and is still there, so the draft stands; Ben posts it.

**Target tracker:** https://github.com/anyttng/exotic_worlds/issues (new issue, "Bug report" form).
Source: the Modrinth project "Exotic Worlds (Toroidal World)", id `CdjG7l5z`, slug `exotic-worlds`, whose
`issues_url` is that tracker (its `source_url` field is empty; the repository is
https://github.com/anyttng/exotic_worlds). No issue there mentions pregeneration. Issue 41 (closed) was a
Distant Horizons 3.3 report that the author answered within two days, so the author does take these.

## Notes for the poster (delete this section before posting)

1. **The form's fields.** The tracker's `bug_report.yml` asks for: Minecraft version and mod loader; Mod
   version; What happened; Steps to reproduce; Log (required); Where it does not happen. The text below
   follows those headings. "What we think is going on", "Workaround" and "Suggestion" are extra; paste them
   at the end of "What happened" or as a first comment.
2. **Attach a log.** The form requires `latest.log`. The excerpt under "Log" is real (from the 2026-10-17 run
   on 1.0.0 with Distant Horizons 3.3.3), but the full logs went with the scratch folders of that run. For
   the attachment, take a fresh `latest.log`: it takes about five minutes on the test VM,
   `tools/review/ew-dh-pregen-repro.py` (private repo; see STATUS.md, build log 2026-10-17), or by hand with the
   steps below.
3. **Tested on the current release: done.** Exotic Worlds 1.0.0+mc1.21.1 (published 2026-10-11, still the
   newest NeoForge 1.21.1 file on 2026-10-17) with Distant Horizons 3.3.1 and 3.3.3 stalls at the same place
   as 0.19.3 did. The release notes' line "Distant Horizons 3.3.3 draws the distant terrain across the world's
   edge" is about drawing the terrain; `/dh pregen` still stalls with 3.3.3.
4. **The reproduction is Deep-Time-free.** The numbers below come from servers with exactly two mods
   (Exotic Worlds and Distant Horizons), Distant Horizons' own generator. Nothing in the draft names our
   mod, which is private. The earlier 0.19.3 measurements (the 75 to 76 chunk run, the 15 minute stall, the
   one-sided squares that finished) were taken with it installed, and the text says so.
5. Nothing in the draft names a person, a path or an address.

---

## Title

`/dh pregen` stalls part-way when its square crosses the world's edge

## Minecraft version and mod loader

Minecraft 1.21.1, NeoForge 21.1.249, Java 21, dedicated server.

## Mod version

- Exotic Worlds 1.0.0+mc1.21.1 (`exotic_worlds-1.0.0+mc1.21.1.jar`, Modrinth version `HxhThaC4`)
- Distant Horizons 3.3.3-1.21.1 (Modrinth version `9w34y8ai`) and 3.3.1-1.21.1 (`IcOcoekl`); both stall
- Seen earlier on Toroidal World 0.19.3+mc1.21.1 (`8YTSYi9y`) with Distant Horizons 3.3.1, at the same point

## What happened

Distant Horizons' server-side pregeneration (`/dh pregen start minecraft:overworld <x> <z> <radius>`)
does not finish when the square it covers reaches past the world's edge. It runs on the near side, then
stops: `/dh pregen status` stops advancing, no error is reported, and it never recovers by itself. A square
of the same size that ends exactly at the edge finishes normally.

What we measured on 2026-10-17, on `exotic_worlds:torus_large` (the log reports `axis=X width_blocks=4096`,
so the X edge is at x = E = 2048), radius 32 chunks (the smallest DH accepts), z = 0, a fresh world, a fresh
server for each square:

| Servers | Centre x | Square | Result |
|---|---|---|---|
| Exotic Worlds 1.0.0 + DH 3.3.3, two mods only | 1728 (E - 320) | crosses the edge | stalled at 53.13 % (radius 23.324 / 32 chunks) 21 s after the start; no change until we stopped it at 222 s |
| the same | 1472 (E - 576) | ends exactly at the edge | finished, `Pregen is not running`, 24 s after the start |
| Exotic Worlds 1.0.0 + DH 3.3.1, two mods only | 1728 | crosses | stalled at 53.13 % after 24 s; no change until 225 s |
| the same | 1472 | ends at the edge | finished after 21 s |

53.13 % of the 256 sections DH counts for this square is 136 sections, the same place the same square
stopped on Toroidal World 0.19.3 with DH 3.3.1 earlier (and again on 2026-10-17: 53.13 % at 21 s, no change
until 222 s; the control finished in 21 s). Exotic Worlds 1.0.0 with DH 3.3.1 and a few other mods installed
did the same (53.13 % at 24 s, stopped at 225 s; control 18 s). The stall point does not move between runs,
mod versions or DH versions.

Also seen on 0.19.3, with an extra mod of ours installed (see the note in "Steps to reproduce"):

- In an earlier run a larger pregeneration stopped once it had reached a radius of about 75 to 76
  chunks, and in another a stalled one showed no progress for 15 minutes.
- A centre exactly on the edge (x = E) is refused by the command, since E is outside the world. That is
  reasonable, but it means any pregeneration around the edge has to be centred on one side and so
  crosses it.

**Expected:** the pregeneration walks the whole square, treating the part past the edge as the world's
other side, and finishes.

**Actual:** it stalls once enough of the square lies past the edge, without an error.

## Steps to reproduce

Run with exactly these two mods on 2026-10-17 (and with other mods on top, with the same numbers):

1. A NeoForge 21.1.249 dedicated server for 1.21.1 with Exotic Worlds 1.0.0 and Distant Horizons 3.3.3 (or
   3.3.1) and nothing else.
2. Set `level-type=exotic_worlds:torus_large` in `server.properties` (the preset's X and Z loop is 4096
   blocks wide, so the edge is at x = E = 2048; with Toroidal World 0.19.3 the same preset is
   `toroidal_world:large`). Any size with room for a 1024-block square works; adjust E to your edge.
3. Start the server, wait for "Done", give Distant Horizons about 15 seconds to start its level, and run
   `/dh pregen start minecraft:overworld 1728 0 32`
   (centre E - 320, z = 0, radius 32 chunks).
4. Watch `/dh pregen status` (we polled it every 3 seconds). It climbs to 53.13 % (radius 23.324) within
   about 25 seconds and does not move again.
5. For comparison, on a fresh server, `/dh pregen start minecraft:overworld 1472 0 32`
   (centre E - 576, so the last 64-block section ends exactly at the edge). This one finishes, in 18 to 25
   seconds on our machine.

## Log

No error appears. The pregeneration lines of the stalled run (Exotic Worlds 1.0.0, Distant Horizons 3.3.3, two
mods, centre 1728), which stop at the last one shown; `dh pregen status` kept answering the 53.13 % line
below for the next 200 seconds, and the "Generation for section ... has expired!" warning never appeared
(counted: 0 lines):

```
[08:35:08] [Server thread/INFO] [Di.se.di.co.ge.PregenManager/]: Generated radius: 2 / 32 chunks (0 cps, 0.4%), ETA: 0s
[08:35:10] [DH-World Gen Thread[12]/INFO] [Di.se.di.co.ge.PregenManager/]: Generated radius: 9 / 32 chunks (39 cps, 7.8%), ETA: 1m 34s
[08:35:13] [DH-World Gen Thread[4]/INFO] [Di.se.di.co.ge.PregenManager/]: Generated radius: 14 / 32 chunks (112 cps, 19%), ETA: 29s
[08:35:15] [DH-World Gen Thread[2]/INFO] [Di.se.di.co.ge.PregenManager/]: Generated radius: 17.9 / 32 chunks (148 cps, 30%), ETA: 19s
[08:35:18] [DH-World Gen Thread[12]/INFO] [Di.se.di.co.ge.PregenManager/]: Generated radius: 20.69 / 32 chunks (149 cps, 42%), ETA: 15s
[08:35:20] [DH-World Gen Thread[2]/INFO] [Di.se.di.co.ge.PregenManager/]: Generated radius: 22.09 / 32 chunks (141 cps, 47.7%), ETA: 15s
[08:35:23] [DH-World Gen Thread[1]/INFO] [Di.se.di.co.ge.PregenManager/]: Generated radius: 23.07 / 32 chunks (132 cps, 52%), ETA: 14s
```

`/dh pregen status` at the end: `Generated radius: 23.324 / 32 chunks (131 cps, 53.13%), ETA: 14s`.
(The full `latest.log` goes with the issue; see the notes for the poster.)

## Where it does not happen

- A pregeneration square that stays on one side of the edge, as in step 5.
- On 0.19.3 with an extra mod of ours installed: pregenerating squares on either side of the edge, each kept
  on its own side, finished and Distant Horizons held every column we checked, so the folded sections
  themselves look fine. Only a square across the edge stalls.
- Players' clients do not use `/dh pregen`, so we have not seen this affect them.

## What we think is going on (our reading, please correct us)

We read the decompiled class files of Distant Horizons 3.3.3 (and, earlier with `javap`, 3.3.1) and of Exotic
Worlds 1.0.0 (and, earlier, Toroidal World 0.19.3). We have not stepped through it in a debugger, so treat
the following as a hypothesis that fits what we measured. The code involved is the same in 0.19.3 and 1.0.0,
and in 3.3.1 and 3.3.3.

- Distant Horizons, `com.seibel.distanthorizons.core.generation.PregenManager$PregenState`:
  `fillPendingQueue()` takes the next section from a spiral and files it in `pendingGenerations`, a Guava
  `Cache<Long, Long>` built with `expireAfterWrite(2, MINUTES)`, under that **raw** position. It then asks
  `GeneratedFullDataSourceProvider.getAsync(pos)` and, for a section that is not fully generated,
  `queuePositionForRetrieval(pos)`. It only adds more while `pendingGenerations.size()` is below
  `Config.Common.MultiThreading.numberOfThreads`. When the retrieval future completes, the callback calls
  `pendingGenerations.invalidate(result.pos)` with the position carried by the `DataSourceRetrievalResult`,
  and the cache's removal listener calls `fillPendingQueue()` again.
- Exotic Worlds,
  `com.exoticworlds.compat.distanthorizons.mixin.GeneratedFullDataSourceProviderMixin.toroidal$retrieveTheOneSection`,
  a `@WrapMethod` on `queuePositionForRetrieval(Long)`, calls the original with
  `DhKeys.foldSection(DhRepoLevel.latticeOf(repo), genPos)` and returns the original's future as it is. As far
  as we can tell the future's `DataSourceRetrievalResult.pos` is then the folded position and is not mapped
  back. (Toroidal World 0.19.3 had the same method with `DhKeys.foldSection` and the same shape.)

So for a section past the edge the entry is filed under the raw position and cleared under the folded one:
`invalidate` finds nothing, the raw entry stays, and the refill that the removal would have triggered
never happens. Each such section permanently takes one of the `numberOfThreads` slots, so once enough have
leaked nothing more is queued. The two-minute expiry should free them in principle, but Guava only evicts
expired entries while the cache is being written to (and occasionally read from), and nothing writes to it
once the queue has stopped. That would be why the expiry warning never appears, even after 200 seconds
(measured: 0 lines). That last part is inference; the stall itself and its position are measured.

We did not trace the other branch (a section found already fully generated, cleared by
`FullDataSourceV2.getPos()`). In the same package `AbstractDhRepoMixin.toroidal$answerTheAskedKey` wraps
`AbstractDhRepo.getByKey` and calls `DhKeys.reseat(dto, key)`, which answers under the asked key, which would
explain why that branch is fine.

## Workaround we use

We keep every pregeneration square on one side of the edge, as two runs: one ending exactly at the edge
and one starting exactly at the opposite edge. With a 32-chunk radius DH's spiral reaches 7 sections
(64 blocks each) below the centre's section and 8 above, so we centre them at E - 9 * 64 and -E + 7 * 64.
Nothing crosses the edge, so nothing leaks.

## Suggestion

If our reading is right, the smallest change might be in your wrapper: when the original future completes,
return a result whose `pos` is the position DH asked for rather than the folded one, the same idea as
`DhKeys.reseat` in `toroidal$answerTheAskedKey`. `DataSourceRetrievalResult.pos` is `final`, but the class has a
`CreateSuccess(long, FullDataSourceV2)` factory (in 3.3.3 as in 3.3.1) that could rebuild it for the success
case. Alternatively the fix could live in DH (`PregenState` invalidating the key it put in, not the one the
result reports); we are happy to raise it there if you think that is the better place. You know better than
we do whether anything else relies on the folded position in that result. We have not tried either change.

Thanks for the Distant Horizons support so far, and for looking at this.
