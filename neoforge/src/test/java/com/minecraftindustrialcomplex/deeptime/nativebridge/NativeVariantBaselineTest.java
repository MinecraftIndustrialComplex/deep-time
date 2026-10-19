// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.nativebridge;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Path;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * {@code -Ddeeptime.natives.variant=baseline} (decisions log 2026-10-18): the baseline build links
 * whatever the CPU has, it reports {@code cpu=baseline}, and the status says it was forced. Its own
 * class, so its own JVM: the library links once per JVM.
 */
class NativeVariantBaselineTest {

    @TempDir
    static Path gameDir;

    @AfterAll
    static void clear() {
        System.clearProperty(NativeLoader.PROP_VARIANT);
    }

    @Test
    void theForcedBaselineLinks() throws Exception {
        System.setProperty(NativeLoader.PROP_VARIANT, "baseline");
        NativeStatus s = NativeLoader.load(gameDir);
        assertTrue(s.isOk(), s::describe);
        assertEquals(NativesManifest.BASELINE, s.variant());
        assertTrue(DeepTimeNative.version().endsWith(";cpu=baseline"), DeepTimeNative.version());
        boolean v3InJar = NativeLoader.readManifest().libraries().stream().anyMatch(NativesManifest.Library::isV3);
        assertEquals(v3InJar ? "forced by -D" + NativeLoader.PROP_VARIANT + "=baseline"
                : "natives.json has no x86-64-v3 build for " + s.platformKey() + " (-D" + NativeLoader.PROP_VARIANT
                        + "=baseline has nothing to choose)", s.variantReason());
        assertEquals(NativeLoader.expectedSelfTest(), s.selfTest());
        System.out.println("[junit] native " + s.describe());
    }
}
