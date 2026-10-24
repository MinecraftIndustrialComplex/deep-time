// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.nativebridge.CancelledException;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeStatus;
import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.preset.ShareCode;
import com.minecraftindustrialcomplex.deeptime.server.Staging;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * P1-11 on the real stages: the Create World editor's models drive a real planet (every stage:
 * plates, climate, surface, volcanoes, impacts, structure, deposits, young, amplify, then the bake)
 * through the JNI the way the screens do, and the timelapse, the layers, the event ticker, the
 * review and staging work on what it produces. The planet is the tests' small real one
 * ({@code tests/presets/planet_world_4k.json}, C = 4,096), cut to 100 Myr. Also: a run started while
 * a cancelled one is still stopping waits for it (one planet in memory at a time), and leaving the
 * Create World screen stops a run nobody accepted.
 */
class RealRunWiringTest {

    @TempDir
    static Path gameDir;

    /** The editor on a small machine: 4 processors (3 simulation threads), 8 GiB free. */
    static final MachineInfo MACHINE = new MachineInfo(8L << 30, 16L << 30, 2L << 30, 4);

    static final List<String> REAL_STAGES = List.of("plates", "climate", "surface", "volcanoes", "impacts", "structure", "deposits",
            "young", "amplify");

    /** A placeholder whose steps take 1.5 s each, so a cancel holds its session that long (dummy pipeline). */
    static final String SLOW = "{\"schema\":0,\"name\":\"slow\",\"placeholder\":true,\"scale\":{\"circumference_blocks\":1024},"
            + "\"dummy\":{\"steps\":1000,\"sleep_ms\":1500,\"stages\":[\"dummy_drift\"],\"world\":{\"circumference\":1024,\"grid_r\":16,"
            + "\"t_start_years\":-1.0e8,\"t_end_years\":0.0,\"keyframe_every_years\":1.0e5,\"keyframe_size\":[64,32]}}}";

    @BeforeAll
    static void load() {
        NativeStatus status = NativeLoader.load(gameDir);
        assertTrue(status.isOk(), status::describe);
    }

    static EditorModel editor() {
        return new EditorModel(new EditorModel.Deps(json -> SimSession.estimate(json, ""), SessionRunHandle::launch, gameDir, MACHINE,
                Runnable::run, Runnable::run, NativeLoader::isLinked, List::of));
    }

    static String planet4k() throws IOException {
        return Files.readString(Path.of(System.getProperty("deeptime.repoRoot"), "crates/deeptime-core/tests/presets/planet_world_4k.json"));
    }

    /** Polls like the render thread until the run stops (or the time is up). */
    static void pollUntilStopped(TimelapseModel run, long seconds) throws InterruptedException {
        long end = System.nanoTime() + TimeUnit.SECONDS.toNanos(seconds);
        while (run.state() == TimelapseModel.State.RUNNING && System.nanoTime() < end) {
            run.poll();
            assertTrue(run.markers().size() <= TimelapseModel.MAX_MARKERS);
            assertTrue(SimSession.launchedLive() <= 1, "one launched session at a time");
            Thread.sleep(50);
        }
        run.poll();
        run.poll();
    }

