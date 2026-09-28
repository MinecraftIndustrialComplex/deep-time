// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.nio.file.Path;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;

/**
 * One simulation run (plan §6.1, §6.2), in its Phase 0 form: the preset and content are JSON
 * strings (the typed {@code Preset}/{@code ContentBundle}/{@code SimOptions} and the keyframe and
 * event lists arrive with Phase 1). {@link #start} creates the native session on the calling thread,
 * so a bad preset throws {@link PresetInvalidException} at once; a daemon {@code deeptime-sim}
 * thread then loops {@code advance(250)}, forwards the native log, and calls
 * {@code finalizeWrite} and {@code destroy}. {@link #progress()} and {@link #cancel()} are cheap and
 * safe from any thread, including the render thread.
 */
public final class SimSession implements AutoCloseable {

    /** Milliseconds per {@code advance} call on the worker. */
    static final int ADVANCE_BUDGET_MS = 250;

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

        static final Progress INITIAL = new Progress(RUNNING, 0, 0, 0, 0, 0, 0, -1, 0, 0, 0, 0);

        static Progress of(long[] a) {
            return new Progress((int) a[0], (int) a[1], (int) a[2], (int) a[3], (int) a[4], a[5], a[6], a[7], a[8],
                    a[9], (int) a[10], a[11]);
        }
    }

    private final long id;
    private final CompletableFuture<String> completion = new CompletableFuture<>();
    private final Thread worker;
    private volatile Progress last = Progress.INITIAL;
    private volatile boolean destroyed;

    private SimSession(long id) {
        this.id = id;
        this.worker = new Thread(this::run, "deeptime-sim");
        this.worker.setDaemon(true);
    }

    /**
     * Creates the native session and starts its worker.
     *
     * @param threads     Rayon threads, 0 = all processors
     * @param memCapBytes native memory cap, 0 = the 4 GiB default
     * @throws PresetInvalidException if the preset or content is malformed
     * @throws IllegalStateException  if the native library is not linked
     */
    public static SimSession start(String presetJson, String contentJson, Path outDir, int threads, long memCapBytes) {
        Objects.requireNonNull(presetJson, "presetJson");
        Objects.requireNonNull(outDir, "outDir");
        if (!NativeLoader.isLinked())
            throw new IllegalStateException("the Deep Time native library is not linked"
                    + NativeLoader.status().map(s -> ": " + s.summary()).orElse(""));
        long id = DeepTimeNative.create(presetJson, contentJson == null ? "" : contentJson,
                outDir.toAbsolutePath().toString(), threads, memCapBytes);
        SimSession s = new SimSession(id);
        s.worker.start();
        return s;
    }

    /** The native session id (diagnostics only). */
    public long id() {
        return id;
    }

    private void run() {
        try {
            while (true) {
                int st = DeepTimeNative.advance(id, ADVANCE_BUDGET_MS);
                refreshProgress();
                NativeLoader.forwardNativeLog(DeepTimeNative.drainLog(id));
                if (st == DeepTimeNative.DONE) {
                    String manifest = DeepTimeNative.finalizeWrite(id);
                    refreshProgress();
                    completion.complete(manifest);
                    return;
                }
                if (st == DeepTimeNative.CANCELLED) {
                    completion.completeExceptionally(new CancelledException("the session was cancelled"));
                    return;
                }
            }
        } catch (Throwable t) {
            completion.completeExceptionally(t);
        } finally {
            try {
                NativeLoader.forwardNativeLog(DeepTimeNative.drainLog(id));
            } catch (RuntimeException ignored) {
                // the log is best effort
            }
            destroyed = true;
            try {
                DeepTimeNative.destroy(id);
            } catch (RuntimeException ignored) {
                // already gone
            }
            try {
                NativeLoader.forwardNativeLog(DeepTimeNative.drainLog(0));
            } catch (RuntimeException ignored) {
                // the log is best effort
            }
        }
    }

    private void refreshProgress() {
        long[] a = new long[DeepTimeNative.PROGRESS_FIELDS];
        DeepTimeNative.progress(id, a);
        last = Progress.of(a);
    }

    /** The latest progress; cheap and safe from the render thread. After the run, the final values. */
    public Progress progress() {
        if (!destroyed) {
            try {
                refreshProgress();
            } catch (RuntimeException e) {
                // destroyed between the check and the call (IllegalStateException) or poisoned
            }
        }
        return last;
    }

    /** Requests cancellation; the worker then stops within one step and {@link #completion()} fails with {@link CancelledException}. */
    public void cancel() {
        if (destroyed)
            return;
        try {
            DeepTimeNative.cancel(id);
        } catch (RuntimeException ignored) {
            // already destroyed or poisoned: nothing left to cancel
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
