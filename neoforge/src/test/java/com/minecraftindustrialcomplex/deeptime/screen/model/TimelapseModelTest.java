// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.Executor;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/** The run page's logic: keyframe ring, scrubbing, event cursor, layers, playback, progress (plan §7.12). */
class TimelapseModelTest {

    @TempDir
    Path dir;

    FakeRunHandle run;

    TimelapseModel model(Executor loader) {
        run = new FakeRunHandle(dir.resolve("staging"));
        return new TimelapseModel(run, "00000000-0000-0000-0000-000000000000", "{}", 16_384,
                List.of("dummy_drift", "dummy_erode"), loader, Runnable::run);
    }

    @Test
    void followsLiveUntilScrubbedBack() {
        TimelapseModel m = model(Runnable::run);
        m.poll();
        assertEquals(-1, m.index());
        assertTrue(m.displayed().isEmpty());
        for (int i = 0; i < 4; i++)
            run.frame(-4e8 + i * 1e8, 8, 4, "elevation");
        m.poll();
        assertEquals(3, m.index(), "live: the newest frame");
        assertTrue(m.following());
        assertEquals(3, m.displayed().orElseThrow().info().n());
        m.scrubTo(1);
        assertFalse(m.following());
        run.frame(0, 8, 4, "elevation");
        m.poll();
        assertEquals(1, m.index(), "scrubbed back: new frames do not move the view");
        m.scrubTo(99);
        assertEquals(4, m.index(), "clamped to the last frame, which follows again");
        assertTrue(m.following());
        m.step(-1);
        assertEquals(3, m.index());
        m.scrubTo(-5);
        assertEquals(0, m.index());
    }

    @Test
    void eventCursorAndFreshness() {
        TimelapseModel m = model(Runnable::run);
        run.frame(-3e8, 8, 4, "elevation");
        run.frame(-2e8, 8, 4, "elevation");
        run.frame(-1e8, 8, 4, "elevation");
        GeoEvent early = run.event(1, -2.5e8, 10, 20);
        GeoEvent mid = run.event(2, -1.5e8, -30, 200);
        GeoEvent global = run.event(3, -1.5e8, Double.NaN, Double.NaN);
        GeoEvent late = run.event(4, -0.5e8, 0, 0);
        m.poll();
        // Following a live run: every event so far.
        assertEquals(List.of(late, global, mid, early), m.visibleEvents(10));
        // Markers: point events of the last 24 Myr before the newest event while live (the others are 50+ Myr old).
        assertEquals(List.of(late), m.markers());
        m.scrubTo(1); // t = −200 Ma, previous frame −300 Ma
        assertEquals(List.of(early), m.visibleEvents(10));
        assertTrue(m.fresh(early));
        m.scrubTo(2); // the last frame of a running run: live again, everything shows
        assertTrue(m.following());
        assertEquals(4, m.visibleEvents(10).size());
        m.scrubTo(0);
        assertTrue(m.visibleEvents(10).isEmpty(), "nothing happened by −300 Ma");
        run.completion.complete("{}");
        m.poll();
        m.scrubTo(2); // done: the last frame is a time like any other (−100 Ma)
        assertFalse(m.following());
        assertEquals(List.of(global, mid, early), m.visibleEvents(10));
        assertEquals(List.of(global), m.visibleEvents(1), "at most max, newest first");
        assertTrue(m.fresh(mid) && m.fresh(global));
        assertFalse(m.fresh(early), "happened before the previous frame");
        assertTrue(m.markers().isEmpty(), "mid ended 50 Myr before −100 Ma: faded");
    }

    @Test
    void layersAndFallback() {
        TimelapseModel m = model(Runnable::run);
        run.frame(-2e8, 8, 4, "elevation");
        m.poll();
        assertEquals(List.of("elevation"), m.layers());
        assertEquals("elevation", m.layer().orElseThrow());
        run.frame(-1e8, 8, 4, "elevation", "plates");
        m.poll();
        assertEquals(List.of("elevation", "plates"), m.layers());
        m.setLayer("plates");
        assertEquals(1, m.displayed().orElseThrow().layer());
        assertEquals(0xff000000 | 1 << 8 | 1, m.displayed().get().abgr()[0], "frame 1, layer 1");
        m.scrubTo(0);
        assertEquals(0, m.displayed().orElseThrow().layer(), "a frame without the layer shows its first");
        m.setLayer("nonexistent");
        assertEquals("plates", m.layer().orElseThrow());
    }

