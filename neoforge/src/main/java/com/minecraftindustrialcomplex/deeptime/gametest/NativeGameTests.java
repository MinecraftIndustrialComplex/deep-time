// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.worlddata.FormatVersion;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.nativebridge.CancelledException;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeStatus;
import com.minecraftindustrialcomplex.deeptime.nativebridge.PresetInvalidException;
import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.CompletionException;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/** The native library inside the game: linked by the mod constructor, self-tested, and a dummy session run (P0-07). */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class NativeGameTests {

    private NativeGameTests() {}

    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void loadsAndSelfTests(GameTestHelper helper) {
        NativeStatus status = NativeLoader.status().orElse(null);
        if (status == null) {
            helper.fail("NativeLoader.load never ran (the mod constructor should call it)");
            return;
        }
        if (!status.isOk()) {
            helper.fail("natives did not load: " + status.describe());
            return;
        }
        long again = NativeLoader.selfTest();
        long expected = NativeLoader.expectedSelfTest();
        DeepTime.LOGGER.info("[gametest] native version={} format={} target={} selftest={}", status.version(),
                status.format(), status.target(), Long.toUnsignedString(again));
        if (again != expected || status.selfTest() != expected)
            helper.fail("self-test " + Long.toUnsignedString(again) + " (at load " + Long.toUnsignedString(status.selfTest())
                    + ") != Java's " + Long.toUnsignedString(expected));
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 400)
    public static void dummySessionRunsAndCancels(GameTestHelper helper) {
        if (!NativeLoader.isLinked()) {
            helper.fail("natives not linked: " + NativeLoader.status().map(NativeStatus::summary).orElse("never loaded"));
            return;
        }
        Path base = FMLPaths.GAMEDIR.get().resolve("deeptime").resolve("gametest")
                .resolve("dummy-" + System.nanoTime());
        Path doneDir = base.resolve("done");
        Path cancelDir = base.resolve("cancelled");

        try {
            SimSession.start("{\"planet\":{}}", "", base.resolve("invalid"), 1, 0).close();
            helper.fail("a non-dummy preset was accepted");
            return;
        } catch (PresetInvalidException expectedInvalid) {
            DeepTime.LOGGER.info("[gametest] dummySessionRunsAndCancels invalid preset -> {}", expectedInvalid.getMessage());
        }

        SimSession done = SimSession.start("{\"dummy\":{\"steps\":5,\"sleep_ms\":10}}", "", doneDir, 2, 0);
        SimSession cancelled = SimSession.start("{\"dummy\":{\"steps\":100000,\"sleep_ms\":10}}", "", cancelDir, 2, 0);
        helper.runAfterDelay(5, cancelled::cancel);

        helper.succeedWhen(() -> {
            helper.assertTrue(done.completion().isDone() && cancelled.completion().isDone(),
                    "sessions still running");
            String manifest;
            try {
                manifest = done.completion().join();
            } catch (CompletionException e) {
                throw new IllegalStateException("the dummy run failed: " + e.getCause(), e);
            }
            helper.assertTrue(manifest.contains("\"format_version\": \"" + FormatVersion.CURRENT + "\""),
                    "manifest lacks format_version " + FormatVersion.CURRENT + ": " + manifest);
            helper.assertTrue(Files.isRegularFile(doneDir.resolve("manifest.json")), "no manifest.json in " + doneDir);
            helper.assertTrue(done.progress().state() == SimSession.Progress.FINALIZED,
                    "finished session state " + done.progress().state());

            Throwable why = null;
            try {
                cancelled.completion().join();
            } catch (CompletionException e) {
                why = e.getCause();
            }
            helper.assertTrue(why instanceof CancelledException, "cancelled session completed with " + why);
            helper.assertTrue(!Files.exists(cancelDir.resolve("manifest.json")), "a cancelled run wrote a manifest");
            DeepTime.LOGGER.info("[gametest] dummySessionRunsAndCancels done: steps=5 manifest ok; cancelled after {} permille",
                    cancelled.progress().totalPermille());
        });
    }
}
