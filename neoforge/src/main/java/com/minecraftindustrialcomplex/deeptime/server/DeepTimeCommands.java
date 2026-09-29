// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.gametest.DeepTimeTestReporter;
import com.minecraftindustrialcomplex.deeptime.gametest.GameTestSupport;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import com.mojang.brigadier.context.CommandContext;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.levelgen.Heightmap;

/**
 * {@code /deeptime ...} for tests and debugging (plan §7.16). Every probe prints one JSON line
 * prefixed {@code [deeptime-probe]} to the command source and the log, which the smoke harness
 * and gametest-planet.sh parse. Permission level 2.
 */
public final class DeepTimeCommands {

    public static final String PROBE_PREFIX = "[deeptime-probe] ";

    private DeepTimeCommands() {}

    public static void register(CommandDispatcher<CommandSourceStack> dispatcher) {
        dispatcher.register(Commands.literal("deeptime").requires(s -> s.hasPermission(2))
                .then(Commands.literal("probe")
                        .then(Commands.literal("column")
                                .then(Commands.argument("x", IntegerArgumentType.integer())
                                        .then(Commands.argument("z", IntegerArgumentType.integer())
                                                .executes(DeepTimeCommands::probeColumn))))
                        .then(Commands.literal("tw").executes(DeepTimeCommands::probeTw))
                        .then(Commands.literal("palette").executes(c -> probePalette(c.getSource()))))
                .then(Commands.literal("gametest").requires(s -> GameTestSupport.enabled())
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

    static int emit(CommandSourceStack source, JsonObject o) {
        String line = PROBE_PREFIX + o;
        DeepTime.LOGGER.info(line);
        source.sendSuccess(() -> Component.literal(line), false);
        return 1;
    }
}
