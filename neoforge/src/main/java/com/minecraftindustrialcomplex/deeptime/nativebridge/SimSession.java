// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.nio.ByteBuffer;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Semaphore;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * One simulation run (plan §6.1, §6.2). The preset and content are JSON strings. A daemon
 * {@code deeptime-sim} thread loops {@code advance(250)}, forwards the native log, and calls
 * {@code finalizeWrite} and {@code destroy}. {@link #progress()} and {@link #cancel()} are cheap and
 * safe from any thread, including the render thread.
 *
 * <p>Two ways to start one:
 * <ul>
 *   <li>{@link #start}: creates the native session on the calling thread, so a bad preset throws
 *       {@link PresetInvalidException} at once (the dedicated server's first start, P1-13).</li>
 *   <li>{@link #launch}: returns at once and creates the session on the worker, as plan §6.2 puts
 *       it (the Create World screen, P1-11: the render thread never waits for {@code create}); a bad
 *       preset then fails {@link #completion()}. With {@code collectTimeline} the worker also
 *       gathers each new keyframe's info and every new event after each step and once more before
 *       it destroys the session, so nothing produced in the last step is lost; the render thread
 *       takes them with {@link #newKeyframeInfos()} and {@link #newEventBatches()}.</li>
 * </ul>
 *
 * Keyframe pixels come from {@link #copyKeyframe} while the session lives; afterwards (it is
 * destroyed as soon as it finishes) they are on disk under {@link #outDir()}'s {@code keyframes/},
 * which {@code worlddata.KeyframeFiles} reads without natives.
 */
public final class SimSession implements AutoCloseable {

    /** Milliseconds per {@code advance} call on the worker. */
    static final int ADVANCE_BUDGET_MS = 250;

    /** Events fetched per {@code eventsSince} call while collecting. */
    static final int EVENT_BATCH = 4096;

    /**
     * The progress block (plan §6.1).
     *
     * @param state          {@link #RUNNING}, {@link #DONE}, {@link #CANCELLED}, {@link #FAILED} or {@link #FINALIZED}
     * @param simulatedTime  simulated time in years × −1
     * @param etaMs          −1 while unknown
     */
    public record Progress(int state, int stageIndex, int stageCount, int stagePermille, int totalPermille,
            long simulatedTime, long elapsedMs, long etaMs, long nativeBytesInUse, long nativeBytesPeak,
            int keyframesReady, long eventsTotal) {

        public static final int RUNNING = DeepTimeNative.RUNNING;
        public static final int DONE = DeepTimeNative.DONE;
        public static final int CANCELLED = DeepTimeNative.CANCELLED;
        public static final int FAILED = DeepTimeNative.FAILED;
        public static final int FINALIZED = DeepTimeNative.FINALIZED;

        public static final Progress INITIAL = new Progress(RUNNING, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0);

        static Progress of(long[] a) {
            return new Progress((int) a[0], (int) a[1], (int) a[2], (int) a[3], (int) a[4], a[5], a[6], a[7], a[8],
                    a[9], (int) a[10], a[11]);
        }
    }

    private interface Creator {
        long create();
    }

    /**
     * Launched sessions (the Create World screen's) run one at a time: a run started while a
     * cancelled one is still stopping (a step, the bake, can take a while) waits for it, so a
     * player's machine never holds two planets' memory at once. The server's {@link #start} is not
     * gated (it runs one session, on its own thread, before any level exists).
     */
    private static final Semaphore LAUNCH_GATE = new Semaphore(1, true);
    private static final AtomicInteger LAUNCHED_LIVE = new AtomicInteger();

    private final Path outDir;
    private final boolean collectTimeline;
    private final boolean gated;
    private volatile boolean waitingForGate;
    private final CompletableFuture<String> completion = new CompletableFuture<>();
    private final Thread worker;
    /** The native id; 0 until created (session ids start at 1). */
    private volatile long id;
    private volatile Progress last = Progress.INITIAL;
    private volatile boolean destroyed;
    private volatile boolean cancelRequested;
    private final Object lifecycle = new Object();

    // Collected by the worker, taken by the render thread (guarded by `this`).
    private final List<String> keyframeInfos = new ArrayList<>();
    private final List<String> eventBatches = new ArrayList<>();
    private int keyframesCollected;
    private long eventsCollected;

    private SimSession(Path outDir, boolean collectTimeline, boolean gated, Creator creator) {
        this.outDir = outDir;
        this.collectTimeline = collectTimeline;
        this.gated = gated;
        this.worker = new Thread(() -> run(creator), "deeptime-sim");
        this.worker.setDaemon(true);
    }

    /**
     * Creates the native session on this thread and starts its worker.
     *
     * @param threads     Rayon threads, 0 = all processors
     * @param memCapBytes native memory cap, 0 = the 4 GiB default
     * @throws PresetInvalidException if the preset or content is malformed
     * @throws IllegalStateException  if the native library is not linked
     */
    public static SimSession start(String presetJson, String contentJson, Path outDir, int threads, long memCapBytes) {
        Objects.requireNonNull(presetJson, "presetJson");
        Objects.requireNonNull(outDir, "outDir");
        requireLinked();
        long created = DeepTimeNative.create(presetJson, contentJson == null ? "" : contentJson,
                outDir.toAbsolutePath().toString(), threads, memCapBytes);
        SimSession s = new SimSession(outDir, false, false, () -> created);
        s.id = created;
        s.worker.start();
        return s;
    }

    /**
     * Starts a worker that creates the native session and runs it; returns at once. A preset the
     * core rejects fails {@link #completion()} with {@link PresetInvalidException}.
     *
     * @param collectTimeline gather keyframe infos and events for {@link #newKeyframeInfos()} and
     *                        {@link #newEventBatches()}
     * @throws IllegalStateException if the native library is not linked
     */
    public static SimSession launch(String presetJson, String contentJson, Path outDir, int threads, long memCapBytes,
            boolean collectTimeline) {
        return launch(presetJson, contentJson, outDir, threads, memCapBytes, collectTimeline, Mode.PLAIN);
    }

    /** How a launched run starts (docs/design/playability-steering.md). */
    public enum Mode {
        /** A new run, no epoch checkpoints. */
        PLAIN(0),
        /** A new run that writes epoch checkpoints, so it can be picked at a keyframe, stopped or continued. */
        EPOCHS(DeepTimeNative.FLAG_EPOCHS),
        /**
         * The run already in the folder, to the preset's {@code timeline.stop_myr}, from its epoch
         * checkpoints (it keeps writing them).
         */
        RESUME(DeepTimeNative.FLAG_EPOCHS | DeepTimeNative.FLAG_RESUME);

        final int flags;

        Mode(int flags) {
            this.flags = flags;
        }
    }

    /** {@link #launch(String, String, Path, int, long, boolean)} in a {@link Mode}. */
    public static SimSession launch(String presetJson, String contentJson, Path outDir, int threads, long memCapBytes,
            boolean collectTimeline, Mode mode) {
        Objects.requireNonNull(presetJson, "presetJson");
        Objects.requireNonNull(outDir, "outDir");
        Objects.requireNonNull(mode, "mode");
        requireLinked();
        String content = contentJson == null ? "" : contentJson;
        String out = outDir.toAbsolutePath().toString();
        SimSession s = new SimSession(outDir, collectTimeline, true, mode == Mode.PLAIN
                ? () -> DeepTimeNative.create(presetJson, content, out, threads, memCapBytes)
                : () -> DeepTimeNative.createWith(presetJson, content, out, threads, memCapBytes, mode.flags));
        s.worker.start();
        return s;
    }

    private static void requireLinked() {
        if (!NativeLoader.isLinked())
            throw new IllegalStateException("the Deep Time native library is not linked"
                    + NativeLoader.status().map(s -> ": " + s.summary()).orElse(""));
    }

    /**
     * The memory and time estimate of a preset (plan §5.7.5), as JSON
     * {@code {"peak_bytes":P,"stages":[{"bytes":B,"name":N,"seconds":S},…]}}, without creating a
     * session. Cheap; safe from any thread.
     *
     * @throws PresetInvalidException if the preset or content is malformed
     * @throws IllegalStateException  if the native library is not linked
     */
    public static String estimate(String presetJson, String contentJson) {
        Objects.requireNonNull(presetJson, "presetJson");
        requireLinked();
        return DeepTimeNative.estimate(presetJson, contentJson == null ? "" : contentJson);
    }

    /** The native session id (diagnostics only); 0 until the worker created it. */
    public long id() {
        return id;
    }

    /** Where the session writes its world data (keyframes while it runs, the rest at the end). */
    public Path outDir() {
        return outDir;
    }

    /**
     * The worker: create, advance until done, finalize, destroy. {@link #completion()} completes
     * only after the native session is destroyed, so whoever waits on it (the first-start
     * generator moving its work folder into the world, the staging mover) never races a session
     * that still holds files in {@link #outDir()} (P1-17: the recorded history's spill file, which
     * on Windows lives there until the session closes it).
     */
    private void run(Creator creator) {
        String manifest = null;
        Throwable failure = null;
        boolean holdsGate = false;
        try {
            if (gated) {
                waitingForGate = true;
                try {
                    while (!(holdsGate = LAUNCH_GATE.tryAcquire(200, TimeUnit.MILLISECONDS))) {
                        if (cancelRequested) {
                            failure = new CancelledException("cancelled before it started");
                            return;
                        }
                    }
                } finally {
                    waitingForGate = false;
                }
                LAUNCHED_LIVE.incrementAndGet();
            }
            if (cancelRequested && id == 0) {
                failure = new CancelledException("cancelled before it started");
                return;
            }
            if (id == 0) {
                long created = creator.create();
                synchronized (lifecycle) {
                    id = created;
                }
                if (cancelRequested)
                    DeepTimeNative.cancel(created);
            }
            while (true) {
                int st = DeepTimeNative.advance(id, ADVANCE_BUDGET_MS);
                refreshProgress();
                collect();
                NativeLoader.forwardNativeLog(DeepTimeNative.drainLog(id));
                if (st == DeepTimeNative.DONE) {
                    manifest = DeepTimeNative.finalizeWrite(id);
                    refreshProgress();
                    collect();
                    return;
                }
                if (st == DeepTimeNative.CANCELLED) {
                    failure = new CancelledException("the session was cancelled");
                    return;
                }
            }
        } catch (Throwable t) {
            failure = t;
        } finally {
            long sid = id;
            if (sid != 0) {
                try {
                    NativeLoader.forwardNativeLog(DeepTimeNative.drainLog(sid));
                } catch (RuntimeException ignored) {
                    // the log is best effort
                }
                synchronized (lifecycle) {
                    destroyed = true;
                    try {
                        DeepTimeNative.destroy(sid);
                    } catch (RuntimeException ignored) {
                        // already gone
                    }
                }
            } else {
                destroyed = true;
            }
            try {
                NativeLoader.forwardNativeLog(DeepTimeNative.drainLog(0));
            } catch (RuntimeException ignored) {
                // the log is best effort
            }
            if (holdsGate) {
                LAUNCHED_LIVE.decrementAndGet();
                LAUNCH_GATE.release();
            }
            if (failure != null)
                completion.completeExceptionally(failure);
            else if (manifest != null)
                completion.complete(manifest);
            else
                completion.completeExceptionally(new IllegalStateException("the simulation worker stopped without a result"));
        }
    }

    /** On the worker, between steps (nothing is being produced then): new keyframe infos and events. */
    private void collect() {
        if (!collectTimeline)
            return;
        Progress p = last;
        int frames = DeepTimeNative.keyframeCount(id);
        List<String> infos = new ArrayList<>();
        for (int n = keyframesCollected; n < frames; n++)
            infos.add(DeepTimeNative.keyframeInfo(id, n));
        List<String> batches = new ArrayList<>();
        long cursor = eventsCollected;
        while (cursor < p.eventsTotal()) {
            int want = (int) Math.min(EVENT_BATCH, p.eventsTotal() - cursor);
            String json = DeepTimeNative.eventsSince(id, cursor, want);
            batches.add(json);
            cursor += want;
        }
        synchronized (this) {
            keyframeInfos.addAll(infos);
            eventBatches.addAll(batches);
            keyframesCollected = frames;
            eventsCollected = cursor;
        }
    }

    private void refreshProgress() {
        long[] a = new long[DeepTimeNative.PROGRESS_FIELDS];
        DeepTimeNative.progress(id, a);
        last = Progress.of(a);
    }

    /** The latest progress; cheap and safe from the render thread. After the run, the final values. */
    public Progress progress() {
        if (!destroyed && id != 0) {
            synchronized (lifecycle) {
                if (!destroyed) {
                    try {
                        refreshProgress();
                    } catch (RuntimeException e) {
                        // poisoned: keep the last values
                    }
                }
            }
        }
        return last;
    }

    /**
     * The keyframe infos ({@code keyframeInfo} JSON, plan §6.1) collected since the last call, in
     * frame order; empty unless the session was launched with {@code collectTimeline}.
     */
    public synchronized List<String> newKeyframeInfos() {
        List<String> out = List.copyOf(keyframeInfos);
        keyframeInfos.clear();
        return out;
    }

    /**
     * The events collected since the last call, as the JSON arrays {@code eventsSince} returned
     * ({@code EVENTS.md}, "JSON"), in emission order; empty unless launched with {@code collectTimeline}.
     */
    public synchronized List<String> newEventBatches() {
        List<String> out = List.copyOf(eventBatches);
        eventBatches.clear();
        return out;
    }

    /**
     * Copies layer {@code layer} of keyframe {@code index} as RGBA8 into a direct buffer of at least
     * w·h·4 bytes (plan §6.1). Safe from any thread while the session lives.
     *
     * @throws IllegalStateException once the session is destroyed (read the frame from disk then)
     */
    public void copyKeyframe(int index, int layer, ByteBuffer dstRgba) {
        synchronized (lifecycle) {
            if (destroyed || id == 0)
                throw new IllegalStateException("the session is " + (destroyed ? "finished" : "not created yet")
                        + "; read keyframe " + index + " from " + outDir.resolve("keyframes"));
            DeepTimeNative.copyKeyframe(id, index, layer, dstRgba);
        }
    }

    /** True while this launched session waits for an earlier one to stop (see {@code LAUNCH_GATE}). */
    public boolean waitingForPrevious() {
        return waitingForGate;
    }

    /** Launched sessions alive now (created or being created, not yet destroyed): 0 or 1. */
    public static int launchedLive() {
        return LAUNCHED_LIVE.get();
    }

    /** True once the worker has finished and destroyed the native session. */
    public boolean isFinished() {
        return destroyed;
    }

    /** Requests cancellation; the worker then stops within one step and {@link #completion()} fails with {@link CancelledException}. */
    public void cancel() {
        cancelRequested = true;
        synchronized (lifecycle) {
            if (destroyed || id == 0)
                return;
            try {
                DeepTimeNative.cancel(id);
            } catch (RuntimeException ignored) {
                // already destroyed or poisoned: nothing left to cancel
            }
        }
    }

    /** Completes with the manifest JSON, or exceptionally ({@link CancelledException}, a native exception). */
    public CompletableFuture<String> completion() {
        return completion;
    }

    /** Cancels if running and waits (up to 10 s) for the worker, which destroys the native session. */
    @Override
    public void close() {
        cancel();
        try {
            worker.join(TimeUnit.SECONDS.toMillis(10));
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
        }
    }
}
