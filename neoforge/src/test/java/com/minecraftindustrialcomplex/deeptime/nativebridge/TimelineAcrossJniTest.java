// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.minecraftindustrialcomplex.deeptime.screen.model.GeoEvent;
import com.minecraftindustrialcomplex.deeptime.screen.model.SessionRunHandle;
import com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeFiles;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;
import com.minecraftindustrialcomplex.deeptime.worlddata.SiteRecords;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * P1-11's timeline plumbing across the real JNI boundary: keyframes read through
 * {@code copyKeyframe} equal the ones the pure-Java {@link KeyframeFiles} reads from disk, a
 * launched {@link SimSession} collects every keyframe and event for the timelapse, and the site
 * records read back with {@link SiteRecords}.
 */
class TimelineAcrossJniTest {

    @TempDir
    static Path gameDir;

    static final String WORLD = "{\"dummy\":{\"steps\":8,\"sleep_ms\":0,\"stages\":[\"a\",\"b\"],\"world\":{"
            + "\"circumference\":1024,\"grid_r\":16,\"t_start_years\":-1.0e8,\"t_end_years\":0.0,"
            + "\"keyframe_every_years\":2.5e7,\"keyframe_size\":[64,32],\"event_every_steps\":2,\"sites\":2}}}";

    @BeforeAll
    static void load() {
        NativeStatus status = NativeLoader.load(gameDir);
        assertTrue(status.isOk(), status::describe);
    }

    @Test
    void jniPixelsEqualDiskPixels() {
        Path out = gameDir.resolve("parity");
        long id = DeepTimeNative.create(WORLD, "", out.toString(), 2, 0);
        try {
            int st;
            do {
                st = DeepTimeNative.advance(id, 50);
            } while (st != DeepTimeNative.DONE && st != DeepTimeNative.CANCELLED);
            int count = DeepTimeNative.keyframeCount(id);
            List<KeyframeFiles.Entry> index = KeyframeFiles.index(out);
            assertEquals(count, index.size(), "every frame is on disk while the session lives");
            for (int n = 0; n < count; n++) {
                KeyframeInfo info = KeyframeInfo.fromJson(DeepTimeNative.keyframeInfo(id, n));
                assertEquals(info, index.get(n).info());
                assertEquals(info, KeyframeFiles.readInfo(out, n));
                ByteBuffer buf = ByteBuffer.allocateDirect(info.pixels() * 4);
                DeepTimeNative.copyKeyframe(id, n, 0, buf);
                assertArrayEquals(KeyframeFiles.abgr(buf, info.pixels()), KeyframeFiles.readAbgr(out, n, 0), "frame " + n);
            }
            String events = DeepTimeNative.eventsSince(id, 0, 1000);
            List<GeoEvent> parsed = GeoEvent.parseArray(events);
            assertEquals(10, parsed.size(), "8 chain events + 2 deposit formations");
            assertTrue(parsed.stream().filter(e -> e.kind().equals("deeptime:deposit_formation")).allMatch(e -> e.site() >= 0));
            assertTrue(parsed.stream().filter(e -> !e.kind().equals("deeptime:deposit_formation")).allMatch(GeoEvent::hasPoint));
            DeepTimeNative.finalizeWrite(id);
            List<SiteRecords.Site> sites = SiteRecords.read(out);
            assertEquals(2, sites.size());
            for (SiteRecords.Site s : sites) {
                assertTrue(s.lat() >= -90 && s.lat() <= 90 && s.lon() >= 0 && s.lon() < 360, s.toString());
                assertTrue(s.x() >= -512 && s.x() < 512, s.toString());
            }
        } finally {
            DeepTimeNative.destroy(id);
        }
    }

    /** The Create World screen's path: launch on the worker, collect, poll, read pixels after the session is gone. */
    @Test
    void launchedSessionFeedsTheTimelapse() throws Exception {
        Path out = gameDir.resolve("launched");
        Files.createDirectories(out);
        SessionRunHandle h = SessionRunHandle.launch(WORLD, out, 2, 0);
        TimelapseModel m = new TimelapseModel(h, "00000000-0000-0000-0000-000000000001", WORLD, 1024, List.of("a", "b"),
                Runnable::run, Runnable::run);
        h.completion().get(60, TimeUnit.SECONDS);
        m.poll();
        m.poll(); // anything the worker collected in its last step
        assertEquals(TimelapseModel.State.DONE, m.state());
        SimSession.Progress p = m.progress();
        assertEquals(SimSession.Progress.FINALIZED, p.state());
        assertEquals(p.keyframesReady(), m.frameCount());
        assertEquals(KeyframeFiles.index(out).size(), m.frameCount());
        assertEquals(p.eventsTotal(), m.events().size());
        assertTrue(m.frameCount() >= 4, "keyframes every 25 Myr over 100 Myr");
        assertEquals(List.of("elevation"), m.layers());
        // The session is destroyed: pixels come from disk.
        m.scrubTo(0);
        int[] px = m.displayed().orElseThrow().abgr();
        assertArrayEquals(KeyframeFiles.readAbgr(out, 0, 0), px);
        assertTrue(Files.isRegularFile(out.resolve("manifest.json")));

        // A preset the core rejects fails the completion (create runs on the worker).
        SessionRunHandle bad = SessionRunHandle.launch("{\"planet\":{}}", gameDir.resolve("bad"), 1, 0);
        var e = org.junit.jupiter.api.Assertions.assertThrows(java.util.concurrent.ExecutionException.class,
                () -> bad.completion().get(30, TimeUnit.SECONDS));
        assertTrue(e.getCause() instanceof PresetInvalidException, String.valueOf(e.getCause()));
        assertTrue(bad.newKeyframes().isEmpty());

        // Cancel before or during the run.
        SessionRunHandle slow = SessionRunHandle.launch("{\"dummy\":{\"steps\":100000,\"sleep_ms\":5}}", gameDir.resolve("slow"), 1, 0);
        slow.cancel();
        var c = org.junit.jupiter.api.Assertions.assertThrows(java.util.concurrent.ExecutionException.class,
                () -> slow.completion().get(30, TimeUnit.SECONDS));
        assertTrue(c.getCause() instanceof CancelledException, String.valueOf(c.getCause()));
        assertFalse(Files.exists(gameDir.resolve("slow/manifest.json")));
    }
}
