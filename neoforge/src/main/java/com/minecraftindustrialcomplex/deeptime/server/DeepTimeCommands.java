// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.BiomeCensus;
import com.minecraftindustrialcomplex.deeptime.compat.sable.SableTestOps;
import com.minecraftindustrialcomplex.deeptime.gametest.DeepTimeTestReporter;
import com.minecraftindustrialcomplex.deeptime.gametest.GameTestSupport;
import com.minecraftindustrialcomplex.deeptime.net.DeepTimeNetwork;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCap;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve;
import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import com.mojang.brigadier.context.CommandContext;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.fml.ModList;

import java.util.ArrayList;
import java.util.List;
import java.util.UUID;

/**
 * {@code /deeptime ...} for tests and debugging (plan §7.16). Every probe prints one JSON line
 * prefixed {@code [deeptime-probe]} to the command source and the log, which the smoke harness
 * and gametest-planet.sh parse. Permission level 2.
 */
public final class DeepTimeCommands {

    public static final String PROBE_PREFIX = "[deeptime-probe] ";

    private DeepTimeCommands() {}

    public static void register(CommandDispatcher<CommandSourceStack> dispatcher) {
        var probe = Commands.literal("probe")
                .then(Commands.literal("column")
                        .then(Commands.argument("x", IntegerArgumentType.integer())
                                .then(Commands.argument("z", IntegerArgumentType.integer())
                                        .executes(DeepTimeCommands::probeColumn))))
                .then(Commands.literal("tw").executes(DeepTimeCommands::probeTw))
                .then(Commands.literal("pole")
                        .executes(c -> probePole(c.getSource(), c.getSource().getPosition().z))
                        .then(Commands.argument("z", IntegerArgumentType.integer())
                                .executes(c -> probePole(c.getSource(), IntegerArgumentType.getInteger(c, "z")))))
                .then(Commands.literal("network").executes(c -> probeNetwork(c.getSource())))
                .then(Commands.literal("palette").executes(c -> probePalette(c.getSource())))
                // Where the level has lava: the active volcanoes' magma, the lava features refused, the distances.
                .then(Commands.literal("magma").executes(c -> MagmaProbe.probe(c.getSource())))
                .then(Commands.literal("site")
                        .then(Commands.argument("x", IntegerArgumentType.integer())
                                .then(Commands.argument("y", IntegerArgumentType.integer())
                                        .then(Commands.argument("z", IntegerArgumentType.integer())
                                                .executes(c -> SiteCommands.probeSite(c.getSource(),
                                                        IntegerArgumentType.getInteger(c, "x"), IntegerArgumentType.getInteger(c, "y"),
                                                        IntegerArgumentType.getInteger(c, "z")))))))
                .then(Commands.literal("claims").executes(c -> SiteCommands.probeClaims(c.getSource())))
                // P1-09: surface biomes against the Köppen map; the grid goes to deeptime-probe/biomemap.json.
                .then(Commands.literal("biomes")
                        .executes(c -> probeBiomes(c.getSource(), 16))
                        .then(Commands.argument("step", IntegerArgumentType.integer(4, 4096))
                                .executes(c -> probeBiomes(c.getSource(), IntegerArgumentType.getInteger(c, "step")))))
                // P1-16: what Deep Time and COE / CDG hold for a chunk (work without the mods too).
                .then(Commands.literal("coe")
                        .then(Commands.argument("chunkX", IntegerArgumentType.integer())
                                .then(Commands.argument("chunkZ", IntegerArgumentType.integer())
                                        .executes(c -> CompatCommands.probeCoe(c.getSource(),
                                                IntegerArgumentType.getInteger(c, "chunkX"), IntegerArgumentType.getInteger(c, "chunkZ"))))))
                .then(Commands.literal("cdg")
                        .then(Commands.argument("chunkX", IntegerArgumentType.integer())
                                .then(Commands.argument("chunkZ", IntegerArgumentType.integer())
                                        .executes(c -> CompatCommands.probeCdg(c.getSource(),
                                                IntegerArgumentType.getInteger(c, "chunkX"), IntegerArgumentType.getInteger(c, "chunkZ"))))));
        var gametest = Commands.literal("gametest").requires(s -> GameTestSupport.enabled());
        // Sable is optional: its probe and test commands exist only when it is loaded, and every
        // Sable call goes through SableTestOps (loaded on first use).
        if (ModList.get().isLoaded(SableTestOps.MOD_ID)) {
            probe.then(Commands.literal("sable").executes(DeepTimeCommands::probeSable));
            gametest.then(Commands.literal("sable")
                    .then(Commands.literal("straddle")
                            .then(Commands.argument("z", IntegerArgumentType.integer())
                                    .executes(DeepTimeCommands::sableStraddle))));
        }
        var sites = Commands.literal("sites")
                .then(Commands.literal("near")
                        .then(Commands.argument("x", IntegerArgumentType.integer())
                                .then(Commands.argument("z", IntegerArgumentType.integer())
                                        .then(Commands.argument("radius", IntegerArgumentType.integer(1, 1 << 20))
                                                .executes(c -> SiteCommands.sitesNear(c.getSource(),
                                                        IntegerArgumentType.getInteger(c, "x"), IntegerArgumentType.getInteger(c, "z"),
                                                        IntegerArgumentType.getInteger(c, "radius")))))));
        var why = Commands.literal("why")
                .then(Commands.argument("siteId", IntegerArgumentType.integer(0))
                        .executes(c -> SiteCommands.why(c.getSource(), IntegerArgumentType.getInteger(c, "siteId"))));
        dispatcher.register(Commands.literal("deeptime").requires(s -> s.hasPermission(2))
                .then(probe)
                .then(sites)
                .then(why)
                .then(gametest
                        .then(Commands.literal("reset").executes(c -> {
                            DeepTimeTestReporter.INSTANCE.reset();
                            JsonObject o = new JsonObject();
                            o.addProperty("probe", "gametest-reset");
                            return emit(c.getSource(), o);
                        }))
                        .then(Commands.literal("summary").executes(c -> {
                            String line = "[gametest-summary] " + DeepTimeTestReporter.INSTANCE.summary();
                            DeepTime.LOGGER.info(line);
                            c.getSource().sendSuccess(() -> Component.literal(line), false);
                            return 1;
                        }))));
    }

