// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.climate.ClimateMapping;
import com.minecraftindustrialcomplex.deeptime.climate.Peatland;
import com.minecraftindustrialcomplex.deeptime.worlddata.Rivers;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;

/**
 * The terrain source of a real Deep Time level: the world data's rasters (plan §7.1). Pure Java
 * over memory-mapped tile packs, so chunk generation never needs the natives. The data is looked
 * up in {@link ActiveWorldData} on first use and then kept, with the world's climate mapping
 * (P1-09, plan §7.2), which the five biome inputs pass through.
 */
public final class WorldDataTerrain implements TerrainSource {

    private final WorldDataRef ref;
    private volatile WorldData data;
    private volatile ClimateMapping mapping;
    private volatile LavaRules lava;
    private volatile Magma magma;
    private volatile MagmaRules magmaRules;
    private volatile SeaCliffGuard.Rules cliffRules;
    private volatile Geothermal geothermal;
    private volatile GeothermalRules geothermalRules;
    private volatile Peatland peat;
    private volatile java.util.function.Supplier<Peatland> peatDecider;
    private final Object peatLock = new Object();
    private volatile VolcanicBiomeRules volcanicBiomeRules;
    private volatile VolcanicZones volcanicZones;
    private volatile WaterCaveShield waterCaveShield;
    private volatile LavaFlows lavaFlows;

    public WorldDataTerrain(WorldDataRef ref) {
        this.ref = ref;
    }

    public WorldData data() {
        WorldData d = data;
        if (d == null) {
            d = ActiveWorldData.get(ref.folder());
            if (d.circumference() != ref.circumferenceBlocks())
                throw new IllegalStateException("world data '" + ref.folder() + "' has C = " + d.circumference()
                        + " but its generator says circumference_blocks = " + ref.circumferenceBlocks());
            if (!d.projection().kind().id().equals(ref.projection()))
                throw new IllegalStateException("world data '" + ref.folder() + "' is a " + d.projection().kind().id()
                        + " world but its generator says projection = " + ref.projection()
                        + " (the level type must match the preset's scale.projection)");
            data = d;
        }
        return d;
    }

    /** The world's climate mapping (identity for worlds without one). */
    public ClimateMapping mapping() {
        ClimateMapping m = mapping;
        if (m == null) {
            data();
            m = ActiveWorldData.mapping(ref.folder());
            mapping = m;
        }
        return m;
    }

    /** The world's lava rules ({@code notes.bake.underground.params}, preset section {@code underground}). */
    @Override
    public LavaRules lava() {
        LavaRules l = lava;
        if (l == null) {
            l = LavaRules.of(data().manifest().notes());
            lava = l;
        }
        return l;
    }

    /** The world's setting for the sea-cliff cave guard. */
    public SeaCliffGuard.Rules cliffRules() {
        SeaCliffGuard.Rules r = cliffRules;
        if (r == null) {
            r = SeaCliffGuard.Rules.of(data().manifest().notes());
            cliffRules = r;
        }
        return r;
    }

    /** The world's settings for lava at its volcanoes. */
    public MagmaRules magmaRules() {
        MagmaRules r = magmaRules;
        if (r == null) {
            r = MagmaRules.of(data().manifest().notes());
            magmaRules = r;
        }
        return r;
    }

    /**
     * The magma under and in the world's active volcanoes, built on first use from the baked list,
     * the terrain height and the water columns (so every chunk sees the same bodies).
     *
     * @param cell   the generator's noise cell width (the water rule's reach)
     * @param minY   the level's floor, {@code seaLevel} its sea level
     */
    public Magma magma(int cell, int minY, int seaLevel) {
        Magma m = magma;
        if (m == null) {
            synchronized (this) {
                if (magma == null) {
                    WorldData wd = data();
                    var spec = com.minecraftindustrialcomplex.deeptime.worlddata.Water.Spec.of(wd, cell);
                    var none = com.minecraftindustrialcomplex.deeptime.worlddata.Water.Kind.NONE;
                    magma = Magma.build(wd.volcanoes().list(), magmaRules(), (x, z) -> wd.height(wrapX(x), z),
                            (x, z) -> com.minecraftindustrialcomplex.deeptime.worlddata.Water.column(wd, spec, wrapX(x), z).kind() != none,
                            wd.circumference(), seaLevel, lava().floodY(minY, seaLevel), minY, wd.manifest().mPerBlockH(),
                            wd.manifest().mPerBlockV());
                    long lakes = magma.bodies().stream().filter(Magma.Body::lake).count();
                    long molten = magma.bodies().stream().filter(b -> b.chamberRh() > 0).count();
                    com.minecraftindustrialcomplex.deeptime.DeepTime.LOGGER.info(
                            "[deeptime] magma: {} active volcanoes, {} with a chamber, {} with a crater lava lake ({})",
                            wd.volcanoes().list().size(), molten, lakes, magmaRules());
                }
                m = magma;
            }
        }
        return m;
    }

