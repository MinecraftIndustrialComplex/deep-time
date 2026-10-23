// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import java.nio.ByteBuffer;

/**
 * The native methods of {@code deeptime_jni} (plan §6.1). Package-private: only
 * {@link NativeLoader}, {@link SimSession} and {@link NativeWorldMap} call it, and only after {@link NativeLoader} linked
 * the library. Errors arrive as the exceptions of this package (plan §6.3), or as
 * {@link IllegalStateException} for a stale or forged session id,
 * {@link IllegalArgumentException} for a bad argument and
 * {@link UnsupportedOperationException} for an entry point that has no implementation yet.
 */
final class DeepTimeNative {

    /** Length of the array {@link #progress} fills. */
    static final int PROGRESS_FIELDS = 12;

    // advance() results; also the first progress field while running, done or cancelled.
    static final int RUNNING = 0;
    static final int STAGE_DONE = 1;
    static final int DONE = 2;
    static final int CANCELLED = 3;
    // Further states of the first progress field.
    static final int FAILED = 4;
    static final int FINALIZED = 5;

    private DeepTimeNative() {}

    /** {@code "<semver>+<git sha>;format=<f>;target=<rust triple>;cpu=<baseline|x86-64-v3>"}. */
    static native String version();

    /** The fixed-input Rayon reduction; must equal {@link NativeLoader#expectedSelfTest()}. */
    static native long selfTest();

    /** JSON {@code {"peak_bytes":P,"stages":[{"bytes":B,"name":N,"seconds":S},…]}} (plan §5.7.5). */
    static native String estimate(String presetJson, String contentJson);

    /** Returns a session id (never a pointer). {@code threads} 0 = all processors; {@code memCapBytes} 0 = 4 GiB. */
    static native long create(String presetJson, String contentJson, String outDir, int threads, long memCapBytes);

    /** {@link #createWith} flag: write epoch checkpoints into {@code <outDir>/epochs/}. */
    static final int FLAG_EPOCHS = 1;
    /**
     * {@link #createWith} flag: pick up the run already in {@code outDir} at the preset's
     * {@code timeline.stop_myr}, from its epoch checkpoints (an epoch picked, a run stopped early or
     * continued; docs/design/playability-steering.md).
     */
    static final int FLAG_RESUME = 2;
    /**
     * {@link #createWith} flags: run the simulation on the CPU only, or on the GPU where one is
     * usable (falling back to the CPU when it is not). Neither is automatic; both is an error
     * (docs/design/gpu-compute.md).
     */
    static final int FLAG_COMPUTE_CPU = 4;
    static final int FLAG_COMPUTE_GPU = 8;

    /** {@link #create} with {@link #FLAG_EPOCHS} and {@link #FLAG_RESUME}. */
    static native long createWith(String presetJson, String contentJson, String outDir, int threads, long memCapBytes,
            int flags);

    /** Runs for about {@code budgetMillis}; returns {@link #RUNNING}, {@link #STAGE_DONE}, {@link #DONE} or {@link #CANCELLED}. */
    static native int advance(long session, int budgetMillis);

    /** Any thread; sets a flag and never blocks on a running {@link #advance}. */
    static native void cancel(long session);

    /** Fills the first {@link #PROGRESS_FIELDS} longs of {@code out}. */
    static native void progress(long session, long[] out);

    /** Keyframes produced so far (frames 0 … count − 1 stay readable after the ring drops them). */
    static native int keyframeCount(long session);

    /**
     * JSON {@code {"h","layers","n","stage","t","w"}} of a keyframe. {@link UnsupportedOperationException}
     * when the preset produces no keyframes at all, {@link IllegalArgumentException} for an index not
     * produced yet.
     */
    static native String keyframeInfo(long session, int index);

    /** Copies layer {@code layer} of a keyframe as RGBA8 into a direct buffer of at least w·h·4 bytes. */
    static native void copyKeyframe(long session, int index, int layer, ByteBuffer dstRgba);

    /**
     * Asks for the live map of {@code layer} (the Create World map while it follows the run;
     * {@code deeptime_core::live}) for the next two seconds; null or "" stops. Returns the newest live
     * frame's number, 0 before the first. Any thread; never blocks on a running {@link #advance}.
     */
    static native long watchLive(long session, String layer);

    /**
     * Copies the newest live frame as RGBA8 into a direct buffer of at least w·h·4 bytes (the
     * keyframes' size) and returns its JSON {@code {"h","layer","seq","stage","t","w"}}; "" before
     * the first. Any thread; never blocks on a running {@link #advance}.
     */
    static native String copyLiveFrame(long session, ByteBuffer dstRgba);

    /**
     * JSON array of at most {@code max} events after {@code cursor}, which counts events in emission
     * order (pass the number already received). The event JSON is specified in {@code EVENTS.md}.
     */
    static native String eventsSince(long session, long cursor, int max);

    /**
     * Empties a native log ring: session 0 is the process-wide ring (panics, create/destroy). Lines
     * are {@code LEVEL<TAB>message} joined by {@code \n}; the empty string means nothing new.
     */
    static native String drainLog(long session);

    /** After {@link #DONE}: writes {@code manifest.json} last and returns it. */
    static native String finalizeWrite(long session);

    static native void destroy(long session);

    /**
     * Parity oracle for tests (plan §6.1, §9.2): column (x, z) of a world-data folder as Rust's column
     * evaluator answers it, the JSON {@code parity/columns.json} holds per column (FORMAT.md,
     * "Columns"). The Java twin is {@code worlddata/Columns}.
     */
    static native String sampleColumn(String worldDataDir, int x, int z);

    /**
     * A world-data folder drawn as the workbench's Final world draws it (deeptime-tiles'
     * {@code BakeMap::render_frame}): {@code layer} {@code "sat"} (the satellite look) or {@code "elev"}
     * over a frame of {@code w × h} pixels whose pixel (i, j) is block (x0 + (i + ½)·s, z0 + (j + ½)·s),
     * RGBA8 row by row from the north into the direct buffer {@code dstRgba} (at least w·h·4 bytes),
     * drawn by {@code threads} threads. Returns JSON
     * {@code {"layer","w","h","zoom","climate","rivers","lakes","ms"}}. Writes nothing to the folder.
     * See {@link NativeWorldMap}.
     */
    static native String renderWorldMap(String worldDataDir, String layer, double x0, double z0, double blocksPerPx,
                                        int w, int h, int threads, ByteBuffer dstRgba);
}
