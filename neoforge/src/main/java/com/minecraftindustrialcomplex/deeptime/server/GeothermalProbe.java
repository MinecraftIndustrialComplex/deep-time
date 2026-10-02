// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.Geothermal;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeothermalPalette;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeothermalPass;
import com.minecraftindustrialcomplex.deeptime.worldgen.Magma;
import com.minecraftindustrialcomplex.deeptime.worldgen.WorldDataTerrain;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.server.level.ServerLevel;

import java.util.Arrays;
import java.util.SplittableRandom;

/**
 * {@code /deeptime probe geothermal} (P2-14): the settings, the palette, every site
 * ({@link Geothermal}), and what it means for a player: from {@link #POINTS} land columns drawn
 * evenly over the planet inside the pole cap's soft line (the seed {@code probe magma} uses, so the
 * two reports describe the same points), the distance to the nearest site of any kind, to the nearest
 * hot-spring pool, to the nearest vent, and, for comparison, to the nearest active volcano on land,
 * as 10th percentile, median and 90th percentile.
 */
public final class GeothermalProbe {

    private GeothermalProbe() {}

    static final int POINTS = 4000;

    public static int probe(CommandSourceStack source) {
        ServerLevel level = source.getLevel();
        JsonObject o = new JsonObject();
        o.addProperty("probe", "geothermal");
        o.addProperty("level", level.dimension().location().toString());
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) || !(g.terrain() instanceof WorldDataTerrain t)) {
            o.addProperty("deepTime", false);
            return DeepTimeCommands.emit(source, o);
        }
        var settings = g.generatorSettings().value();
        int minY = settings.noiseSettings().minY(), sea = settings.seaLevel();
        Geothermal geo = t.geothermal(g.waterReach(), minY, sea);
        Magma magma = t.magma(g.waterReach(), minY, sea);
        var wd = t.data();
        o.addProperty("deepTime", true);
        o.addProperty("rules", t.geothermalRules().toString());
        o.add("palette", GeothermalPalette.get().toJson());
        o.addProperty("activeVolcanoes", wd.volcanoes().list().size());
        int[] n = geo.counts();
        o.addProperty("sites", geo.sites().size());
        o.addProperty("pools", n[0]);
        o.addProperty("vents", n[1]);
        o.addProperty("patches", n[2]);
        o.addProperty("sinterPools", geo.sites().stream().filter(s -> s.kind() == Geothermal.Kind.POOL && s.sinter()).count());
        o.addProperty("travertinePools", geo.sites().stream().filter(s -> s.kind() == Geothermal.Kind.POOL && !s.sinter()).count());
        o.addProperty("volcanoesWithSites", geo.sites().stream().filter(s -> s.volcano() != Geothermal.HEAT_FLOW_SOURCE)
                .map(Geothermal.Site::volcano).distinct().count());
        o.addProperty("heatFlowSites", geo.heatFlowSites());
        long[] totals = GeothermalPass.totals();
        o.addProperty("poolColumnsPlaced", totals[0]);
        o.addProperty("ventColumnsPlaced", totals[1]);
        o.addProperty("groundColumnsPlaced", totals[2]);
        o.addProperty("chunksWithFeatures", totals[3]);
        JsonArray list = new JsonArray();
        for (Geothermal.Site s : geo.sites()) {
            JsonObject e = new JsonObject();
            e.addProperty("id", s.id());
            e.addProperty("kind", s.kind().name().toLowerCase());
            e.addProperty("volcano", s.volcano());
            e.addProperty("x", s.x());
            e.addProperty("z", s.z());
            e.addProperty("r", s.r());
            e.addProperty("apron", s.apron());
            if (s.kind() == Geothermal.Kind.POOL) {
                e.addProperty("level", s.level());
                e.addProperty("terrace", s.sinter() ? "sinter" : "travertine");
            }
            list.add(e);
        }
        o.add("siteList", list);

        // What it means for a player: land columns drawn evenly over the playable planet.
        int c = t.circumference();
        int zMax = (int) (c / 4 * (75.0 / 90.0));
        SplittableRandom rnd = new SplittableRandom(20261001L);
        double[] any = new double[POINTS], pool = new double[POINTS], vent = new double[POINTS], volcano = new double[POINTS];
        int count = 0;
        for (int tries = 0; count < POINTS && tries < POINTS * 40; tries++) {
            int x = rnd.nextInt(-c / 2, c / 2), z = rnd.nextInt(-zMax, zMax);
            if (t.height(x, z) <= sea + 1)
                continue;
            double dAny = Double.POSITIVE_INFINITY, dPool = dAny, dVent = dAny, dVolcano = dAny;
            for (Geothermal.Site s : geo.sites()) {
                if (Math.abs(s.z()) > zMax)
                    continue;   // beyond the pole cap's line
                double d = Math.hypot(geo.dx(x, s), z - s.z());
                dAny = Math.min(dAny, d);
                if (s.kind() == Geothermal.Kind.POOL)
                    dPool = Math.min(dPool, d);
                else if (s.kind() == Geothermal.Kind.VENT)
                    dVent = Math.min(dVent, d);
            }
            for (Magma.Body b : magma.bodies()) {
                if (b.chamberRh() <= 0 || b.submerged() || Math.abs(b.z()) > zMax)
                    continue;
                dVolcano = Math.min(dVolcano, Math.hypot(magma.dx(x, b), z - b.z()));
            }
            any[count] = dAny;
            pool[count] = dPool;
            vent[count] = dVent;
            volcano[count] = dVolcano;
            count++;
        }
        JsonObject reach = new JsonObject();
        reach.addProperty("landPoints", count);
        reach.addProperty("metresPerBlock", wd.manifest().mPerBlockH());
        reach.add("toNearestSiteBlocks", stats(any, count));
        reach.add("toNearestHotSpringPoolBlocks", stats(pool, count));
        reach.add("toNearestVentBlocks", stats(vent, count));
        reach.add("toActiveVolcanoOnLandBlocks", stats(volcano, count));
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
}
