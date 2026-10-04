// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.create.wrap;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.wrapspike.WrapSpikeGameTests;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/**
 * Create across the wrap engine's turning seams (docs/design/wrap-engine.md, "Create across turning
 * seams"): shaft and cog lines and a pump-driven pipe line across a half-turn seam, shaft and cog lines
 * across a quarter-turn seam, each compared block by block with the same build on a site with no seam;
 * and the connections the seam refuses (belts, track, contraptions).
 *
 * <p>Namespace {@code deeptime_wrapspike}: run by {@code ./gametest.sh -PwithMic -PwrapSpike} (Create is in
 * the MIC set). Under plain {@code -PwrapSpike} Create is absent and each test passes with a log line
 * saying it was skipped. This class names no Create type; {@link CreateSeamChecks} does, and is loaded
 * only when Create is installed.
 */
@GameTestHolder(DeepTime.MODID)
@PrefixGameTestTemplate(false)
public final class CreateSeamGameTests {

    private static final String NS = WrapSpikeGameTests.NS;
    private static final String TEMPLATE = WrapSpikeGameTests.TEMPLATE;

    private CreateSeamGameTests() {}

    private static boolean skip(GameTestHelper helper, String test) {
        if (ModList.get().isLoaded("create"))
            return false;
        WrapSpikeGameTests.log("create %s SKIPPED: Create is not installed (run with -PwithMic -PwrapSpike)", test);
        helper.succeed();
        return true;
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void createKineticsAcrossHalfTurn(GameTestHelper helper) {
        if (!skip(helper, "kinetics half-turn"))
            CreateSeamChecks.kineticsAcrossHalfTurn(helper);
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 300)
    public static void createKineticsAcrossQuarterTurn(GameTestHelper helper) {
        if (!skip(helper, "kinetics quarter-turn"))
            CreateSeamChecks.kineticsAcrossQuarterTurn(helper);
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 900)
    public static void createFluidsAcrossHalfTurn(GameTestHelper helper) {
        if (!skip(helper, "fluids half-turn"))
            CreateSeamChecks.fluidsAcrossHalfTurn(helper);
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 200)
    public static void createRefusalsAtSeam(GameTestHelper helper) {
        if (!skip(helper, "refusals"))
            CreateSeamChecks.refusalsAtSeam(helper);
    }
}
