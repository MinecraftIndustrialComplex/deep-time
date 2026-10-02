// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * The world's settings for lava at its volcanoes ({@code notes.bake.underground.params}, preset
 * section {@code underground}; plain Java). {@link Magma} builds the bodies from them.
 *
 * @param chamberDepthKm     depth of a magma chamber's top below the crater floor, km
 *                           ({@code magma_chamber_depth_km})
 * @param chamberSize        multiplies every chamber; 0: no magma at all ({@code magma_chamber_size})
 * @param craterLakeShare    share of the active volcanoes on land that hold a crater lava lake, the
 *                           most active first ({@code crater_lake_share})
 * @param removeLavaFeatures whether the biomes' random lava lakes and springs are removed
 *                           ({@code remove_lava_features})
 * @param lensCount          the most stacked lava lenses a volcano's crust-deep column holds below its
 *                           chamber; 0: no column, the chamber and conduit only ({@code magma_lens_count})
 * @param lensSpacingBlocks  vertical distance between lenses, blocks ({@code magma_lens_spacing_blocks});
 *                           widened where the count would stop the column short of the lava floor
 * @param lensWidth          multiplies every lens's radius; 0: no column ({@code magma_lens_width})
 * @param mushWidth          multiplies the radius of the body of altered hot rock around the column;
 *                           0: none ({@code magma_mush_width})
 */
public record MagmaRules(double chamberDepthKm, double chamberSize, double craterLakeShare, boolean removeLavaFeatures,
                         int lensCount, double lensSpacingBlocks, double lensWidth, double mushWidth) {

    public static final MagmaRules DEFAULT = new MagmaRules(3.0, 1.0, 0.25, true, 24, 28.0, 1.0, 1.0);

    public MagmaRules {
        chamberDepthKm = Math.clamp(chamberDepthKm, 0.5, 15.0);
        chamberSize = Math.clamp(chamberSize, 0.0, 4.0);
        craterLakeShare = Math.clamp(craterLakeShare, 0.0, 1.0);
        lensCount = Math.clamp(lensCount, 0, 64);
        lensSpacingBlocks = Math.clamp(lensSpacingBlocks, 8.0, 200.0);
        lensWidth = Math.clamp(lensWidth, 0.0, 4.0);
        mushWidth = Math.clamp(mushWidth, 0.0, 4.0);
    }

    /** The first four settings with the default crust-deep column. */
    public MagmaRules(double chamberDepthKm, double chamberSize, double craterLakeShare, boolean removeLavaFeatures) {
        this(chamberDepthKm, chamberSize, craterLakeShare, removeLavaFeatures, DEFAULT_LENS_COUNT, DEFAULT_LENS_SPACING,
                DEFAULT_LENS_WIDTH, DEFAULT_MUSH_WIDTH);
    }

    private static final int DEFAULT_LENS_COUNT = 24;
    private static final double DEFAULT_LENS_SPACING = 28.0, DEFAULT_LENS_WIDTH = 1.0, DEFAULT_MUSH_WIDTH = 1.0;

    /** The same rules without the crust-deep column (the chamber and conduit alone). */
    public MagmaRules withoutColumn() {
        return new MagmaRules(chamberDepthKm, chamberSize, craterLakeShare, removeLavaFeatures, 0, lensSpacingBlocks, lensWidth, mushWidth);
    }

    /** Whether volcanoes get the stacked lenses. */
    public boolean hasColumn() {
        return lensCount > 0 && lensWidth > 0.0;
    }

    /** The world's rules; {@link #DEFAULT} for every key that is absent or of the wrong type. */
    public static MagmaRules of(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "underground", "params"}) {
            if (e == null || !e.isJsonObject())
                return DEFAULT;
            e = e.getAsJsonObject().get(key);
        }
        if (e == null || !e.isJsonObject())
            return DEFAULT;
        JsonObject p = e.getAsJsonObject();
        JsonElement remove = p.get("remove_lava_features");
        return new MagmaRules(number(p, "magma_chamber_depth_km", DEFAULT.chamberDepthKm),
                number(p, "magma_chamber_size", DEFAULT.chamberSize),
                number(p, "crater_lake_share", DEFAULT.craterLakeShare),
                remove != null && remove.isJsonPrimitive() && remove.getAsJsonPrimitive().isBoolean()
                        ? remove.getAsBoolean() : DEFAULT.removeLavaFeatures,
                (int) Math.round(number(p, "magma_lens_count", DEFAULT.lensCount)),
                number(p, "magma_lens_spacing_blocks", DEFAULT.lensSpacingBlocks),
                number(p, "magma_lens_width", DEFAULT.lensWidth),
                number(p, "magma_mush_width", DEFAULT.mushWidth));
    }

    private static double number(JsonObject p, String key, double fallback) {
        JsonElement v = p.get(key);
        if (v == null || !v.isJsonPrimitive() || !v.getAsJsonPrimitive().isNumber())
            return fallback;
        double d = v.getAsDouble();
        return Double.isFinite(d) ? d : fallback;
    }
}
