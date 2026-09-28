// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime;

import com.minecraftindustrialcomplex.deeptime.gametest.GameTestSupport;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Deep Time's entry point. The mod registers no blocks or items (decisions file,
 * "Rock blocks"): everything it adds is worldgen, data, screens and hooks.
 */
@Mod(DeepTime.MODID)
public final class DeepTime {

    public static final String MODID = "deeptime";

    public static final Logger LOGGER = LoggerFactory.getLogger(MODID);

    public DeepTime(IEventBus modBus) {
        GameTestSupport.init(modBus);
    }
}