    private static int probeColumn(CommandContext<CommandSourceStack> ctx) {
        ServerLevel level = ctx.getSource().getLevel();
        int x = IntegerArgumentType.getInteger(ctx, "x");
        int z = IntegerArgumentType.getInteger(ctx, "z");
        JsonObject o = new JsonObject();
        o.addProperty("probe", "column");
        o.addProperty("level", level.dimension().location().toString());
        o.addProperty("x", x);
        o.addProperty("z", z);
        var generator = level.getChunkSource().getGenerator();
        o.addProperty("generator", generator.getClass().getName());
        if (generator instanceof GeologicChunkGenerator g) {
            var terrain = g.terrain();
            o.addProperty("terrain", terrain.getClass().getSimpleName());
            o.addProperty("circumference", terrain.circumference());
            o.addProperty("terrainH", terrain.height(x, z));
            o.addProperty("surfaceY", generator.getBaseHeight(x, z, Heightmap.Types.OCEAN_FLOOR_WG, level,
                    level.getChunkSource().randomState()) - 1);
            for (String layer : new String[]{"temperature", "vegetation", "continents", "erosion", "ridges"})
                o.addProperty(layer, terrain.raster("climate/" + layer, x, z));
        }
        // The loaded chunk's own heightmap, without loading anything (null when not loaded).
        var chunk = level.getChunkSource().getChunkNow(x >> 4, z >> 4);
        o.addProperty("chunkLoaded", chunk != null);
        if (chunk != null)
            o.addProperty("worldSurface", chunk.getHeight(Heightmap.Types.WORLD_SURFACE, x & 15, z & 15));
        var biome = level.getBiome(new BlockPos(x, generator.getSeaLevel(), z));
        o.addProperty("biome", biome.unwrapKey().map(k -> k.location().toString()).orElse("?"));
        return emit(ctx.getSource(), o);
    }

