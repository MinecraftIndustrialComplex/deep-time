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

    /** Completes with the manifest JSON; exceptionally when cancelled or failed. */
    CompletableFuture<String> completion();
}
