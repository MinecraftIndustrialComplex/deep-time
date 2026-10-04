// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrain;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.Level;
import net.neoforged.bus.api.EventPriority;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.common.NeoForge;
import net.neoforged.neoforge.event.level.LevelEvent;
import net.neoforged.neoforge.event.server.ServerStartedEvent;
import net.neoforged.neoforge.event.server.ServerStoppedEvent;

import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;

/**
 * Distant Horizons far terrain (plan §10.5 P3-04; decisions file, "Distant Horizons"): for every
 * Deep Time level that generates from world data, registers a world-generator override with
 * Distant Horizons, so its level-of-detail data comes from the baked rasters
 * ({@link FarTerrain}) instead of from real chunks generated far away.
 *
 * <p><b>Optional.</b> Nothing here runs unless the mod {@value #MOD_ID} is installed
 * ({@link #init}), and no class of this package names a Distant Horizons type: the API is bound by
 * name ({@link DhApiBinding}). Without Distant Horizons, or with an API this cannot bind, Deep Time
 * behaves as it did, and Distant Horizons generates its LODs itself from real chunks.
 * {@code -Ddeeptime.dh=false} switches the override off.
 *
 * <p><b>When.</b> Distant Horizons creates its level at NeoForge's {@code LevelEvent.Load} and, half
 * a second later on its own thread, the generation queue, which takes the generator registered for
 * the level at that moment (its own, if none is). So the override is registered in the same event,
 * at the lowest priority (after DH's listener), and once more at {@code ServerStartedEvent} for a
 * level DH had not loaded yet. Only server levels are bound: a client connected to a server
 * generates nothing, the server does (DH's server side stores the LODs and sends them to clients).
 *
 * <p><b>Toroidal World.</b> TW 0.19.3 wraps {@code WorldGeneratorInjector.bind} and refuses every
 * generator that is not Distant Horizons' own on a level it wraps ("only its own generator runs
 * this world's terrain"), from DH 3.3.1 on (with 3.3.0 TW's mixins missed that jar's renamed
 * classes). Deep Time patches that guard to let this generator through, and only it, when the
 * level's loop is the world data's circumference ({@link TwDhGuard}; Ben, 2026-10-02). If the patch
 * did not apply or its check says no, the refusal is logged once with TW's message and Distant
 * Horizons keeps generating from real chunks: correct, slower.
 */
public final class DhFarTerrain {

    public static final String MOD_ID = "distanthorizons";

    /** What happened for one level. */
    public record Bound(ResourceKey<Level> level, String dhLevel, boolean registered, String message, DhLodGenerator generator,
                        Object levelWrapper, FarTerrain terrain) {}

    private static final Map<ResourceKey<Level>, Bound> BOUND = new ConcurrentHashMap<>();
    private static volatile Optional<DhApiBinding> api;
    private static volatile String unavailable;

    private DhFarTerrain() {}

    public static boolean installed() {
        return ModList.get() != null && ModList.get().isLoaded(MOD_ID);
    }

    /** Off with {@code -Ddeeptime.dh=false}. */
    public static boolean enabled() {
        return !"false".equalsIgnoreCase(System.getProperty("deeptime.dh", "true"));
    }

    /** The largest data detail level offered ({@code -Ddeeptime.dh.maxDetail}, default DH's own 12). */
    static int maxDetail() {
        return Integer.getInteger("deeptime.dh.maxDetail", DhLodGenerator.DEFAULT_MAX_DETAIL);
    }

    /** Whether DH validates every column it is given ({@code -Ddeeptime.dh.validate}, default on: gaps and overlaps throw). */
    static boolean validate() {
        return !"false".equalsIgnoreCase(System.getProperty("deeptime.dh.validate", "true"));
    }

    /** Called from the mod's constructor; does nothing without Distant Horizons. */
    public static void init() {
        if (!installed())
            return;
        if (!enabled()) {
            DeepTime.LOGGER.info("[deeptime] Distant Horizons is installed; far terrain is switched off (-Ddeeptime.dh=false)");
            return;
        }
        NeoForge.EVENT_BUS.addListener(EventPriority.LOWEST, LevelEvent.Load.class, e -> {
            if (e.getLevel() instanceof ServerLevel level)
                register(level, false);
        });
        NeoForge.EVENT_BUS.addListener(ServerStartedEvent.class, e -> {
            for (ServerLevel level : e.getServer().getAllLevels())
                register(level, true);
        });
        NeoForge.EVENT_BUS.addListener(LevelEvent.Unload.class, e -> {
            if (e.getLevel() instanceof ServerLevel level)
                BOUND.remove(level.dimension());
        });
        NeoForge.EVENT_BUS.addListener(ServerStoppedEvent.class, e -> BOUND.clear());
        DeepTime.LOGGER.info("[deeptime] Distant Horizons is installed: far terrain will come from the world data");
    }

