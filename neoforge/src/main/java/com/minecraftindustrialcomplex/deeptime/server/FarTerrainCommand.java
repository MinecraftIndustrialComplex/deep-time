// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhApiBinding;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhFarTerrain;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrain;
import com.minecraftindustrialcomplex.deeptime.farterrain.FarTerrainCheck;
import com.mojang.brigadier.CommandDispatcher;
import com.mojang.brigadier.arguments.IntegerArgumentType;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.Commands;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * The far-terrain probes (plan §10.5 P3-04, §7.16), merged into {@code /deeptime probe}:
 *
 * <ul>
 *   <li>{@code /deeptime probe far <x> <z>}: the far-terrain column at a block and the generated
 *       chunk's column there (the chunk is loaded or generated).</li>
 *   <li>{@code /deeptime probe far bench <x> <z> <detail> <sections>}: builds that many sections
 *       (64 × 64 columns of 2^detail blocks) starting at the block, on the calling thread, and
 *       reports the time a section takes.</li>
 *   <li>{@code /deeptime probe dh}: whether Distant Horizons is installed, which API was bound, and
 *       for every bound level whether the override registered, how many sections Distant Horizons
 *       has asked for and how long they took.</li>
 *   <li>{@code /deeptime probe dh check <x> <z> <n>}: n × n chunks, four columns in each,
 *       from block (x, z): first what Distant Horizons holds for each (its own store, so only
 *       after it generated the area, e.g. {@code /dh pregen start}), then far terrain's column,
 *       then the generated chunk. {@code stored} says how many columns DH held and how many of
 *       them are far terrain's layer for layer; {@code vs_real} is the agreement with real chunks.
 *       The stored columns are read before any chunk is generated: a generated chunk replaces
 *       DH's data for its columns.</li>
 * </ul>
 */
public final class FarTerrainCommand {

    private FarTerrainCommand() {}

    /** The four columns `probe dh check` samples in each chunk. */
    private static final int[][] OFFSETS = {{5, 9}, {13, 2}, {1, 14}, {9, 6}};

    public static void register(CommandDispatcher<CommandSourceStack> dispatcher) {
        dispatcher.register(Commands.literal("deeptime").requires(s -> s.hasPermission(2))
                .then(Commands.literal("probe")
                        .then(Commands.literal("far")
                                .then(Commands.literal("land")
                                        .then(Commands.argument("x", IntegerArgumentType.integer())
                                                .then(Commands.argument("z", IntegerArgumentType.integer())
                                                        .executes(c -> land(c.getSource(), IntegerArgumentType.getInteger(c, "x"),
                                                                IntegerArgumentType.getInteger(c, "z"))))))
                                .then(Commands.literal("bench")
                                        .then(Commands.argument("x", IntegerArgumentType.integer())
                                                .then(Commands.argument("z", IntegerArgumentType.integer())
                                                        .then(Commands.argument("detail", IntegerArgumentType.integer(0, 16))
                                                                .then(Commands.argument("sections", IntegerArgumentType.integer(1, 4096))
                                                                        .executes(c -> bench(c.getSource(), IntegerArgumentType.getInteger(c, "x"),
                                                                                IntegerArgumentType.getInteger(c, "z"),
                                                                                IntegerArgumentType.getInteger(c, "detail"),
                                                                                IntegerArgumentType.getInteger(c, "sections"))))))))
                                .then(Commands.argument("x", IntegerArgumentType.integer())
                                        .then(Commands.argument("z", IntegerArgumentType.integer())
                                                .executes(c -> far(c.getSource(), IntegerArgumentType.getInteger(c, "x"),
                                                        IntegerArgumentType.getInteger(c, "z"))))))
                        .then(Commands.literal("dh")
                                .executes(c -> dh(c.getSource()))
                                .then(Commands.literal("check")
                                        .then(Commands.argument("x", IntegerArgumentType.integer())
                                                .then(Commands.argument("z", IntegerArgumentType.integer())
                                                        .then(Commands.argument("n", IntegerArgumentType.integer(1, 64))
                                                                .executes(c -> dhCheck(c.getSource(), IntegerArgumentType.getInteger(c, "x"),
                                                                        IntegerArgumentType.getInteger(c, "z"),
                                                                        IntegerArgumentType.getInteger(c, "n"))))))))));
    }

    private static JsonObject header(String probe, ServerLevel level) {
        JsonObject o = new JsonObject();
        o.addProperty("probe", probe);
        o.addProperty("level", level.dimension().location().toString());
        return o;
    }

    private static String id(BlockState s) {
        return BuiltInRegistries.BLOCK.getKey(s.getBlock()).toString();
    }

