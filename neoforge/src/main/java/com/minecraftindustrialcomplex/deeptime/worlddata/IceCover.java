// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.Optional;

/**
 * The ice cover of a world: how many blocks of snow and ice {@code worldgen/IcePass} lays over the
 * ground at a block column, whatever the pack's biome source drew there (Ben, 2026-09-30, "Force
 * snow/ice cover": Terralith has no flat inland ice biome, so an ice cap's flats would show gravel).
 *
 * <p>This is the small interface between the climate and the surface pass. The bake writes the
 * cover's thickness as the layer {@code climate/ice_cover} (L2, u8, blocks) from the climate's land-ice
 * thickness ({@code climate/ice_thickness_m}; an older run without ice fields falls back to the
 * ground's height above its equilibrium line; {@code bake/ice.rs}). The pass reads nothing of the
 * climate but this layer, so a climate that changes how it carries ice does not change the pass. The table that shaped the cover (the placement file's {@code ice_cover}
 * section: the step, the ceiling, the snow and ice blocks) is recorded in
 * {@code notes.bake.climate.ice_cover}; FORMAT.md, "The ice cover".
 */
public interface IceCover {

    /** The baked thickness layer. */
    String LAYER = "climate/ice_cover";

    /** Blocks of cover at block column (x, z); 0 for none. */
    int blocks(int x, int z);

    /** What the cover is made of. */
    Spec spec();

    /** What a cover is made of: the top {@code snowBlocks} snow, the base blue ice when thick, packed ice between. */
    record Spec(double stepM, int maxBlocks, int snowBlocks, int blueIceFromBlocks, int blueIceBlocks, String snow, String ice,
                String deepIce) {

        public static final int SNOW = 0, ICE = 1, DEEP_ICE = 2;

        /**
         * The kind of block {@code index} blocks below the top of a cover {@code total} blocks thick:
         * {@link #SNOW} for the top {@code snowBlocks}, {@link #DEEP_ICE} for the last
         * {@code blueIceBlocks} of a cover of at least {@code blueIceFromBlocks}, else {@link #ICE}.
         */
        public int kind(int index, int total) {
            if (index < snowBlocks)
                return SNOW;
            if (total >= blueIceFromBlocks && index >= total - blueIceBlocks)
                return DEEP_ICE;
            return ICE;
        }

        /** The block id of a {@link #kind}. */
        public String blockId(int kind) {
            return switch (kind) {
                case SNOW -> snow;
                case DEEP_ICE -> deepIce;
                default -> ice;
            };
        }

        /** {@code notes.bake.climate.ice_cover} of a manifest's notes, or null when the bake wrote none. */
        public static Spec read(JsonObject notes) {
            JsonObject o = object(object(object(notes, "bake"), "climate"), "ice_cover");
            if (o == null || !o.has("max_blocks"))
                return null;
            return new Spec(o.get("step_m").getAsDouble(), o.get("max_blocks").getAsInt(), o.get("snow_blocks").getAsInt(),
                    o.get("blue_ice_from_blocks").getAsInt(), o.get("blue_ice_blocks").getAsInt(), o.get("snow").getAsString(),
                    o.get("ice").getAsString(), o.get("deep_ice").getAsString());
        }

        private static JsonObject object(JsonObject o, String key) {
            if (o == null)
                return null;
            JsonElement e = o.get(key);
            return e != null && e.isJsonObject() ? e.getAsJsonObject() : null;
        }
    }

    /**
     * The world's ice cover: its {@code climate/ice_cover} layer read bilinearly (so the edge of a
     * cover is a slope of the layer's cells, not their 8-block steps) and rounded, with the table
     * recorded in the manifest; empty when the world has no such layer (no climate stage, or a
     * synthetic world).
     */
    static Optional<IceCover> of(WorldData wd) {
        if (!wd.has(LAYER))
            return Optional.empty();
        Spec spec = Spec.read(wd.manifest().notes());
        if (spec == null)
            return Optional.empty();
        RasterSampler layer = wd.layer(LAYER);
        return Optional.of(new IceCover() {
            @Override
            public int blocks(int x, int z) {
                double v = layer.bilinear(x, z);
                if (Double.isNaN(v))
                    return 0;
                return (int) Math.max(0, Math.min(spec.maxBlocks(), Math.round(v)));
            }

            @Override
            public Spec spec() {
                return spec;
            }
        });
    }
}
