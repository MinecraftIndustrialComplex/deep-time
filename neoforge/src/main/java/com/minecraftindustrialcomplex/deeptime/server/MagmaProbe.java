// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.LavaFeatures;
import com.minecraftindustrialcomplex.deeptime.worldgen.Magma;
import com.minecraftindustrialcomplex.deeptime.worldgen.MagmaPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.server.level.ServerLevel;

import java.util.Arrays;
import java.util.SplittableRandom;

/**
 * {@code /deeptime probe magma}: where the level has lava (Ben, decisions log 2026-10-01, "Volcanoes
 * is a go ahead"). The settings, every active volcano's magma body ({@link Magma}), the lava
 * features refused so far ({@link LavaFeatures}), and what it means for a player: from
 * {@link #POINTS} land columns drawn evenly over the planet (a fixed seed), the distance to the
 * nearest crater lava lake, to the nearest active volcano on land (lava six blocks under its
 * crater floor, under the plug) and straight down to the lava floor, as median and 90th percentile.
 */
public final class MagmaProbe {

    private MagmaProbe() {}

    static final int POINTS = 4000;

    public static int probe(CommandSourceStack source) {
        ServerLevel level = source.getLevel();
        JsonObject o = new JsonObject();
        o.addProperty("probe", "magma");
        o.addProperty("level", level.dimension().location().toString());
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !(g.terrain() instanceof WorldDataTerrain t)) {
            o.addProperty("deepTime", false);
            return DeepTimeCommands.emit(source, o);
        }
        var settings = g.generatorSettings().value();
        int minY = settings.noiseSettings().minY(), sea = settings.seaLevel();
        Magma magma = t.magma(g.waterReach(), minY, sea);
        int lavaFloor = t.lava().floodY(minY, sea);
        var wd = t.data();
        double mH = wd.manifest().mPerBlockH();
        o.addProperty("deepTime", true);
        o.addProperty("rules", t.magmaRules().toString());
        o.addProperty("lavaRules", t.lava().toString());
        o.addProperty("lavaFloorY", lavaFloor);
        o.addProperty("highestLavaLakeY", t.lava().highestLakeY(minY));
        o.addProperty("activeVolcanoes", wd.volcanoes().list().size());
        o.addProperty("removesLavaFeatures", g.removesLavaFeatures());
        JsonArray refused = new JsonArray();
        LavaFeatures.seen().stream().sorted().forEach(refused::add);
        o.add("lavaFeaturesRefused", refused);
        long[] totals = MagmaPass.totals();
        o.addProperty("lavaBlocksPlaced", totals[0]);
        o.addProperty("chunksWithMagma", totals[1]);
        JsonArray bodies = new JsonArray();
        int withMagma = 0, lakes = 0, underWater = 0;
        for (Magma.Body b : magma.bodies()) {
            JsonObject e = new JsonObject();
            e.addProperty("id", b.id());
            e.addProperty("kind", b.kind());
            e.addProperty("activity", b.activity());
            e.addProperty("x", b.x());
            e.addProperty("z", b.z());
            e.addProperty("floorY", b.floorY());
            e.addProperty("submerged", b.submerged());
            if (b.chamberRh() > 0) {
                withMagma++;
                e.addProperty("chamberY", b.chamberY());
                e.addProperty("chamberRh", b.chamberRh());
                e.addProperty("chamberRv", b.chamberRv());
                e.addProperty("chamberBlocks", b.chamberBlocks());
                e.addProperty("conduitTop", b.conduitTop());
                e.addProperty("lenses", b.lensCount());
                e.addProperty("columnBlocks", b.columnBlocks());
                e.addProperty("bodyBottomY", b.minY());
                e.addProperty("reach", b.reach());
            }
            e.addProperty("lake", b.lake());
            if (b.lake()) {
                lakes++;
                e.addProperty("lakeRadius", b.lakeRadius());
                e.addProperty("lakeTop", b.lakeTop());
                e.addProperty("lakeBottom", b.lakeBottom());
            }
            if (b.submerged())
                underWater++;
            bodies.add(e);
        }
        o.addProperty("withMagma", withMagma);
        o.addProperty("craterLakes", lakes);
        o.addProperty("underWater", underWater);
        o.add("bodies", bodies);

        // What it means for a player: land columns drawn evenly over the playable planet.
        int c = t.circumference();
        int zMax = (int) t.zHalf();
        var wp = t.data().projection();
        SplittableRandom rnd = new SplittableRandom(20261001L);
        double[] toLake = new double[POINTS], toVolcano = new double[POINTS], down = new double[POINTS], any = new double[POINTS];
        int n = 0;
        for (int tries = 0; n < POINTS && tries < POINTS * 40; tries++) {
            int x = rnd.nextInt(-c / 2, c / 2), z = rnd.nextInt(-zMax, zMax);
            if (!onPlanet(wp, x, z))
                continue;
            double h = t.height(x, z);
            if (h <= sea + 1)
                continue;
            double lake = Double.POSITIVE_INFINITY, volcano = Double.POSITIVE_INFINITY;
            for (Magma.Body b : magma.bodies()) {
                if (b.chamberRh() <= 0 || b.submerged())
                    continue;   // no magma, or under water
                double d = Math.hypot(magma.dx(x, b), z - b.z());
                volcano = Math.min(volcano, d);
                if (b.lake())
                    lake = Math.min(lake, Math.max(0.0, d - b.lakeRadius()));
            }
            toLake[n] = lake;
            toVolcano[n] = volcano;
            down[n] = Math.ceil(h) - 1 - lavaFloor;
            any[n] = Math.min(volcano, down[n]);
            n++;
        }
        JsonObject reach = new JsonObject();
        reach.addProperty("landPoints", n);
        reach.addProperty("metresPerBlock", mH);
        reach.add("toCraterLakeBlocks", stats(toLake, n));
        reach.add("toActiveVolcanoOnLandBlocks", stats(toVolcano, n));
        reach.add("straightDownToLavaFloorBlocks", stats(down, n));
        reach.add("nearestOfVolcanoOrFloorBlocks", stats(any, n));
        o.add("reach", reach);
        return DeepTimeCommands.emit(source, o);
    }

    private static JsonObject stats(double[] v, int n) {
        JsonObject s = new JsonObject();
        if (n == 0)
            return s;
        double[] a = Arrays.copyOf(v, n);
        Arrays.sort(a);
        for (var q : new Object[][]{{"p10", 0.10}, {"median", 0.50}, {"p90", 0.90}}) {
            double x = a[Math.min(n - 1, (int) Math.floor((double) q[1] * n))];
            if (Double.isFinite(x))
                s.addProperty((String) q[0], Math.round(x));
            else
                s.addProperty((String) q[0], "none");
        }
        return s;
    }

    /**
     * Whether column (x, z) is one of the planet's own blocks, not a folded copy of one (a void square
     * of the cube's net): the projection's canonical point of its centre is the centre itself.
     */
    static boolean onPlanet(com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection wp, int x, int z) {
        double[] q = wp.canonical(x + 0.5, z + 0.5);
        return q[0] == x + 0.5 && q[1] == z + 0.5;
    }
}
