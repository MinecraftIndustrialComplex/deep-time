// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;

import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * A running simulation as the timelapse sees it (plan §6.1, §6.2): {@link SessionRunHandle} over a
 * {@link SimSession} in the game, scripted fakes in tests. Everything but {@link #pixels} is cheap
 * and safe from the render thread; {@link #pixels} may read a file and runs on a loader thread.
 */
public interface RunHandle {

    /** Where the run writes its world data (the staging folder). */
    Path outDir();

    SimSession.Progress progress();

    /** Keyframes produced since the last call, in frame order. */
    List<KeyframeInfo> newKeyframes();

    /** Events produced since the last call, in emission order. */
    List<GeoEvent> newEvents();

    /** Layer {@code layer} of frame {@code f} as ABGR ints ({@code w × h}); may block on I/O. */
    int[] pixels(KeyframeInfo f, int layer);

    void cancel();

    /** True while the run waits for an earlier (cancelled) run to stop before it starts. */
    default boolean waitingForPrevious() {
        return false;
    }

    /**
     * A live frame ({@code deeptime_core::live}): the one layer the map shows, drawn between the
     * keyframes while the view follows the run. Not a keyframe: it is not kept and cannot be scrubbed
     * to, stopped at or picked.
     *
     * @param seq  the session's live frame number, from 1 (a larger one is newer)
     * @param info its time, stage and size; {@code n} is −1 and {@code layers} holds its one layer
     * @param abgr the pixels, {@code w × h}
     */
    record LiveFrame(long seq, KeyframeInfo info, int[] abgr) {

        public String layer() {
            return info.layers().get(0);
        }
    }

    /**
     * Asks for live frames of {@code layer} (null: no more) and returns the newest one's number, 0
     * when there is none. The request lapses after two seconds, so the caller renews it on every poll
     * while it wants them. Cheap, safe from the render thread; a handle without a live map returns 0.
     */
    default long watchLive(String layer) {
        return 0;
    }

    /**
     * The newest live frame, or null (none, or the run is over). {@code like} is one of the run's
     * keyframes: the live frame has its size. May block briefly; runs on a loader thread.
     */
    default LiveFrame liveFrame(KeyframeInfo like) {
        return null;
    }

    /** Completes with the manifest JSON; exceptionally when cancelled or failed. */
    CompletableFuture<String> completion();
}
