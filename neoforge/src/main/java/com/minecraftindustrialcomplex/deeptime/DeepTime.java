// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime;

import com.minecraftindustrialcomplex.deeptime.compat.CompatHooks;
import com.minecraftindustrialcomplex.deeptime.gametest.GameTestSupport;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.net.DeepTimeNetwork;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLogSink;
import com.minecraftindustrialcomplex.deeptime.ore.OreBlockResolver;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.server.DeepTimeCommands;
import com.minecraftindustrialcomplex.deeptime.server.SpawnHandler;
import com.minecraftindustrialcomplex.deeptime.server.WorldDataBinder;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCap;
import net.neoforged.api.distmarker.Dist;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;
import net.neoforged.fml.loading.FMLEnvironment;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.RegisterCommandsEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Deep Time's entry point. The mod registers no blocks or items (decisions file,
 * "Rock blocks"): everything it adds is worldgen, data, screens, hooks and two required network
 * channels (plan §7.15), which is what makes it required on the client too.
 */
@Mod(DeepTime.MODID)
public final class DeepTime {

    public static final String MODID = "deeptime";

    public static final Logger LOGGER = LoggerFactory.getLogger(MODID);

    /** Native log lines drained from the Rust side (plan §6.3). */
    private static final Logger NATIVE_LOGGER = LoggerFactory.getLogger(NativeLogSink.NATIVE);

    public DeepTime(IEventBus modBus) {
        DeepTimeRegistries.register(modBus);
        NeoForge.EVENT_BUS.addListener(RegisterCommandsEvent.class, e -> DeepTimeCommands.register(e.getDispatcher()));
        // /deeptime climate (P1-25): merges into the same /deeptime node.
        NeoForge.EVENT_BUS.addListener(RegisterCommandsEvent.class,
                e -> com.minecraftindustrialcomplex.deeptime.server.ClimateCommand.register(e.getDispatcher()));
        WorldDataBinder.init();
        SpawnHandler.init();
        RockPalette.init();
        com.minecraftindustrialcomplex.deeptime.worldgen.GeothermalPalette.init();
        com.minecraftindustrialcomplex.deeptime.worldgen.MagmaPalette.init();
        // The volcanic biomes' role lists are tags: resolve them again after a reload (decisions log 2026-10-03).
        NeoForge.EVENT_BUS.addListener(net.neoforged.neoforge.event.TagsUpdatedEvent.class,
                e -> com.minecraftindustrialcomplex.deeptime.climate.VolcanicBiomes.tagsUpdated());
        OreBlockResolver.init();
        GameTestSupport.init(modBus);
        // Wrap engine (docs/design/wrap-engine.md): nothing without -Ddeeptime.wrapengine=true.
        com.minecraftindustrialcomplex.deeptime.wrapengine.WrapEngine.init();
        DeepTimeNetwork.init(modBus);
        PoleCap.init();
        // COE and CDG hooks (P1-16): their mixins' targets load now, so a drifted signature fails the boot.
        CompatHooks.preloadTargets();
        // Distant Horizons far terrain (P3-04): LODs from the world data; nothing without Distant Horizons.
        com.minecraftindustrialcomplex.deeptime.compat.dh.DhFarTerrain.init();
        NeoForge.EVENT_BUS.addListener(RegisterCommandsEvent.class,
                e -> com.minecraftindustrialcomplex.deeptime.server.FarTerrainCommand.register(e.getDispatcher()));
        // The F3 line and the polar storm; never loaded on a dedicated server.
        if (FMLEnvironment.dist == Dist.CLIENT)
            com.minecraftindustrialcomplex.deeptime.client.DeepTimeClient.init(modBus);
        loadNatives();
    }

    /**
     * Links the native library once, on both sides (plan §6.5). Never throws: a failure is logged
     * once with its guidance, and everything that reads world data keeps working (plan §6.6).
     */
    private static void loadNatives() {
        try {
            NativeLoader.setLogSink(DeepTime::logNative);
            NativeLoader.load(FMLPaths.GAMEDIR.get());
        } catch (Throwable t) {
            LOGGER.error("Deep Time natives: unexpected error while loading", t);
        }
    }

    private static void logNative(String logger, System.Logger.Level level, String message, Throwable thrown) {
        Logger l = NativeLogSink.NATIVE.equals(logger) ? NATIVE_LOGGER : LOGGER;
        switch (level) {
            case ERROR -> l.error(message, thrown);
            case WARNING -> l.warn(message, thrown);
            case DEBUG, TRACE -> l.debug(message, thrown);
            default -> l.info(message, thrown);
        }
    }
}
