// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import net.neoforged.neoforge.common.ModConfigSpec;

/** The dev map's client settings ({@code config/deeptime_devmap-client.toml}). */
final class ClientConfig {

    private static final ModConfigSpec.Builder BUILDER = new ModConfigSpec.Builder();

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

    private ClientConfig() {}

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
