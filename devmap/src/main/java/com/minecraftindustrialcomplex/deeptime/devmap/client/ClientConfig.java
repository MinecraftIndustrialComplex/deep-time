// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import com.minecraftindustrialcomplex.deeptime.devmap.model.MapLayer;
import net.neoforged.neoforge.common.ModConfigSpec;

/** The dev map's client settings ({@code config/deeptime_devmap-client.toml}). */
final class ClientConfig {

    private static final ModConfigSpec.Builder BUILDER = new ModConfigSpec.Builder();

    static final ModConfigSpec.EnumValue<MapLayer> MAP_LAYER = BUILDER
            .comment("The picture under the dev map and the corner minimap: SATELLITE (true colour, the workbench's look)",
                    "or ELEVATION (the terrain height ramp). Switch in game with S on the map screen or its button.")
            .defineEnum("mapLayer", MapLayer.DEFAULT);

    static final ModConfigSpec.BooleanValue HUD = BUILDER
            .comment("Show the dev map's corner minimap (toggle in game with the HUD key, period by default).")
            .define("hud", false);

    static final ModConfigSpec.IntValue HUD_BLOCKS = BUILDER
            .comment("How many blocks the corner minimap spans across.")
            .defineInRange("hudBlocks", 512, 32, 16384);

    static final ModConfigSpec.IntValue HUD_SIZE = BUILDER
            .comment("The corner minimap's size in GUI pixels.")
            .defineInRange("hudSize", 128, 48, 512);

    static final ModConfigSpec SPEC = BUILDER.build();

    /** The picture until the config has loaded (and when it cannot be saved). */
    private static MapLayer layerFallback = MapLayer.DEFAULT;

    private ClientConfig() {}

    /** The picture the player chose (satellite until they switch). */
    static MapLayer mapLayer() {
        return SPEC.isLoaded() ? MAP_LAYER.get() : layerFallback;
    }

    /** Remembers {@code layer} as the player's picture (saved to the config file when it is loaded). */
    static void mapLayer(MapLayer layer) {
        layerFallback = layer;
        if (SPEC.isLoaded()) {
            MAP_LAYER.set(layer);
            MAP_LAYER.save();
        }
    }

    static boolean hud() {
        return SPEC.isLoaded() && HUD.get();
    }

    static void hud(boolean on) {
        if (SPEC.isLoaded()) {
            HUD.set(on);
            HUD.save();
        }
    }

    static int hudBlocks() {
        return SPEC.isLoaded() ? HUD_BLOCKS.get() : 512;
    }

    static int hudSize() {
        return SPEC.isLoaded() ? HUD_SIZE.get() : 128;
    }
}
