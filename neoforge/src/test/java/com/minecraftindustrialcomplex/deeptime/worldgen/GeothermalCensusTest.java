// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.Water;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;

import java.nio.file.Path;
import java.util.Map;
import java.util.TreeMap;

/**
 * A measuring tool, not a check: counts the geothermal sites of a baked world by kind and by heat
 * source (volcano or the ground's heat flow), with the heat-flow fields on and off, and the share
 * of continental land that reaches the heat-flow threshold. Runs only when
 * {@code DEEPTIME_CENSUS_WORLD} names a world-data folder (for example one written by
 * {@code deeptime run --preset earthlike_quick_16k ... --out <dir>}); prints to stdout.
 *
 * <pre>DEEPTIME_CENSUS_WORLD=&lt;folder&gt; ./gradlew test --tests '*GeothermalCensusTest' -i</pre>
 */
class GeothermalCensusTest {

    @Test
    @EnabledIfEnvironmentVariable(named = "DEEPTIME_CENSUS_WORLD", matches = ".+")
    void census() {
        WorldData wd = WorldData.open(Path.of(System.getenv("DEEPTIME_CENSUS_WORLD")));
        int sea = 63, minY = Integer.getInteger("deeptime.minY", -512), cell = 4;
        GeothermalRules rules = GeothermalRules.of(wd.manifest().notes());
        var spec = Water.Spec.of(wd, cell);
        var none = Water.Kind.NONE;
        int c = wd.circumference();
        Magma.Wet wet = (x, z) -> Water.column(wd, spec, Math.floorMod(x + c / 2, c) - c / 2, z).kind() != none;
        Magma.Heights h = (x, z) -> wd.height(Math.floorMod(x + c / 2, c) - c / 2, z);
        LavaRules lava = LavaRules.of(wd.manifest().notes());
        Magma magma = Magma.build(wd.volcanoes().list(), MagmaRules.of(wd.manifest().notes()), h, wet, c, sea, lava.floodY(minY, sea),
                wd.manifest().mPerBlockH(), wd.manifest().mPerBlockV());
        GeothermalEnv env = new GeothermalEnv(wd);
        System.out.println("CENSUS world C=" + c + " volcanoes=" + wd.volcanoes().list().size() + " rules=" + rules);
        for (double heatDensity : new double[]{0.0, rules.heatDensity()}) {
            GeothermalRules r = new GeothermalRules(rules.density(), rules.reach(), rules.size(), rules.springMinPrecipMm(),
                    rules.heatFlowMinMwM2(), heatDensity);
            long t0 = System.nanoTime();
            Geothermal g = Geothermal.build(wd.volcanoes().list(), r, magma, h, wet, env, c, sea, wd.manifest().mPerBlockH());
            long ms = (System.nanoTime() - t0) / 1_000_000L;
            Map<String, Integer> by = new TreeMap<>();
            for (Geothermal.Site s : g.sites()) {
                String src = s.volcano() == Geothermal.HEAT_FLOW_SOURCE ? "heat-flow" : "volcano";
                by.merge(src + " " + s.kind(), 1, Integer::sum);
            }
            long volcanoes = g.sites().stream().filter(s -> s.volcano() != Geothermal.HEAT_FLOW_SOURCE).map(Geothermal.Site::volcano)
                    .distinct().count();
            System.out.println("CENSUS heatDensity=" + heatDensity + " sites=" + g.sites().size() + " heatFlowSites=" + g.heatFlowSites()
                    + " volcanoesWithSites=" + volcanoes + " by=" + by + " buildMs=" + ms);
        }
        // The hot ground: the share of land columns (a coarse grid) at each heat-flow level, over the world's own
        // z span (the projection's, as the fields use).
        int step = 16;
        long land = 0, withData = 0, nanContinental = 0, nanNearestHas = 0, nanOceanic = 0;
        double[] bars = {41, 45, 50, 55, 60, 70, 80, 100, 120, 150};
        long[] over = new long[bars.length];
        double qMax = Double.NEGATIVE_INFINITY;
        int pole = (int) (Double.isNaN(env.zHalf()) ? com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Lambert.radius(c) : env.zHalf());
        var crust = wd.has("meta/crust_type") ? wd.layer("meta/crust_type") : null;
        var heat = wd.has("meta/heat_flow_mw_m2") ? wd.layer("meta/heat_flow_mw_m2") : null;
        for (int z = -pole + step / 2; z < pole; z += step)
            for (int x = -c / 2 + step / 2; x < c / 2; x += step) {
                if (Magma.top(h.at(x, z)) < sea + 5 || wet.at(x, z))
                    continue;
                land++;
                double q = env.heatFlowMwM2(x, z);
                if (Double.isNaN(q)) {
                    // Why no heat flow: oceanic crust under the land, or a continental column beside it (the bilinear
                    // read is NaN when one of its four cells is oceanic).
                    double ct = crust == null ? Double.NaN : crust.nearest(x, z);
                    if (ct == 1.0)
                        nanContinental++;
                    else if (ct == 0.0)
                        nanOceanic++;
                    if (heat != null && !Double.isNaN(heat.nearest(x, z)))
                        nanNearestHas++;
                    continue;
                }
                withData++;
                qMax = Math.max(qMax, q);
                for (int k = 0; k < bars.length; k++)
                    if (q >= bars[k])
                        over[k]++;
            }
        StringBuilder sb = new StringBuilder("CENSUS dry land samples=" + land + " continental=" + withData + " zHalf=" + pole
                + String.format(" qMax=%.1f", qMax) + " noHeatFlow: crustContinental=" + nanContinental + " crustOceanic="
                + nanOceanic + " nearestCellHasHeatFlow=" + nanNearestHas);
        for (int k = 0; k < bars.length; k++)
            sb.append(String.format(" >=%.0f:%.4f", bars[k], land == 0 ? 0.0 : (double) over[k] / land));
        System.out.println(sb);
        // The layer itself, on every land and sea column with data (the four corners of its bilinear read continental).
        long any = 0, nan = 0;
        long[] layerOver = new long[bars.length];
        for (int z = -pole + step / 2; z < pole; z += step)
            for (int x = -c / 2 + step / 2; x < c / 2; x += step) {
                double q = env.heatFlowMwM2(x, z);
                if (Double.isNaN(q)) {
                    nan++;
                    continue;
                }
                any++;
                for (int k = 0; k < bars.length; k++)
                    if (q >= bars[k])
                        layerOver[k]++;
            }
        sb = new StringBuilder("CENSUS layer samples=" + (any + nan) + " continental=" + any);
        for (int k = 0; k < bars.length; k++)
            sb.append(String.format(" >=%.0f:%.4f", bars[k], any == 0 ? 0.0 : (double) layerOver[k] / any));
        System.out.println(sb);
        // The fields' own funnel (Geothermal.heatFlowFields' cells and draws, read here): of the draws, those on
        // continental crust, at the threshold, and also dry land clear of ice; the cells with one such draw.
        double side = Math.sqrt(Geothermal.HEAT_FIELD_AREA_KM2 / rules.heatDensity()) * 1000.0 / wd.manifest().mPerBlockH();
        int nx = Math.max(1, (int) Math.round(c / side)), nz = Math.max(1, (int) Math.round(2.0 * pole / side));
        double wx = (double) c / nx, wz = 2.0 * pole / nz;
        long draws = 0, cont = 0, hot = 0, hotDry = 0, cells = 0, hotCells = 0;
        for (int gz = 0; gz < nz; gz++)
            for (int gx = 0; gx < nx; gx++) {
                java.util.SplittableRandom rng = new java.util.SplittableRandom(
                        com.minecraftindustrialcomplex.deeptime.rock.PaletteRules.mix(0x4EA7_F10EL ^ ((long) gx << 32) ^ (gz & 0xFFFFFFFFL)));
                boolean got = false;
                for (int draw = 0; draw < Geothermal.HEAT_DRAWS; draw++) {
                    int fx = Math.floorMod((int) Math.round(-c / 2.0 + (gx + rng.nextDouble()) * wx) + c / 2, c) - c / 2;
                    int fz = (int) Math.round(-pole + (gz + rng.nextDouble()) * wz);
                    draws++;
                    double q = env.heatFlowMwM2(fx, fz);
                    if (Double.isNaN(q))
                        continue;
                    cont++;
                    if (!(q >= rules.heatFlowMinMwM2()))
                        continue;
                    hot++;
                    if (Magma.top(h.at(fx, fz)) < sea + Geothermal.MIN_ABOVE_SEA + 2 || wet.at(fx, fz) || env.iced(fx, fz))
                        continue;
                    hotDry++;
                    got = true;
                }
                cells++;
                if (got)
                    hotCells++;
            }
        System.out.println("CENSUS fields cells=" + cells + " (" + nx + "x" + nz + ", side " + Math.round(side) + " blocks) draws=" + draws
                + " continental=" + cont + " atThreshold=" + hot + " atThresholdDry=" + hotDry + " cellsWithOne=" + hotCells);
        // What the draws could find (Geothermal.heatFlowOdds, the GameTest's figure): each cell's share of hot dry
        // ground on a 4-block grid, the expected count of cells with a field and the chance that none has one.
        System.out.println("CENSUS " + Geothermal.heatFlowOdds(rules, h, wet, env, c, sea, wd.manifest().mPerBlockH(), 4));
        // Each active volcano: its crust (the volcano list's flag and whether the heat-flow layer has data there), the
        // heat flow at its summit and the hottest within its base, and the summit's height above the sea.
        for (var v : wd.volcanoes().list()) {
            if (!(v.activity() > 0.0))
                continue;
            int vx = Math.floorMod((int) Math.round(v.x()) + c / 2, c) - c / 2, vz = (int) Math.round(v.z());
            double r = Math.max(6.0, v.radiusM() / wd.manifest().mPerBlockH());
            double q0 = env.heatFlowMwM2(vx, vz), qm = Double.NaN;
            for (int dz = (int) -r; dz <= r; dz += 4)
                for (int dx = (int) -r; dx <= r; dx += 4)
                    if (dx * dx + dz * dz <= r * r) {
                        double q = env.heatFlowMwM2(Math.floorMod(vx + dx + c / 2, c) - c / 2, vz + dz);
                        if (!Double.isNaN(q) && !(q <= qm))
                            qm = q;
                    }
            System.out.println(String.format("CENSUS volcano id=%d kind=%s at=(%d,%d) activity=%.2f oceanic=%s r=%.0f q=%.1f qMaxInBase=%.1f top=%d",
                    v.id(), v.kind(), vx, vz, v.activity(), v.oceanic(), r, q0, qm, Magma.top(h.at(vx, vz)) - sea));
        }
    }
}
