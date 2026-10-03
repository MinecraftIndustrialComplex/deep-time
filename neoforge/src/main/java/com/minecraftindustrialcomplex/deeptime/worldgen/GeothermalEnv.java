// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.IceCover;
import com.minecraftindustrialcomplex.deeptime.worlddata.RasterSampler;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

import java.util.List;
import java.util.Optional;
import java.util.Set;

/**
 * What {@link Geothermal} asks of a world: the baked rain and mean temperature (a world without a
 * climate answers NaN: warm and wet), the ice cover, and the rock under a column from the strata
 * horizons (carbonate rock makes a hot spring's terrace travertine; everything else, sinter), and
 * the plate model's heat flow on continental crust ({@code meta/heat_flow_mw_m2}; a world without
 * the layer answers NaN: no heat-flow fields).
 */
final class GeothermalEnv implements Geothermal.Env {

    static final String PRECIP = "climate/p_mm", MEAN_C = "climate/t_mean_c", HEAT_FLOW = "meta/heat_flow_mw_m2";

    /** The rock classes of calcium carbonate that a spring can precipitate travertine from. */
    static final Set<String> CARBONATE = Set.of("limestone", "dolostone", "chalk", "marble", "travertine");

    private final WorldData wd;
    private final RasterSampler precip, mean, strata, heat;
    private final Optional<IceCover> ice;
    private final List<String> classes;

    GeothermalEnv(WorldData wd) {
        this.wd = wd;
        this.precip = wd.has(PRECIP) ? wd.layer(PRECIP) : null;
        this.mean = wd.has(MEAN_C) ? wd.layer(MEAN_C) : null;
        this.heat = wd.has(HEAT_FLOW) ? wd.layer(HEAT_FLOW) : null;
        this.ice = wd.iceCover();
        this.strata = wd.has(WorldData.HORIZONS) ? wd.layer(WorldData.HORIZONS) : null;
        this.classes = strata == null ? List.of() : wd.classes();
    }

    @Override
    public double precipMm(int x, int z) {
        return precip == null ? Double.NaN : precip.bilinear(x, z);
    }

    @Override
    public double meanC(int x, int z) {
        return mean == null ? Double.NaN : mean.bilinear(x, z);
    }

    @Override
    public double heatFlowMwM2(int x, int z) {
        return heat == null ? Double.NaN : heat.bilinear(x, z);
    }

    @Override
    public boolean iced(int x, int z) {
        return ice.isPresent() && ice.get().blocks(x, z) > 0;
    }

    @Override
    public boolean carbonate(int x, int z, int groundY) {
        if (strata == null)
            return false;
        StrataPass.Column col = new StrataPass.Column(Math.max(1, strata.stackDepth()));
        StrataPass.column(wd, strata, x, z, col);
        for (int h = 0; h < col.count; h++) {
            if (col.tops[h] < groundY - Geothermal.CARBONATE_WINDOW)
                break;
            int cls = col.classes[h];
            if (cls >= 0 && cls < classes.size() && CARBONATE.contains(classes.get(cls)))
                return true;
        }
        return false;
    }
}
