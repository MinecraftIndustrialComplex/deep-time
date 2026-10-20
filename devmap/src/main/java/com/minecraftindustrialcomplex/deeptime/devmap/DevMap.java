// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap;

import com.minecraftindustrialcomplex.deeptime.devmap.net.DevMapNetwork;
import com.minecraftindustrialcomplex.deeptime.devmap.server.ServerMaps;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Deep Time's internal developer map (Ben, 2026-10-20: "an internal use map mod that can show both
 * the players location on a globe / on whatever projection is currently getting used by the
 * world"). A separate mod on top of Deep Time, for development only: never in Deep Time's jar or
 * the MIC pack. Its projection, world data, colour ramps and globe are Deep Time's own classes.
 * The client half is {@code client.DevMapClient}.
 */
@Mod(DevMap.MODID)
public final class DevMap {

    public static final String MODID = "deeptime_devmap";

    public static final Logger LOGGER = LoggerFactory.getLogger(MODID);

    public DevMap(IEventBus modBus) {
        DevMapNetwork.init(modBus);
        ServerMaps.init();
    }
}
