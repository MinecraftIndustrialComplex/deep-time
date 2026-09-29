// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.CompletionException;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * Links the library once for this JVM (Gradle forks a JVM per test class) through
 * {@link NativeLoader#load}, then exercises the Phase 0 entry points of plan §6.1 and the
 * exception mapping of §6.3 across the real JNI boundary.
 */
class NativeLinkTest {

    @TempDir
    static Path gameDir;

    static NativeStatus status;

    @BeforeAll
    static void load() {
        status = NativeLoader.load(gameDir);
        assertTrue(status.isOk(), () -> status.describe());
    }

    @Test
    void linksOnceAndReportsVersionFormatAndSelfTest() {
        assertEquals(System.getProperty("deeptime.test.version"), status.version());
        assertEquals(System.getProperty("deeptime.test.formatVersion"), status.format());
        assertEquals(1_997_318_690_096_807_936L, NativeLoader.expectedSelfTest());
        assertEquals(NativeLoader.expectedSelfTest(), status.selfTest());
        assertEquals(status.selfTest(), NativeLoader.selfTest());
        assertTrue(status.target() != null && !status.target().isBlank());
        assertTrue(DeepTimeNative.version().startsWith(status.version() + "+"), DeepTimeNative.version());
        assertSame(status, NativeLoader.load(gameDir), "load() runs once per JVM");
        assertTrue(NativeLoader.status().isPresent());
        System.out.println("[junit] native " + status.describe());
    }

    @Test
    void phaseZeroEntryPoints() {
        String dummy = "{\"dummy\":{\"steps\":1,\"sleep_ms\":0}}";
        // P1-01: the estimate lists every stage (plan §6.1 "per-stage bytes and seconds").
        assertEquals("{\"peak_bytes\":0,\"stages\":[{\"bytes\":0,\"name\":\"dummy\",\"seconds\":0.0}]}",
                DeepTimeNative.estimate(dummy, ""));
        long id = DeepTimeNative.create(dummy, "", gameDir.resolve("entry").toString(), 1, 0);
        try {
            assertEquals(0, DeepTimeNative.keyframeCount(id));
            assertEquals("[]", DeepTimeNative.eventsSince(id, 0, 10));
            assertThrows(UnsupportedOperationException.class, () -> DeepTimeNative.keyframeInfo(id, 0));
            // P1-08: the column evaluator answers; a folder without world data is an I/O error.
            assertThrows(DeepTimeIoException.class,
                    () -> DeepTimeNative.sampleColumn(gameDir.resolve("nowhere").toString(), 0, 0));
            assertThrows(IllegalStateException.class, () -> DeepTimeNative.finalizeWrite(id), "not done yet");
            long[] p = new long[DeepTimeNative.PROGRESS_FIELDS];
            DeepTimeNative.progress(id, p);
            assertEquals(DeepTimeNative.RUNNING, p[0]);
            assertEquals(1, p[2], "stage count");
            assertThrows(IllegalArgumentException.class, () -> DeepTimeNative.progress(id, new long[3]));
            assertEquals(DeepTimeNative.DONE, advanceUntilSettled(id));
            String manifest = DeepTimeNative.finalizeWrite(id);
            assertTrue(manifest.contains("\"format_version\": \"" + status.format() + "\""), manifest);
            assertTrue(Files.isRegularFile(gameDir.resolve("entry").resolve("manifest.json")));
            assertTrue(DeepTimeNative.drainLog(id).contains("manifest written"));
        } finally {
            DeepTimeNative.destroy(id);
        }
        // Stale and forged ids raise IllegalStateException, never touch freed memory.
        assertThrows(IllegalStateException.class, () -> DeepTimeNative.advance(id, 1));
        assertThrows(IllegalStateException.class, () -> DeepTimeNative.destroy(id));
        assertThrows(IllegalStateException.class, () -> DeepTimeNative.cancel(0xdead_beefL));
    }

    /**
     * P1-01: a dummy preset with a toy world produces keyframes (RGBA8 through a direct buffer) and
     * events across JNI, and its world data holds the event segment and the keyframe index.
     */
    @Test
    void keyframesAndEventsCrossTheBoundary() {
        String world = "{\"dummy\":{\"steps\":8,\"sleep_ms\":0,\"stages\":[\"a\",\"b\"],\"world\":{"
                + "\"circumference\":1024,\"grid_r\":16,\"t_start_years\":-1.0e8,\"t_end_years\":0.0,"
                + "\"keyframe_every_years\":2.5e7,\"keyframe_size\":[64,32],\"event_every_steps\":2,\"sites\":2}}}";
        Path out = gameDir.resolve("world");
        long id = DeepTimeNative.create(world, "", out.toString(), 2, 0);
        try {
            assertThrows(IllegalArgumentException.class, () -> DeepTimeNative.keyframeInfo(id, 0),
                    "no keyframe produced yet");
            assertEquals(DeepTimeNative.DONE, advanceUntilSettled(id));
            int count = DeepTimeNative.keyframeCount(id);
            assertTrue(count > 0, "keyframes produced");
            String info = DeepTimeNative.keyframeInfo(id, count - 1);
            assertTrue(info.contains("\"w\":64") && info.contains("\"layers\":[\"elevation\"]"), info);
            ByteBuffer rgba = ByteBuffer.allocateDirect(64 * 32 * 4);
            DeepTimeNative.copyKeyframe(id, 0, 0, rgba);
            assertEquals((byte) 255, rgba.get(3), "opaque first pixel");
            assertThrows(IllegalArgumentException.class,
                    () -> DeepTimeNative.copyKeyframe(id, 0, 0, ByteBuffer.allocateDirect(16)), "short buffer");
            assertThrows(IllegalArgumentException.class,
                    () -> DeepTimeNative.copyKeyframe(id, 0, 0, ByteBuffer.allocate(64 * 32 * 4)), "heap buffer");
            assertThrows(IllegalArgumentException.class, () -> DeepTimeNative.keyframeInfo(id, count));
            String events = DeepTimeNative.eventsSince(id, 0, 1000);
            assertTrue(events.contains("\"kind\":\"deeptime:deposit_formation\""), events);
            assertEquals("[]", DeepTimeNative.eventsSince(id, 1000, 10));
            long[] p = new long[DeepTimeNative.PROGRESS_FIELDS];
            DeepTimeNative.progress(id, p);
            assertEquals(count, p[10], "keyframes ready");
            assertEquals(10, p[11], "events total: 8 in the chain + 2 deposit formations");
            DeepTimeNative.finalizeWrite(id);
            assertTrue(Files.isRegularFile(out.resolve("events/1-0.dtev")));
            assertTrue(Files.isRegularFile(out.resolve("keyframes/index.json")));
            assertTrue(Files.isRegularFile(out.resolve("sites/sites.dtsr")));
        } finally {
            DeepTimeNative.destroy(id);
        }
    }

    private static int advanceUntilSettled(long id) {
        for (int i = 0; i < 1000; i++) {
            int st = DeepTimeNative.advance(id, 50);
            if (st == DeepTimeNative.DONE || st == DeepTimeNative.CANCELLED)
                return st;
        }
        throw new AssertionError("session never settled");
    }

    @Test
    void errorsMapToTheirExceptions() {
        PresetInvalidException invalid = assertThrows(PresetInvalidException.class,
                () -> DeepTimeNative.create("{\"planet\":{}}", "", gameDir.toString(), 1, 0));
        assertEquals("$.planet", invalid.fieldPath());
        assertThrows(PresetInvalidException.class,
                () -> DeepTimeNative.create("{\"dummy\":{\"sleep_ms\":1}}", "", gameDir.toString(), 1, 0));

        long capped = DeepTimeNative.create("{\"dummy\":{\"steps\":2,\"sleep_ms\":0,\"reserve_bytes\":1000000}}", "",
                gameDir.resolve("cap").toString(), 1, 1);
        try {
            MemoryCapException cap = assertThrows(MemoryCapException.class, () -> DeepTimeNative.advance(capped, 50));
            assertTrue(cap.getMessage().contains("cap 1 bytes"), cap.getMessage());
        } finally {
            DeepTimeNative.destroy(capped);
        }

        long panicking = DeepTimeNative.create("{\"dummy\":{\"steps\":2,\"sleep_ms\":0,\"panic_at_step\":0}}", "",
                gameDir.resolve("panic").toString(), 1, 0);
        long healthy = DeepTimeNative.create("{\"dummy\":{\"steps\":2,\"sleep_ms\":0}}", "",
                gameDir.resolve("healthy").toString(), 1, 0);
        try {
            NativePanicException panic = assertThrows(NativePanicException.class,
                    () -> DeepTimeNative.advance(panicking, 50));
            assertTrue(panic.getMessage().contains("panic_at_step 0"), panic.getMessage());
            assertTrue(panic.getMessage().contains(".rs:"), "the location is part of the message: " + panic.getMessage());
            assertThrows(SessionPoisonedException.class, () -> DeepTimeNative.advance(panicking, 50));
            assertThrows(SessionPoisonedException.class, () -> DeepTimeNative.progress(panicking, new long[12]));
            assertTrue(DeepTimeNative.drainLog(panicking).contains("poisoned"));
            // Other sessions and the JVM carry on.
            assertEquals(DeepTimeNative.DONE, advanceUntilSettled(healthy));
            assertTrue(DeepTimeNative.drainLog(0).contains("native panic"));
        } finally {
            DeepTimeNative.destroy(panicking);
            DeepTimeNative.destroy(healthy);
        }
    }

    @Test
    void simSessionRunsAndCancels() throws Exception {
        Path out = gameDir.resolve("sim");
        try (SimSession s = SimSession.start("{\"dummy\":{\"steps\":5,\"sleep_ms\":5}}", "", out, 2, 0)) {
            String manifest = s.completion().get(30, TimeUnit.SECONDS);
            assertTrue(manifest.contains("\"layers\": []"), manifest);
            assertEquals(SimSession.Progress.FINALIZED, s.progress().state());
            assertEquals(1000, s.progress().totalPermille());
        }
        assertTrue(Files.isRegularFile(out.resolve("manifest.json")));

        Path cancelled = gameDir.resolve("sim-cancelled");
        try (SimSession s = SimSession.start("{\"dummy\":{\"steps\":100000,\"sleep_ms\":5}}", "", cancelled, 2, 0)) {
            Thread.sleep(50);
            s.cancel();
            CompletionException e = assertThrows(CompletionException.class, () -> s.completion().join());
            assertTrue(e.getCause() instanceof CancelledException, String.valueOf(e.getCause()));
            assertEquals(SimSession.Progress.CANCELLED, s.progress().state());
        }
        assertFalse(Files.exists(cancelled.resolve("manifest.json")));
    }

    /**
     * The JNI oracle (plan §6.1, §9.2) answers exactly what {@code deeptime parity} wrote for the
     * real test planet: the same evaluator behind both, the same JSON text.
     */
    @Test
    void sampleColumnIsTheParityFixture() throws Exception {
        Path world = Path.of(System.getProperty("deeptime.testWorlds")).resolve("planet4k");
        String text = Files.readString(world.resolve("parity/columns.json"));
        var cols = com.google.gson.JsonParser.parseString(text).getAsJsonObject().getAsJsonArray("columns");
        int checked = 0;
        for (var e : cols) {
            var o = e.getAsJsonObject();
            String json = DeepTimeNative.sampleColumn(world.toString(), o.get("x").getAsInt(), o.get("z").getAsInt());
            assertTrue(text.contains(json), "sampleColumn(" + o.get("x") + ", " + o.get("z") + ") = " + json);
            checked++;
        }
        assertEquals(500, checked);
        System.out.println("[junit] sampleColumnIsTheParityFixture columns=" + checked);
    }
}