    /**
     * The lava flows on the world's active volcanoes ({@link LavaFlows}), indexed on first use from the baked
     * list; each volcano's flows are traced from the terrain height, the water columns and the rain the first
     * time a column within its reach is asked about, so every chunk and far terrain see the same flows.
     *
     * @param cell the generator's noise cell width (the water rule's reach)
     */
    public LavaFlows lavaFlows(int cell) {
        LavaFlows f = lavaFlows;
        if (f == null) {
            synchronized (this) {
                if (lavaFlows == null) {
                    WorldData wd = data();
                    var spec = com.minecraftindustrialcomplex.deeptime.worlddata.Water.Spec.of(wd, cell);
                    var none = com.minecraftindustrialcomplex.deeptime.worlddata.Water.Kind.NONE;
                    var projection = wd.projection();
                    var precip = wd.has(GeothermalEnv.PRECIP) ? wd.layer(GeothermalEnv.PRECIP) : null;
                    double mH = wd.manifest().mPerBlockH();
                    int c = wd.circumference();
                    LavaFlows.Env env = new LavaFlows.Env() {
                        /** x folded into [−C/2, C/2) as {@link #wrapX(int)} folds a block's. */
                        private double fold(double x) {
                            return x - c * Math.floor((x + c / 2) / (double) c);
                        }

                        @Override
                        public double height(double x, double z) {
                            return wd.height(fold(x), z);
                        }

                        @Override
                        public boolean wet(int x, int z) {
                            return com.minecraftindustrialcomplex.deeptime.worlddata.Water.column(wd, spec, wrapX(x), z).kind() != none;
                        }

                        @Override
                        public double precipMm(double x, double z) {
                            return precip == null ? Double.NaN : precip.bilinear(fold(x), z);
                        }

                        @Override
                        public double[] metric(double x, double z) {
                            return LavaFlows.sphereMetric(projection, x, z, mH);
                        }
                    };
                    // a cube world's flows cross its seams (cube plan G16); its z range is the net window's
                    var cube = projection instanceof com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Cube cp ? cp.cube() : null;
                    lavaFlows = new LavaFlows(wd.volcanoes().list(), env, cube, wd.circumference(), projection.zMin(), projection.zMax(), mH,
                            wd.manifest().mPerBlockV(), line -> DeepTime.LOGGER.info("[deeptime] lava flows: {}", line));
                    DeepTime.LOGGER.info("[deeptime] lava flows: {} active volcanoes indexed ({} climate); each traces its flows on first use",
                            lavaFlows.size(), precip == null ? "no" : "the baked");
                }
                f = lavaFlows;
            }
        }
        return f;
    }

    /** The world's settings for geothermal features. */
    public GeothermalRules geothermalRules() {
        GeothermalRules r = geothermalRules;
        if (r == null) {
            r = GeothermalRules.of(data().manifest().notes());
            geothermalRules = r;
        }
        return r;
    }