    private static int probeTw(CommandContext<CommandSourceStack> ctx) {
        ServerLevel level = ctx.getSource().getLevel();
        JsonObject o = new JsonObject();
        o.addProperty("probe", "tw");
        o.addProperty("level", level.dimension().location().toString());
        var generator = level.getChunkSource().getGenerator();
        o.addProperty("generator", generator.getClass().getName());
        var shape = com.toroidalworld.api.v1.ToroidalWorldApi.shapeOf(level);
        // A NoiseBasedChunkGenerator subclass is never rebuilt by TW (exact-class check), only
        // stamped: a shape on a Geologic level therefore means "stamped".
        o.addProperty("stamped", shape.isPresent() && generator instanceof GeologicChunkGenerator);
        shape.ifPresent(s -> {
            o.addProperty("loopsX", s.loops(net.minecraft.core.Direction.Axis.X));
            o.addProperty("loopsZ", s.loops(net.minecraft.core.Direction.Axis.Z));
            o.addProperty("widthChunks", s.widthChunks(net.minecraft.core.Direction.Axis.X));
            o.addProperty("minChunkX", s.minChunk(net.minecraft.core.Direction.Axis.X));
            o.addProperty("maxChunkX", s.maxChunk(net.minecraft.core.Direction.Axis.X));
        });
        // TW's climate compression matches vanilla's climate density functions only; Deep Time's
        // climate is deeptime:raster, so compression cannot apply.
        o.addProperty("climateCompression", "off");
        return emit(ctx.getSource(), o);
    }

    /**
     * Every rock class → its resolved blocks (plan §7.16): {@code classes} (class → block states,
     * the primary first; what {@code deeptime-inspect rock --palette} reads), {@code source},
     * {@code state}, {@code unresolved} and {@code problems} ({@link RockPalette#toJson}).
     */
    public static int probePalette(CommandSourceStack source) {
        JsonObject o = new JsonObject();
        o.addProperty("probe", "palette");
        RockPalette.get().toJson().entrySet().forEach(e -> o.add(e.getKey(), e.getValue()));
        return emit(source, o);
    }

    /**
     * {@code /deeptime probe sable}: every loaded Sable sub-level of the level with its id, block
     * count (non-air blocks in its plot) and pose position. Registered only with Sable loaded.
     */
    private static int probeSable(CommandContext<CommandSourceStack> ctx) {
        ServerLevel level = ctx.getSource().getLevel();
        JsonObject o = new JsonObject();
        o.addProperty("probe", "sable");
        o.addProperty("level", level.dimension().location().toString());
        JsonArray subLevels = new JsonArray();
        for (SableTestOps.SubLevelInfo s : SableTestOps.list(level)) {
            JsonObject e = new JsonObject();
            e.addProperty("id", s.id().toString());
            e.addProperty("blocks", s.blocks());
            e.addProperty("x", s.position().x);
            e.addProperty("y", s.position().y);
            e.addProperty("z", s.position().z);
            e.addProperty("removed", s.removed());
            subLevels.add(e);
        }
        o.add("subLevels", subLevels);
        return emit(ctx.getSource(), o);
    }