    static JsonObject describe(FarTerrain.Column c) {
        JsonObject o = new JsonObject();
        o.addProperty("ground", c.ground());
        o.addProperty("ground_block", id(c.groundBlock()));
        o.addProperty("top", c.top());
        o.addProperty("top_block", id(c.topBlock()));
        o.addProperty("water", c.water());
        if (c.water())
            o.addProperty("water_top", c.waterTop());
        o.addProperty("biome", c.biome().unwrapKey().map(k -> k.location().toString()).orElse("?"));
        JsonArray layers = new JsonArray();
        c.layers().forEach(layers::add);
        o.add("layers", layers);
        return o;
    }

    static JsonObject describe(FarTerrainCheck.RealColumn r) {
        JsonObject o = new JsonObject();
        o.addProperty("ground", r.ground());
        o.addProperty("ground_block", id(r.groundBlock()));
        o.addProperty("water", r.water());
        if (r.water())
            o.addProperty("water_top", r.waterTop());
        o.addProperty("snow_layer", r.snowLayer());
        o.addProperty("biome", r.biome().unwrapKey().map(k -> k.location().toString()).orElse("?"));
        return o;
    }

    static int far(CommandSourceStack source, int x, int z) {
        ServerLevel level = source.getLevel();
        JsonObject o = header("far", level);
        o.addProperty("x", x);
        o.addProperty("z", z);
        Optional<FarTerrain> terrain = FarTerrain.of(level);
        o.addProperty("deep_time_level", terrain.isPresent());
        if (terrain.isPresent()) {
            o.add("far", describe(terrain.get().column(x, z)));
            o.add("real", describe(FarTerrainCheck.real(level.getChunk(x >> 4, z >> 4), x, z)));
        }
        return DeepTimeCommands.emit(source, o);
    }

    /**
     * {@code /deeptime probe far land <x> <z>}: the share of dry columns in the 512 × 512 blocks
     * around a block, from far terrain alone (no chunk is generated), so a check can pick land.
     */
    static int land(CommandSourceStack source, int x, int z) {
        ServerLevel level = source.getLevel();
        JsonObject o = header("far-land", level);
        o.addProperty("x", x);
        o.addProperty("z", z);
        Optional<FarTerrain> terrain = FarTerrain.of(level);
        o.addProperty("deep_time_level", terrain.isPresent());
        if (terrain.isPresent()) {
            int[] n = new int[2];
            terrain.get().build(x - 256, z - 256, 8, 64, (relX, relZ, count, bottom, top, block, material, light, biome) -> {
                n[0]++;
                boolean water = false;
                for (int k = 0; k < count; k++)
                    water |= material[k] == com.minecraftindustrialcomplex.deeptime.worlddata.FarColumn.WATER
                            || material[k] == com.minecraftindustrialcomplex.deeptime.worlddata.FarColumn.WATER_ICE;
                if (!water)
                    n[1]++;
            });
            o.addProperty("land_share", Math.round(1000.0 * n[1] / n[0]) / 1000.0);
        }
        return DeepTimeCommands.emit(source, o);
    }

    static int bench(CommandSourceStack source, int x, int z, int detail, int sections) {
        ServerLevel level = source.getLevel();
        JsonObject o = header("far-bench", level);
        Optional<FarTerrain> terrain = FarTerrain.of(level);
        o.addProperty("deep_time_level", terrain.isPresent());
        if (terrain.isPresent()) {
            int step = 1 << detail, side = (int) Math.ceil(Math.sqrt(sections));
            long[] layers = {0};
            FarTerrain.Sink sink = (relX, relZ, count, bottom, top, block, material, light, biome) -> layers[0] += count;
            // One pass to warm the tile caches and the JIT, one timed.
            double best = Double.MAX_VALUE;
            for (int pass = 0; pass < 2; pass++) {
                layers[0] = 0;
                long t0 = System.nanoTime();
                for (int k = 0; k < sections; k++)
                    terrain.get().build(x + (k % side) * 64 * step, z + (k / side) * 64 * step, step, 64, sink);
                best = Math.min(best, (System.nanoTime() - t0) / 1e6 / sections);
            }
            o.addProperty("detail", detail);
            o.addProperty("sections", sections);
            o.addProperty("ms_per_section", Math.round(best * 1000.0) / 1000.0);
            o.addProperty("layers_per_column", Math.round(layers[0] / (sections * 4096.0) * 100.0) / 100.0);
        }
        return DeepTimeCommands.emit(source, o);
    }

