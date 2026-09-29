// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.cdg.CdgBridge;
import com.minecraftindustrialcomplex.deeptime.compat.coe.CoeBridge;
import net.neoforged.fml.ModList;

import java.util.List;
import java.util.Map;

/**
 * Loads the classes Deep Time's COE and CDG mixins target while the mod is constructed (P1-16).
 * Mixin applies a mixin when its target class loads, and COE's generator, its vein search and CDG's
 * oil data load only when a player first drills, scans or pumps; the injectors have
 * {@code defaultRequire: 1}, so a target whose signature drifted must fail the boot (plan §7.14),
 * not the first player who touches it. Loading without initialising runs no static code of theirs.
 * A target class that is gone fails the boot too.
 */
public final class CompatHooks {

    private CompatHooks() {}

    /** Mod id → the classes its mixin package targets. */
    static final Map<String, List<String>> TARGETS = Map.of(
            CoeBridge.MOD_ID, List.of("com.tom.createores.OreVeinGenerator", "com.tom.createores.OreData",
                    "com.tom.createores.util.RandomSpreadGenerator"),
            CdgBridge.MOD_ID, List.of("com.jesz.createdieselgenerators.world.OilChunksSavedData"));

    public static void preloadTargets() {
        ModList mods = ModList.get();
        if (mods == null)
            return;
        ClassLoader loader = CompatHooks.class.getClassLoader();
        for (var e : TARGETS.entrySet()) {
            if (!mods.isLoaded(e.getKey()))
                continue;
            for (String cls : e.getValue()) {
                try {
                    Class.forName(cls, false, loader);
                } catch (ClassNotFoundException ex) {
                    throw new IllegalStateException(e.getKey() + " is installed but " + cls
                            + " is gone: Deep Time's hooks for it cannot apply (plan §7.14)", ex);
                }
            }
            DeepTime.LOGGER.info("[deeptime] {} hooks bound: {}", e.getKey(), e.getValue());
        }
    }
}
