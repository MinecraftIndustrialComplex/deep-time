// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.client;

import com.minecraftindustrialcomplex.deeptime.devmap.DevMap;
import com.minecraftindustrialcomplex.deeptime.devmap.net.DevMapState;
import com.mojang.blaze3d.platform.InputConstants;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.resources.ResourceLocation;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.ModContainer;
import net.neoforged.fml.common.Mod;
import net.neoforged.fml.config.ModConfig;
import net.neoforged.neoforge.client.event.ClientPlayerNetworkEvent;
import net.neoforged.neoforge.client.event.ClientTickEvent;
import net.neoforged.neoforge.client.event.RegisterGuiLayersEvent;
import net.neoforged.neoforge.client.event.RegisterKeyMappingsEvent;
import net.neoforged.neoforge.common.NeoForge;
import org.lwjgl.glfw.GLFW;

/**
 * The dev map's client half: two keys (comma opens the map, period toggles the corner minimap; M,
 * the usual map key, is taken in MIC by Antique Atlas, Simple Voice Chat and Cobblemon), the HUD
 * layer, the client config, and forgetting the server's data when the client leaves.
 */
@Mod(value = DevMap.MODID, dist = Dist.CLIENT)
public final class DevMapClient {

    static final String CATEGORY = "key.categories.deeptime_devmap";
    static final KeyMapping OPEN = new KeyMapping("key.deeptime_devmap.open", InputConstants.Type.KEYSYM, GLFW.GLFW_KEY_COMMA, CATEGORY);
    static final KeyMapping HUD = new KeyMapping("key.deeptime_devmap.hud", InputConstants.Type.KEYSYM, GLFW.GLFW_KEY_PERIOD, CATEGORY);

    /** The HUD switch until the config has loaded (and when it cannot be saved). */
    private static boolean hudFallback;

    public DevMapClient(IEventBus modBus, ModContainer container) {
        container.registerConfig(ModConfig.Type.CLIENT, ClientConfig.SPEC);
        modBus.addListener(RegisterKeyMappingsEvent.class, e -> {
            e.register(OPEN);
            e.register(HUD);
        });
        modBus.addListener(RegisterGuiLayersEvent.class, e ->
                e.registerAboveAll(ResourceLocation.fromNamespaceAndPath(DevMap.MODID, "minimap"), HudMinimap::render));
        NeoForge.EVENT_BUS.addListener(ClientTickEvent.Post.class, e -> tick());
        NeoForge.EVENT_BUS.addListener(ClientPlayerNetworkEvent.LoggingOut.class, e -> {
            DevMapState.reset();
            MapTextures.reset();
        });
        ShotDriver.init();
    }

    private static void tick() {
        Minecraft mc = Minecraft.getInstance();
        while (OPEN.consumeClick())
            if (mc.screen == null && mc.player != null)
                mc.setScreen(new DevMapScreen());
        while (HUD.consumeClick())
            if (mc.player != null)
                toggleHud();
    }

    static boolean hudOn() {
        return ClientConfig.SPEC.isLoaded() ? ClientConfig.hud() : hudFallback;
    }

    static void toggleHud() {
        boolean on = !hudOn();
        hudFallback = on;
        ClientConfig.hud(on);
    }

    static void hud(boolean on) {
        hudFallback = on;
        ClientConfig.hud(on);
    }
}