    static int dh(CommandSourceStack source) {
        JsonObject o = header("dh", source.getLevel());
        JsonArray spawn = new JsonArray();
        spawn.add(source.getLevel().getSharedSpawnPos().getX());
        spawn.add(source.getLevel().getSharedSpawnPos().getZ());
        o.add("spawn", spawn);   // the smoke harness checks an area beside the spawn chunks
        o.addProperty("installed", DhFarTerrain.installed());
        o.addProperty("enabled", DhFarTerrain.enabled());
        if (DhFarTerrain.installed() && DhFarTerrain.enabled()) {
            Optional<DhApiBinding> api = DhFarTerrain.api();
            o.addProperty("api_bound", api.isPresent());
            if (api.isPresent()) {
                o.addProperty("api_naming", api.get().naming());
                o.addProperty("version", api.get().version());
            } else {
                o.addProperty("api_error", DhFarTerrain.unavailable());
            }
            JsonArray levels = new JsonArray();
            DhFarTerrain.all(source.getServer()).forEach((key, b) -> {
                JsonObject l = new JsonObject();
                l.addProperty("level", key.location().toString());
                l.addProperty("dh_level", b.dhLevel());
                l.addProperty("registered", b.registered());
                if (!b.registered())
                    l.addProperty("message", b.message());
                if (b.generator() != null) {
                    l.addProperty("max_detail", b.generator().maxDetail());
                    l.addProperty("sections", b.generator().sections());
                    l.addProperty("failures", b.generator().failures());
                    if (b.generator().firstFailure() != null)
                        l.addProperty("first_failure", b.generator().firstFailure());
                    if (b.generator().sections() > 0) {
                        l.addProperty("mean_ms_per_section", Math.round(b.generator().meanMillis() * 1000.0) / 1000.0);
                        l.addProperty("points_per_column", Math.round(b.generator().meanPointsPerColumn() * 100.0) / 100.0);
                        l.addProperty("by_detail", b.generator().byDetail());
                    }
                }
                levels.add(l);
            });
            o.add("levels", levels);
        }
        return DeepTimeCommands.emit(source, o);
    }

    /** Whether DH's stored column is far terrain's, layer for layer (neighbouring layers of one block merged on both sides). */
    static boolean sameLayers(List<DhApiBinding.StoredPoint> stored, FarTerrain terrain, int x, int z, StringBuilder why) {
        List<int[]> a = new ArrayList<>();
        List<Object> ab = new ArrayList<>();
        List<DhApiBinding.StoredPoint> sorted = new ArrayList<>(stored);
        sorted.sort((p, q) -> Integer.compare(p.bottomY(), q.bottomY()));
        for (DhApiBinding.StoredPoint p : sorted) {
            Object pb = p.block() == null ? Blocks.AIR.defaultBlockState() : p.block();   // DH's air wraps nothing
            if (!ab.isEmpty() && ab.get(ab.size() - 1) == pb && a.get(a.size() - 1)[1] == p.bottomY())
                a.get(a.size() - 1)[1] = p.topY();
            else {
                a.add(new int[]{p.bottomY(), p.topY()});
                ab.add(pb);
            }
        }
        List<int[]> f = new ArrayList<>();
        List<Object> fb = new ArrayList<>();
        Holder<?>[] biome = new Holder<?>[1];
        terrain.build(x, z, 1, 1, (relX, relZ, count, bottom, top, block, material, light, b) -> {
            biome[0] = b;
            for (int k = 0; k < count; k++) {
                if (!fb.isEmpty() && fb.get(fb.size() - 1) == block[k] && f.get(f.size() - 1)[1] == bottom[k])
                    f.get(f.size() - 1)[1] = top[k];
                else {
                    f.add(new int[]{bottom[k], top[k]});
                    fb.add(block[k]);
                }
            }
        });
        if (a.size() != f.size()) {
            why.append("(").append(x).append(", ").append(z).append("): ").append(a.size()).append(" stored layers, ").append(f.size())
                    .append(" far");
            return false;
        }
        for (int k = 0; k < a.size(); k++) {
            if (a.get(k)[0] != f.get(k)[0] || a.get(k)[1] != f.get(k)[1] || ab.get(k) != fb.get(k)) {
                why.append("(").append(x).append(", ").append(z).append(") layer ").append(k).append(": stored ").append(a.get(k)[0])
                        .append("..").append(a.get(k)[1]).append(" ").append(ab.get(k)).append(", far ").append(f.get(k)[0]).append("..")
                        .append(f.get(k)[1]).append(" ").append(fb.get(k));
                return false;
            }
        }
        for (DhApiBinding.StoredPoint p : sorted) {
            if (p.biome() != null && !p.biome().equals(biome[0])) {
                why.append("(").append(x).append(", ").append(z).append("): stored biome ").append(p.biome()).append(", far ").append(biome[0]);
                return false;
            }
        }
        return true;
    }

