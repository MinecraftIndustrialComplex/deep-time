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
import com.minecraftindustrialcomplex.deeptime.worldgen.Geothermal;
import com.minecraftindustrialcomplex.deeptime.worldgen.LavaFlowPalette;
import com.minecraftindustrialcomplex.deeptime.worldgen.LavaFlows;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.tags.BlockTags;
import net.minecraft.tags.FluidTags;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.fml.ModList;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * Lake ice and salt flats on the real planet with the pack's biomes and features
 * ({@code deeptime:test_planet_real}; Ben, decisions log 2026-10-01): a lake the climate freezes
 * ({@link LakeIce}) has ice on it whatever the biome, a lake it keeps open has water on top even in
 * a biome cold enough to freeze water (vanilla's {@code freeze_top_layer} ran over it), and the
 * floor of a playa is the salt the bake's {@code playa} layer puts there, except where an active
 * volcano's lava flow lies on it: a flow is laid after the salt ({@code LavaFlowPass} runs after
 * {@code PlayaPass}), as a flow that ran onto a salt flat stands over it (ledger VOL-162).
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

    /**
     * The flat checked round a geothermal site: its claim's reach and this many blocks (the field's role biome reached 4
     * beyond the claim, BIO-110, and its lake features 14 and more beyond that).
     */
    static final int RING = 20;

    private record Spot(int x, int z, int top) {}

    /**
     * A column's ground: its top block under air, plants, a snow layer and any tree (leaves and logs: a neighbour's
     * canopy overhangs a column it does not stand in, and the salt is what lies under it), and the block on it.
     */
    private record Ground(int y, BlockState top, BlockState above, boolean canopy) {
        /**
         * What is wrong with this column of a salt flat: null when water or the ice cover lies on it (not the flat's
         * surface), "" when it is salt, else what is there instead. A trunk standing on the flat is a tree growing on
         * salt, a violation of its own (a playa grows no trees).
         */
        String wrong(BlockState salt) {
            if (!top.getFluidState().isEmpty() || top.is(Blocks.SNOW_BLOCK) || top.is(Blocks.PACKED_ICE) || top.is(Blocks.BLUE_ICE))
                return null;
            if (above.is(BlockTags.LOGS))
                return "tree-on-salt:" + BuiltInRegistries.BLOCK.getKey(above.getBlock());
            if (salt == null || top != salt && !packSalt(top))
                return "not-salt:" + BuiltInRegistries.BLOCK.getKey(top.getBlock());
            return "";
        }
    }

    /**
     * The pack's own salt laid over Deep Time's: Expanded Ecosphere's {@code salt} patch turns calcite (halite's vanilla
     * stand-in) under air at y 62 into white concrete powder, the salt of its salt lakes (its surface rules use the same
     * block). Salt over salt, so the flat is still salt (41b11137's test planet, -752 62 198).
     */
    static boolean packSalt(BlockState top) {
        return ModList.get().isLoaded(BiomeRuleGameTests.EE) && (top.is(Blocks.WHITE_CONCRETE_POWDER) || top.is(Blocks.WHITE_CONCRETE));
    }

    /**
     * The block an active volcano's lava flow paints on column (x, z), as {@code LavaFlowPass} lays it, or null where no
     * flow shows (none, or one under the vegetation). A salt column whose top is this block is a flow over the flat.
     */
    private static BlockState flowBlock(LavaFlows flows, LavaFlowPalette palette, int x, int z) {
        int code = flows.code(x, z);
        return code > 0 ? palette.block(code, LavaFlowPalette.key(flows.wrap(x), z)) : null;
    }

    private static Ground ground(ServerLevel level, int x, int z) {
        LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        int y = chunk.getHeight(Heightmap.Types.WORLD_SURFACE, x & 15, z & 15);
        BlockState b = chunk.getBlockState(pos.set(x, y, z));
        BlockState above = Blocks.AIR.defaultBlockState();
        boolean canopy = false;
        while (y > level.getMinBuildHeight() && b.getFluidState().isEmpty()
                && (b.isAir() || !b.isCollisionShapeFullBlock(level, pos) || b.is(BlockTags.LEAVES) || b.is(BlockTags.LOGS))) {
            above = b;
            canopy |= b.is(BlockTags.LEAVES);
            b = chunk.getBlockState(pos.set(x, --y, z));
        }
        return new Ground(y, b, above, canopy);
    }

    /** The biome on a column's ground, for the failure's detail. */
    private static String biome(ServerLevel level, int x, int y, int z) {
        return level.getBiome(new BlockPos(x, y + 1, z)).unwrapKey().map(k -> k.location().toString()).orElse("?");
    }

    /** A salt column's blocks from the top down past its flat's depth, and that depth, so a red names what lies there. */
    private static String column(ServerLevel level, RasterSampler playa, int x, int y, int z) {
        double v = playa.nearest(x, z);
        int n = Double.isNaN(v) ? 0 : (int) Math.round(v);
        LevelChunk chunk = level.getChunk(x >> 4, z >> 4);
        StringBuilder b = new StringBuilder(" salt ").append(n).append(" blocks; y ").append(y + 1).append(" down:");
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        for (int k = y + 1; k >= y - n - 1; k--)
            b.append(' ').append(BuiltInRegistries.BLOCK.getKey(chunk.getBlockState(pos.set(x, k, z)).getBlock()).getPath());
        return b.toString();
    }

    /**
     * Whether something opened column (x, z) through its salt: its ground lies at or under the salt's bottom (the
     * bake's terrain top less the salt's depth), so no flat's surface is left to read (a carver's mouth on the flat;
     * 649baf54's test planet with MIC: air from the bake's top at y 64 and 65 down to the crater breccia at 62, at two
     * pairs of columns by its springs), as water or the ice cover over it are not.
     */
    private static boolean opened(WorldDataTerrain w, RasterSampler playa, Ground gr, int x, int z) {
        double v = playa.nearest(x, z);
        int n = Double.isNaN(v) ? 0 : (int) Math.round(v);
        return gr.y() <= (int) Math.floor(w.height(x, z)) - n;
    }

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
        var settings = g.generatorSettings().value();
        Geothermal geo = w.geothermal(g.waterReach(), settings.noiseSettings().minY(), settings.seaLevel());
        LavaFlows flows = w.lavaFlows(g.waterReach());
        LavaFlowPalette flowPalette = LavaFlowPalette.get();
        // Lake columns a block or more deep, frozen and open, and playa cells, spread over the planet.
        List<Spot> frozen = new ArrayList<>(), open = new ArrayList<>(), salt = new ArrayList<>();
        RasterSampler levels = wd.layer(WorldData.WATER_LEVEL), playa = wd.layer(Playa.LAYER);
        var grid = levels.desc().grid();
        long lakeCells = 0, playaCells = 0;
        int onSpring = 0;
        for (int j = 0; j < grid.h(); j++) {
            for (int i = 0; i < grid.w(); i++) {
                boolean lake = !Double.isNaN(levels.get(i, j)), flat = playa.get(i, j) > 0;
                if (lake)
                    lakeCells++;
                if (flat)
                    playaCells++;
                if ((!lake && !flat) || Math.floorMod(i * 31 + j * 17, 7) != 0)
                    continue;
                // The block the samplers take to cell (i, j). On a cube grid the rows stack the six faces, so the
                // flat origin + i·r is that block only on the first face: the cell's centre on the net, the block east
                // and south of it (r is even). The flat sum passed while the sampled flats lay on the first face; on
                // the planet after the M5 merge (2026-10-26) all 24 read dry ground elsewhere on the net.
                int x, z;
                if (grid.isCube()) {
                    double[] p = grid.cubePoint(i, j);
                    x = (int) Math.floor(p[0]);
                    z = (int) Math.floor(p[1]);
                } else {
                    x = (int) (grid.originBlockX() + (long) i * grid.r());
                    z = (int) (grid.originBlockZ() + (long) j * grid.r());
                }
                if (flat && salt.size() < MOST) {
                    // A hot spring's own pool, rim and terraces are not the flat: GeothermalPass paints them over the salt
                    // (sinter or travertine; the old rule passed there only because their vanilla stand-in, calcite, is
                    // also halite's).
                    if (geo.partAt(x, z) != Geothermal.Part.NONE)
                        onSpring++;
                    else
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
        int wrong = 0, frozenWarm = 0, openCold = 0, frozenRead = 0, openRead = 0, saltRead = 0, overhung = 0, underFlow = 0, openedColumns = 0;
        StringBuilder flowDetail = new StringBuilder();
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
            }   // else: not water or ice at the lake's top, which riversHold's dry-shore count fails
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
            Ground gr = ground(level, s.x(), s.z());
            if (gr.canopy())
                overhung++;
            String bad = gr.wrong(saltBlock);
            if (bad == null)
                continue;   // under water or under the ice cover: not this pass's surface
            saltRead++;
            if (!bad.isEmpty() && gr.top() == flowBlock(flows, flowPalette, s.x(), s.z())) {
                underFlow++;   // a lava flow over the flat
                continue;
            }
            if (!bad.isEmpty() && opened(w, playa, gr, s.x(), s.z())) {
                openedColumns++;   // opened through its salt: no flat's surface to read
                continue;
            }
            if (!bad.isEmpty()) {
                wrong++;
                detail.append(' ').append(bad).append('(').append(s.x()).append(',').append(gr.y()).append(',').append(s.z()).append(' ')
                        .append(biome(level, s.x(), gr.y(), s.z())).append(column(level, playa, s.x(), gr.y(), s.z())).append(')');
            }
        }
        // Round every hot spring by a playa, the flat it does not claim is salt too: no role biome comes within
        // VolcanicBiomes.PLAYA_MARGIN of a playa (BIO-165; before it, Biomes O' Plenty's hot springs dug ponds into the
        // flat and ringed them with thermal calcite, which the sparse grid above found only where it happened to fall).
        int springs = 0, ringRead = 0, ringWrong = 0, ringUnderFlow = 0, ringOpened = 0;
        for (Geothermal.Site site : geo.sites()) {
            if (springs >= MOST)
                break;
            int reach = site.reach() + RING;
            List<int[]> flat = new ArrayList<>();
            for (int dz = -reach; dz <= reach; dz++)
                for (int dx = -reach; dx <= reach; dx++) {
                    int x = site.x() + dx, z = site.z() + dz;
                    if (playa.nearest(x, z) > 0 && geo.partAt(x, z) == Geothermal.Part.NONE)
                        flat.add(new int[]{x, z});
                }
            if (flat.isEmpty())
                continue;
            springs++;
            for (int[] c : flat) {
                Ground gr = ground(level, c[0], c[1]);
                String bad = gr.wrong(saltBlock);
                if (bad == null)
                    continue;
                ringRead++;
                if (bad.isEmpty())
                    continue;
                if (gr.top() == flowBlock(flows, flowPalette, c[0], c[1])) {
                    if (ringUnderFlow++ < 4)
                        flowDetail.append(" flow:").append(LavaFlows.Paint.of(flows.code(c[0], c[1])).id).append(':')
                                .append(BuiltInRegistries.BLOCK.getKey(gr.top().getBlock())).append('(').append(c[0]).append(',')
                                .append(gr.y()).append(',').append(c[1]).append(" spring ").append(site.kind()).append('@')
                                .append(site.x()).append(',').append(site.z()).append(')');
                    continue;   // a lava flow over the flat beside the spring
                }
                if (opened(w, playa, gr, c[0], c[1])) {
                    ringOpened++;   // opened through its salt: no flat's surface to read
                    continue;
                }
                if (ringWrong++ < 12)
                    detail.append(" spring-ring:").append(bad).append('(').append(c[0]).append(',').append(gr.y()).append(',').append(c[1])
                            .append(' ').append(biome(level, c[0], gr.y(), c[1])).append(" spring ").append(site.kind()).append('@')
                            .append(site.x()).append(',').append(site.z())
                            .append(column(level, playa, c[0], gr.y() + 4, c[1])).append(')');
            }
        }
        wrong += ringWrong;
        DeepTime.LOGGER.info("[gametest] lakeIceAndSaltFollowTheClimate lakeCells={} playaCells={} frozenLakeColumns={} (iceRead={} inWarmBiome={}) "
                        + "openLakeColumns={} (waterRead={} inColdBiome={}) playaColumns={} (saltRead={} underCanopy={} onSpring={} opened={} salt={}) "
                        + "underFlow={} springsOnPlaya={} (ringRead={} ringWrong={} ringUnderFlow={} ringOpened={}) threshold={} wrong={}{}{}",
                lakeCells, playaCells, frozen.size(), frozenRead, frozenWarm, open.size(), openRead, openCold, salt.size(), saltRead, overhung, onSpring, openedColumns,
                saltBlock == null ? "none" : BuiltInRegistries.BLOCK.getKey(saltBlock.getBlock()), underFlow, springs, ringRead, ringWrong,
                ringUnderFlow, ringOpened, ice.threshold(), wrong, flowDetail, detail);
        // The opened columns are a few cave mouths, not the flats: a carve-out that sets apart more than a quarter
        // of what it read would hide the rule (Graphide's review of PR #41).
        helper.assertTrue(openedColumns * 4 <= saltRead && ringOpened * 4 <= ringRead, "opened through the salt: "
                + openedColumns + " of " + saltRead + " flat columns, " + ringOpened + " of " + ringRead + " ring columns");
        helper.assertTrue(wrong == 0, wrong + " columns break the lake-ice or salt rule:" + detail);
        helper.succeed();
    }
}
