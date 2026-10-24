// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.client.ClientConfig;
import com.minecraftindustrialcomplex.deeptime.mixin.client.CreateWorldScreenInvoker;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.worldgen.PlanetGenerators;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;

/**
 * The one-click Create World flow (Ben, 2026-10-19: "it should default to deeptime, you click
 * create"; docs/design/create-world-ux.md), called from {@code mixin.client.CreateWorldScreenMixin}:
 * <ol>
 *   <li>{@link #fresh}: a new Create World screen starts on {@code deeptime:planet_cube} (the cube is the
 *       only world shape that ships, decisions log 2026-10-24; the client setting {@code defaultWorldType},
 *       on by default), unless the natives did not load here, in
 *       which case the game's own default stays so Create still works.</li>
 *   <li>{@link #interceptCreate}: Create on a Deep Time world that has no world data yet opens the
 *       generation screen, which simulates the planet with the default settings at once; its Play
 *       binds the finished run and creates the world through {@link #create}.</li>
 * </ol>
 * Customize (the World tab's button) still opens the planet editor's Customize page first.
 */
public final class CreateWorldFlow {

    private CreateWorldFlow() {}

    /** A fresh Create World screen: select Deep Time when the setting is on and the natives loaded. */
    public static void fresh(CreateWorldScreen screen) {
        if (!ClientConfig.defaultWorldType() || !NativeLoader.isLinked())
            return;
        WorldCreationUiState ui = screen.getUiState();
        ui.getNormalPresetList().stream()
                .filter(e -> e.preset() != null && e.preset().is(PlanetGenerators.PLANET_CUBE))
                .findFirst()
                .ifPresentOrElse(ui::setWorldType,
                        () -> DeepTime.LOGGER.warn("Deep Time: deeptime:planet_cube is not among the world types; Create World keeps the game's default"));
    }

    /**
     * The Create button (or Enter) was pressed. Returns true when Deep Time handles it: a Deep Time
     * world without world data opens the generation screen; without the natives nothing happens (the
     * button is disabled and says why). Any other world goes on to the game's own creation.
     */
    public static boolean interceptCreate(CreateWorldScreen screen) {
        return switch (CreateWorldGuard.block(screen.getUiState())) {
            case NONE -> false;
            case NATIVES -> true;
            case NOT_SIMULATED, STAGING_MISSING -> {
                Minecraft.getInstance().setScreen(PlanetEditorScreen.generate(screen));
                yield true;
            }
        };
    }

    /** Creates the world as the Create button does (Play, once the run is bound). */
    static void create(CreateWorldScreen screen) {
        ((CreateWorldScreenInvoker) screen).deeptime$onCreate();
    }
}
