// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.farterrain;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.QuartPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.tags.BlockTags;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.material.Fluids;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Far terrain against real chunks (plan §10.5 P3-04): reads a generated chunk's column the way far
 * terrain describes one (the ground under the vegetation, the water over it, the snow layer, the
 * biome) and tallies how the two agree. The GameTest and the {@code /deeptime probe far|dh}
 * commands share it; nothing here asserts, the callers decide what a tally must hold.
 */
public final class FarTerrainCheck {

    private FarTerrainCheck() {}

    /**
     * A generated column.
     *
     * @param ground      the y of the top block of the ground: under water, ice on water, snow layers,
     *                    plants, leaves and logs
     * @param groundBlock that block
     * @param water       water (or the ice on it) stands over the ground
     * @param waterTop    the y of the top water or ice block, when {@code water}
     * @param snowLayer   a snow layer lies on the ground
     * @param biome       the biome the chunk stores at the ground
     */
    public record RealColumn(int ground, BlockState groundBlock, boolean water, int waterTop, boolean snowLayer, Holder<Biome> biome) {}

    /** What is over the ground rather than the ground: plants, trees, snow layers, anything a player walks through. */
    static boolean overGround(BlockState s) {
        return s.isAir() || s.is(BlockTags.LEAVES) || s.is(BlockTags.LOGS) || s.is(Blocks.SNOW) || !s.blocksMotion()
                || s.is(Blocks.BAMBOO) || s.is(Blocks.CACTUS) || s.is(Blocks.MUSHROOM_STEM) || s.is(Blocks.RED_MUSHROOM_BLOCK)
                || s.is(Blocks.BROWN_MUSHROOM_BLOCK) || s.is(Blocks.BEE_NEST) || s.is(Blocks.PUMPKIN) || s.is(Blocks.MELON);
    }

    static boolean waterLike(BlockState s) {
        return s.getFluidState().is(Fluids.WATER) || s.getFluidState().is(Fluids.FLOWING_WATER);
    }

    /** Reads column (x, z) of {@code chunk} (which holds it). */
    public static RealColumn real(ChunkAccess chunk, int x, int z) {
        int minY = chunk.getMinBuildHeight();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int y = Math.min(chunk.getMaxBuildHeight() - 1, chunk.getHeight(Heightmap.Types.WORLD_SURFACE, x, z));
        boolean water = false, snow = false;
        int waterTop = Integer.MIN_VALUE;
        for (; y >= minY; y--) {
            BlockState s = chunk.getBlockState(pos.set(x, y, z));
            if (waterLike(s) || s.is(Blocks.ICE) && (water || waterLike(chunk.getBlockState(pos.set(x, y - 1, z))))) {
                if (!water)
                    waterTop = y;
                water = true;
                continue;
            }
            if (water && !s.blocksMotion())
                continue;   // seagrass, kelp
            if (s.is(Blocks.SNOW))
                snow = true;
            if (!water && overGround(s))
                continue;
            break;
        }
        BlockState ground = y >= minY ? chunk.getBlockState(pos.set(x, y, z)) : Blocks.AIR.defaultBlockState();
        int by = Math.max(minY, y);
        Holder<Biome> biome = chunk.getNoiseBiome(QuartPos.fromBlock(x), QuartPos.fromBlock(water ? waterTop : by), QuartPos.fromBlock(z));
        return new RealColumn(y, ground, water, waterTop, snow, biome);
    }

    static String id(BlockState s) {
        return BuiltInRegistries.BLOCK.getKey(s.getBlock()).toString();
    }

    /** How far terrain and generated chunks agree over a set of columns. */
    public static final class Tally {
        public int columns, land, heightExact, heightWithin1, heightWithin2, heightMax;
        public int waterAgree, waterBoth, waterTopExact, snowAgree, biomeSame, groundBlockSame, landGroundBlockSame;
        public String worstHeight = "";
        private final Map<String, Integer> blockPairs = new HashMap<>(), biomePairs = new HashMap<>();

        public void add(FarTerrain.Column far, RealColumn real) {
            columns++;
            boolean landBoth = !far.water() && !real.water();
            int d = Math.abs(far.ground() - real.ground());
            if (d == 0)
                heightExact++;
            if (d <= 1)
                heightWithin1++;
            if (d <= 2)
                heightWithin2++;
            if (d > heightMax) {
                heightMax = d;
                worstHeight = "(" + far.x() + ", " + far.z() + ") far " + far.ground() + " real " + real.ground();
            }
            if (far.water() == real.water())
                waterAgree++;
            if (far.water() && real.water()) {
                waterBoth++;
                if (far.waterTop() == real.waterTop())
                    waterTopExact++;
            }
            boolean farSnow = far.topBlock().is(Blocks.SNOW);
            if (farSnow == real.snowLayer())
                snowAgree++;
            if (far.biome().equals(real.biome()))
                biomeSame++;
            else
                biomePairs.merge(name(far.biome()) + " -> " + name(real.biome()), 1, Integer::sum);
            boolean same = far.groundBlock().getBlock() == real.groundBlock().getBlock();
            if (same)
                groundBlockSame++;
            else
                blockPairs.merge(id(far.groundBlock()) + " -> " + id(real.groundBlock()) + " in " + name(real.biome()), 1, Integer::sum);
            if (landBoth) {
                land++;
                if (same)
                    landGroundBlockSame++;
            }
        }

        static String name(Holder<Biome> b) {
            return b.unwrapKey().map(k -> k.location().toString()).orElse("?");
        }

        public double share(int count) {
            return columns == 0 ? 0.0 : (double) count / columns;
        }

        private static JsonArray top(Map<String, Integer> pairs, int n) {
            List<Map.Entry<String, Integer>> list = new java.util.ArrayList<>(pairs.entrySet());
            list.sort((a, b) -> b.getValue() != a.getValue().intValue() ? b.getValue() - a.getValue() : a.getKey().compareTo(b.getKey()));
            JsonArray out = new JsonArray();
            for (int i = 0; i < Math.min(n, list.size()); i++)
                out.add(list.get(i).getValue() + "x " + list.get(i).getKey());
            return out;
        }

        public JsonObject toJson() {
            JsonObject o = new JsonObject();
            o.addProperty("columns", columns);
            o.addProperty("height_exact", heightExact);
            o.addProperty("height_within_1", heightWithin1);
            o.addProperty("height_within_2", heightWithin2);
            o.addProperty("height_max_abs", heightMax);
            o.addProperty("height_worst", worstHeight);
            o.addProperty("water_agree", waterAgree);
            o.addProperty("water_both", waterBoth);
            o.addProperty("water_top_exact", waterTopExact);
            o.addProperty("snow_layer_agree", snowAgree);
            o.addProperty("biome_same", biomeSame);
            o.addProperty("ground_block_same", groundBlockSame);
            o.addProperty("land", land);
            o.addProperty("land_ground_block_same", landGroundBlockSame);
            o.add("ground_block_differences", top(blockPairs, 8));
            o.add("biome_differences", top(biomePairs, 4));
            return o;
        }

        @Override
        public String toString() {
            return toJson().toString();
        }
    }
}
