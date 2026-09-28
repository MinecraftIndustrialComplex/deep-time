// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime;

import com.minecraftindustrialcomplex.deeptime.gametest.GameTestSupport;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLogSink;
import com.minecraftindustrialcomplex.deeptime.server.DeepTimeCommands;
import net.neoforged.bus.api.IEventBus;
import net.neoforged.fml.common.Mod;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.RegisterCommandsEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Deep Time's entry point. The mod registers no blocks or items (decisions file,
 * "Rock blocks"): everything it adds is worldgen, data, screens and hooks.
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
        GameTestSupport.init(modBus);
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