    /** The binding, resolved once; empty when the installed Distant Horizons has no API this knows. */
    public static Optional<DhApiBinding> api() {
        Optional<DhApiBinding> a = api;
        if (a == null) {
            synchronized (DhFarTerrain.class) {
                if (api == null) {
                    StringBuilder why = new StringBuilder();
                    api = DhApiBinding.find(DhFarTerrain.class.getClassLoader(), why);
                    if (api.isEmpty()) {
                        unavailable = why.toString();
                        DeepTime.LOGGER.error("[deeptime] Distant Horizons' world-generator API was not found ({}); "
                                + "Distant Horizons generates its own LODs from real chunks", unavailable);
                    } else {
                        DeepTime.LOGGER.info("[deeptime] Distant Horizons {} API bound by its {}", api.get().version(), api.get().naming());
                    }
                }
                a = api;
            }
        }
        return a;
    }

    /** Why {@link #api} is empty, or null. */
    public static String unavailable() {
        return unavailable;
    }

    public static Bound bound(ResourceKey<Level> level) {
        return BOUND.get(level);
    }

    /**
     * Registers the override for {@code level} when it is a Deep Time world-data level that Distant
     * Horizons has loaded and that is not bound yet.
     *
     * @param late the server has started: say so when the level is still unknown to Distant Horizons
     */
    static void register(ServerLevel level, boolean late) {
        if (BOUND.containsKey(level.dimension()))
            return;
        Optional<FarTerrain> terrain;
        try {
            terrain = FarTerrain.of(level);
        } catch (RuntimeException e) {
            DeepTime.LOGGER.error("[deeptime] far terrain for {} cannot be set up", level.dimension().location(), e);
            return;
        }
        if (terrain.isEmpty() || api().isEmpty())
            return;
        DhApiBinding dh = api().get();
        try {
            Object wrapper = null;
            for (Object w : dh.loadedLevelWrappers())
                if (dh.wrappedLevel(w) == level)
                    wrapper = w;
            if (wrapper == null) {
                if (late)
                    DeepTime.LOGGER.warn("[deeptime] Distant Horizons has not loaded {}: no far terrain override for it",
                            level.dimension().location());
                return;
            }
            Bound b = bind(dh, level, wrapper, terrain.get());
            BOUND.put(level.dimension(), b);
            if (b.registered())
                DeepTime.LOGGER.info("[deeptime] far terrain: Distant Horizons' LODs of {} ({}) come from the world data "
                                + "(detail levels 0 to {}, surface table: {})", level.dimension().location(), b.dhLevel(),
                        b.generator().maxDetail(), terrain.get().surface().describe());
            else
                DeepTime.LOGGER.warn("[deeptime] far terrain: Distant Horizons refused the override for {} ({}): {} "
                                + "Distant Horizons generates this level's LODs itself, from real chunks.",
                        level.dimension().location(), b.dhLevel(), b.message());
        } catch (ReflectiveOperationException | RuntimeException | LinkageError e) {
            BOUND.put(level.dimension(), new Bound(level.dimension(), "?", false, e.toString(), null, null, terrain.get()));
            DeepTime.LOGGER.error("[deeptime] far terrain: registering with Distant Horizons failed for {}; "
                    + "Distant Horizons generates this level's LODs itself", level.dimension().location(), e);
        }
    }

    private static Bound bind(DhApiBinding dh, ServerLevel level, Object wrapper, FarTerrain terrain) throws ReflectiveOperationException {
        int minY = dh.minHeight(wrapper);
        int height = level.getMaxBuildHeight() - minY;
        DhLodGenerator.Wrappers wrappers = new DhLodGenerator.Wrappers() {
            @Override
            public Object block(Object blockState) throws ReflectiveOperationException {
                return dh.blockWrapper(blockState, wrapper);
            }

            @Override
            public Object biome(Object biomeHolder) throws ReflectiveOperationException {
                return dh.biomeWrapper(biomeHolder, wrapper);
            }

            @Override
            public int minY() {
                return minY;
            }

            @Override
            public int height() {
                return height;
            }
        };
        DhLodGenerator.Source source = (x0, z0, step, n, sink) -> terrain.build(x0, z0, step, n,
                (relX, relZ, count, bottom, top, block, material, light, biome) ->
                        sink.column(relX, relZ, count, bottom, top, block, light, biome));
        DhLodGenerator generator = new DhLodGenerator(dh, source, wrappers, maxDetail(), validate(),
                level.dimension().location().toString());
        DhApiBinding.Result r = dh.register(wrapper, generator.proxy());
        return new Bound(level.dimension(), dh.levelId(wrapper), r.success(), r.message(), generator, wrapper, terrain);
    }

    /** The server's levels that are bound, for the probe. */
    public static Map<ResourceKey<Level>, Bound> all(MinecraftServer server) {
        return Map.copyOf(BOUND);
    }
}
