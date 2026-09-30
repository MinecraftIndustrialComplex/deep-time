// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.rock.RockPalette;
import com.minecraftindustrialcomplex.deeptime.server.SpawnHandler;
import com.minecraftindustrialcomplex.deeptime.worlddata.Columns;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.StrataPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.core.BlockPos;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.status.ChunkStatus;
import net.minecraft.world.level.levelgen.Heightmap;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.io.IOException;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * The generator on a real baked planet (P1-08): the {@code deeptime:test_planet_real} level reads
 * the bake of {@code tests/presets/planet_world_4k.json} (every stage of the pipeline, format 0.1)
 * through the pure-Java reader, with the pack's biomes and surface rules. The parity fixture
 * {@code deeptime parity} wrote next to it (Rust's column evaluator, plan §9.2) drives the rock
 * check: the blocks the strata pass placed are the classes Rust decided.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class RealPlanetGameTests {

    private RealPlanetGameTests() {}

    static ServerLevel level(GameTestHelper helper) {
        ServerLevel level = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_REAL);
        if (level == null)
            helper.fail("level " + GameTestSupport.TEST_PLANET_REAL.location() + " does not exist");
        return level;
    }

    static WorldDataTerrain terrain(GameTestHelper helper, ServerLevel level) {
        if (level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g && g.terrain() instanceof WorldDataTerrain w)
            return w;
        helper.fail("test_planet_real is not generating from world data");
        throw new IllegalStateException();
    }

    /**
     * The terrain is the baked height, detail term included: at noise-cell corner columns (x ≡ z ≡ 0
     * mod 4) the top block is ⌈H⌉ − 1 exactly; elsewhere the density is interpolated between
     * corners, so the difference is only logged, with a loose bound.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 2400)
    public static void realPlanetHeightMatchesTheBake(GameTestHelper helper) {
        ServerLevel level = level(helper);
        WorldDataTerrain t = terrain(helper, level);
        WorldData wd = t.data();
        helper.assertTrue(wd.detailNoise() != null, "the real planet has a detail term");
        int c = t.circumference();
        int[] detailed = {0};
        ChunkSampling.HeightTally tally = new ChunkSampling.HeightTally();
        ChunkSampling.forEach(helper, 96, i -> {
            boolean corner = i % 2 == 0;
            int x = (-c / 2 + 8 + i * (c / 98)) & ~3;
            int z = (((i * 37) % 97 - 48) * 18) & ~3;
            if (!corner) {
                x += 1 + (i % 3);
                z += 2;
            }
            if (wd.detail(t.wrapX(x), z) != 0.0)
                detailed[0]++;
            int expected = (int) Math.ceil(t.height(x, z)) - 1;
            if (expected < level.getMinBuildHeight() + 32)
                return false;   // the floor band: the router's bottom slide, not the bake
            int actual = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.NOISE, true)
                    .getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z);
            tally.add(x, z, corner, expected, actual, t.height(x, z));
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] realPlanetHeightMatchesTheBake columns={} cornerColumns={} cornerMaxAbsDiff={} "
                    + "maxAbsDiff={} withDetail={} worst={}", tally.checked, tally.corners, tally.cornerMax, tally.maxDiff, detailed[0], tally.worst);
            helper.assertTrue(tally.corners >= 24, "only " + tally.corners + " corner columns above the floor band");
            helper.assertTrue(tally.cornerMax == 0, "corner columns differ by up to " + tally.cornerMax + ": " + tally.worst);
            helper.assertTrue(tally.maxDiff <= 12, "columns differ by up to " + tally.maxDiff + ": " + tally.worst);
            helper.succeed();
        });
    }

    /** The class a rock run of the fixture gives block y, or null (unclassified crust). */
    static String fixtureClass(JsonArray rock, int y) {
        for (JsonElement e : rock) {
            JsonArray r = e.getAsJsonArray();
            if (y <= r.get(0).getAsInt() && y >= r.get(1).getAsInt())
                return r.get(2).isJsonNull() ? null : r.get(2).getAsString();
        }
        return null;
    }

    /**
     * Placed rock = Rust's decision (plan §9.2: "the same file drives a GameTest that compares placed
     * blocks with the expected classes"). For fixture columns inside the planet's span, at 20, 60 and
     * 150 blocks below the generated surface and in the middle of every body run: the Java
     * evaluator names the fixture's class, the strata pass's expected block ({@code predictedRock})
     * belongs to that class in the palette, and the placed block is that block (or the crust where
     * the class resolves to nothing). Blocks the pack's surface rules placed (not crust, not rock)
     * are counted apart.
     */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 4800)
    public static void realPlanetRockMatchesTheParityFixture(GameTestHelper helper) {
        ServerLevel level = level(helper);
        WorldData wd = terrain(helper, level).data();
        JsonObject doc;
        try {
            doc = JsonParser.parseString(Files.readString(wd.folder().resolve("parity/columns.json"))).getAsJsonObject();
        } catch (IOException e) {
            helper.fail("no parity/columns.json next to the real planet (gametest.sh writes it): " + e.getMessage());
            return;
        }
        RockPalette palette = RockPalette.get();
        int c = wd.circumference();
        int[] n = new int[5];   // columns, checked, bodyChecks, surface-rule blocks, wrong
        List<String> detail = new ArrayList<>();
        JsonArray fixture = doc.getAsJsonArray("columns");
        ChunkSampling.forEach(helper, fixture.size(), k -> {
            if (n[0] >= 48)
                return false;
            JsonObject col = fixture.get(k).getAsJsonObject();
            int x = col.get("wx").getAsInt(), z = col.get("z").getAsInt();
            if (Math.abs(z) > c / 4 - 32)
                return false;   // past the pole edge: the beyond-pole mask, not the bake
            n[0]++;
            ChunkAccess chunk = level.getChunkSource().getChunk(x >> 4, z >> 4, ChunkStatus.SURFACE, true);
            int surface = chunk.getHeight(Heightmap.Types.OCEAN_FLOOR_WG, x, z);
            Columns.Sample java = Columns.sample(wd, x, z);
            List<Integer> ys = new ArrayList<>(List.of(surface - 20, surface - 60, surface - 150));
            for (JsonElement b : col.getAsJsonArray("bodies")) {
                JsonArray r = b.getAsJsonArray();
                int mid = (r.get(1).getAsInt() + r.get(2).getAsInt()) / 2;
                if (mid < surface - 8) {
                    ys.add(mid);
                    n[2]++;
                }
            }
            for (int y : ys) {
                if (y < chunk.getMinBuildHeight())
                    continue;
                String cls = fixtureClass(col.getAsJsonArray("rock"), y);
                String javaCls = fixtureClass(runsOf(java), y);
                BlockState expected = StrataPass.predictedRock(wd, x, y, z);
                BlockState actual = chunk.getBlockState(new BlockPos(x, y, z));
                n[1]++;
                boolean ok = Objects.equals(cls, javaCls)
                        && (cls == null ? expected == null : expected == null || palette.classesOf(expected).contains(cls));
                boolean placed = expected == null ? StrataPass.isCrust(actual) : actual == expected;
                if (!placed && !StrataPass.isCrust(actual) && palette.classesOf(actual).isEmpty()) {
                    n[3]++;   // a surface rule's block (sand, terracotta, …): not the strata pass's to paint
                    continue;
                }
                if (!ok || !placed) {
                    n[4]++;
                    if (detail.size() < 8)
                        detail.add("(" + x + ", " + y + ", " + z + ") fixture " + cls + " java " + javaCls + " expected "
                                + expected + " placed " + actual);
                }
            }
            return true;
        }, () -> {
            DeepTime.LOGGER.info("[gametest] realPlanetRockMatchesTheParityFixture columns={} checked={} bodyChecks={} "
                    + "surfaceRuleBlocks={} wrong={} {}", n[0], n[1], n[2], n[3], n[4], detail);
            helper.assertTrue(n[0] >= 32 && n[1] >= 96, "only " + n[0] + " columns, " + n[1] + " samples");
            helper.assertTrue(n[4] == 0, n[4] + " samples differ: " + detail);
            helper.succeed();
        });
    }

    static JsonArray runsOf(Columns.Sample s) {
        JsonArray out = new JsonArray();
        for (Columns.Run r : s.rock()) {
            JsonArray a = new JsonArray();
            a.add(r.top());
            a.add(r.bottom());
            a.add(r.rockClass());
            out.add(a);
        }
        return out;
    }

    /** The bake's spawn candidates give a spawn on dry land. */
    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void realPlanetSpawnOnLand(GameTestHelper helper) {
        ServerLevel level = level(helper);
        var pos = SpawnHandler.chooseSpawn(level, terrain(helper, level).data());
        int sea = level.getChunkSource().getGenerator().getSeaLevel();
        DeepTime.LOGGER.info("[gametest] realPlanetSpawnOnLand spawn={} seaLevel={}", pos.orElse(null), sea);
        helper.assertTrue(pos.isPresent() && pos.get().getY() > sea, "no dry-land spawn: " + pos);
        helper.succeed();
    }
}
