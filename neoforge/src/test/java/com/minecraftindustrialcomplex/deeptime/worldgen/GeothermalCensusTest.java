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
                wd.manifest().mPerBlockH(), wd.manifest().mPerBlockV(), (int) Math.abs(com.minecraftindustrialcomplex.deeptime.worlddata.Projection.zOfLatitude(c, com.minecraftindustrialcomplex.deeptime.wrap.PoleCapCurve.DEFAULT_CAP_DEG)));
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
        // The hot ground: the share of land columns (a coarse grid) at each heat-flow level.
        int step = 16;
        long land = 0, withData = 0;
        long[] over = new long[6];
        double[] bars = {70, 80, 90, 100, 120, 150};
        int pole = (int) com.minecraftindustrialcomplex.deeptime.worlddata.Projection.poleZ(c);
        for (int z = -pole + step / 2; z < pole; z += step)
            for (int x = -c / 2 + step / 2; x < c / 2; x += step) {
                if (Magma.top(h.at(x, z)) < sea + 5 || wet.at(x, z))
                    continue;
                land++;
                double q = env.heatFlowMwM2(x, z);
                if (Double.isNaN(q))
                    continue;
                withData++;
                for (int k = 0; k < bars.length; k++)
                    if (q >= bars[k])
                        over[k]++;
            }
        StringBuilder sb = new StringBuilder("CENSUS dry land samples=" + land + " continental=" + withData);
        for (int k = 0; k < bars.length; k++)
            sb.append(String.format(" >=%.0f:%.4f", bars[k], land == 0 ? 0.0 : (double) over[k] / land));
        System.out.println(sb);
    }
}
