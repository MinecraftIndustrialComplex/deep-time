// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

/** The mod graph boots: Deep Time and Toroidal World are both loaded (P0-03). */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class BootGameTests {

    private BootGameTests() {}

    @GameTest(template = GameTestSupport.TEMPLATE)
    public static void toroidalWorldLoaded(GameTestHelper helper) {
        boolean dt = ModList.get().isLoaded(DeepTime.MODID);
        boolean tw = ModList.get().isLoaded("toroidal_world");
        DeepTime.LOGGER.info("[gametest] toroidalWorldLoaded deeptime={} toroidal_world={}", dt, tw);
        if (!dt || !tw)
            helper.fail("expected deeptime and toroidal_world loaded, got deeptime=" + dt + " toroidal_world=" + tw);
        helper.succeed();
    }
}
