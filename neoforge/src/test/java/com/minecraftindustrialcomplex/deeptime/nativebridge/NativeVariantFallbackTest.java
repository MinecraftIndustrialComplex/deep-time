// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assumptions.assumeTrue;

import java.nio.file.Path;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * An x86-64-v3 build that cannot link falls back to the baseline build (decisions log 2026-10-18),
 * through {@code -Ddeeptime.natives.forceFail=v3_link}; the status keeps the CPU's answer and adds
 * the failure. Needs a machine the probe gives v3 (the test VM's EPYC, every CI x86-64 runner); its
 * own class, so its own JVM.
 */
class NativeVariantFallbackTest {

    @TempDir
    static Path gameDir;

    @AfterAll
    static void clear() {
        System.clearProperty(NativeLoader.PROP_FORCE_FAIL);
    }

    @Test
    void aV3BuildThatCannotLinkFallsBackToTheBaseline() {
        ExtractResult r = NativeLoader.extract(gameDir);
        assumeTrue(NativesManifest.X86_64_V3.equals(r.variant()), "this machine does not get v3: " + r.variantReason());

        System.setProperty(NativeLoader.PROP_FORCE_FAIL, "v3_link");
        NativeStatus s = NativeLoader.load(gameDir);
        assertTrue(s.isOk(), s::describe);
        assertEquals(NativesManifest.BASELINE, s.variant());
        assertTrue(DeepTimeNative.version().endsWith(";cpu=baseline"), DeepTimeNative.version());
        assertEquals("the CPU has every x86-64-v3 feature; the x86-64-v3 build failed to link ("
                + r.candidates().get(0).file() + ": cannot load the x86-64-v3 build [forced by -D"
                + NativeLoader.PROP_FORCE_FAIL + "=v3_link]), so the baseline build", s.variantReason());
        System.out.println("[junit] native " + s.describe());
    }
}
