// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The world data of the running server, by folder name (plan §7.1, "Lazy binding"). Filled at
 * {@code ServerAboutToStartEvent} by {@code WorldDataBinder}, before any level exists, and cleared
 * when the server stops. Density functions get neither the level nor the seed, so the terrain
 * source looks its data up here by the folder its generator names.
 */
public final class ActiveWorldData {

    private static final Map<String, WorldData> BY_FOLDER = new ConcurrentHashMap<>();

    private ActiveWorldData() {}

    public static void put(String folder, WorldData data) {
        BY_FOLDER.put(folder, data);
    }

    /** The bound world data, or an exception naming the folder: never plausible garbage. */
    public static WorldData get(String folder) {
        WorldData wd = BY_FOLDER.get(folder);
        if (wd == null)
            throw new IllegalStateException("Deep Time world data '" + folder + "' is not bound: the world folder has no "
                    + folder + "/manifest.json, or it was sampled before ServerAboutToStartEvent");
        return wd;
    }

    public static boolean has(String folder) {
        return BY_FOLDER.containsKey(folder);
    }

    public static void clear() {
        BY_FOLDER.clear();
    }
}
