// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules;
import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;

import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.SplittableRandom;

/**
 * Where a Deep Time world has geothermal surface features (P2-14; plan §10.4): hot springs with
 * sinter or travertine terraces, fumaroles with native sulfur, and bare altered ground, in plain
 * Java (no Minecraft types, so JUnit can test it). Built once per level from the same source of
 * truth as the magma ({@link Magma}): the baked list of active volcanoes ({@link Volcanoes}), the
 * world's settings ({@link GeothermalRules}), the terrain height and the water columns, so every
 * chunk and every thread sees the same sites; {@link GeothermalPass} places the blocks. A site's
 * every block is decided from its record and the column's position alone, and once.
 *
 * <p><b>Where.</b> Only where the simulation has heat. The bake exports two heat sources: the
 * active volcanoes ({@code volcanoes/active.json}) and the plate model's heat flow on continental
 * crust ({@code meta/heat_flow_mw_m2}; Ben, decisions log 2026-10-02, "Export heat flow"; its
 * spreading ridges are under the sea). Each volcano gets a few <i>fields</i> (more at the more
 * active and larger ones, times {@code geothermal_density}), the first near the summit, the others
 * out on the flanks as far as {@code geothermal_reach} basal radii. Beyond the volcanoes, ground
 * whose baked heat flow reaches {@code geothermal_heat_flow_min_mw_m2} (rifts, arcs, hotspot swells
 * and young belts in the plate model) takes at most one field for each {@link #HEAT_FIELD_AREA_KM2}
 * (divided by {@code geothermal_heat_density}): the cells of a grid of that area are each
 * given a field at the first of {@link #HEAT_DRAWS} random points that is hot enough, dry and
 * clear of ice, so the fields follow the hot ground.
 *
 * <p><b>What.</b> The setting at a field decides its kind:
 * <ul>
 *   <li>near the summit, or where the year's rain is under {@code geothermal_spring_min_precip_mm}, or
 *       where the mean temperature is under 3 °C ({@link #POOL_MIN_C}): a <b>fumarole field</b>: vents (hot ground,
 *       a crust of native sulfur around each) in discs of bleached and stained altered ground;</li>
 *   <li>elsewhere, where groundwater is plentiful and the ground does not freeze: a <b>hot-spring
 *       field</b>: a main pool and one to three smaller ones, in a chain down the slope where there
 *       is one (a terrace), each pool ringed by sinter (white, siliceous) or, where carbonate rock
 *       lies under the site, travertine; a vent or a patch of altered ground now and then.
 *       Silicic volcanoes' altered ground carries stains of cinnabar, the mercury ore of
 *       epithermal systems.</li>
 * </ul>
 * Nothing stands in or beside water (the sea, a lake, a river, a bank), under ice, within a few
 * blocks of a magma body's open crater or plug, or where a pool on a slope would need more than
 * {@link #CUT} blocks of ground cut away uphill or {@link #DAM} blocks of dam built downhill to sit level.
 * On a cube world every column of a site is on the six faces ({@link Env#onPlanet}): a site off them would
 * never be built, and one across a cut seam would be cut in two.
 *
 * <p><b>Water safety.</b> A pool's water is bounded on every side by its rim, its bed or the rock
 * under it, and every block of a pool and its rim is in the chunk's carving masks. It lies at
 * {@code level}, the mean ground of its footprint: the ground above is cut away, and where the ground
 * is lower the rim and bed are built up to the water (the dam of a travertine terrace), so the
 * water has a solid block on every side whatever the slope. A footprint never touches another
 * site's pool, rim or vent, and a vent's magma block never touches water.
 *
 * <p><b>Open water.</b> A hot spring's water is hot: {@link #poolSurface} says which block of the world is
 * the surface of a pool's water, and {@code LakeFreeze} (through {@code BiomeFreezeMixin}) never lets
 * vanilla's freezing, at generation or in weather, turn that block to ice, whatever the biome's temperature.
 * The ground round a pool may still take snow.
 */
public final class Geothermal {

