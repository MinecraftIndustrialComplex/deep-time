# Draft issue: `/dh pregen` stalls when its square crosses the world's edge

**Status: DRAFT. Not posted.** Nobody has opened, commented or written to anyone about this.

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
2. **Attach a log.** The form requires `latest.log`. None is kept in this repo (the measurements were
   taken on a test machine that has since been rebuilt). Take it from a fresh reproduction.
3. **Test on the current release first.** Everything below was measured on Toroidal World 0.19.3 and
   Distant Horizons 3.3.1. The project has since been renamed Exotic Worlds and 1.0.0 was published on
   2026-10-11; its notes say "Distant Horizons 3.3.3 draws the distant terrain across the world's edge".
   The stall may already be gone. One run of the steps below on 1.0.0 with DH 3.3.3 settles it, and the
   draft should then be updated to the versions it was seen on.
4. **The reproduction is not yet Deep-Time-free in practice.** The steps below need only the two mods, but
   every measurement was taken with an experimental worldgen mod of ours also installed. The draft says
   so and does not name that mod (its repository is private); naming it is your call.
5. Nothing in the draft names a person, a path or an address.

---

## Title

`/dh pregen` stalls part-way when its square crosses the world's edge

## Minecraft version and mod loader

Minecraft 1.21.1, NeoForge 21.1.249, Java 21, dedicated server.

## Mod version

- Toroidal World 0.19.3+mc1.21.1 (`toroidal_world-0.19.3+mc1.21.1.jar`, Modrinth version `8YTSYi9y`;
  the Modrinth page is now called Exotic Worlds)
- Distant Horizons 3.3.1-1.21.1 (Modrinth version `IcOcoekl`)

We have not tried Exotic Worlds 1.0.0 or Distant Horizons 3.3.3 for this, so it may already be fixed there.

## What happened

Distant Horizons' server-side pregeneration (`/dh pregen start minecraft:overworld <x> <z> <radius>`)
does not finish when the square it covers reaches past the world's edge. It runs on the near side, then
stops: `/dh pregen status` stops advancing, no error is reported, and it never recovers by itself. A square
of the same size that ends exactly at the edge finishes normally.

What we measured, on a world whose X loop has its edge at x = E:

- A square of radius 32 chunks (the smallest DH accepts) centred 320 blocks before the edge stopped at
  53.13 % after 136 sections and did not move for 200 seconds.
- A square of the same size ending exactly at the edge finished all 257 sections in under 10 seconds
  (in our setup).
- In an earlier run a larger pregeneration stopped once it had reached a radius of about 75 to 76
  chunks, and in another a stalled one showed no progress for 15 minutes.
- A centre exactly on the edge (x = E) is refused by the command, since E is outside the world. That is
  reasonable, but it means any pregeneration around the edge has to be centred on one side and so
  crosses it.

**Expected:** the pregeneration walks the whole square, treating the part past the edge as the world's
other side, and finishes.

**Actual:** it stalls once enough of the square lies past the edge, without an error.

## Steps to reproduce

We expect these to reproduce it with only the two mods. We have **not** run them in exactly this form: our
measurements were taken on a server that also had a mod of ours which registers its own Distant Horizons
world generator, using a small patch of ours to `DhGenerators.requireOwn` so that your guard lets that
generator through. From our reading of the code (below) the failing step happens before any generator is
involved, so we think DH's own generator hits the same thing, but that is a reading, not a test.

1. A NeoForge 21.1.249 dedicated server for 1.21.1 with Toroidal World 0.19.3 and Distant Horizons 3.3.1
   and nothing else of interest.
2. Set `level-type=toroidal_world:large` in `server.properties` (a toroidal preset; by our reading of the
   preset it loops x from chunk -128 to 128, so the edge is at x = E = 2048). Any size with room for a
   1024-block square works; adjust E to your edge.
3. Start the server, let the spawn load, and run
   `/dh pregen start minecraft:overworld 1728 0 32`
   (centre E - 320, z = 0, radius 32 chunks).
