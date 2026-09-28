// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.FileTime;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CyclicBarrier;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * Extraction (plan §6.5) in temporary game directories, and one test per failure class through its
 * forcing property (plan §6.6). Only {@link #versionMismatch} links the library; Gradle runs every
 * test class in its own JVM ({@code forkEvery = 1}).
 */
class NativeLoaderTest {

    @TempDir
    Path gameDir;

    @AfterEach
    void clearProperties() {
        System.clearProperty(NativeLoader.PROP_FORCE_FAIL);
        System.clearProperty(NativeLoader.PROP_DELETE_AFTER_WRITE);
        System.clearProperty(NativeLoader.PROP_PATH);
    }

    private static ExtractResult.Extracted only(ExtractResult r) {
        assertTrue(r.isOk(), () -> "extraction failed: " + r.failure().map(NativeStatus::describe).orElse(""));
        assertEquals(1, r.candidates().size(), "the dev jar carries only the host library");
        return r.candidates().get(0);
    }

    // ------------------------------------------------------------------ the 4 extraction cases

    @Test
    void firstExtractionWritesAVerifiedFile() throws Exception {
        ExtractResult r = NativeLoader.extract(gameDir);
        ExtractResult.Extracted c = only(r);
        assertTrue(c.written(), "a fresh game directory must be written");
        assertTrue(Files.isRegularFile(c.file()));
        assertEquals(c.sha256(), NativeLoader.sha256(c.file()));
        Path dir = c.file().getParent();
        assertEquals(NativeLoader.nativesRoot(gameDir), dir.getParent());
        assertEquals(System.getProperty("deeptime.test.version", r.expectedVersion()) + "-" + c.sha256().substring(0, 12),
                dir.getFileName().toString());
        assertEquals(r.expectedFormat(), System.getProperty("deeptime.test.formatVersion", r.expectedFormat()));
        assertTrue(Files.exists(dir.resolve(".lock")));
        assertFalse(Files.exists(dir.resolve(c.file().getFileName() + ".tmp")), "no temporary file is left");
        assertTrue(PlatformKey.detect().candidateKeys().contains(c.platform()), c.platform());
    }

    @Test
    void secondExtractionReusesWithoutRewriting() throws Exception {
        ExtractResult.Extracted first = only(NativeLoader.extract(gameDir));
        FileTime old = FileTime.fromMillis(1_000_000_000_000L);
        Files.setLastModifiedTime(first.file(), old);
        ExtractResult.Extracted second = only(NativeLoader.extract(gameDir));
        assertFalse(second.written(), "a verified file must be reused");
        assertEquals(first.file(), second.file());
        assertEquals(old, Files.getLastModifiedTime(second.file()), "the reused file was rewritten");
    }

    @Test
    void corruptedFileIsReExtracted() throws Exception {
        ExtractResult.Extracted first = only(NativeLoader.extract(gameDir));
        Files.write(first.file(), new byte[] {1, 2, 3, 4});
        ExtractResult.Extracted again = only(NativeLoader.extract(gameDir));
        assertTrue(again.written(), "a corrupted file must be rewritten");
        assertEquals(again.sha256(), NativeLoader.sha256(again.file()));
    }

    /** Two game instances sharing a directory: the lock serialises them, exactly one writes. */
    @Test
    void concurrentExtractionsShareTheLockAndWriteOnce() throws Exception {
        int n = 4;
        CyclicBarrier barrier = new CyclicBarrier(n);
        ExecutorService pool = Executors.newFixedThreadPool(n);
        try {
            List<Future<ExtractResult>> futures = new ArrayList<>();
            for (int i = 0; i < n; i++)
                futures.add(pool.submit(() -> {
                    barrier.await(10, TimeUnit.SECONDS);
                    return NativeLoader.extract(gameDir);
                }));
            int written = 0;
            Path file = null;
            for (Future<ExtractResult> f : futures) {
                ExtractResult.Extracted c = only(f.get(60, TimeUnit.SECONDS));
                if (c.written())
                    written++;
                if (file == null)
                    file = c.file();
                assertEquals(file, c.file());
            }
            assertEquals(1, written, "exactly one of the concurrent extractions writes the file");
            assertNotNull(file);
            assertEquals(only(NativeLoader.extract(gameDir)).sha256(), NativeLoader.sha256(file));
        } finally {
            pool.shutdownNow();
        }
    }

    // ------------------------------------------------------------------ the 5 failure classes

    private NativeStatus loadWith(String property, String value) {
        System.setProperty(property, value);
        NativeStatus s = NativeLoader.link(NativeLoader.extract(gameDir));
        assertFalse(s.isOk(), "expected a failure, got " + s);
        assertFalse(s.guidance().isBlank(), "every failure carries guidance");
        assertTrue(s.describe().contains(s.failureClass().name()));
        assertTrue(s.summary().equals("status=FAILED class=" + s.failureClass()));
        return s;
    }

    @Test
    void unsupportedPlatform() {
        NativeStatus s = loadWith(NativeLoader.PROP_FORCE_FAIL, "unsupported");
        assertEquals(FailureClass.UNSUPPORTED_PLATFORM, s.failureClass());
        assertTrue(s.guidance().contains("deeptime run"), s.guidance());
    }

    @Test
    void libcMismatch() {
        NativeStatus s = loadWith(NativeLoader.PROP_FORCE_FAIL, "libc_mismatch");
        assertEquals(FailureClass.LIBC_MISMATCH, s.failureClass());
        assertTrue(s.cause() instanceof UnsatisfiedLinkError);
        assertTrue(s.cause().getMessage().contains("GLIBC_2.99"), s.cause().getMessage());
        assertTrue(s.guidance().contains("2.17"), s.guidance());
    }

    @Test
    void antivirusSuspected() {
        NativeStatus s = loadWith(NativeLoader.PROP_DELETE_AFTER_WRITE, "true");
        assertEquals(FailureClass.ANTIVIRUS_SUSPECTED, s.failureClass());
        assertNotNull(s.attemptedPath());
        assertFalse(Files.exists(s.attemptedPath()));
        assertTrue(s.guidance().contains("exclusion"), s.guidance());
    }

    @Test
    void ioError() throws Exception {
        NativeStatus s = loadWith(NativeLoader.PROP_FORCE_FAIL, "io");
        assertEquals(FailureClass.IO_ERROR, s.failureClass());
        assertNotNull(s.cause());
        // The forced read-only directory is writable again afterwards.
        Path dir = s.attemptedPath().getParent();
        assertTrue(Files.isWritable(dir), dir + " was left read-only");
    }

    @Test
    void versionMismatch() {
        NativeStatus s = loadWith(NativeLoader.PROP_FORCE_FAIL, "version_mismatch");
        assertEquals(FailureClass.VERSION_MISMATCH, s.failureClass());
        assertTrue(s.cause().getMessage().contains("0.0.0-forced-mismatch"), s.cause().getMessage());
        assertTrue(NativeLoader.isLinked(), "the version check needs the library linked");
    }
}