    /** The most ground a pool may cut away uphill to sit level, blocks, and the most dam its rim and bed may build downhill. */
    public static final int CUT = 6, DAM = 6; // ledger: GEO-034 (game-choice)
    /** Ground under this many blocks above the sea takes no site (coast, flood plain, tidal reach). */
    public static final int MIN_ABOVE_SEA = 3; // ledger: GEO-017 (game-choice)
    /**
     * A field whose annual mean temperature is under this, °C, has no pools. Not physics: real hot
     * springs do not freeze at a 3 °C mean. It is a game rule (ledger: GEO-020, game-choice). It began as a
     * guard against vanilla's top-layer freezing putting ice on a hot pool; since 2026-10-20 a pool's
     * water stays open whatever the biome's temperature ({@link #poolSurface}, GEO-062; a pack's
     * hot-spring biome can be colder than the field's climate), so the threshold no longer keeps
     * ice off the pools and stays only because Ben confirmed it as a game choice on 2026-10-02.
     */
    public static final double POOL_MIN_C = 3.0;
    /**
     * Fields nearer the summit than this share of the basal radius are fumarolic whatever the rain.
     * The pattern (steam vents and acid-sulfate ground near the summit, neutral-chloride springs
     * lower down) is read in Paulson and Ingebritsen 1991 (USGS WRI 90-4201); the 0.35 is a game
     * choice (ledger: GEO-018).
     */
    public static final double SUMMIT_ZONE = 0.35;
    /** The rock window checked for carbonate under a field, blocks under the ground. */
    public static final int CARBONATE_WINDOW = 150; // ledger: GEO-023 (game-choice)
    /**
     * The area of hot continental ground (heat flow at or over {@code geothermal_heat_flow_min_mw_m2})
     * that takes one field at the default heat density, km2: the cells of the grid are this big
     * (100 km across). A game choice (ledger: GEO-053): real high-temperature fields in a province
     * stand from a few hundred to some thousand km2 apart (not sourced here).
     */
    public static final double HEAT_FIELD_AREA_KM2 = 10_000.0;
    /** Random points tried in a grid cell for a hot, dry, ice-free place to put its field. */
    public static final int HEAT_DRAWS = 6; // ledger: GEO-053 (game-choice)
    /** The {@link Site#volcano()} of a site whose heat is the ground's heat flow, not a volcano. */
    public static final int HEAT_FLOW_SOURCE = -1;

    /**
     * Peak warming over the climate, K, at the heart of each kind of site (the warm-ground signal).
     * Capped far below a real vent (Yellowstone's fumaroles reach 138 °C) so that the temperature
     * stays playable (ledger: GEO-044, game-choice).
     */
    public static final double WARM_POOL = 10.0, WARM_VENT = 12.0, WARM_PATCH = 6.0;
    /** The warm-ground signal reaches this far (blocks) beyond a site's altered ground, falling linearly. */
    public static final int WARM_FADE = 4;

    public enum Kind {
        POOL, VENT, PATCH
    }

    /** What a block column of a site is, with the priority that settles two sites' claims on it. */
    public enum Part {
        NONE(0), POOL_CORE(100), POOL_MAT(99), POOL_EDGE(98), RIM(90), VENT(80), SULFUR(70), APRON(50), APRON_STAIN(45),
        ALTERED(40), STAIN(39), CINNABAR(41);

        public final int priority;

        Part(int priority) {
            this.priority = priority;
        }

        /** Water stands here. */
        public boolean pool() {
            return this == POOL_CORE || this == POOL_MAT || this == POOL_EDGE;
        }

        /** Blocks of water in the column (the bed is under them). */
        public int depth() {
            return switch (this) {
                case POOL_CORE, POOL_MAT -> 2;
                case POOL_EDGE -> 1;
                default -> 0;
            };
        }
    }

    /** What the world says about the ground and climate at a column (the bake's rasters in the game). */
    public interface Env {
        /** The year's rain at the column, mm; NaN when the world has no climate (then there is water enough). */
        double precipMm(int x, int z);

        /** The annual mean temperature at the column, °C; NaN without a climate (then it is warm enough). */
        double meanC(int x, int z);

        /**
         * The half-extent of the world in z, blocks: the cube net's 3s/2, or the poles R = C/2π of a
         * Lambert-frame world; NaN to take the Lambert frame's poles of the circumference.
         */
        default double zHalf() {
            return Double.NaN;
        }

        /**
         * The plate model's heat flow at the column, mW/m2, on continental crust; NaN on oceanic
         * crust and in a world without the layer (then the ground has no heat of its own).
         */
        default double heatFlowMwM2(int x, int z) {
            return Double.NaN;
        }

        /** Whether the column lies under glacial ice. */
        boolean iced(int x, int z);

        /**
         * Whether column (x, z) is the planet's own ground, where a site is built: on a cube world a column of one of
         * its six faces. Off them a chunk is a turned copy of its partner's ground ({@code CubeFill}, ledger SCL-210),
         * made by the partner's own passes at the partner's place, so the geothermal pass never paints a site at the
         * outside place, while the world's rasters there read the partner's values (so a draw there finds the
         * partner's hot dry ground). Every column of any other world.
         */
        default boolean onPlanet(int x, int z) {
            return true;
        }

        /** Whether carbonate rock lies under the column within {@link Geothermal#CARBONATE_WINDOW} blocks of the ground. */
        boolean carbonate(int x, int z, int groundY);

