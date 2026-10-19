// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * The build this machine gets (decisions log 2026-10-18), linked for real: the CPU probe's answer
 * agrees with the kernel's {@code /proc/cpuinfo} on Linux, the dev jar carries both builds and the
 * probe on an x86-64 host, and {@link NativeLoader#load} links the chosen build, which reports the
 * same level in its version string, and says why. The forced baseline and the v3 fallback have
 * their own classes ({@link NativeVariantBaselineTest}, {@link NativeVariantFallbackTest}): each
 * links once in its own JVM ({@code forkEvery = 1}).
 */
class NativeVariantTest {

    @TempDir
    static Path gameDir;

    /** {@code /proc/cpuinfo}'s flag for each of {@link CpuProbe#FEATURES}, in order. */
    private static final List<String> CPUINFO_FLAGS = List.of("pni", "ssse3", "sse4_1", "sse4_2", "popcnt", "cx16",
            "lahf_lm", "avx", "avx2", "bmi1", "bmi2", "f16c", "fma", "abm", "movbe", "xsave");

    /** Whether the kernel lists every x86-64-v3 flag, when this is Linux on x86-64. */
    static Optional<Boolean> cpuinfoHasV3() throws Exception {
        Path cpuinfo = Path.of("/proc/cpuinfo");
        if (!Files.isReadable(cpuinfo) || !PlatformKey.detect().arch().equals("x86_64"))
            return Optional.empty();
        Optional<String> flags = Files.readAllLines(cpuinfo).stream().filter(l -> l.startsWith("flags")).findFirst();
        if (flags.isEmpty())
            return Optional.empty();
        Set<String> have = Arrays.stream(flags.get().substring(flags.get().indexOf(':') + 1).trim().split("\\s+"))
                .collect(Collectors.toSet());
        // The kernel drops avx and everything built on it when it does not enable the YMM state,
        // which the probe's XGETBV check sees the same way.
        return Optional.of(have.containsAll(CPUINFO_FLAGS));
    }

    @Test
    void linksTheBuildTheCpuCanRunAndSaysWhy() throws Exception {
        assertEquals(CpuProbe.FEATURES.length, CPUINFO_FLAGS.size());
        PlatformKey platform = PlatformKey.detect();
        NativesManifest manifest = NativeLoader.readManifest();
        assertTrue(manifest.library(platform.key(), NativesManifest.BASELINE).isPresent(), manifest.toString());
        boolean v3InJar = Boolean.parseBoolean(System.getProperty("deeptime.test.v3Native", "false"));
        assertEquals(v3InJar, manifest.library(platform.key(), NativesManifest.X86_64_V3).isPresent(),
                "the Gradle build packs the x86-64-v3 build on an x86-64 host");
        assertEquals(v3InJar, manifest.probe(platform.key()).isPresent(), "and the CPU probe beside it");

        ExtractResult r = NativeLoader.extract(gameDir);
        assertTrue(r.isOk(), () -> r.failure().orElseThrow().describe());
        Optional<Boolean> kernel = cpuinfoHasV3();
        if (v3InJar && kernel.isPresent())
            assertEquals(kernel.get(), NativesManifest.X86_64_V3.equals(r.variant()),
                    "the probe and /proc/cpuinfo disagree: " + r.variantReason());

        NativeStatus s = NativeLoader.load(gameDir);
        assertTrue(s.isOk(), s::describe);
        assertEquals(r.variant(), s.variant());
        assertEquals(r.variantReason(), s.variantReason());
        assertTrue(DeepTimeNative.version().endsWith(";cpu=" + s.variant()), DeepTimeNative.version());
        assertTrue(s.summary().endsWith(" build=" + s.variant()), s.summary());
        assertTrue(s.describe().endsWith("(" + s.variant() + ": " + s.variantReason() + ")"), s.describe());
        if (v3InJar)
            assertTrue(Files.isRegularFile(r.probeFile()), "the probe sits under the game directory");
        assertEquals(r.candidates().get(0).file(), s.attemptedPath(), "the first candidate links");
        System.out.println("[junit] native " + s.describe());
    }
}
