// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

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
import java.util.Optional;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * Extraction (plan §6.5) in temporary game directories, the choice between the baseline and the
 * x86-64-v3 build (decisions log 2026-10-18), and one test per failure class through its forcing
 * property (plan §6.6). Only {@link #versionMismatch} links the Deep Time library (extraction links
 * the CPU probe, a library of its own); Gradle runs every test class in its own JVM
 * ({@code forkEvery = 1}).
 */
class NativeLoaderTest {

    @TempDir
    Path gameDir;

    @AfterEach
    void clearProperties() {
        System.clearProperty(NativeLoader.PROP_FORCE_FAIL);
        System.clearProperty(NativeLoader.PROP_DELETE_AFTER_WRITE);
        System.clearProperty(NativeLoader.PROP_PATH);
        System.clearProperty(NativeLoader.PROP_VARIANT);
        System.clearProperty(NativeLoader.PROP_TEST_CPU_FEATURES);
    }

    /** True when the dev jar has an x86-64-v3 build for this host (x86-64, no -PbaselineNativeOnly). */
    private static boolean jarHasV3() throws Exception {
        return NativeLoader.readManifest().libraries().stream().anyMatch(NativesManifest.Library::isV3);
    }

    /**
     * The first candidate. The dev jar carries the host's library only: its baseline build, and on
     * an x86-64 host its x86-64-v3 build too, which comes first when the CPU has the level.
     */
    private static ExtractResult.Extracted firstCandidate(ExtractResult r) {
        assertTrue(r.isOk(), () -> "extraction failed: " + r.failure().map(NativeStatus::describe).orElse(""));
        List<String> variants = r.candidates().stream().map(ExtractResult.Extracted::variant).toList();
        assertEquals(NativesManifest.X86_64_V3.equals(r.variant())
                ? List.of(NativesManifest.X86_64_V3, NativesManifest.BASELINE) : List.of(NativesManifest.BASELINE),
                variants, r.variantReason());
        return r.candidates().get(0);
    }

    // ------------------------------------------------------------------ the 4 extraction cases

    @Test
    void firstExtractionWritesAVerifiedFile() throws Exception {
        ExtractResult r = NativeLoader.extract(gameDir);
        ExtractResult.Extracted c = firstCandidate(r);
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
        ExtractResult.Extracted first = firstCandidate(NativeLoader.extract(gameDir));
        FileTime old = FileTime.fromMillis(1_000_000_000_000L);
        Files.setLastModifiedTime(first.file(), old);
        ExtractResult.Extracted second = firstCandidate(NativeLoader.extract(gameDir));
        assertFalse(second.written(), "a verified file must be reused");
        assertEquals(first.file(), second.file());
        assertEquals(old, Files.getLastModifiedTime(second.file()), "the reused file was rewritten");
    }

    @Test
    void corruptedFileIsReExtracted() throws Exception {
        ExtractResult.Extracted first = firstCandidate(NativeLoader.extract(gameDir));
        Files.write(first.file(), new byte[] {1, 2, 3, 4});
        ExtractResult.Extracted again = firstCandidate(NativeLoader.extract(gameDir));
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
                ExtractResult.Extracted c = firstCandidate(f.get(60, TimeUnit.SECONDS));
                if (c.written())
                    written++;
                if (file == null)
                    file = c.file();
                assertEquals(file, c.file());
            }
            assertEquals(1, written, "exactly one of the concurrent extractions writes the file");
            assertNotNull(file);
            assertEquals(firstCandidate(NativeLoader.extract(gameDir)).sha256(), NativeLoader.sha256(file));
        } finally {
            pool.shutdownNow();
        }
    }

    // ------------------------------------------------------------------ the build for this CPU

    @Test
    void forcedBaselineExtractsOnlyTheBaseline() throws Exception {
        System.setProperty(NativeLoader.PROP_VARIANT, "baseline");
        ExtractResult r = NativeLoader.extract(gameDir);
        assertEquals(NativesManifest.BASELINE, firstCandidate(r).variant());
        assertEquals(1, r.candidates().size());
        assertTrue(r.variantReason().startsWith(jarHasV3() ? "forced by -D" + NativeLoader.PROP_VARIANT
                + "=baseline" : "natives.json has no x86-64-v3 build"), r.variantReason());
    }

    @Test
    void aCpuWithoutTheLevelGetsTheBaselineAndAForcedV3IsRefused() throws Exception {
        assumeTrue(jarHasV3(), "no x86-64-v3 build in this jar");
        System.setProperty(NativeLoader.PROP_TEST_CPU_FEATURES, "0x" + Integer.toHexString(CpuProbe.ALL & ~(1 << 8)));
        ExtractResult r = NativeLoader.extract(gameDir);
        assertEquals(NativesManifest.BASELINE, r.variant());
        assertEquals("the CPU lacks avx2 of x86-64-v3", r.variantReason());
        assertEquals(List.of(NativesManifest.BASELINE),
                r.candidates().stream().map(ExtractResult.Extracted::variant).toList());

        System.setProperty(NativeLoader.PROP_VARIANT, "x86-64-v3");
        r = NativeLoader.extract(gameDir);
        assertEquals(NativesManifest.BASELINE, r.variant());
        assertTrue(r.variantReason().contains("refused: the CPU lacks avx2"), r.variantReason());
    }

    @Test
    void aCpuWithTheLevelGetsV3FirstAndTheBaselineAfter() throws Exception {
        assumeTrue(jarHasV3(), "no x86-64-v3 build in this jar");
        System.setProperty(NativeLoader.PROP_TEST_CPU_FEATURES, Integer.toString(CpuProbe.ALL));
        ExtractResult r = NativeLoader.extract(gameDir);
        assertEquals(NativesManifest.X86_64_V3, firstCandidate(r).variant());
        assertEquals("the CPU has every x86-64-v3 feature", r.variantReason());

        System.setProperty(NativeLoader.PROP_VARIANT, "v3");
        r = NativeLoader.extract(gameDir);
        assertEquals("forced by -D" + NativeLoader.PROP_VARIANT + "=v3; the CPU has every x86-64-v3 feature",
                r.variantReason());
    }

    @Test
    void theOverrideReadsThePropertyThenTheEnvironment() {
        assertEquals(Optional.empty(), NativeLoader.VariantOverride.parse(null, null));
        assertEquals(Optional.empty(), NativeLoader.VariantOverride.parse(" ", "auto"));
        assertEquals(NativesManifest.BASELINE, NativeLoader.VariantOverride.parse(null, "Baseline").orElseThrow().variant());
        assertEquals(NativeLoader.ENV_VARIANT + "=baseline",
                NativeLoader.VariantOverride.parse(null, "baseline").orElseThrow().source());
        assertEquals(NativesManifest.X86_64_V3, NativeLoader.VariantOverride.parse("v3", "baseline").orElseThrow().variant());
        assertEquals(Optional.empty(), NativeLoader.VariantOverride.parse("auto", "baseline"), "auto cancels the variable");
        assertEquals(Optional.empty(), NativeLoader.VariantOverride.parse("avx512", null), "unknown: no override");
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
