// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.wrapengine;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngineGameTests.Site;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.List;

/**
 * Create across the wrap engine's turning seams (docs/design/wrap-engine.md §11): shaft and cog lines
 * and a pump-driven pipe line across a half-turn seam, shaft and cog lines across a quarter-turn seam,
 * each compared block by block with the same build on a site with no seam; and the connections the
 * seam refuses (belts, track, contraptions).
 *
 * <p>Namespace {@code deeptime_wrapengine}: run by {@code ./gametest.sh -PwithMic -PwrapEngine} (Create
 * is in the MIC set). Under plain {@code -PwrapEngine} Create is absent and each test passes with a log
 * line saying it was skipped. This class names no Create type; {@link CreateSeamChecks} does, and is
 * loaded only when Create is installed. Its glued sites join the engine tests' topology
 * ({@link WrapEngineGameTests#TOPOLOGY}, through {@link #topologies}).
 */
@GameTestHolder(DeepTime.MODID)
@PrefixGameTestTemplate(false)
public final class CreateSeamGameTests {

    private static final String NS = WrapEngineGameTests.NS;
    private static final String TEMPLATE = WrapEngineGameTests.TEMPLATE;

    private CreateSeamGameTests() {}

    // Sites 8.. of the engine tests' grid (x = 20480 + 512 i); controls at z 24576 and 26624. Built on
    // each call (no static fields), so WrapEngineGameTests' static initialiser can ask for them in any
    // class-loading order.
    static Site kinetics() {
        return WrapEngineGameTests.half("create-kinetics", 20480 + 512 * 8, 20480, 32);
    }

    static Site kineticsControl() {
        return unglued(WrapEngineGameTests.half("create-kinetics-control", 20480 + 512 * 8, 24576, 32));
    }

    static Site fluids() {
        return WrapEngineGameTests.half("create-fluids", 20480 + 512 * 9, 20480, 32);
    }

    static Site fluidsControl() {
        return unglued(WrapEngineGameTests.half("create-fluids-control", 20480 + 512 * 9, 24576, 32));
    }

    static Site quarter() {
        return WrapEngineGameTests.quarter("create-quarter", 20480 + 512 * 10, 22528, 48);
    }

    static Site quarterControl() {
        return unglued(WrapEngineGameTests.quarter("create-quarter-control", 20480 + 512 * 10, 26624, 48));
    }

    static Site refusals() {
        return WrapEngineGameTests.half("create-refusals", 20480 + 512 * 11, 20480, 32);
    }

    private static Site unglued(Site c) {
        return new Site(c.name(), c.kind(), c.ox(), c.oz(), c.uMin(), c.uMax(), c.local(), false, c.boxX0(), c.boxX1(), c.boxZ0(), c.boxZ1());
    }

    /** The glued sites' topologies, for the engine tests' union. */
    static List<WrapTopology> topologies() {
        return List.of(kinetics().local(), fluids().local(), quarter().local(), refusals().local());
    }

    private static boolean skip(GameTestHelper helper, String test) {
        if (ModList.get().isLoaded("create"))
            return false;
        WrapEngineGameTests.log("create %s SKIPPED: Create is not installed (run with -PwithMic -PwrapEngine)", test);
        helper.succeed();
        return true;
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void createKineticsAcrossHalfTurn(GameTestHelper helper) {
        if (!skip(helper, "kinetics half-turn"))
            CreateSeamChecks.kineticsAcrossHalfTurn(helper, kinetics(), kineticsControl());
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void createKineticsAcrossQuarterTurn(GameTestHelper helper) {
        if (!skip(helper, "kinetics quarter-turn"))
            CreateSeamChecks.kineticsAcrossQuarterTurn(helper, quarter(), quarterControl());
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 1000)
    public static void createFluidsAcrossHalfTurn(GameTestHelper helper) {
        if (!skip(helper, "fluids half-turn"))
            CreateSeamChecks.fluidsAcrossHalfTurn(helper, fluids(), fluidsControl());
    }

    @GameTest(templateNamespace = NS, template = TEMPLATE, timeoutTicks = 400)
    public static void createRefusalsAtSeam(GameTestHelper helper) {
        if (!skip(helper, "refusals"))
            CreateSeamChecks.refusalsAtSeam(helper, refusals());
    }
}
