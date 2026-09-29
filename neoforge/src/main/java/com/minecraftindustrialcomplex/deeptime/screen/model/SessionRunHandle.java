// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeFiles;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;

import java.nio.ByteBuffer;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CompletableFuture;

/**
 * A {@link SimSession} launched for the Create World screen (plan §6.2): created and stepped on its
 * {@code deeptime-sim} worker, which also collects keyframe infos and events. Pixels come across
 * JNI ({@code copyKeyframe} into a direct buffer) while the session lives and from
 * {@code keyframes/kf-<n>.dtk} ({@link KeyframeFiles}) once it is destroyed.
 */
public final class SessionRunHandle implements RunHandle {

    private final SimSession session;

    private SessionRunHandle(SimSession session) {
        this.session = session;
    }

    /** Launches a run; see {@link SimSession#launch}. */
    public static SessionRunHandle launch(String presetJson, Path outDir, int threads, long capBytes) {
        return new SessionRunHandle(SimSession.launch(presetJson, "", outDir, threads, capBytes, true));
    }

    @Override
    public Path outDir() {
        return session.outDir();
    }

    @Override
    public SimSession.Progress progress() {
        return session.progress();
    }

    @Override
    public List<KeyframeInfo> newKeyframes() {
        List<KeyframeInfo> out = new ArrayList<>();
        for (String json : session.newKeyframeInfos())
            out.add(KeyframeInfo.fromJson(json));
        return out;
    }

    @Override
    public List<GeoEvent> newEvents() {
        List<GeoEvent> out = new ArrayList<>();
        for (String batch : session.newEventBatches())
            out.addAll(GeoEvent.parseArray(batch));
        return out;
    }

    @Override
    public int[] pixels(KeyframeInfo f, int layer) {
        if (!session.isFinished()) {
            try {
                ByteBuffer buf = ByteBuffer.allocateDirect(f.pixels() * 4);
                session.copyKeyframe(f.n(), layer, buf);
                return KeyframeFiles.abgr(buf, f.pixels());
            } catch (IllegalStateException e) {
                // finished between the check and the copy: read it from disk
            }
        }
        return KeyframeFiles.readAbgr(session.outDir(), f.n(), layer);
    }

    @Override
    public void cancel() {
        session.cancel();
    }

    @Override
    public CompletableFuture<String> completion() {
        return session.completion();
    }
}
