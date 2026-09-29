// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.nativebridge.CancelledException;
import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicInteger;

/** A scripted run: the test pushes frames and events and finishes it. Pixels encode (frame, layer). */
final class FakeRunHandle implements RunHandle {

    final Path out;
    final List<KeyframeInfo> pendingFrames = new ArrayList<>();
    final List<GeoEvent> pendingEvents = new ArrayList<>();
    final CompletableFuture<String> completion = new CompletableFuture<>();
    final AtomicInteger pixelReads = new AtomicInteger();
    SimSession.Progress progress = SimSession.Progress.INITIAL;
    boolean cancelled;
    int frames;

    FakeRunHandle(Path out) {
        this.out = out;
    }

    /** Adds a frame of {@code w × h} at time {@code t} with the given layers. */
    KeyframeInfo frame(double t, int w, int h, String... layers) {
        KeyframeInfo f = new KeyframeInfo(frames++, t, "dummy_drift", w, h, List.of(layers));
        pendingFrames.add(f);
        return f;
    }

    GeoEvent event(long id, double t, double lat, double lon) {
        GeoEvent e = new GeoEvent(id, t, t, "deeptime:rift_onset", Double.isNaN(lat) ? "global" : "point", lat, lon, -1, 0.5);
        pendingEvents.add(e);
        return e;
    }

    @Override
    public Path outDir() {
        return out;
    }

    @Override
    public SimSession.Progress progress() {
        return progress;
    }

    @Override
    public List<KeyframeInfo> newKeyframes() {
        List<KeyframeInfo> o = List.copyOf(pendingFrames);
        pendingFrames.clear();
        return o;
    }

    @Override
    public List<GeoEvent> newEvents() {
        List<GeoEvent> o = List.copyOf(pendingEvents);
        pendingEvents.clear();
        return o;
    }

    /** Every pixel of frame n, layer l is {@code 0xff000000 | n << 8 | l}. */
    @Override
    public int[] pixels(KeyframeInfo f, int layer) {
        pixelReads.incrementAndGet();
        int[] px = new int[f.pixels()];
        java.util.Arrays.fill(px, 0xff000000 | f.n() << 8 | layer);
        return px;
    }

    @Override
    public void cancel() {
        cancelled = true;
        completion.completeExceptionally(new CancelledException("cancelled"));
    }

    @Override
    public CompletableFuture<String> completion() {
        return completion;
    }
}