    /**
     * A real planet's frames: 24 layers on every step of the coupled stages, then one frame each
     * from deposits (favourability, sites), young and amplify. The satellite view is the default;
     * the list shows one group at a time; picking a layer the frame lacks moves to a frame with it.
     */
    @Test
    void realLayersGroupsAndJumps() {
        TimelapseModel m = model(Runnable::run);
        String[] main = {"satellite", "elevation", "plates", "crust_age", "climate/koppen", "climate/t_mean_c",
                "surface/erosion_m_myr", "structure/rock", "provinces", "rivers"};
        for (int i = 0; i < 4; i++)
            run.frame(-4e8 + i * 1e8, 8, 4, main);
        run.frame(0, 8, 4, "deposits/favourability/porphyry_cu_mo_au", "deposits/sites", "input/elevation_m");
        run.frame(0, 8, 4, "satellite", "amplify/elevation", "rivers");
        m.poll();
        assertEquals("satellite", m.layer().orElseThrow(), "the satellite view first, as in the workbench");
        assertEquals(List.of("", "climate", "surface", "structure", "deposits", "input", "amplify"), m.groups());
        assertEquals("", m.group());
        assertEquals(List.of("satellite", "elevation", "plates", "crust_age", "provinces", "rivers"), m.groupLayers());
        m.cycleGroup(1);
        assertEquals("climate", m.group());
        assertEquals(List.of("climate/koppen", "climate/t_mean_c"), m.groupLayers());
        assertEquals("satellite", m.layer().orElseThrow(), "showing another group does not change the layer");
        m.cycleGroup(-2);
        assertEquals("amplify", m.group(), "wraps");

        m.scrubTo(2);
        assertTrue(m.currentHas("plates") && !m.currentHas("deposits/sites"));
        m.setLayer("deposits/sites");
        assertEquals(4, m.index(), "the only frame with the sites");
        assertEquals("deposits", m.group());
        assertEquals(1, m.displayed().orElseThrow().layer(), "layer 1 of the deposits frame");
        m.setLayer("climate/koppen");
        assertEquals(3, m.index(), "the nearest frame with Köppen");
        m.setLayer("plates");
        assertEquals(3, m.index(), "the frame has plates: no move");
        // A layer chosen sticks when new layers arrive.
        run.frame(1, 8, 4, "elevation");
        m.poll();
        assertEquals("plates", m.layer().orElseThrow());
    }

    /** Markers fade 24 Myr after an event ends and are capped, so 10,000 events do not all draw on every frame. */
    @Test
    void markersFadeAndAreCapped() {
        TimelapseModel m = model(Runnable::run);
        run.frame(-1e8, 8, 4, "satellite");
        run.frame(0, 8, 4, "satellite");
        long id = 0;
        // A long event (an orogeny phase) still on the map while it lasts and 24 Myr after it ended.
        run.pendingEvents.add(new GeoEvent(id++, -3e8, -1.1e8, "deeptime:orogeny_phase", "point", 10, 10, -1, 1));
        run.pendingEvents.add(new GeoEvent(id++, -3e8, -1.5e8, "deeptime:orogeny_phase", "point", 10, 20, -1, 1));
        for (int i = 0; i < 2 * TimelapseModel.MAX_MARKERS; i++) {
            double t = -20e6 * i / (2.0 * TimelapseModel.MAX_MARKERS);
            run.pendingEvents.add(new GeoEvent(id++, t, t, "deeptime:volcano_caldera", "point", 0, i % 360, -1, 1));
        }
        run.completion.complete("{}");
        m.poll();
        m.scrubTo(0); // t = −100 Ma
        List<GeoEvent> at100 = m.markers();
        assertTrue(at100.stream().anyMatch(e -> e.id() == 0), "ended 10 Myr earlier: still shown");
        assertTrue(at100.stream().noneMatch(e -> e.id() == 1), "ended 50 Myr earlier: faded");
        m.scrubTo(1); // t = 0
        List<GeoEvent> now = m.markers();
        assertEquals(TimelapseModel.MAX_MARKERS, now.size(), "capped");
        assertTrue(now.stream().allMatch(e -> e.tStart() >= -TimelapseModel.MARKER_FADE_YEARS), "only the last 24 Myr");
        for (int i = 1; i < now.size(); i++)
            assertTrue(now.get(i - 1).tStart() <= now.get(i).tStart(), "oldest first, newest drawn on top");
        assertEquals(0, now.get(now.size() - 1).tStart(), 1e-6, "the newest is kept");
    }