    @Test
    void realPlanetThroughTheEditor() throws Exception {
        EditorModel m = editor();
        assertTrue(m.importShareCode(ShareCode.encode(planet4k())).isEmpty(), "the real preset imports");
        assertEquals(4096, m.circumference());
        m.setStart("scattered_continents");
        m.setLengthText("100");
        m.refreshEstimate();
        assertTrue(m.estimateError().isEmpty(), "the core accepts it: " + m.estimateError().orElse(""));
        assertTrue(m.canRun(), "Run enabled: " + m.problems());
        Estimate est = m.estimate().orElseThrow();
        assertEquals(REAL_STAGES, est.stages().stream().map(Estimate.Stage::name).toList(), "the real stages, in order");
        List<String> editable = new ArrayList<>();
        for (EditorFields.Field f : EditorFields.ALL)
            if (m.available(f))
                editable.add(f.id());
        assertTrue(editable.size() >= 20, "the real preset lights up the editor's fields: " + editable);
        // The effective preset is what runs: the timeline the editor set, the rest as imported.
        JsonObject eff = Presets.parseObject(m.effectivePresetJson());
        assertEquals(new Presets.Timeline("scattered_continents", 100.0), Presets.info(eff).timeline().orElseThrow());

        TimelapseModel run = m.startRun().orElseThrow(() -> new AssertionError("launch failed: " + m.problems()));
        Path staging = run.run().outDir();
        pollUntilStopped(run, 900);
        assertEquals(TimelapseModel.State.DONE, run.state(), run.failure().orElse(""));
        SimSession.Progress p = run.progress();
        assertEquals(p.keyframesReady(), run.frameCount(), "every keyframe reached the timelapse");
        assertEquals(p.eventsTotal(), run.events().size(), "every event reached the ticker");
        assertTrue(run.events().size() > 10, "a real planet logs events: " + run.events().size());
        assertTrue(p.nativeBytesPeak() <= m.capBytes(), "native peak " + p.nativeBytesPeak() + " under the cap");
        assertTrue(SimSession.launchedLive() == 0, "the session is gone");

        // Layers: the real ones, the satellite view first, grouped as in the workbench; all named.
        List<String> layers = run.layers();
        for (String l : List.of("satellite", "elevation", "plates", "climate/koppen", "structure/rock", "provinces", "deposits/sites",
                "amplify/elevation"))
            assertTrue(layers.contains(l), "layer " + l + " in " + layers);
        assertEquals("satellite", run.layer().orElseThrow());
        assertTrue(run.groups().containsAll(List.of("", "climate", "surface", "structure", "deposits")), "groups " + run.groups());
        JsonObject lang = LayerCatalogTest.lang();
        List<String> unnamed = new ArrayList<>();
        for (String l : layers)
            if (!lang.has(LayerCatalog.key(l)) && !lang.has(LayerCatalog.wordKey(l)))
                unnamed.add(l);
        Set<String> kinds = new HashSet<>();
        run.events().forEach(e -> kinds.add(e.kindPath()));
        for (String k : kinds)
            if (!lang.has("deeptime.event." + k))
                unnamed.add("event " + k);
        assertTrue(unnamed.isEmpty(), "no name for " + unnamed + " (add it to LayerCatalogTest.REAL_LAYERS and en_us.json)");

        // Every frame decodes; the satellite view is a picture, not a flat colour; markers stay bounded.
        for (int i = 0; i < run.frameCount(); i++) {
            run.scrubTo(i);
            assertTrue(run.displayed().isPresent(), "frame " + i + " decodes");
            assertTrue(run.markers().size() <= TimelapseModel.MAX_MARKERS);
        }
        run.scrubTo(run.frameCount() - 1);
        int[] sat = run.displayed().orElseThrow().abgr();
        assertTrue(distinct(sat) > 50, "the satellite view has " + distinct(sat) + " colours");
        // The deposits layers live in the deposits frame only: picking one goes there.
        run.scrubTo(0);
        run.setLayer("deposits/sites");
        assertTrue(run.currentHas("deposits/sites"), "moved to the deposits frame");
        assertEquals("deposits", run.group());
        assertEquals("deposits/sites", run.current().orElseThrow().layers().get(run.displayed().orElseThrow().layer()));

        // Review and Accept: the staged world data.
        ReviewModel review = m.openReview();
        assertTrue(review.ready() && review.error().isEmpty(), "review: " + review.error().orElse(""));
        assertEquals("world_data", review.map().orElseThrow().source());
        assertTrue(review.spawn().isPresent(), "a spawn marker");
        EditorModel.Accepted a = m.accept();
        assertEquals(run.stagingId(), a.stagingId());
        assertEquals(4096, a.circumference());
        assertTrue(Staging.complete(staging), "the staging folder holds a complete world");
        // Leaving Create World keeps the accepted run's folder (the server moves it).
        m.abandon();
        assertTrue(Files.isRegularFile(staging.resolve("manifest.json")));
    }

