// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.client;

import net.neoforged.neoforge.common.ModConfigSpec;

/**
 * Deep Time's client settings ({@code config/deeptime-client.toml}; a screen under Mods, Deep Time,
 * Config): the Distant Horizons curvature switch and whether Create World starts on Deep Time.
 * Client-only: the class is loaded from {@link DeepTimeClient}, never on a dedicated server.
 */
public final class ClientConfig {

    private static final ModConfigSpec.Builder BUILDER = new ModConfigSpec.Builder();

    /** Whether Deep Time sets Distant Horizons' world curvature to the planet's (decisions log 2026-10-03). */
    public static final ModConfigSpec.BooleanValue DH_AUTO_CURVATURE = BUILDER
            .comment("In a Deep Time world, bend Distant Horizons' far terrain to the planet's own curvature "
                    + "(a sphere of circumference / 2 pi blocks), unless you have set Distant Horizons' Earth Curve Ratio yourself. "
                    + "Deep Time uses Distant Horizons' API override, so its config file is never rewritten, and the "
                    + "setting is put back when you leave the world. Turn off to leave Distant Horizons' curvature alone.")
            .translation("deeptime.configuration.dhAutoCurvature")
            .define("dhAutoCurvature", true);

    /** Whether Create World opens with Deep Time as the world type (Ben, 2026-10-19; docs/design/create-world-ux.md). */
    public static final ModConfigSpec.BooleanValue DEFAULT_WORLD_TYPE = BUILDER
            .comment("Open the Create World screen with Deep Time Planet as the world type. Clicking Create then simulates "
                    + "the planet's history (a few minutes) and plays it. The other world types stay in the World tab. "
                    + "Turn off to start on the game's own default world type.")
            .translation("deeptime.configuration.defaultWorldType")
            .define("defaultWorldType", true);

    public static final ModConfigSpec SPEC = BUILDER.build();

    private ClientConfig() {}

    /** The switch's value; true (the default) until the config file has loaded. */
    public static boolean dhAutoCurvature() {
        return !SPEC.isLoaded() || DH_AUTO_CURVATURE.get();
    }

    /** Whether Create World starts on Deep Time; true (the default) until the config file has loaded. */
    public static boolean defaultWorldType() {
        return !SPEC.isLoaded() || DEFAULT_WORLD_TYPE.get();
    }
}