        /** A world without a climate, ice or carbonate: warm, wet and silicic. */
        Env OPEN = new Env() {
            @Override
            public double precipMm(int x, int z) {
                return Double.NaN;
            }

            @Override
            public double meanC(int x, int z) {
                return Double.NaN;
            }

            @Override
            public boolean iced(int x, int z) {
                return false;
            }

            @Override
            public boolean carbonate(int x, int z, int groundY) {
                return false;
            }
        };
    }

    /**
     * One site.
     *
     * @param volcano the active volcano whose heat it is (its id)
     * @param x       the centre column (x canonical)
     * @param r       a pool's mean radius, a vent's radius (core and crust), a patch's 0
     * @param apron   mean radius of the terrace or the altered ground, blocks
     * @param level   a pool's water surface: the mean ground top of its pool and rim columns (0 for the others)
     * @param groundY the ground's top block at the centre when the site was drawn
     * @param sinter  a pool's terrace is sinter (siliceous); else travertine
     * @param silicic the volcano is silicic: its altered ground can carry cinnabar
     * @param seed    the site's own seed (its edge, its stains)
     */
    public record Site(int id, Kind kind, int volcano, int x, int z, int r, int apron, int level, int groundY,
                       boolean sinter, boolean silicic, long seed, double phase1, double phase2) {

        /** Horizontal radius that holds every column of the site (the outline wobbles up to a third wider). */
        public int reach() {
            return (int) Math.ceil((apron + 2) * 1.34) + 1;
        }

        /** Radius of the part that must stay clear of other sites' water: pool and rim, vent and crust. */
        public int hard() {
            return switch (kind) {
                case POOL -> (int) Math.ceil((r + 2) * 1.34);
                case VENT -> (int) Math.ceil((r + 1.3) * 1.34) + 1;
                case PATCH -> 0;
            };
        }

        private double wobble(double th) {
            return 1.0 + 0.16 * Math.sin(3.0 * th + phase1) + 0.09 * Math.sin(5.0 * th + phase2);
        }

        /** The distance from the centre as a share of the outline's local radius, in blocks of the mean outline. */
        double dd(int dx, int dz) {
            if (dx == 0 && dz == 0)
                return 0.0;
            return Math.sqrt((double) dx * dx + (double) dz * dz) / wobble(Math.atan2(dz, dx));
        }

        private boolean inPool(int dx, int dz) {
            return kind == Kind.POOL && dd(dx, dz) <= r;
        }

        /** What column (dx, dz) from the centre is. */
        public Part part(int dx, int dz) {
            int reach = reach();
            if (Math.abs(dx) > reach || Math.abs(dz) > reach)
                return Part.NONE;
            double d = dd(dx, dz);
            switch (kind) {
                case POOL -> {
                    if (d <= r) {
                        double f = d / r;
                        return f <= 0.45 ? Part.POOL_CORE : f <= 0.75 ? Part.POOL_MAT : Part.POOL_EDGE;
                    }
                    boolean touches = false;
                    for (int a = -1; a <= 1 && !touches; a++)
                        for (int b = -1; b <= 1 && !touches; b++)
                            touches = (a != 0 || b != 0) && inPool(dx + a, dz + b);
                    if (touches || d <= r + 1.5)
                        return Part.RIM;
                    if (d > apron)
                        return Part.NONE;
                    if (d > apron - 1.5 && unit(seed, dx, dz, 1) < 0.5)
                        return Part.NONE;
                    return unit(seed, dx, dz, 2) > 0.93 ? Part.APRON_STAIN : Part.APRON;
                }
                case VENT -> {
                    if (d <= r - 0.9)
                        return Part.VENT;
                    if (d <= r + 1.3)
                        return unit(seed, dx, dz, 3) < 0.8 ? Part.SULFUR : altered(dx, dz);
                    return ground(dx, dz, d);
                }
                default -> {
                    return ground(dx, dz, d);
                }
            }
        }

        private Part ground(int dx, int dz, double d) {
            if (d > apron)
                return Part.NONE;
            if (d > apron - 1.5 && unit(seed, dx, dz, 1) < 0.5)
                return Part.NONE;
            return altered(dx, dz);
        }

        private Part altered(int dx, int dz) {
            double h = unit(seed, dx, dz, 4);
            if (silicic && h < 0.06)
                return Part.CINNABAR;
            return h < 0.30 ? Part.STAIN : Part.ALTERED;
        }

        /** The warm-ground signal at the block at offset (dx, dz) and height y, K over the climate; 0 outside the site. */
        public double warmth(int dx, int dz, int y) {
            if (y < groundY - 10 - (kind == Kind.POOL ? 2 : 0) || y > groundY + 6)
                return 0.0;
            double d = Math.sqrt((double) dx * dx + (double) dz * dz);
            double inner = switch (kind) {
                case POOL -> r;
                case VENT -> r + 1.3;
                case PATCH -> apron * 0.5;
            };
            double outer = apron * 1.1 + WARM_FADE;
            if (d >= outer)
                return 0.0;
            double peak = switch (kind) {
                case POOL -> WARM_POOL;
                case VENT -> WARM_VENT;
                case PATCH -> WARM_PATCH;
            };
            return peak * Math.clamp((outer - d) / Math.max(1.0, outer - inner), 0.0, 1.0);
        }
    }

