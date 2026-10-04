// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.worlddata.Craters;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.Comparator;
import java.util.HashSet;
import java.util.List;
import java.util.Set;

/**
 * Meteor impacts in the placed world (P2-08): the real test planet's craters
 * ({@code impacts/craters.json}; {@code planet_world_4k.json} runs the impact stage at forty times
 * Earth's rate so that a planet this small has some).
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class ImpactGameTests {

    private ImpactGameTests() {}

    /** Columns of placed blocks read under the crater's floor: the centre and four around it. */
    private static final int[][] FLOOR_COLUMNS = {{0, 0}, {1, 0}, {-1, 0}, {0, 1}, {0, -1}};

    /**
     * The crater that shows best on the test planet is a bowl in the terrain the game reads, and
     * the rock under its floor is impact breccia:
     * <ul>
     *   <li>the list is sane: every crater has a size, a state the reader knows and no more relief
     *       than it formed with;</li>
     *   <li>the terrain at its centre lies below the mean of sixteen points on its rim by at least
     *       a third of the relief the simulation says it has left;</li>
     *   <li>within its breccia lens under the floor, placed blocks are the block the palette gives
     *       {@code impact_breccia} (a body of the bake for a crater too small for the strata log).</li>
     * </ul>
     * A planet whose craters have all faded, or whose showing craters are all in the strata log's
     * cells or under water, passes with a line saying so: the shapes themselves are the core's
     * tests' ({@code surface::amplify::impact}).
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void aCraterIsABowlOverBreccia(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_CAVES);
        if (level == null) {
            helper.fail("level " + GameTestSupport.TEST_PLANET_CAVES.location() + " does not exist");
            return;
        }
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain t)) {
            helper.fail("test_planet_caves is not generating from world data");
            return;
        }
        WorldData wd = t.data();
        double mH = wd.manifest().mPerBlockH(), mV = wd.manifest().mPerBlockV();
        int c = wd.circumference(), sea = g.generatorSettings().value().seaLevel();
        List<Craters.Crater> all = wd.craters().list();
        Set<String> fates = Set.of("fresh", "degraded", "buried", "eroded_to_roots");
        for (Craters.Crater k : all) {
            if (!(k.diameterM() > 0) || !fates.contains(k.fate()) || k.reliefM() > 1.01 * k.depthM() + 1 || k.reliefM() < 0
                    || Math.abs(k.x()) > c / 2.0 || Math.abs(k.z()) > com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c)) {
                helper.fail("crater " + k.id() + " is not sane: " + k);
                return;
            }
        }
        long showing = all.stream().filter(Craters.Crater::shows).count();
        // The best one to read: on land, clear of the poles, its bowl at least three blocks deep and
        // its lens at least three blocks thick, drawn by the amplifier with a body under it.
        Craters.Crater best = all.stream()
                .filter(k -> k.shows() && !k.inSurface() && k.reliefM() >= 3 * mV && k.brecciaM() >= 3 * mV)
                .filter(k -> Math.abs(k.z()) < com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c) - 64 - 3 * k.diameterM() / mH)
                .filter(k -> t.height((int) Math.round(k.x()), (int) Math.round(k.z())) > sea + 2)
                .max(Comparator.comparingDouble(Craters.Crater::reliefM)).orElse(null);
        if (best == null) {
            DeepTime.LOGGER.info("[gametest] aCraterIsABowlOverBreccia craters={} showing={}: none on land with a bowl of three "
                    + "blocks and a body; nothing to read back", all.size(), showing);
            helper.succeed();
            return;
        }
        int cx = (int) Math.round(best.x()), cz = (int) Math.round(best.z());
        double rb = best.diameterM() / 2 / mH;
        double centre = t.height(cx, cz), rim = 0;
        for (int i = 0; i < 16; i++) {
            double a = 2 * Math.PI * i / 16;
            rim += t.height((int) Math.round(best.x() + rb * Math.cos(a)), (int) Math.round(best.z() + rb * Math.sin(a))) / 16;
        }
        double bowl = rim - centre, want = best.reliefM() / mV / 3;
        var result = RockPalette.get().result("impact_breccia").orElse(null);
        if (result == null || result.blocks().isEmpty()) {
            helper.fail("the palette has no block for impact_breccia");
            return;
        }
        Set<BlockState> breccia = new HashSet<>(result.blocks());
        int lens = (int) Math.max(3, Math.min(24, Math.floor(best.brecciaM() / mV)));
        int[] found = new int[FLOOR_COLUMNS.length], read = new int[FLOOR_COLUMNS.length];
        // What the columns hold under the floor, by block, for the log: the lens is breccia, the rest soil and strata.
        java.util.Map<String, Integer> seen = new java.util.TreeMap<>();
        StringBuilder profile = new StringBuilder();
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        double rimF = rim;
        ChunkSampling.forEach(helper, FLOOR_COLUMNS.length, i -> {
            int x = cx + FLOOR_COLUMNS[i][0], z = cz + FLOOR_COLUMNS[i][1];
            ChunkAccess chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.SURFACE, true);
            int top = (int) Math.ceil(t.height(x, z));
            if (i == 0)
                profile.append(columnProfile(chunk, x, z, top + 8, top - 40)).append(" heightmap ")
                        .append(chunk.getHeight(net.minecraft.world.level.levelgen.Heightmap.Types.WORLD_SURFACE_WG, x & 15, z & 15));
            // From the floor down through the lens (the top blocks are the surface rules' soil).
            for (int y = top; y >= Math.max(level.getMinBuildHeight(), top - lens - 2); y--) {
                BlockState s = chunk.getBlockState(pos.set(x, y, z));
                read[i]++;
                seen.merge(BuiltInRegistries.BLOCK.getKey(s.getBlock()).getPath(), 1, Integer::sum);
                if (breccia.contains(s))
                    found[i]++;
            }
            return true;
        }, () -> {
            int columns = 0, blocks = 0;
            for (int n : found) {
                columns += n > 0 ? 1 : 0;
                blocks += n;
            }
            DeepTime.LOGGER.info("[gametest] aCraterIsABowlOverBreccia craters={} showing={} read=crater {} ({} m across, {}, {} Myr, "
                            + "{} m of {} m relief, lens {} m) at ({}, {}): centre y {} rim y {} bowl {} blocks (want {}); breccia {} "
                            + "blocks in {} of {} columns, {} read each ({}); read {}; centre column {}", all.size(), showing, best.id(),
                    Math.round(best.diameterM()), best.fate(), best.ageMyr(), Math.round(best.reliefM()), Math.round(best.depthM()),
                    Math.round(best.brecciaM()), cx, cz, String.format("%.1f", centre), String.format("%.1f", rimF),
                    String.format("%.1f", bowl), String.format("%.1f", want), blocks, columns, FLOOR_COLUMNS.length, read[0],
                    result.blocks().stream().map(StrataGameTests::id).toList(), seen, profile);
            if (bowl < want)
                helper.fail("crater " + best.id() + ": its centre is " + bowl + " blocks below its rim, " + want + " wanted");
            if (columns == 0)
                helper.fail("crater " + best.id() + ": no impact breccia in the " + lens + " blocks under its floor");
            helper.succeed();
        });
    }

    /** The column from y {@code from} down to {@code to} as runs of one block, top first. */
    static String columnProfile(ChunkAccess chunk, int x, int z, int from, int to) {
        StringBuilder b = new StringBuilder();
        BlockPos.MutableBlockPos p = new BlockPos.MutableBlockPos();
        String last = null;
        int top = from, y = from;
        for (; y >= Math.max(to, chunk.getMinBuildHeight()); y--) {
            String id = BuiltInRegistries.BLOCK.getKey(chunk.getBlockState(p.set(x, y, z)).getBlock()).getPath();
            if (!id.equals(last)) {
                if (last != null)
                    b.append(' ').append(last).append('[').append(top).append("..").append(y + 1).append(']');
                last = id;
                top = y;
            }
        }
        if (last != null)
            b.append(' ').append(last).append('[').append(top).append("..").append(y + 1).append(']');
        return b.toString();
    }
}