    /**
     * Cube plan G13 on a real run: the real 4k planet with the editor's projection set to the cube
     * (what a cube world type picks), cut to 50 Myr. The finished run has the cube's outline for the
     * globe (twelve edges, six of them the border); the review's final map is the net, 4 : 3, from the
     * world data, with the void transparent, the hints as a picture over it and the spawn on a face;
     * Accept binds a cube world of period 4,096.
     */
    @Test
    void aCubeRunThroughTheEditor() throws Exception {
        EditorModel m = editor();
        assertTrue(m.importShareCode(ShareCode.encode(planet4k())).isEmpty(), "the real preset imports");
        m.setChoice(EditorFields.byId("projection"), "cube");
        m.setStart("supercontinent");
        m.setLengthText("50");
        m.refreshEstimate();
        assertTrue(m.estimateError().isEmpty(), "the core accepts it: " + m.estimateError().orElse(""));
        TimelapseModel run = m.startRun().orElseThrow(() -> new AssertionError("launch failed: " + m.problems()));
        pollUntilStopped(run, 900);
        assertEquals(TimelapseModel.State.DONE, run.state(), run.failure().orElse(""));

        CubeOutline o = run.cubeOutline().orElseThrow(() -> new AssertionError("a finished cube run has its outline"));
        assertEquals(12, o.edges().size());
        assertEquals(6, o.edges().stream().filter(e -> e.kind() == CubeOutline.Kind.BORDER).count());
        assertEquals(4, o.corners().stream().filter(CubeOutline.Corner::reachable).count());

        ReviewModel review = m.openReview();
        assertTrue(review.ready() && review.error().isEmpty(), "review: " + review.error().orElse(""));
        NetView net = review.net().orElseThrow(() -> new AssertionError("the review of a cube world is the net"));
        ReviewModel.MapImage map = review.map().orElseThrow();
        assertEquals("world_data", map.source());
        assertEquals(NetView.W, map.w());
        assertEquals(NetView.H, map.h());
        int onFaces = 0, painted = 0;
        for (int y = 0; y < map.h(); y++)
            for (int x = 0; x < map.w(); x++) {
                int c = map.abgr()[y * map.w() + x];
                if (net.faceAt((x + 0.5) / map.w(), (y + 0.5) / map.h()) == null) {
                    assertEquals(0, c, "the void at " + x + "," + y + " is transparent");
                } else {
                    onFaces++;
                    painted += c != 0 ? 1 : 0;
                }
            }
        assertEquals(6 * 128 * 128, onFaces);
        assertEquals(onFaces, painted, "every face pixel has the world's elevation");
        ReviewModel.MapImage hints = review.hintsOverlay().orElseThrow();
        assertEquals(map.w(), hints.w());
        double[] sp = review.spawn().orElseThrow(() -> new AssertionError("a spawn marker"));
        double[] f = net.fractionOfLatLon(sp[0], sp[1]);
        assertTrue(net.faceAt(f[0], f[1]) != null, "the spawn is on a face: " + f[0] + "," + f[1]);

        EditorModel.Accepted a = m.accept();
        assertEquals("cube", a.projection());
        assertEquals(4096, a.worldCircumference());
        m.abandon();
    }