    /** A hash of (seed, dx, dz, salt) as a number in [0, 1). */
    static double unit(long seed, int dx, int dz, int salt) {
        long h = PaletteRules.mix(seed ^ PaletteRules.mix(((long) dx << 32) ^ (dz & 0xFFFFFFFFL) ^ ((long) salt << 56)));
        return (h >>> 11) * 0x1.0p-53;
    }

    public static final Geothermal NONE = new Geothermal(List.of(), 0);

    private final List<Site> sites;
    private final int circumference;
    /** The water levels of the pools (a pool's surface block is at its site's level), so most heights are refused at once. */
    private final Set<Integer> poolLevels;

    private Geothermal(List<Site> sites, int circumference) {
        this.sites = List.copyOf(sites);
        this.circumference = circumference;
        Set<Integer> levels = new HashSet<>();
        for (Site s : this.sites)
            if (s.kind == Kind.POOL)
                levels.add(s.level);
        this.poolLevels = Set.copyOf(levels);
    }

    public List<Site> sites() {
        return sites;
    }

    /** x − sx folded into [−C/2, C/2): the offset east of a site's centre across the seam. */
    public int dx(int x, Site s) {
        return fold(x - s.x, circumference);
    }

    static int fold(int d, int c) {
        return c <= 0 ? d : Math.floorMod(d + c / 2, c) - c / 2;
    }

    /** The sites whose reach touches block box [x0, x1] × [z0, z1]. */
    public List<Site> near(int x0, int z0, int x1, int z1) {
        List<Site> out = null;
        for (Site s : sites) {
            int reach = s.reach();
            if (s.z + reach < z0 || s.z - reach > z1)
                continue;
            int lo = dx(x0, s), hi = lo + (x1 - x0);
            if (hi < -reach || lo > reach)
                continue;
            if (out == null)
                out = new ArrayList<>(4);
            out.add(s);
        }
        return out == null ? List.of() : out;
    }

    /** The site that wins column (x, z) (the highest {@link Part#priority}, the first on a tie), or null. */
    public Site claim(int x, int z) {
        Part best = Part.NONE;
        Site winner = null;
        for (Site s : sites) {
            int reach = s.reach();
            if (Math.abs(z - s.z) > reach)
                continue;
            int dx = dx(x, s);
            if (Math.abs(dx) > reach)
                continue;
            Part p = s.part(dx, z - s.z);
            if (p.priority > best.priority) {
                best = p;
                winner = s;
            }
        }
        return winner;
    }

    /** The part of the best-claiming site at column (x, z), or {@link Part#NONE}. */
    public Part partAt(int x, int z) {
        Site s = claim(x, z);
        return s == null ? Part.NONE : s.part(dx(x, s), z - s.z);
    }

    /**
     * Whether block (x, y, z) is the surface block of a hot-spring pool's water: the column is a pool column of the
     * site that wins it ({@link #claim}) and {@code y} is that site's water level, where {@link GeothermalPass} puts
     * the top block of the water. The water of a hot spring is hot, so it never freezes whatever the biome's
     * temperature (ledger: GEO-062, game-choice); {@code LakeFreeze} asks this before vanilla's top-layer freezing
     * and snowfall may ice the block. Only the surface block: a pool is one or two blocks deep, and the second
     * block of water under it is not reached by the cold. False for every other block, rim and apron included
     * (snow may lie on the dry ground).
     */
    public boolean poolSurface(int x, int y, int z) {
        if (!poolLevels.contains(y))
            return false;
        Site s = claim(x, z);
        return s != null && s.kind == Kind.POOL && s.level == y && s.part(dx(x, s), z - s.z).pool();
    }

    /**
     * The warm-ground signal at block (x, y, z): how many kelvin warmer than the climate's
     * temperature it is there (the most any site gives); 0 away from every site.
     */
    public double warmthC(int x, int y, int z) {
        double best = 0.0;
        for (Site s : sites) {
            int reach = s.reach() + WARM_FADE;
            if (Math.abs(z - s.z) > reach || y < s.groundY - 13 || y > s.groundY + 6)
                continue;
            int dx = dx(x, s);
            if (Math.abs(dx) > reach)
                continue;
            best = Math.max(best, s.warmth(dx, z - s.z, y));
        }
        return best;
    }

    /** Sites whose heat is the ground's heat flow, not a volcano. */
    public long heatFlowSites() {
        return sites.stream().filter(s -> s.volcano == HEAT_FLOW_SOURCE).count();
    }

