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
import net.minecraft.world.level.chunk.CarvingMask;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.levelgen.GenerationStep;
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
     *                    plants, leaves and logs, and under water the blocks of a coral reef
     * @param groundBlock that block
     * @param water       water (or the ice on it) stands over the ground
     * @param waterTop    the y of the top water or ice block, when {@code water}
     * @param snowLayer   a snow layer lies on the ground
     * @param biome       the biome the chunk stores at the ground
     */
    public record RealColumn(int ground, BlockState groundBlock, boolean water, int waterTop, boolean snowLayer, Holder<Biome> biome) {}

    /** What is over the ground rather than the ground: plants, trees, snow layers, anything a player walks through. */
    static boolean overGround(BlockState s) {
        return s.isAir() || s.is(BlockTags.LEAVES) || s.is(BlockTags.LOGS) || s.is(Blocks.MANGROVE_ROOTS) || s.is(Blocks.SNOW) || !s.blocksMotion()
                || s.is(Blocks.BAMBOO) || s.is(Blocks.CACTUS) || s.is(Blocks.MUSHROOM_STEM) || s.is(Blocks.RED_MUSHROOM_BLOCK)
                || s.is(Blocks.BROWN_MUSHROOM_BLOCK) || s.is(Blocks.BEE_NEST) || s.is(Blocks.PUMPKIN) || s.is(Blocks.MELON);
    }

    /**
     * A block of a coral reef: the warm oceans' feature, up to several blocks over the sea bed. Live
     * coral only (the tags hold no dead coral): a dead coral block is ground where a surface rule
     * paints it (Expanded Ecosphere's warm ocean floors from y 58 to 62), and far terrain draws it.
     */
    static boolean coral(BlockState s) {
        return s.is(BlockTags.CORAL_BLOCKS) || s.is(BlockTags.CORALS) || s.is(BlockTags.WALL_CORALS);
    }

    /**
     * Whether a carver opened block (x, y, z) of a chunk still being generated: it is open (air or a
     * fluid) and in the chunk's air carving mask. False once the chunk is promoted (the masks are
     * gone). Deep Time's own passes put the blocks they guard in that mask too, but those stay solid
     * or keep their water over the bed, so an open block there at the height of far terrain's ground
     * is a carver's cut.
     */
    public static boolean carved(ChunkAccess chunk, int x, int y, int z) {
        if (!(chunk instanceof ProtoChunk proto) || y < chunk.getMinBuildHeight() || y >= chunk.getMaxBuildHeight())
            return false;
        CarvingMask air = proto.getCarvingMask(GenerationStep.Carving.AIR);
        if (air == null || !air.get(x & 15, y, z & 15))
            return false;
        BlockState s = chunk.getBlockState(new BlockPos(x, y, z));
        return s.isAir() || !s.getFluidState().isEmpty();
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
            if (water && (!s.blocksMotion() || coral(s)))
                continue;   // seagrass, kelp, a coral reef (a feature far terrain does not draw)
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
        public int waterAgree, waterBoth, waterTopExact, snowAgree, biomeSame, groundBlockSame, landGroundBlockSame, landHeightWithin1;
        public String worstHeight = "";
        private final Map<String, Integer> blockPairs = new HashMap<>(), biomePairs = new HashMap<>();

        /** Columns where a feature of ice stands in the water far terrain shows (an iceberg): counted apart. */
        public int icebergs;

        /**
         * Columns whose ground a carver cut more than two blocks below far terrain's (a canyon under a
         * shallow sea, a cave mouth): counted apart. Far terrain draws the column rule the chunk is
         * built by, not the carvers, which are random per chunk; the caller bounds their share.
         */
        public int carved;
        public String carvedAt = "";

        public void add(FarTerrain.Column far, RealColumn real) {
            add(far, real, null);
        }

        /**
         * As {@link #add(FarTerrain.Column, RealColumn)}; {@code chunk}, while still generating, tells a carver's cut apart:
         * real ground more than the one block of tolerance under far terrain's, where a carver opened far terrain's
         * ground block. (Until 2026-10-29 the cut had to be over 2 blocks, so a carver taking the top two blocks of a
         * beach counted as far terrain's miss: the -PwithMic seam rows of de67c78f's test planet, far 66, real 64.)
         */
        public void add(FarTerrain.Column far, RealColumn real, ChunkAccess chunk) {
            if (chunk != null && real.ground() < far.ground() - 1 && carved(chunk, far.x(), far.ground(), far.z())) {
                carved++;
                if (carvedAt.length() < 200)
                    carvedAt += (carvedAt.isEmpty() ? "" : " ") + "(" + far.x() + ", " + far.z() + ") far " + far.ground() + " real " + real.ground();
                return;
            }
            // An iceberg is a feature (vanilla's, in frozen oceans): packed or blue ice from the sea
            // floor to above the surface, where the world data and far terrain have open water.
            if (far.water() && !real.water() && real.ground() > far.ground()
                    && (real.groundBlock().is(Blocks.PACKED_ICE) || real.groundBlock().is(Blocks.BLUE_ICE)
                    || real.groundBlock().is(Blocks.SNOW_BLOCK) || real.groundBlock().is(Blocks.ICE))) {
                icebergs++;
                return;
            }
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
                if (d <= 1)
                    landHeightWithin1++;
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
            o.addProperty("icebergs_set_aside", icebergs);
            o.addProperty("carved_set_aside", carved);
            if (carved > 0)
                o.addProperty("carved_at", carvedAt);
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
            o.addProperty("land_height_within_1", landHeightWithin1);
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