    /**
     * {@code /deeptime gametest sable straddle <z>} (P0-06 scenario {@code sable_straddle}): builds
     * a 6 × 3 oak deck with a one-block mast across x = C/2 (three columns each side), two blocks
     * above the higher of the ground on either side, and assembles it into one sub-level. The deck
     * is given to Sable in raw coordinates (x = C/2 − 3 … C/2 + 2); the level folds the west half
     * when the blocks are placed. Needs the seam chunks loaded (the harness forceloads them).
     */
    private static int sableStraddle(CommandContext<CommandSourceStack> ctx) {
        ServerLevel level = ctx.getSource().getLevel();
        int z = IntegerArgumentType.getInteger(ctx, "z");
        JsonObject o = new JsonObject();
        o.addProperty("probe", "sable-assemble");
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g)) {
            o.addProperty("error", "not a Deep Time planet");
            return emit(ctx.getSource(), o);
        }
        int c = g.dataRef().circumferenceBlocks();
        var shape = com.toroidalworld.api.v1.ToroidalWorldApi.shapeOf(level).orElse(null);
        if (shape == null) {
            o.addProperty("error", "no Toroidal World shape");
            return emit(ctx.getSource(), o);
        }
        int ground = Integer.MIN_VALUE;
        for (int x = c / 2 - 3; x <= c / 2 + 2; x++)
            for (int dz = -1; dz <= 1; dz++) {
                BlockPos f = shape.fold(new BlockPos(x, 0, z + dz));
                var chunk = level.getChunkSource().getChunkNow(f.getX() >> 4, f.getZ() >> 4);
                if (chunk == null) {
                    o.addProperty("error", "chunk of " + f.toShortString() + " is not loaded (forceload the seam first)");
                    return emit(ctx.getSource(), o);
                }
                ground = Math.max(ground, chunk.getHeight(Heightmap.Types.MOTION_BLOCKING, f.getX(), f.getZ()));
            }
        int y = ground + 2;
        List<BlockPos> deck = new ArrayList<>();
        for (int x = c / 2 - 3; x <= c / 2 + 2; x++)
            for (int dz = -1; dz <= 1; dz++)
                deck.add(new BlockPos(x, y, z + dz));
        BlockPos anchor = new BlockPos(c / 2 - 1, y, z);
        deck.add(anchor.above());
        for (BlockPos p : deck)
            level.setBlock(p, Blocks.OAK_PLANKS.defaultBlockState(), 3);
        UUID id = SableTestOps.assemble(level, anchor, deck);
        o.addProperty("id", id.toString());
        o.addProperty("placed", deck.size());
        o.addProperty("blocks", SableTestOps.blockCount(level, id));
        var pos = SableTestOps.position(level, id);
        if (pos != null) {
            o.addProperty("x", pos.x);
            o.addProperty("y", pos.y);
            o.addProperty("z", pos.z);
        }
        o.addProperty("seamX", c / 2);
        return emit(ctx.getSource(), o);
    }

    /**
     * The pole cap (plan §7.11) of the source's level at block z (default: the source's own z):
     * the lines, the latitude and what the cap does there.
     */
    private static int probePole(CommandSourceStack source, double z) {
        ServerLevel level = source.getLevel();
        JsonObject o = new JsonObject();
        o.addProperty("probe", "pole");
        o.addProperty("level", level.dimension().location().toString());
        PoleCapCurve cap = PoleCap.capOf(level);
        o.addProperty("planet", cap != null);
        if (cap != null) {
            o.addProperty("circumference", cap.circumference());
            o.addProperty("capDeg", cap.capDeg());
            o.addProperty("stopDeg", cap.stopDeg());
            o.addProperty("capZ", cap.capZ());
            o.addProperty("stopZ", cap.stopZ());
            o.addProperty("z", z);
            o.addProperty("latitude", cap.latitudeDeg(z));
            o.addProperty("zone", cap.zone(z).name());
            o.addProperty("depth", cap.depth(z));
            o.addProperty("storm", cap.storm(z));
            o.addProperty("push", cap.push(z));
            o.addProperty("wind", cap.wind(z));
        }
        return emit(source, o);
    }

    /**
     * Every payload channel NeoForge has registered (plan §7.15): protocol, id, version, flow and
     * whether it is optional. gametest-planet.sh's {@code refuse_client} scenario builds its fake
     * clients' channel lists from this.
     */
    private static int probeNetwork(CommandSourceStack source) {
        JsonObject o = new JsonObject();
        o.addProperty("probe", "network");
        JsonArray channels = new JsonArray();
        for (DeepTimeNetwork.Channel c : DeepTimeNetwork.registrations()) {
            JsonObject j = new JsonObject();
            j.addProperty("protocol", c.protocol().name());
            j.addProperty("id", c.id().toString());
            j.addProperty("version", c.version());
            j.addProperty("flow", c.flow().map(Enum::name).orElse(null));
            j.addProperty("optional", c.optional());
            channels.add(j);
        }
        o.add("channels", channels);
        return emit(source, o);
    }

    /**
     * {@code /deeptime probe biomes [step]} (P1-09): the overworld's surface biomes every
     * {@code step} blocks against its Köppen map ({@link BiomeCensus}); the whole grid is written to
     * {@code <server>/deeptime-probe/biomemap.json} for the review page.
     */
    private static int probeBiomes(CommandSourceStack source, int step) {
        ServerLevel level = source.getServer().overworld();
        JsonObject o;
        try {
            long t0 = System.nanoTime();
            var r = BiomeCensus.run(level, step, 64,
                    source.getServer().getServerDirectory().resolve("deeptime-probe").resolve("biomemap.json"));
            o = r.json();
            o.addProperty("seconds", Math.round((System.nanoTime() - t0) / 1e7) / 100.0);
        } catch (Exception e) {
            o = new JsonObject();
            o.addProperty("error", e.toString());
        }
        o.addProperty("probe", "biomes");
        return emit(source, o);
    }

    public static int emit(CommandSourceStack source, JsonObject o) {
        String line = PROBE_PREFIX + o;
        DeepTime.LOGGER.info(line);
        source.sendSuccess(() -> Component.literal(line), false);
        return 1;
    }
}