    /** Sites of each kind, for reports: {pool, vent, patch}. */
    public int[] counts() {
        int[] n = new int[3];
        for (Site s : sites)
            n[s.kind.ordinal()]++;
        return n;
    }

    // ---------------------------------------------------------------------------------------

    private record Ctx(GeothermalRules rules, Magma magma, Magma.Heights heights, Magma.Wet wet, Env env, int circumference,
                       int seaLevel, double mH, List<Site> sites) {

        int wrap(int x) {
            return circumference <= 0 ? x : Math.floorMod(x + circumference / 2, circumference) - circumference / 2;
        }

        double h(int x, int z) {
            return heights.at(wrap(x), z);
        }
    }

    /**
     * Builds the world's sites.
     *
     * @param magma the world's magma bodies: sites keep clear of every open crater and plug
     * @param mH    metres per block across (a volcano's basal radius is given in metres)
     */
    public static Geothermal build(List<Volcanoes.Volcano> volcanoes, GeothermalRules rules, Magma magma, Magma.Heights heights,
                                   Magma.Wet wet, Env env, int circumference, int seaLevel, double mH) {
        if (rules.density() <= 0.0)
            return new Geothermal(List.of(), circumference);
        Ctx c = new Ctx(rules, magma, heights, wet, env, circumference, seaLevel, mH, new ArrayList<>());
        for (Volcanoes.Volcano v : volcanoes)
            volcano(c, v);
        heatFlowFields(c);
        return new Geothermal(c.sites, circumference);
    }

    /** What a field needs to know of its heat: the volcano's id (or {@link #HEAT_FLOW_SOURCE}) and whether it is silicic. */
    private record Source(int id, boolean silicic) {
    }

    /**
     * The fields on hot continental ground. The world (its period by its z extent, {@link Env#zHalf}) is
     * cut into cells
     * of {@link #HEAT_FIELD_AREA_KM2} / {@code geothermal_heat_density} square km; each cell, from its
     * own seed, tries {@link #HEAT_DRAWS} random points and takes the first where the baked heat flow
     * reaches the setting and the ground is dry land clear of ice. Fields so placed are kinds of the
     * same machinery as a volcano's, judged by the rain and the mean temperature of the place; a
     * heat-flow field is never in a summit zone, so it is hot springs wherever those allow.
     */
    private static void heatFlowFields(Ctx c) {
        HeatGrid grid = HeatGrid.of(c);
        if (grid == null)
            return;
        for (int gz = 0; gz < grid.nz; gz++)
            for (int gx = 0; gx < grid.nx; gx++) {
                long seed = PaletteRules.mix(0x4EA7_F10EL ^ ((long) gx << 32) ^ (gz & 0xFFFFFFFFL));
                SplittableRandom rng = new SplittableRandom(seed);
                for (int draw = 0; draw < HEAT_DRAWS; draw++) {
                    int fx = c.wrap((int) Math.round(-c.circumference / 2.0 + (gx + rng.nextDouble()) * grid.wx));
                    int fz = (int) Math.round(grid.z0 + (gz + rng.nextDouble()) * grid.wz);
                    if (!heatFlowSpot(c, fx, fz))
                        continue;
                    field(c, new Source(HEAT_FLOW_SOURCE, false), rng, fx, fz, 1.0);
                    break;
                }
            }
    }

    /**
     * The heat-flow fields' grid: {@code nx} by {@code nz} cells of {@code wx} by {@code wz} blocks from z
     * {@code z0}, over the circumference and the world's z span; null when the world takes no such fields.
     */
    private record HeatGrid(int nx, int nz, double wx, double wz, double z0) {
        static HeatGrid of(Ctx c) {
            if (c.rules.heatDensity() <= 0.0 || c.circumference <= 0)
                return null;
            double side = Math.sqrt(HEAT_FIELD_AREA_KM2 / c.rules.heatDensity()) * 1000.0 / c.mH;   // blocks
            double pole = Double.isNaN(c.env.zHalf())
                    ? com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection.Lambert.radius(c.circumference) : c.env.zHalf();
            int nx = Math.max(1, (int) Math.round(c.circumference / side)), nz = Math.max(1, (int) Math.round(2 * pole / side));
            return new HeatGrid(nx, nz, (double) c.circumference / nx, 2 * pole / nz, -pole);
        }
    }

    /**
     * Whether a heat-flow field may stand at (fx, fz): the planet's own ground ({@link Env#onPlanet}: a cube world's
     * cells cover its whole net window, whose void squares read their partners' ground), the heat flow at the setting,
     * dry land, clear of ice.
     */
    private static boolean heatFlowSpot(Ctx c, int fx, int fz) {
        if (!c.env.onPlanet(fx, fz))
            return false;
        double q = c.env.heatFlowMwM2(fx, fz);
        if (!(q >= c.rules.heatFlowMinMwM2()))
            return false;
        return !(Magma.top(c.h(fx, fz)) < c.seaLevel + MIN_ABOVE_SEA + 2 || c.wet.at(fx, fz) || c.env.iced(fx, fz));
    }

