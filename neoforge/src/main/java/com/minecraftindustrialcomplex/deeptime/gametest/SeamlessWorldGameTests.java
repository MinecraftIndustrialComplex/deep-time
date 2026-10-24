// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.gametest;

import com.minecraftindustrialcomplex.deeptime.api.climate.DeepTimeClimate;
import com.minecraftindustrialcomplex.deeptime.api.planet.DeepTimePlanet;
import com.minecraftindustrialcomplex.deeptime.net.WorldInfoPayload;
import com.minecraftindustrialcomplex.deeptime.worlddata.PetroffGuyou;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldLayout;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import com.minecraftindustrialcomplex.deeptime.wrap.EdgeBorder;
import com.minecraftindustrialcomplex.deeptime.wrap.PoleCap;
import net.minecraft.gametest.framework.GameTest;
import net.minecraft.gametest.framework.GameTestHelper;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.GameType;
import net.neoforged.neoforge.gametest.GameTestHolder;
import net.neoforged.neoforge.gametest.PrefixGameTestTemplate;

import java.util.ArrayList;
import java.util.List;

/**
 * The seamless Petroff–Guyou world in the game (format 0.5; Ben, decisions log 2026-10-04):
 * {@code test_planet_real} is the 4k test planet drawn as a 3,072 × 1,536 world. Its generator,
 * world data and layout agree; the terrain past a turning edge is the folded inside; the temporary
 * edge border stops a survival player a block inside the north and south edges and warns before
 * them, lets a spectator pass, and is absent on a Lambert level; the pole cap is off; the climate
 * API's latitude depends on x and z.
 */
@GameTestHolder(GameTestSupport.NS)
@PrefixGameTestTemplate(false)
public final class SeamlessWorldGameTests {

    private SeamlessWorldGameTests() {}

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void seamlessWorldLayoutAndFold(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        WorldDataTerrain t = RealPlanetGameTests.terrain(helper, level);
        GeologicChunkGenerator g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        WorldData wd = t.data();
        helper.assertTrue(g.dataRef().petroffGuyou() && wd.projection() instanceof PetroffGuyou,
                "test_planet_real is a Petroff–Guyou world: " + wd.projection());
        PetroffGuyou pg = (PetroffGuyou) wd.projection();
        long b = pg.hemisphere();
        helper.assertTrue(wd.circumference() == 2 * b && g.dataRef().circumferenceBlocks() == 2 * b,
                "the x period is 2b: " + wd.circumference() + ", b " + b);
        WorldLayout layout = wd.layout();
        helper.assertTrue(layout != null && layout.edges().size() == 4 && layout.edge("north").halfTurn()
                && "toroidal_world".equals(layout.edge("west").owner()) && "wrap_engine".equals(layout.edge("north").owner()),
                "the layout block: " + layout);
        helper.assertTrue(layout.border() != null && layout.border().temporary(), "a temporary border");
        // The cone points are in open sea (the bake chose λc for that).
        int sea = g.getSeaLevel();
        for (WorldLayout.ConePoint cp : layout.conePoints())
            for (long[] img : cp.images()) {
                long x = Math.max(-b, Math.min(b - 1, img[0])), z = Math.max(-b / 2, Math.min(b / 2 - 1, img[1]));
                double h = t.height((int) x, (int) z);
                helper.assertTrue(h < sea, "cone point " + cp.name() + " at " + x + ", " + z + " is under the sea: " + h);
            }
        // Past a turning edge is the glued inside: (x, −b/2 − d) is (−x, −b/2 + d).
        double h2 = b / 2.0;
        for (int k = 0; k < 64; k++) {
            int x = (int) (-b + 37 + k * (2 * b / 64));
            for (int d : new int[]{1, 9, 40}) {
                double out = wd.height(x, -h2 - d), in = wd.height(-x, -h2 + d);
                helper.assertTrue(Math.abs(out - in) < 1e-6, "north fold at x " + x + ", d " + d + ": " + out + " vs " + in);
                out = wd.height(x, h2 + d);
                in = wd.height(-x, h2 - d);
                helper.assertTrue(Math.abs(out - in) < 1e-6, "south fold at x " + x + ", d " + d + ": " + out + " vs " + in);
            }
        }
        // No pole cap, nothing beyond a pole; latitude depends on x and z.
        helper.assertTrue(PoleCap.capOf(level) == null, "a Petroff–Guyou world has no pole cap");
        // The planet API reads the folded world as one sphere: past the north edge is the glued inside.
        DeepTimePlanet planet = DeepTimePlanet.of(level).orElse(null);
        double radius = pg.radius() * wd.manifest().mPerBlockH();
        helper.assertTrue(planet != null && Math.abs(planet.radiusMetres() - radius) <= 1e-9 * radius,
                "the planet API's sphere: " + planet + " vs R " + radius + " m");
        double fold = planet.distanceMetres(-b + 37.5, -h2 - 9.5, b - 37.5, -h2 + 9.5);
        helper.assertTrue(fold < 1e-3 && planet.canonical(-b + 37.5, -h2 - 9.5).equals(planet.canonical(b - 37.5, -h2 + 9.5)),
                "(x, -b/2 - d) and (-x, -b/2 + d) are one place: " + fold + " m apart, canonical "
                        + planet.canonical(-b + 37.5, -h2 - 9.5) + " and " + planet.canonical(b - 37.5, -h2 + 9.5));
        double northPole = DeepTimeClimate.latitudeDeg(level, -b / 2.0, -h2 + 0.01);
        double equator = DeepTimeClimate.latitudeDeg(level, -b / 2.0 - 300, 0);
        helper.assertTrue(northPole > 89 && Math.abs(equator) < 1e-6, "latitudes: pole " + northPole + ", equator " + equator);
        WorldInfoPayload info = WorldInfoPayload.forLevel(level);
        helper.assertTrue(info.petroffGuyou() && info.circumference() == 2 * b && info.poleCap() == null
                && info.lambdaCDeg() == pg.lambdaCDeg(), "the planet info: " + info);
        helper.succeed();
    }