    /**
     * Cancel, restart and leaving the Create World screen. Cancel keeps Run disabled until the run
     * stopped, then deletes its folder. Leaving the screen cancels a run nobody accepted; a run a
     * new Create World screen starts meanwhile waits for it (a player's machine never holds two
     * planets); and a run cancelled while it waits never starts a session.
     */
    @Test
    void cancelRestartAndAbandon() throws Exception {
        EditorModel m = slowEditor();
        TimelapseModel first = m.startRun().orElseThrow();
        awaitStarted(first);
        m.cancelRun();
        assertFalse(m.canRun(), "Run stays disabled until the cancelled run stopped");
        assertTrue(first.cancelRequested(), "the setup page says the run is stopping");
        pollUntilStopped(first, 30);
        assertEquals(TimelapseModel.State.CANCELLED, first.state());
        awaitGone(first.run().outDir());
        assertTrue(m.canRun(), "Run again: " + m.problems());

        TimelapseModel second = m.startRun().orElseThrow();
        awaitStarted(second);
        // The Create World screen goes away (no Accept) while the run is in a 1.5 s step ...
        m.abandon();
        assertTrue(m.run().isEmpty());
        // ... and a new Create World screen starts a run at once: it waits for the old one.
        EditorModel m2 = slowEditor();
        TimelapseModel third = m2.startRun().orElseThrow();
        Thread.sleep(300);
        assertTrue(third.run().waitingForPrevious(), "the new run waits while the abandoned one finishes its step");
        assertEquals(1, SimSession.launchedLive());
        third.poll();
        assertTrue(third.progressLine(m2.capBytes()).startsWith("waiting for the previous run to stop"), third.progressLine(m2.capBytes()));
        ExecutionException e = assertThrows(ExecutionException.class, () -> second.run().completion().get(30, TimeUnit.SECONDS));
        assertTrue(e.getCause() instanceof CancelledException, String.valueOf(e.getCause()));
        awaitGone(second.run().outDir());
        awaitStarted(third);
        assertFalse(third.run().waitingForPrevious());
        assertEquals(1, SimSession.launchedLive());

        // A run cancelled while it still waits never starts a session.
        m2.abandon();
        EditorModel m3 = slowEditor();
        TimelapseModel fourth = m3.startRun().orElseThrow();
        Thread.sleep(300);
        assertTrue(fourth.run().waitingForPrevious());
        m3.cancelRun();
        for (TimelapseModel r : List.of(third, fourth)) {
            ExecutionException x = assertThrows(ExecutionException.class, () -> r.run().completion().get(30, TimeUnit.SECONDS));
            assertTrue(x.getCause() instanceof CancelledException, String.valueOf(x.getCause()));
            awaitGone(r.run().outDir());
        }
        assertEquals(0, fourth.progress().stageCount(), "the waiting run never created a session");
        long end = System.nanoTime() + TimeUnit.SECONDS.toNanos(10);
        while (SimSession.launchedLive() != 0 && System.nanoTime() < end)
            Thread.sleep(20);
        assertEquals(0, SimSession.launchedLive());
    }

    static EditorModel slowEditor() {
        EditorModel m = editor();
        assertTrue(m.importShareCode(ShareCode.encode(SLOW)).isEmpty());
        m.setStart("supercontinent");
        m.setLengthText("100");
        m.refreshEstimate();
        assertTrue(m.canRun(), "Run enabled: " + m.problems());
        return m;
    }

    static void awaitStarted(TimelapseModel run) throws InterruptedException {
        long end = System.nanoTime() + TimeUnit.SECONDS.toNanos(60);
        while (run.progress().stageCount() == 0 && System.nanoTime() < end) {
            run.poll();
            Thread.sleep(20);
        }
        assertTrue(run.progress().stageCount() > 0, "the run started");
    }

    /** Folder deletion runs when the completion completes, on the thread that completed it. */
    static void awaitGone(Path dir) throws InterruptedException {
        long end = System.nanoTime() + TimeUnit.SECONDS.toNanos(10);
        while (Files.exists(dir) && System.nanoTime() < end)
            Thread.sleep(20);
        assertFalse(Files.exists(dir), dir + " is deleted");
    }

    static int distinct(int[] px) {
        Set<Integer> s = new HashSet<>();
        for (int i = 0; i < px.length; i += 7)
            s.add(px[i]);
        return s.size();
    }
}