    /**
     * What the heat-flow fields could find on a world: the ground where a field may stand ({@link #heatFlowSpot}),
     * sampled every {@code step} blocks in each of the fields' cells, and from each cell's share {@code f} of it the
     * fields' odds, the draws being {@link #HEAT_DRAWS} uniform points of the cell: the expected count of cells that
     * find a place, {@code sum(1 - (1 - f)^draws)}, and the chance that none does, {@code prod((1 - f)^draws)}
     * (ledger: GEO-064). A place found can still be refused by the placement (a slope, a crater, water), so the
     * fields' count can only be lower. {@code samples} and {@code continental} count the grid points and those on the
     * planet's own ground ({@link Env#onPlanet}) with a heat flow (continental crust). On a cube world the cells cover
     * the whole net window and half of it is void squares, which hold no field.
     */
    public record HeatFlowOdds(long samples, long continental, double hotDryKm2, int cells, int hotCells, double expected,
                               double pNone) {
        @Override
        public String toString() {
            return String.format("hot dry ground %.0f km2 in %d of %d cells (%d of %d samples with a heat flow), "
                    + "expected cells with a field %.2f, P(none) %.4f", hotDryKm2, hotCells, cells, continental, samples, expected, pNone);
        }
    }

    /** See {@link HeatFlowOdds}; the arguments are {@link #build}'s. */
    public static HeatFlowOdds heatFlowOdds(GeothermalRules rules, Magma.Heights heights, Magma.Wet wet, Env env, int circumference,
                                            int seaLevel, double mH, int step) {
        Ctx c = new Ctx(rules, null, heights, wet, env, circumference, seaLevel, mH, new ArrayList<>());
        HeatGrid grid = HeatGrid.of(c);
        if (grid == null)
            return new HeatFlowOdds(0, 0, 0.0, 0, 0, 0.0, 1.0);
        double cellKm2 = grid.wx * grid.wz * (mH / 1000.0) * (mH / 1000.0), km2 = 0.0, expected = 0.0, logNone = 0.0;
        long samples = 0, continental = 0;
        int hotCells = 0;
        for (int gz = 0; gz < grid.nz; gz++)
            for (int gx = 0; gx < grid.nx; gx++) {
                long all = 0, good = 0;
                double zEnd = grid.z0 + (gz + 1) * grid.wz, xEnd = -circumference / 2.0 + (gx + 1) * grid.wx;
                for (double bz = grid.z0 + gz * grid.wz + step / 2.0; bz < zEnd; bz += step)
                    for (double bx = -circumference / 2.0 + gx * grid.wx + step / 2.0; bx < xEnd; bx += step) {
                        int fx = c.wrap((int) Math.round(bx)), fz = (int) Math.round(bz);
                        all++;   // the draws are uniform over the cell, its part off the planet included
                        if (env.onPlanet(fx, fz) && !Double.isNaN(env.heatFlowMwM2(fx, fz)))
                            continental++;
                        if (heatFlowSpot(c, fx, fz))
                            good++;
                    }
                samples += all;
                double f = all == 0 ? 0.0 : (double) good / all;
                if (good > 0)
                    hotCells++;
                km2 += f * cellKm2;
                expected += 1.0 - Math.pow(1.0 - f, HEAT_DRAWS);
                logNone += f >= 1.0 ? Double.NEGATIVE_INFINITY : HEAT_DRAWS * Math.log1p(-f);
            }
        return new HeatFlowOdds(samples, continental, km2, grid.nx * grid.nz, hotCells, expected, Math.exp(logNone));
    }

    private static void volcano(Ctx c, Volcanoes.Volcano v) {
        GeothermalRules rules = c.rules;
        int sx = c.wrap((int) Math.round(v.x())), sz = (int) Math.round(v.z());
        long seed = PaletteRules.mix(0x6E07_4E21L ^ ((long) v.id() << 20) ^ ((long) sx << 40) ^ (sz & 0xFFFFFFL));
        SplittableRandom rng = new SplittableRandom(seed);
        double radius = Math.max(6.0, v.radiusM() / c.mH);
        double activity = Math.clamp(v.activity(), 0.05, 1.0);
        double expected = rules.density() * (0.7 + 2.3 * activity) * Math.clamp(Math.sqrt(radius / 45.0), 0.6, 2.2);
        int fields = (int) Math.floor(expected + rng.nextDouble());
        for (int i = 0; i < fields; i++) {
            // A field goes on the planet's dry land that is not under ice: the first of a dozen draws that is.
            for (int attempt = 0; attempt < 12; attempt++) {
                double frac = i == 0 ? 0.08 + 0.27 * rng.nextDouble() : 0.25 + 0.75 * Math.sqrt(rng.nextDouble());
                double angle = 2.0 * Math.PI * rng.nextDouble();
                double rad = frac * radius * rules.reach();
                int fx = c.wrap((int) Math.round(sx + rad * Math.cos(angle))), fz = (int) Math.round(sz + rad * Math.sin(angle));
                if (!c.env.onPlanet(fx, fz) || Magma.top(c.h(fx, fz)) < c.seaLevel + MIN_ABOVE_SEA + 2 || c.wet.at(fx, fz)
                        || c.env.iced(fx, fz))
                    continue;
                field(c, new Source(v.id(), v.silicic()), rng, fx, fz, rad / radius);
                break;
            }
        }
    }