4. Watch `/dh pregen status`. It climbs to roughly half and stops there.
5. For comparison, `/dh pregen stop`, then `/dh pregen start minecraft:overworld 1472 0 32`
   (centre E - 576, so the last 64-block section ends exactly at the edge). This one finishes.

## Log

(Attach `latest.log` from a fresh run. We have not kept an excerpt from our runs; we saw no error
reported while it was stuck.)

## Where it does not happen

- A pregeneration square that stays on one side of the edge, as in step 5.
- Pregenerating squares on either side of the edge, each kept on its own side, finished and Distant
  Horizons held every column we checked, so the folded sections themselves look fine. Only a square
  across the edge stalls.
- Players' clients do not use `/dh pregen`, so we have not seen this affect them.

## What we think is going on (our reading, please correct us)

We read the class files of Distant Horizons 3.3.1 and Toroidal World 0.19.3 with `javap`. We have not
stepped through it in a debugger, so treat the following as a hypothesis that fits what we measured.

- Distant Horizons, `com.seibel.distanthorizons.core.generation.PregenManager$PregenState`:
  `fillPendingQueue()` takes the next section from a spiral and files it in `pendingGenerations`, a Guava
  `Cache<Long, Long>` built with `expireAfterWrite(2, MINUTES)`, under that **raw** position. It then asks
  `GeneratedFullDataSourceProvider.getAsync(pos)` and, for a section that is not fully generated,
  `queuePositionForRetrieval(pos)`. It only adds more while `pendingGenerations.size()` is below
  `Config.Common.MultiThreading.numberOfThreads`. When the retrieval future completes,
  `lambda$fillPendingQueue$1` calls `pendingGenerations.invalidate(result.pos)` with the position carried
  by the `DataSourceRetrievalResult`, and the cache's removal listener (`lambda$new$0`) calls
  `fillPendingQueue()` again.
- Toroidal World,
  `com.toroidalworld.compat.distanthorizons.mixin.GeneratedFullDataSourceProviderMixin.toroidal$retrieveTheOneSection`,
  a `@WrapMethod` on `queuePositionForRetrieval(Long)`, folds `genPos` with `DhKeys.foldSection` before
  calling the original. As far as we can tell the returned future's `DataSourceRetrievalResult.pos` is
  then the folded position and is not mapped back.

So for a section past the edge the entry is filed under the raw position and cleared under the folded one:
`invalidate` finds nothing, the raw entry stays, and the refill that the removal would have triggered
never happens. Each such section permanently takes one of the `numberOfThreads` slots, so once enough have
leaked nothing more is queued. The two-minute expiry should free them in principle, but Guava only evicts
expired entries while the cache is being written to (and occasionally read from), and nothing writes to it
once the queue has stopped. That would be why we saw no expiry message ("Generation for section ... has expired!")
even after 200 seconds. That last part is inference; the stall itself and its position are measured.

We did not trace the other branch (a section found already fully generated, cleared by
`FullDataSourceV2.getPos()`); `AbstractDhRepoMixin.toroidal$answerTheAskedKey` appears to answer under the
asked key there, which would explain why that branch is fine.

## Workaround we use

We keep every pregeneration square on one side of the edge, as two runs: one ending exactly at the edge
and one starting exactly at the opposite edge. With a 32-chunk radius DH's spiral reaches 7 sections
(64 blocks each) below the centre's section and 8 above, so we centre them at E - 9 * 64 and -E + 7 * 64.
Nothing crosses the edge, so nothing leaks.

## Suggestion

If our reading is right, the smallest change might be in your wrapper: when the original future completes,
return a result whose `pos` is the position DH asked for rather than the folded one. `DataSourceRetrievalResult.pos`
is `final`, but the class has a `CreateSuccess(long, FullDataSourceV2)` factory that could rebuild it for
the success case. Alternatively the fix could live in DH (`PregenState` invalidating the key it put in,
not the one the result reports); we are happy to raise it there if you think that is the better place. You
know better than we do whether anything else relies on the folded position in that result. We have not
tried either change.

Thanks for the Distant Horizons support so far, and for looking at this.