    /**
     * The geothermal sites of the world's active volcanoes, built on first use from the same list,
     * terrain height and water columns as the magma (so every chunk sees the same sites).
     *
     * @param cell   the generator's noise cell width (the water rule's reach)
     * @param minY   the level's floor, {@code seaLevel} its sea level
     */
    public Geothermal geothermal(int cell, int minY, int seaLevel) {
        Geothermal g = geothermal;
        if (g == null) {
            synchronized (this) {
                if (geothermal == null) {
                    WorldData wd = data();
                    Magma m = magma(cell, minY, seaLevel);
                    var spec = com.minecraftindustrialcomplex.deeptime.worlddata.Water.Spec.of(wd, cell);
                    var none = com.minecraftindustrialcomplex.deeptime.worlddata.Water.Kind.NONE;
                    long t0 = System.nanoTime();
                    geothermal = Geothermal.build(wd.volcanoes().list(), geothermalRules(), m, (x, z) -> wd.height(wrapX(x), z),
                            (x, z) -> com.minecraftindustrialcomplex.deeptime.worlddata.Water.column(wd, spec, wrapX(x), z).kind() != none,
                            new GeothermalEnv(wd), wd.circumference(), seaLevel, wd.manifest().mPerBlockH());
                    int[] n = geothermal.counts();
                    com.minecraftindustrialcomplex.deeptime.DeepTime.LOGGER.info(
                            "[deeptime] geothermal: {} sites ({} from hot ground) from {} active volcanoes: {} pools, {} vents, {} patches of altered ground ({} ms; {})",
                            geothermal.sites().size(), geothermal.heatFlowSites(), wd.volcanoes().list().size(), n[0], n[1], n[2],
                            (System.nanoTime() - t0) / 1_000_000L, geothermalRules());
                }
                g = geothermal;
            }
        }
        return g;
    }

    /**
     * What the heat-flow fields of {@link #geothermal} could find on this world ({@link Geothermal#heatFlowOdds}), on the
     * same terrain height, water columns and climate, sampled every {@code step} blocks (a whole-world scan: for checks
     * and reports, not chunk generation).
     */
    public Geothermal.HeatFlowOdds geothermalHeatFlowOdds(int cell, int seaLevel, int step) {
        WorldData wd = data();
        var spec = com.minecraftindustrialcomplex.deeptime.worlddata.Water.Spec.of(wd, cell);
        var none = com.minecraftindustrialcomplex.deeptime.worlddata.Water.Kind.NONE;
        return Geothermal.heatFlowOdds(geothermalRules(), (x, z) -> wd.height(wrapX(x), z),
                (x, z) -> com.minecraftindustrialcomplex.deeptime.worlddata.Water.column(wd, spec, wrapX(x), z).kind() != none,
                new GeothermalEnv(wd), wd.circumference(), seaLevel, wd.manifest().mPerBlockH(), step);
    }

    /**
     * How the level asks its biome source where the pack has a peatland biome (BIO-106): set by the
     * generator, called at the first flagged cold-wetland column of a world whose mapping has no verdict
     * yet. It answers null while it cannot (TerraBlender's trees are not built).
     */
    public void peatlandDecider(java.util.function.Supplier<Peatland> decider) {
        peatDecider = decider;
    }

    /** The world's peatland verdict, from its mapping or decided now and stored there; null while undecided. */
    public Peatland peatland() {
        Peatland p = peat;
        if (p != null)
            return p;
        ClimateMapping m = mapping();
        if (m.peatland() != null)
            return peat = m.peatland();
        java.util.function.Supplier<Peatland> d = peatDecider;
        if (d == null || m.isIdentity())
            return null;
        synchronized (peatLock) {
            if (peat != null)
                return peat;
            Peatland v = d.get();
            if (v == null)
                return null;
            ClimateMapping updated = m.withPeatland(v);
            ActiveWorldData.putMapping(ref.folder(), updated);
            mapping = updated;
            try {
                updated.write(data().folder().resolve(ClimateMapping.FILE));
            } catch (java.io.IOException e) {
                DeepTime.LOGGER.error("Cannot write {} ({}): the peatland verdict holds for this start only", ClimateMapping.FILE, e.getMessage());
            }
            return peat = v;
        }
    }

    /** The peatland verdict where it applies to cold wetland at (x, z): flagged ground in a peatland region, else null. */
    private Peatland peatlandAt(ClimateMapping m, int x, int z) {
        WorldData d = data();
        if (!d.has(Peatland.LAYER) || Math.round(d.layer(Peatland.LAYER).nearest(x, z)) != Peatland.COLD_WETLAND)
            return null;
        Peatland p = peatland();
        if (p == null || p.regions().isEmpty())
            return null;
        String region = Peatland.NO_TABLE;
        if (m.regions() != null && d.has("meta/province")) {
            double v = d.layer("meta/province").nearest(x, z);
            region = Double.isNaN(v) ? "?" : m.regions().regionOf((int) v);
        }
        return p.appliesTo(region) ? p : null;
    }