    @GameTest(template = GameTestSupport.TEMPLATE, timeoutTicks = 1200)
    public static void edgeBorderStopsAtTheTurningEdges(GameTestHelper helper) {
        ServerLevel level = RealPlanetGameTests.level(helper);
        double stop = EdgeBorder.stopZ(level);
        GeologicChunkGenerator g = (GeologicChunkGenerator) level.getChunkSource().getGenerator();
        double half = g.dataRef().hemisphereBlocks() / 2.0;
        helper.assertTrue(!Double.isNaN(stop) && Math.abs(stop - (half - EdgeBorder.INSET)) < 1e-9,
                "the stop line is a block inside the edge: " + stop + " (edge " + half + ")");
        ServerLevel lambert = helper.getLevel().getServer().getLevel(GameTestSupport.TEST_PLANET_DATA);
        helper.assertTrue(lambert != null && Double.isNaN(EdgeBorder.stopZ(lambert)), "no edge border on a Lambert level");

        int x = 100;
        double y = level.getMaxBuildHeight() - 20.5;
        List<long[]> forced = new ArrayList<>();
        for (double z : new double[]{-half - 3, -half + 10, half + 3})
            for (int dx = 0; dx <= 1; dx++) {
                long cx = (x >> 4) + dx, cz = (long) Math.floor(z) >> 4;
                level.setChunkForced((int) cx, (int) cz, true);
                forced.add(new long[]{cx, cz});
            }
        List<MockPlayers.Mock> mocks = new ArrayList<>();
        MockPlayers.Mock north = MockPlayers.create(level, GameType.SURVIVAL);
        MockPlayers.Mock near = MockPlayers.create(level, GameType.SURVIVAL);
        MockPlayers.Mock south = MockPlayers.create(level, GameType.CREATIVE);
        MockPlayers.Mock ghost = MockPlayers.create(level, GameType.SPECTATOR);
        mocks.add(north);
        mocks.add(near);
        mocks.add(south);
        mocks.add(ghost);
        north.player.teleportTo(level, x + 0.5, y, -half - 3, 0, 0);
        near.player.teleportTo(level, x + 0.5, y, -half + 10, 0, 0);
        south.player.teleportTo(level, x + 0.5, y, half + 3, 0, 0);
        ghost.player.teleportTo(level, x + 1.5, y, -half - 3, 0, 0);
        for (MockPlayers.Mock m : mocks)
            m.forgetSent();
        try {
            EdgeBorder.tick(level, stop);
            helper.assertTrue(north.player.getZ() >= -stop && north.player.getZ() < -stop + 1,
                    "the north player is put back inside: z " + north.player.getZ());
            helper.assertTrue(south.player.getZ() <= stop && south.player.getZ() > stop - 1,
                    "the creative south player too: z " + south.player.getZ());
            helper.assertTrue(Math.abs(ghost.player.getZ() - (-half - 3)) < 1e-9, "a spectator passes: z " + ghost.player.getZ());
            helper.assertTrue(north.sawActionBar(EdgeBorder.KEY_STOP), "the stop says why");
            helper.assertTrue(near.sawActionBar(EdgeBorder.KEY_NEAR) && Math.abs(near.player.getZ() - (-half + 10)) < 1e-9,
                    "the warning band warns and does not move: z " + near.player.getZ());
            helper.succeed();
        } finally {
            MockPlayers.removeAll(mocks);
            for (long[] c : forced)
                level.setChunkForced((int) c[0], (int) c[1], false);
        }
    }
}
