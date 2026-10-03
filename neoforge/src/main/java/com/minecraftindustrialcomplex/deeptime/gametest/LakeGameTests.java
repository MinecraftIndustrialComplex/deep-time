// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.LakeIce;
import com.minecraftindustrialcomplex.deeptime.worlddata.Playa;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * Lake ice and salt flats on the real planet with the pack's biomes and features
 * ({@code deeptime:test_planet_real}; Ben, decisions log 2026-10-01): a lake the climate freezes
 * ({@link LakeIce}) has ice on it whatever the biome, a lake it keeps open has water on top even in
 * a biome cold enough to freeze water (vanilla's {@code freeze_top_layer} ran over it), and the
 * floor of a playa is the salt the bake's {@code playa} layer puts there.
 *
 * <p>The test asserts the rule on whatever the planet holds (a planet may have no frozen lake, no
 * open lake in a cold biome, or no playa: the log line says how many of each it checked);
 * {@code DEEPTIME_TEST_PLANET_SET="seed=…"} bakes another planet.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class LakeGameTests {

    /** At most this many columns of each kind are generated and read. */
    static final int MOST = 24;

    private LakeGameTests() {}

    private record Spot(int x, int z, int top) {}

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 4800)
    public static void lakeIceAndSaltFollowTheClimate(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_REAL);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_REAL.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain w)) {
            helper.fail("test_planet_real is not generating from world data");
            return;
        }
        WorldData wd = w.data();
        LakeIce ice = LakeIce.of(wd);
        if (ice == null || !wd.has(WorldData.WATER_LEVEL) || !wd.has(Playa.LAYER)) {
            helper.fail("the real test planet has no lake-ice rule, water_level or playa layer");
            return;
        }
        Water.Spec spec = Water.Spec.of(wd, g.waterReach());
        // Lake columns a block or more deep, frozen and open, and playa cells, spread over the planet.
        List<Spot> frozen = new ArrayList<>(), open = new ArrayList<>(), salt = new ArrayList<>();
        RasterSampler levels = wd.layer(WorldData.WATER_LEVEL), playa = wd.layer(Playa.LAYER);
        var grid = levels.desc().grid();
        long lakeCells = 0, playaCells = 0;
        for (int j = 0; j < grid.h(); j++) {
            for (int i = 0; i < grid.w(); i++) {
                boolean lake = !Double.isNaN(levels.get(i, j)), flat = playa.get(i, j) > 0;
                if (lake)
                    lakeCells++;
                if (flat)
                    playaCells++;
                if ((!lake && !flat) || Math.floorMod(i * 31 + j * 17, 7) != 0)
                    continue;
                int x = (int) (grid.originBlockX() + (long) i * grid.r()), z = (int) (grid.originBlockZ() + (long) j * grid.r());
                if (flat && salt.size() < MOST) {
                    salt.add(new Spot(x, z, 0));
                    continue;
                }
                if (!lake)
                    continue;
                Water.Column col = Water.column(wd, spec, x, z);
                if (col.kind() != Water.Kind.LAKE || col.top() == null || col.top() <= col.bed())
                    continue;
                List<Spot> to = ice.frozen(x, z) ? frozen : open;
                if (to.size() < MOST)
                    to.add(new Spot(x, z, col.top()));
            }
        }
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        StringBuilder detail = new StringBuilder();
        int wrong = 0, frozenWarm = 0, openCold = 0, frozenRead = 0, openRead = 0, saltRead = 0;
        for (Spot s : frozen) {
            BlockState top = level.getChunk(s.x() >> 4, s.z() >> 4).getBlockState(pos.set(s.x(), s.top(), s.z()));
            boolean cold = level.getBiome(pos).value().coldEnoughToSnow(pos);
            if (top.getFluidState().is(FluidTags.WATER)) {
                wrong++;
                detail.append(" open-but-frozen(").append(s.x()).append(',').append(s.top()).append(',').append(s.z()).append(')');
            } else if (top.is(Blocks.ICE)) {
                frozenRead++;
                if (!cold)
                    frozenWarm++;
            }   // else the generated ground reaches the lake's top there: dry shore
        }
        for (Spot s : open) {
            BlockState top = level.getChunk(s.x() >> 4, s.z() >> 4).getBlockState(pos.set(s.x(), s.top(), s.z()));
            boolean cold = level.getBiome(pos).value().coldEnoughToSnow(pos);
            if (top.is(Blocks.ICE)) {
                wrong++;
                detail.append(" iced-but-open(").append(s.x()).append(',').append(s.top()).append(',').append(s.z()).append(cold ? " cold biome)" : ")");
            } else if (top.getFluidState().is(FluidTags.WATER)) {
                openRead++;
                if (cold)
                    openCold++;
            }
        }
        PaletteRules.Result<BlockState>[] found = RockPalette.get().forClasses(List.of(Playa.rockClass(wd.manifest().notes())));
        BlockState saltBlock = found.length == 0 || found[0] == null ? null : found[0].pick(0L);
        for (Spot s : salt) {
            LevelChunk chunk = level.getChunk(s.x() >> 4, s.z() >> 4);
            // The ground's top block: under air, plants and a snow layer.
            int y = chunk.getHeight(Heightmap.Types.WORLD_SURFACE, s.x() & 15, s.z() & 15);
            BlockState b = chunk.getBlockState(pos.set(s.x(), y, s.z()));
            while (y > level.getMinBuildHeight() && (b.isAir() || !b.isCollisionShapeFullBlock(level, pos)) && b.getFluidState().isEmpty())
                b = chunk.getBlockState(pos.set(s.x(), --y, s.z()));
            if (!b.getFluidState().isEmpty() || b.is(Blocks.SNOW_BLOCK) || b.is(Blocks.PACKED_ICE) || b.is(Blocks.BLUE_ICE))
                continue;   // under water or under the ice cover: not this pass's surface
            saltRead++;
            if (saltBlock == null || b != saltBlock) {
                wrong++;
                detail.append(" not-salt(").append(s.x()).append(',').append(y).append(',').append(s.z()).append(' ')
                        .append(BuiltInRegistries.BLOCK.getKey(b.getBlock())).append(')');
            }
        }
        DeepTime.LOGGER.info("[gametest] lakeIceAndSaltFollowTheClimate lakeCells={} playaCells={} frozenLakeColumns={} (iceRead={} inWarmBiome={}) "
                        + "openLakeColumns={} (waterRead={} inColdBiome={}) playaColumns={} (saltRead={} salt={}) threshold={} wrong={}{}",
                lakeCells, playaCells, frozen.size(), frozenRead, frozenWarm, open.size(), openRead, openCold, salt.size(), saltRead,
                saltBlock == null ? "none" : BuiltInRegistries.BLOCK.getKey(saltBlock.getBlock()), ice.threshold(), wrong, detail);
        helper.assertTrue(wrong == 0, wrong + " columns break the lake-ice or salt rule:" + detail);
        helper.succeed();
    }
}
