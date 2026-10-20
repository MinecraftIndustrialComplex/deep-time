// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.client;

import com.minecraftindustrialcomplex.deeptime.net.ClientPlanetState;
import com.minecraftindustrialcomplex.deeptime.net.PlanetDebugLine;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import net.minecraft.client.Minecraft;
import net.minecraft.world.entity.Entity;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.ModContainer;
import net.neoforged.fml.ModLoadingContext;
import net.neoforged.fml.config.ModConfig;
import net.neoforged.neoforge.client.gui.ConfigurationScreen;
import net.neoforged.neoforge.client.gui.IConfigScreenFactory;
import net.neoforged.neoforge.client.event.ClientPlayerNetworkEvent;
import net.neoforged.neoforge.client.event.ClientTickEvent;
import net.neoforged.neoforge.client.event.CustomizeGuiOverlayEvent;
import net.neoforged.neoforge.client.event.ViewportEvent;
import net.neoforged.neoforge.common.NeoForge;

/**
 * Deep Time's client side (plan §7.11, §7.15): the F3 latitude/longitude line and the polar storm
 * ({@link PolarStorm}). Loaded only on a physical client ({@code DeepTime} checks
 * {@code FMLEnvironment.dist}); everything it shows comes from {@link ClientPlanetState}.
 *
 * <p>Nothing here can be checked by the headless tests (plan §9.8): it compiles against the client
 * classes and the dedicated server never loads it.
 */
public final class DeepTimeClient {

    private DeepTimeClient() {}

    public static void init(IEventBus modBus) {
        NeoForge.EVENT_BUS.addListener(CustomizeGuiOverlayEvent.DebugText.class, DeepTimeClient::onDebugText);
        NeoForge.EVENT_BUS.addListener(ClientTickEvent.Pre.class, PolarStorm::onClientTick);
        NeoForge.EVENT_BUS.addListener(ViewportEvent.RenderFog.class, PolarStorm::onRenderFog);
        NeoForge.EVENT_BUS.addListener(ViewportEvent.ComputeFogColor.class, PolarStorm::onFogColor);
        NeoForge.EVENT_BUS.addListener(ClientPlayerNetworkEvent.LoggingOut.class, e -> {
            ClientPlanetState.reset();
            PolarStorm.reset();
            DhCurvatureClient.release();
        });
        // Client settings and their screen (Mods, Deep Time, Config); Distant Horizons' curvature follows the planet,
        // and its world generator stays on there (DhGenerationGate: it switches itself off when Chunky is installed).
        ModContainer container = ModLoadingContext.get().getActiveContainer();
        container.registerConfig(ModConfig.Type.CLIENT, ClientConfig.SPEC);
        container.registerExtensionPoint(IConfigScreenFactory.class, ConfigurationScreen::new);
        DhCurvatureClient.init();
        DhGenerationClient.init();
    }

    /** True when the planet info the server sent is for the level the client shows. */
    static boolean onPlanet(Minecraft mc, WorldInfoPayload info) {
        return info != null && info.planet() && mc.level != null
                && mc.level.dimension().location().toString().equals(info.dimension());
    }

    private static void onDebugText(CustomizeGuiOverlayEvent.DebugText event) {
        Minecraft mc = Minecraft.getInstance();
        WorldInfoPayload info = ClientPlanetState.worldInfo();
        Entity camera = mc.getCameraEntity();
        if (camera == null || !onPlanet(mc, info))
            return;
        String line = PlanetDebugLine.format(info, camera.getX(), camera.getZ(), PolarStorm.shown());
        if (line != null)
            event.getLeft().add(line);
    }
}