    /** One field around (fx, fz), {@code share} of a basal radius from the summit. */
    private static void field(Ctx c, Source v, SplittableRandom rng, int fx, int fz, double share) {
        GeothermalRules rules = c.rules;
        double size = rules.size();
        double precip = c.env.precipMm(fx, fz), mean = c.env.meanC(fx, fz);
        boolean groundwater = Double.isNaN(precip) || precip >= rules.springMinPrecipMm();
        boolean thawed = Double.isNaN(mean) || mean >= POOL_MIN_C;
        boolean springs = share >= SUMMIT_ZONE && groundwater && thawed;
        boolean sinter = !c.env.carbonate(fx, fz, (int) Math.floor(c.h(fx, fz)));
        long fieldSeed = rng.nextLong();
        if (springs) {
            int r = clamp((int) Math.round((2.4 + 2.2 * rng.nextDouble()) * size), 2, 9);
            Site main = place(c, v, Kind.POOL, fx, fz, r, r + 3 + rng.nextInt(3), 0.0, sinter, fieldSeed, rng, 8);
            int cx = main == null ? fx : main.x, cz = main == null ? fz : main.z;
            // The chain: down the slope where there is one (a terrace), else any way.
            double gx = c.h(cx + 5, cz) - c.h(cx - 5, cz), gz = c.h(cx, cz + 5) - c.h(cx, cz - 5);
            double down = Math.hypot(gx, gz) > 0.4 ? Math.atan2(-gz, -gx) : 2.0 * Math.PI * rng.nextDouble();
            boolean chain = rng.nextDouble() < 0.5;
            double ax = cx, az = cz;
            int prev = r;
            int pools = 1 + rng.nextInt(3);
            for (int k = 0; k < pools; k++) {
                int rs = clamp((int) Math.round((1.6 + 1.4 * rng.nextDouble()) * size), 2, 6);
                double dist = prev + rs + 4 + 3 * rng.nextDouble();
                double a = chain ? down + 0.35 * rng.nextGaussian() : 2.0 * Math.PI * rng.nextDouble();
                int sx2 = (int) Math.round((chain ? ax : cx) + dist * Math.cos(a));
                int sz2 = (int) Math.round((chain ? az : cz) + dist * Math.sin(a));
                Site s = place(c, v, Kind.POOL, sx2, sz2, rs, rs + 3 + rng.nextInt(2), 2.5, sinter, rng.nextLong(), rng, 5);
                if (s != null && chain) {
                    ax = s.x;
                    az = s.z;
                    prev = rs;
                }
            }
            if (rng.nextDouble() < 0.3) {
                double a = 2.0 * Math.PI * rng.nextDouble(), dist = r + 8 + 4 * rng.nextDouble();
                place(c, v, Kind.VENT, (int) Math.round(cx + dist * Math.cos(a)), (int) Math.round(cz + dist * Math.sin(a)), 1,
                        clamp((int) Math.round((3.5 + 2.0 * rng.nextDouble()) * size), 4, 9), 2.0, sinter, rng.nextLong(), rng, 5);
            }
            if (rng.nextDouble() < 0.4) {
                double a = 2.0 * Math.PI * rng.nextDouble(), dist = r + 10 + 6 * rng.nextDouble();
                place(c, v, Kind.PATCH, (int) Math.round(cx + dist * Math.cos(a)), (int) Math.round(cz + dist * Math.sin(a)), 0,
                        clamp((int) Math.round((3.5 + 3.0 * rng.nextDouble()) * size), 3, 10), 2.0, sinter, rng.nextLong(), rng, 5);
            }
        } else {
            double spread = (5.0 + 5.0 * rng.nextDouble()) * size;
            int vents = 2 + rng.nextInt(3);
            for (int k = 0; k < vents; k++) {
                int r = rng.nextDouble() < 0.6 ? 1 : 2;
                place(c, v, Kind.VENT, fx, fz, r, Math.max(r + 3, clamp((int) Math.round((3.5 + 3.5 * rng.nextDouble()) * size), 3, 10)),
                        spread, sinter, rng.nextLong(), rng, 6);
            }
            int patches = 1 + rng.nextInt(2);
            for (int k = 0; k < patches; k++)
                place(c, v, Kind.PATCH, fx, fz, 0, clamp((int) Math.round((4.0 + 4.0 * rng.nextDouble()) * size), 3, 11),
                        spread * 1.4, sinter, rng.nextLong(), rng, 6);
        }
    }

