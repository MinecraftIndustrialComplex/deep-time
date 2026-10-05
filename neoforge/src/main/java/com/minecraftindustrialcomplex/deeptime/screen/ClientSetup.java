// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.screen.model.MachineInfo;
import com.minecraftindustrialcomplex.deeptime.server.Staging;
import com.minecraftindustrialcomplex.deeptime.worldgen.PlanetGenerators;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;
import net.neoforged.fml.event.lifecycle.FMLClientSetupEvent;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.client.event.RegisterPresetEditorsEvent;

import java.time.Clock;

/**
 * Deep Time's client entry point (plan §7.12, P1-11), constructed only on the physical client, so
 * no screen class is ever loaded by a dedicated server: registers the planet editor as the preset
 * editor of {@code deeptime:planet} (and {@code planet_32k}), guards the Create button, and removes
 * abandoned staging folders in the background. Also starts the first background free-memory
 * reading the setup page shows ({@link MachineInfo#prime()}).
 */
@Mod(value = DeepTime.MODID, dist = Dist.CLIENT)
public final class ClientSetup {

    public ClientSetup(IEventBus modBus) {
        modBus.addListener(RegisterPresetEditorsEvent.class, e -> {
            e.register(PlanetGenerators.PLANET, PlanetEditorScreen::create);
            e.register(PlanetGenerators.PLANET_32K, PlanetEditorScreen::create);
        });
        modBus.addListener(FMLClientSetupEvent.class, e -> {
            cleanupInBackground();
            // The first free-memory reading (macOS: a vm_stat process) and the classes behind it, off the render thread.
            MachineInfo.prime();
        });
        CreateWorldGuard.init();
        // Leaving the Create World flow stops runs nobody can reach any more (threads, native memory).
        net.neoforged.neoforge.common.NeoForge.EVENT_BUS.addListener(net.neoforged.neoforge.client.event.ScreenEvent.Opening.class, e -> {
            if (endsCreateWorld(e.getNewScreen()))
                PlanetEditorScreen.abandonAll();
        });
        ScreenshotDriver.init();
    }

    /** Screens that mean the Create World screen is gone for good. */
    static boolean endsCreateWorld(net.minecraft.client.gui.screens.Screen s) {
        return s instanceof net.minecraft.client.gui.screens.TitleScreen
                || s instanceof net.minecraft.client.gui.screens.worldselection.SelectWorldScreen
                || s instanceof net.minecraft.client.gui.screens.multiplayer.JoinMultiplayerScreen
                || s instanceof net.minecraft.client.gui.screens.LevelLoadingScreen;
    }

    private static void cleanupInBackground() {
        Thread t = new Thread(() -> {
            var removed = Staging.cleanupAbandoned(FMLPaths.GAMEDIR.get(), Staging.ABANDONED_AFTER,
                    PlanetEditorScreen.heldStagingIds(), Clock.systemUTC());
            if (!removed.isEmpty())
                DeepTime.LOGGER.info("Removed {} abandoned Deep Time staging folder(s): {}", removed.size(), removed);
        }, "deeptime-staging-cleanup");
        t.setDaemon(true);
        t.start();
    }
}