    static int dhCheck(CommandSourceStack source, int x0, int z0, int n) {
        ServerLevel level = source.getLevel();
        JsonObject o = header("dh-check", level);
        DhFarTerrain.Bound bound = DhFarTerrain.installed() ? DhFarTerrain.bound(level.dimension()) : null;
        o.addProperty("bound", bound != null);
        o.addProperty("registered", bound != null && bound.registered());
        Optional<FarTerrain> terrain = FarTerrain.of(level);
        o.addProperty("deep_time_level", terrain.isPresent());
        if (terrain.isEmpty())
            return DeepTimeCommands.emit(source, o);
        FarTerrain far = terrain.get();
        int spacing = 16;   // n x n chunks, four columns in each
        // 1. What Distant Horizons holds, before any chunk is generated here.
        int held = 0, equal = 0, generated = 0;
        int[] coarser = new int[4];
        String[] example = new String[1];
        StringBuilder why = new StringBuilder();
        if (bound != null && bound.levelWrapper() != null && DhFarTerrain.api().isPresent()) {
            DhApiBinding api = DhFarTerrain.api().get();
            for (int j = 0; j < n; j++) {
                for (int i = 0; i < 4 * n; i++) {
                    int x = x0 + (i / 4) * spacing + OFFSETS[i % 4][0], z = z0 + j * spacing + OFFSETS[i % 4][1];
                    if (level.getChunkSource().getChunkNow(x >> 4, z >> 4) != null) {
                        generated++;   // a loaded chunk: Distant Horizons holds the chunk's own column there
                        continue;
                    }
                    try {
                        List<DhApiBinding.StoredPoint> stored = api.storedColumn(bound.levelWrapper(), x, z);
                        if (stored == null || stored.isEmpty()
                                || stored.stream().allMatch(p -> p.block() instanceof BlockState s && s.is(Blocks.AIR) || p.block() == null))
                            continue;
                        held++;
                        StringBuilder w = new StringBuilder();
                        if (sameLayers(stored, far, x, z, w)) {
                            equal++;
                        } else {
                            if (why.length() < 400)
                                why.append(w).append("; ");
                            // Is it another column's? One Distant Horizons merged it with (the
                            // column at the corner of its 2, 4 or 8 block cell), or a neighbour.
                            for (int k = 1; k <= 3; k++) {
                                int mask = ~((1 << k) - 1);
                                if (sameLayers(stored, far, x & mask, z & mask, new StringBuilder())) {
                                    coarser[k]++;
                                    break;
                                }
                            }
                            if (example[0] == null) {
                                StringBuilder sb = new StringBuilder("(" + x + ", " + z + ") stored:");
                                stored.forEach(p -> sb.append(" ").append(p.bottomY()).append("..").append(p.topY()).append(" ")
                                        .append(p.block() instanceof BlockState s ? id(s) : "air"));
                                sb.append(" | far: ").append(String.join(", ", far.column(x, z).layers()));
                                example[0] = sb.toString();
                            }
                        }
                    } catch (ReflectiveOperationException | RuntimeException e) {
                        if (why.length() < 400)
                            why.append(e).append("; ");
                    }
                }
            }
        }
        JsonObject stored = new JsonObject();
        stored.addProperty("asked", 4 * n * n - generated);
        stored.addProperty("skipped_loaded_chunks", generated);
        stored.addProperty("held", held);
        stored.addProperty("equal_to_far_terrain", equal);
        stored.addProperty("equal_to_the_2_block_cell_corner", coarser[1]);
        stored.addProperty("equal_to_the_4_block_cell_corner", coarser[2]);
        stored.addProperty("equal_to_the_8_block_cell_corner", coarser[3]);
        if (why.length() > 0)
            stored.addProperty("differences", why.toString());
        if (example[0] != null)
            stored.addProperty("example", example[0]);
        o.add("stored", stored);
        // 2. Far terrain against the generated chunks.
        FarTerrainCheck.Tally tally = new FarTerrainCheck.Tally();
        for (int j = 0; j < n; j++) {
            for (int i = 0; i < 4 * n; i++) {
                int x = x0 + (i / 4) * spacing + OFFSETS[i % 4][0], z = z0 + j * spacing + OFFSETS[i % 4][1];
                tally.add(far.column(x, z), FarTerrainCheck.real(level.getChunk(x >> 4, z >> 4), x, z));
            }
        }
        o.add("vs_real", tally.toJson());
        return DeepTimeCommands.emit(source, o);
    }
}
