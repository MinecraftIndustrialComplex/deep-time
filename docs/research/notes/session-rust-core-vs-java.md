# Rust simulation core against a full-Java implementation

Date: 2026-09-28 (the engine choice) and 2026-10-01 (a retrospective assessment); both from the
transcript's UTC timestamps. The decisions log dates the choice 2026-09-27, which is the local date.
Status: the coordinator's assessment, recorded from a working session. Not a literature review.

## Question

Should the simulation core be written in Rust and called from Java through JNI (the Java Native
Interface, which lets Java call a native library), or should the whole simulation be written in
Java? The question came up twice. The first time was before any code existed, when the engine had
to be chosen. The second was on 2026-10-01, when a large part of the Rust core was already built
and the owner asked whether the Rust side had been worth it.

## Assessment

Design time, 2026-09-28. The coordinator set out six routes: (A) a pure Java core, (B) Java plus a
GPU accelerator, (C) a native C++ or Rust core loaded through JNI, (D) Rust or C++ compiled to
WebAssembly and run inside Java by the Chicory runtime, (E) a bundled Python helper process, and
(F) a separate desktop program. Its first recommendation was route A, with the GPU path of route B
added only for previews if a benchmark missed the time budget.

The arguments it gave for route A were these. It needs no native binaries or launch flags, runs the
same on Windows, macOS, Linux and servers without a GPU, and gives results that are identical on
every machine. Against it, everything had to be ported, about 17,000 lines of reference code. Java
21 has no usable SIMD (vectorised arithmetic), because the Vector API is still an incubator module
that needs a launch flag a mod cannot set. And nobody had measured Java's speed on these
calculations.

The arguments it gave against route C were these. It needs a build per operating system and CPU.
A native solver in another mod in the target pack had crashed with an illegal-instruction signal
on a virtualised server and had no macOS build. Java's newer native interface was only a preview
in Java 21, so JNI was the only option. A Java fallback would mean maintaining two engines.

The owner then asked for an investigation of how Create Aeronautics handles its Rust code, and
said he expected a faster engine than Java would be needed. The coordinator reported that the Rust
lives in Sable, Aeronautics' physics library. One nested jar carries six binaries (Windows, macOS
and Linux, each on x64 and ARM; 9 MB compressed). Java calls them with plain JNI once per game
tick and waits for the call to return. Inside the call Rust spreads work over all cores with the
Rayon library. The binaries are unpacked to a folder and loaded with `System.load`, and all six
are cross-compiled on one Linux CI runner. Problems reported for Sable: no build for the musl C
library (so Alpine-based servers cannot run it), one antivirus product flagged a DLL, unpacking
repeatedly fails on Apple-silicon Macs, and a failed load crashes the game with no fallback. Sable's
own code is under a source-available licence that forbids competing uses, so the coordinator
proposed copying the pattern and not the code.

Retrospective, 2026-10-01. The coordinator said the Rust core was worth it, but that raw speed was
the smaller reason and that on speed alone it would be a close call. Its points were:

- Speed: tight numeric loops in Java usually run about 1.2 to 2 times slower than in Rust, and
  worse if the code uses many small objects. A 500-million-year Earth-like quick run took about
  7 minutes on a busy Apple-silicon desktop, so Java might take 10 to 15 minutes. It called this an
  estimate, not a measurement.
- Memory, which it called the bigger win: the simulation holds several GB while it runs. In Java
  that memory would sit in Minecraft's own heap beside a large modpack, forcing players to raise
  their memory setting just to create a world and causing garbage-collection pauses. Native memory
  sits outside the Java heap and is freed when the bake finishes. It said Java 21 has no finished
  way to work outside the heap (that arrives in Java 22).
- Tooling: the command-line tool, the browser workbench and headless runs all reuse the core
  without Minecraft. It noted a pure-Java core could do that too, so this advantage is smaller
  than it looks.
- Costs: native builds for every platform; a native crash takes down the game with no Java stack
  trace; the Java side has to re-implement reading of the baked file format, and the climate
  lookup must match the Rust one bit for bit, which needs its own tests; two toolchains and large
  build folders.

It also corrected a word: the Rust core is a native library loaded into the game's process, not a
sidecar process, and it runs once at world creation and writes a baked file. Chunk generation is
plain Java reading that file. It would make the same call again, because the simulation is the
product and runs as one batch per world. It said pure Java would win if the heavy work happened per
chunk or if the simulation were small.

## What was sourced and what was not

- Route comparison (2026-09-28): sub-agent research, saved as
  `docs/research/notes/runtime_packaging_options.md`. That note inspected the bytecode of two mods
  in the target pack and read web sources (for example the Vector API status and the Chicory
  project). Line counts of reference code come from the research report in
  `docs/research/geologic-history-worldgen-report.md`.
- How Sable works: a sub-agent (Sonnet model) investigated by inspecting the jars and reading the
  public repositories and issue tracker. Its report is saved as
  `docs/research/notes/aeronautics_rust_bridge.md`, which tags each claim as verified or inferred.
  The coordinator relayed it; the transcript does not show the coordinator checking it again.
- The retrospective figures (1.2 to 2 times, 10 to 15 minutes, "Java 22") are the coordinator's own
  estimates and knowledge. No source was consulted; this is the model's own knowledge and has not
  been checked. The 7-minute figure is a recollection of an earlier run. No run was read in that
  exchange.
- No Java version of any stage was written or timed, so there is no measured comparison.

## What was decided

On 2026-09-28 Ben chose "Rust engine via JNI", following Sable's pattern: a synchronous JNI call on
a background thread with Rayon inside, Java owning the screens, generator and Minecraft hooks,
musl builds added, a conservative CPU baseline, and a guidance message instead of a crash when the
native library fails to load. This is recorded in the "Engine and runtime" table of
`docs/deeptime-design-decisions.md` (rows Engine, Native targets, Standalone, Determinism). The
2026-10-01 retrospective produced no change of course and no log entry. A later 2026-10-01 entry
in the same log relaxes bit-reproducibility across machines, which removes one of the original
arguments for pure Java.

## Open

- A measured comparison does not exist. The coordinator offered to port one hot stage (river-flow
  routing or the climate step) to Java and time both; no record shows this was requested or done.
- The 1.2 to 2 times figure and the 10 to 15 minute estimate are unchecked.
- The claim about Java 22 and off-heap memory is unchecked.
- Whether the native library loads without incident on every target platform in real use is not
  established by this session; the Sable problems listed above are the risks the project accepted.
