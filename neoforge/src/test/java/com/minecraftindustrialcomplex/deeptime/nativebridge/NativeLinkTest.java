// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertSame;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

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
        assertEquals("{\"peak_bytes\":0,\"stages\":[]}", DeepTimeNative.estimate(dummy, ""));
        long id = DeepTimeNative.create(dummy, "", gameDir.resolve("entry").toString(), 1, 0);
        try {
            assertEquals(0, DeepTimeNative.keyframeCount(id));
            assertEquals("[]", DeepTimeNative.eventsSince(id, 0, 10));
            assertThrows(UnsupportedOperationException.class, () -> DeepTimeNative.keyframeInfo(id, 0));
            assertThrows(UnsupportedOperationException.class, () -> DeepTimeNative.sampleColumn("x", 0, 0));
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
}