    private static int clamp(int v, int lo, int hi) {
        return Math.max(lo, Math.min(hi, v));
    }

    /**
     * Tries to put a site of this kind about (cx, cz): the first attempt at the centre, the others a
     * gaussian {@code spread} blocks about it; returns the accepted site (added to the list) or null.
     */
    private static Site place(Ctx c, Source v, Kind kind, int cx, int cz, int r, int apron, double spread, boolean sinter,
                              long seed, SplittableRandom rng, int attempts) {
        for (int a = 0; a < attempts; a++) {
            int x = cx, z = cz;
            if (a > 0 || spread > 0.0) {   // spread 0: the first try is exactly the centre
                double s = spread > 0.0 ? spread : 2.5;
                x = c.wrap((int) Math.round(cx + s * rng.nextGaussian()));
                z = (int) Math.round(cz + s * rng.nextGaussian());
            }
            x = c.wrap(x);
            Site draft = new Site(c.sites.size(), kind, v.id(), x, z, r, apron, 0, 0, sinter, v.silicic(), seed,
                    2.0 * Math.PI * (seed >>> 11 & 0xFFFF) / 65536.0, 2.0 * Math.PI * (seed >>> 27 & 0xFFFF) / 65536.0);
            Site ok = accept(c, draft);
            if (ok != null) {
                c.sites.add(ok);
                return ok;
            }
        }
        return null;
    }

    /** The draft with its level and ground filled in, or null when the place will not do. */
    private static Site accept(Ctx c, Site s) {
        int reach = s.reach(), hard = s.hard();
        int groundY = Magma.top(c.h(s.x, s.z));
        if (groundY < c.seaLevel + MIN_ABOVE_SEA)
            return null;
        // Clear of every open crater and plug.
        if (c.magma != null)
            for (Magma.Body b : c.magma.bodies()) {
                if (b.chamberRh() <= 0 || Math.abs(b.z() - s.z) > reach + b.reach() + 4)
                    continue;
                double d = Math.hypot(fold(b.x() - s.x, c.circumference), b.z() - s.z);
                double keep = (b.lake() ? b.lakeRadius() + 2 : 2) + 3 + hard;
                if (d < keep)
                    return null;
            }
        // Clear of other sites' water, rims and vents.
        for (Site o : c.sites) {
            if (Math.abs(o.z - s.z) > o.hard() + hard + 2)
                continue;
            double d = Math.hypot(fold(o.x - s.x, c.circumference), o.z - s.z);
            if (d < o.hard() + hard + 1)
                return null;
        }
        // Every column on the planet's own ground (a cube world's faces); dry ground, not iced, above the coast; the
        // pool's footprint decides its level.
        int lowest = Integer.MAX_VALUE, highest = Integer.MIN_VALUE, count = 0;
        long sum = 0;
        for (int dx = -reach; dx <= reach; dx++)
            for (int dz = -reach; dz <= reach; dz++) {
                Geothermal.Part p = s.part(dx, dz);
                if (p == Part.NONE)
                    continue;
                int x = c.wrap(s.x + dx), z = s.z + dz;
                if (!c.env.onPlanet(x, z))
                    return null;   // off a cube's faces: that column would never be built (Env#onPlanet)
                boolean exact = p.pool() || p == Part.RIM || p == Part.VENT || p == Part.SULFUR;
                if (!exact && ((dx & 1) != 0 || (dz & 1) != 0))
                    continue;
                if (c.wet.at(x, z))
                    return null;
                if (c.env.iced(x, z))
                    return null;
                int top = Magma.top(c.h(x, z));
                if (top < c.seaLevel + MIN_ABOVE_SEA)
                    return null;
                if (p.pool() || p == Part.RIM) {
                    lowest = Math.min(lowest, top);
                    highest = Math.max(highest, top);
                    sum += top;
                    count++;
                }
            }
        int level = 0;
        if (s.kind == Kind.POOL) {
            if (lowest == Integer.MAX_VALUE)
                return null;
            level = (int) Math.round((double) sum / count);
            if (highest - level > CUT || level - lowest > DAM)
                return null;
        }
        return new Site(s.id, s.kind, s.volcano, s.x, s.z, s.r, s.apron, level, groundY, s.sinter, s.silicic, s.seed, s.phase1,
                s.phase2);
    }
}