    /** Frames decode on the loader; until then the previous frame stays on screen. */
    @Test
    void ringLoadsOffThreadAndKeepsThePreviousFrame() {
        List<Runnable> queued = new ArrayList<>();
        TimelapseModel m = model(queued::add);
        run.frame(-2e8, 8, 4, "elevation");
        m.poll();
        assertTrue(m.displayed().isEmpty(), "not decoded yet");
        long v0 = m.displayVersion();
        queued.remove(0).run();
        assertEquals(0, m.displayed().orElseThrow().info().n());
        assertTrue(m.displayVersion() > v0);
        run.frame(-1e8, 8, 4, "elevation");
        m.poll();
        assertEquals(1, m.index());
        assertEquals(0, m.displayed().orElseThrow().info().n(), "the old frame until the new one is decoded");
        m.poll();
        assertEquals(1, queued.size(), "one request per frame, not per poll");
        queued.remove(0).run();
        assertEquals(1, m.displayed().orElseThrow().info().n());
        m.scrubTo(0);
        assertEquals(0, queued.size(), "frame 0 is still in the ring");
        assertEquals(0, m.displayed().orElseThrow().info().n());
    }

    @Test
    void ringIsBounded() {
        TimelapseModel m = model(Runnable::run);
        int frames = (int) (TimelapseModel.RING_BYTES / (1024 * 512 * 4)) + 8;
        for (int i = 0; i < frames; i++)
            run.frame(-1e9 + i * 1e6, 1024, 512, "elevation");
        m.poll();
        for (int i = 0; i < frames; i++)
            m.scrubTo(i);
        assertTrue(m.ringSize() <= TimelapseModel.RING_BYTES / (1024 * 512 * 4), "ring holds " + m.ringSize());
        int reads = run.pixelReads.get();
        m.scrubTo(frames - 1);
        assertEquals(reads, run.pixelReads.get(), "the newest frame is still cached");
        m.scrubTo(0);
        assertEquals(reads + 1, run.pixelReads.get(), "the oldest was dropped and is read again");
    }

    @Test
    void playbackAdvancesAndStopsAtTheEnd() {
        TimelapseModel m = model(Runnable::run);
        for (int i = 0; i < 3; i++)
            run.frame(-3e8 + i * 1e8, 8, 4, "elevation");
        run.completion.complete("{}");
        m.poll();
        assertEquals(TimelapseModel.State.DONE, m.state());
        m.togglePlay();
        assertTrue(m.playing());
        assertEquals(0, m.index(), "from the start when at the end");
        m.tick(TimelapseModel.PLAY_FRAME_MS - 1);
        assertEquals(0, m.index());
        m.tick(1);
        assertEquals(1, m.index());
        m.tick(10 * TimelapseModel.PLAY_FRAME_MS);
        assertEquals(2, m.index());
        assertFalse(m.playing());
    }

    @Test
    void cancelledAndFailedRunsDeleteTheirFolder() throws Exception {
        TimelapseModel m = model(Runnable::run);
        Files.createDirectories(run.outDir().resolve("keyframes"));
        m.cancel();
        m.poll();
        assertEquals(TimelapseModel.State.CANCELLED, m.state());
        assertFalse(Files.exists(run.outDir()));

        TimelapseModel f = model(Runnable::run);
        Files.createDirectories(run.outDir());
        run.completion.completeExceptionally(new IllegalStateException("boom"));
        assertFalse(Files.exists(run.outDir()), "deleted without a poll");
        f.poll();
        assertEquals(TimelapseModel.State.FAILED, f.state());
        assertTrue(f.failure().orElseThrow().contains("boom"));
        assertTrue(f.progressLine(4L << 30).startsWith("failed: IllegalStateException: boom"), f.progressLine(4L << 30));
    }

    @Test
    void progressLine() {
        TimelapseModel m = model(Runnable::run);
        assertTrue(m.progressLine(4L << 30).startsWith("starting"), m.progressLine(4L << 30));
        run.progress = new SimSession.Progress(SimSession.Progress.RUNNING, 1, 2, 370, 685, 312_000_000L, 72_000, 160_000,
                3L << 20, 5L << 20, 4, 11);
        m.poll();
        assertEquals("dummy_erode 37% (−312 Ma) · total 68% · 1m12s elapsed · ~2m40s left · native 3.0 MiB of 4.0 GiB",
                m.progressLine(4L << 30));
        assertEquals(0.685, m.fraction(), 1e-9);
        run.completion.complete("{}");
        m.poll();
        assertTrue(m.progressLine(4L << 30).startsWith("done · 0 keyframes · 0 events"), m.progressLine(4L << 30));
        assertEquals(1.0, m.fraction());
    }
}
