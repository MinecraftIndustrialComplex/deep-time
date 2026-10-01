// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.Collections;
import java.util.Map;
import java.util.Optional;
import java.util.WeakHashMap;

/**
 * Which lakes are frozen (Ben, decisions log 2026-10-01, "freeze them and size them by climate";
 * FORMAT.md "Lake ice"): a lake column is under ice where the world's mean annual temperature
 * ({@code climate/t_mean_c}, bilinear at the column) is below the preset's
 * {@code hydro.lake_freeze_mean_c}, which the bake records in
 * {@code notes.bake.water.params.lake_freeze_mean_c}. It is the rule the bake's statistics and the
 * workbench's maps use ({@code deeptime_core::hydro}, {@code HydroParams::frozen}).
 *
 * <p>The game applies it on both sides: {@code worldgen/WaterBodies} ices the top block of a frozen
 * lake column whatever the biome, and {@code worldgen/LakeFreeze} keeps the other lake columns open
 * in a frozen biome. Rivers and the sea are left to the biome. Plain Java, no Minecraft,
 * thread-safe.
 */
public final class LakeIce {

    /** The layer the rule reads. */
    public static final String LAYER = "climate/t_mean_c";

    private static final Map<WorldData, Optional<LakeIce>> CACHE = Collections.synchronizedMap(new WeakHashMap<>());

    private final RasterSampler tMean;
    private final double threshold;

    private LakeIce(RasterSampler tMean, double threshold) {
        this.tMean = tMean;
        this.threshold = threshold;
    }

    /**
     * {@code notes.bake.water.params.lake_freeze_mean_c} of a manifest's notes, or null when the
     * bake wrote none (no rivers and lakes, or a synthetic world).
     */
    public static Double threshold(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "water", "params", "lake_freeze_mean_c"}) {
            if (e == null || !e.isJsonObject())
                return null;
            e = e.getAsJsonObject().get(key);
        }
        return e != null && e.isJsonPrimitive() && e.getAsJsonPrimitive().isNumber() ? e.getAsDouble() : null;
    }

    /** Whether a lake at a mean annual temperature of {@code tMean} °C is frozen (never for NaN). */
    public static boolean frozen(double tMean, double threshold) {
        return tMean < threshold;
    }

    /** The world's rule, or null when it has none (no threshold recorded, or no temperature layer). */
    public static LakeIce of(WorldData wd) {
        return CACHE.computeIfAbsent(wd, w -> {
            Double t = threshold(w.manifest().notes());
            return t == null || !w.has(LAYER) ? Optional.empty() : Optional.of(new LakeIce(w.layer(LAYER), t));
        }).orElse(null);
    }

    /** Whether a lake column at block (x, z) is under ice. */
    public boolean frozen(long x, long z) {
        return frozen(tMean.bilinear(x, z), threshold);
    }

    /**
     * Whether block (x, y, z) is the surface block of a lake of {@code wd} that the rule keeps
     * open: the lake's top block by the shore rule ({@link Water#levelAt} with the generator's
     * noise cell width as {@code reach}), not frozen. False in a world without the rule.
     */
    public static boolean keepsOpen(WorldData wd, int reach, int x, int y, int z) {
        if (!wd.has(WorldData.WATER_LEVEL))
            return false;
        LakeIce ice = of(wd);
        if (ice == null)
            return false;
        double level = Water.levelAt(wd.layer(WorldData.WATER_LEVEL), x, z, reach);
        return Double.isFinite(level) && y == Water.topBlock(level) && !ice.frozen(x, z);
    }

    /** The temperature below which a lake is frozen, °C. */
    public double threshold() {
        return threshold;
    }
}