    /** The world's setting for the volcanic biomes. */
    public VolcanicBiomeRules volcanicBiomeRules() {
        VolcanicBiomeRules r = volcanicBiomeRules;
        if (r == null) {
            r = VolcanicBiomeRules.of(data().manifest().notes());
            volcanicBiomeRules = r;
        }
        return r;
    }

    /**
     * Where the world's volcanic biomes go (Ben, decisions log 2026-10-03), built on first use from the
     * active volcanoes, their geothermal sites and their magma (so every biome query sees the same zones).
     *
     * @param cell   the generator's noise cell width (the water rule's reach)
     * @param minY   the level's floor, {@code seaLevel} its sea level
     */
    public VolcanicZones volcanicZones(int cell, int minY, int seaLevel) {
        VolcanicZones z = volcanicZones;
        if (z == null) {
            synchronized (this) {
                if (volcanicZones == null) {
                    WorldData wd = data();
                    Magma m = magma(cell, minY, seaLevel);
                    Geothermal g = geothermal(cell, minY, seaLevel);
                    volcanicZones = VolcanicZones.build(wd.volcanoes().list(), g.sites(), m.bodies(), volcanicBiomeRules(),
                            wd.manifest().mPerBlockH(), wd.circumference(), wd.projection(),
                            new VolcanicZones.Ground((gx, gz) -> wd.height(wrapX(gx), gz), seaLevel, wd.manifest().mPerBlockV()));
                    java.util.Map<VolcanicZones.Role, Long> byRole = new java.util.EnumMap<>(VolcanicZones.Role.class);
                    for (VolcanicZones.Zone zn : volcanicZones.zones())
                        byRole.merge(zn.role(), 1L, Long::sum);
                    com.minecraftindustrialcomplex.deeptime.DeepTime.LOGGER.info("[deeptime] volcanic biome zones: {} ({})", byRole,
                            volcanicBiomeRules());
                }
                z = volcanicZones;
            }
        }
        return z;
    }

    @Override
    public int circumference() {
        return ref.circumferenceBlocks();
    }

    @Override
    public boolean cube() {
        return ref.cube();
    }

    @Override
    public double height(int x, int z) {
        return data().height(wrapX(x), z);
    }

    @Override
    public boolean closedBasin(int x, int z) {
        return data().closedBasin(wrapX(x), z);
    }

    /** The cave-entrance shield beside the world's water (HYD-078), with the planet's 4-block noise cell as the shore reach. */
    @Override
    public double waterShield(int x, int z) {
        WaterCaveShield s = waterCaveShield;
        if (s == null) {
            WorldData wd = data();
            s = new WaterCaveShield(wd, com.minecraftindustrialcomplex.deeptime.worlddata.Water.Spec.of(wd, PLANET_CELL));
            waterCaveShield = s;
        }
        return s.at(x, z);
    }

    /** {@code deeptime:planet}'s noise cell width, blocks (size_horizontal 1), which the shore rule's reach equals. */
    private static final int PLANET_CELL = 4;

    /**
     * A biome input through the world's mapping. In a world the P1-09 bake wrote, the weirdness is
     * sampled as an odd field ({@code RasterSampler.bilinearOdd}): its sign flips at the variant
     * noise's zero line, and a plain bilinear ramp through 0 there would draw lines of river biome
     * (the valley slice) across the land.
     */
    @Override
    public double raster(String layer, int x, int z) {
        ClimateMapping m = mapping();
        if (ClimateMapping.RIDGES.equals(layer)) {
            // P1-22: a channel wide enough for a river biome puts the column in the valley slice
            // (weirdness 0, the middle of the river slice of every overworld-style parameter list).
            Rivers r = data().rivers();
            if (r != null && r.sample(wrapX(x), z).biome())
                return 0.0;
        }
        if (!m.isIdentity() && ClimateMapping.RIDGES.equals(layer))
            return m.apply(layer, data().layer(layer).bilinearOdd(wrapX(x), z));
        if (ClimateMapping.EROSION.equals(layer) && !m.isIdentity()) {
            // Cold wetland (BIO-133, BIO-135): taiga in the ordinary erosion; the swamp band, where the pack
            // draws its bog or muskeg, in the regions of the pack that have one.
            Peatland p = peatlandAt(m, wrapX(x), z);
            if (p != null)
                return p.erosion();
        }
        return m.apply(layer, data().raster(layer, wrapX(x), z));
    }
}
