// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Where a Deep Time world's volcanic biomes go (Ben, decisions log 2026-10-03, "Tie them to real
 * volcanoes"): discs around the active volcanoes, their craters and calderas, their hot-spring pools
 * and their magma, each disc carrying a {@link Role}. Plain Java geometry, built once per level from
 * the baked volcano list ({@code volcanoes/active.json}), the geothermal sites ({@link Geothermal}) and
 * the magma bodies ({@link Magma}); the biome a role puts there is the pack's business
 * ({@code climate.VolcanicBiomes}, the tags {@code #deeptime:volcanic/<role>}).
 *
 * <p>Every zone is a disc in blocks, at least {@link #MIN_RADIUS} across in radius (three of the
 * biome grid's 4-block cells), so a role never makes a speck of one or two biome cells; a ring (the
 * ash and lava fields round a cone) is a disc of lower priority than the cone's, and is left out when
 * it would be narrower than {@link #MIN_RING}. Where discs overlap, {@link Role}'s order decides.
 * Every number here is a game choice (ledger BIO-104 to BIO-113).
 */
public final class VolcanicZones {

    /**
     * A volcanic biome's place, in priority order (the first role whose disc holds a column and whose
     * biome the pack has wins there). The roles, their order, which volcanoes get which and the
     * climate gate are game choices (ledger: BIO-113).
     */
    public enum Role {
        /**
         * A field of hot-spring pools (from a volcano or from hot ground). First, so a field inside a caldera or a
         * crater zone keeps its own biome.
         */
        HOT_SPRING_FIELD("hot_spring_field", false, false),
        /** The flat floor of a caldera. */
        CALDERA("caldera", false, false),
        /** A summit crater that is not a caldera. */
        CRATER("crater", false, false),
        /** An active cone's summit and upper flanks. */
        CONE("cone", false, false),
        /** Fresh ash on the lower flanks of an active explosive volcano, where the climate allows its biome. */
        ASH_FIELD("ash_field", true, false),
        /** Young lava on the lower flanks of an active shield, where the climate allows its biome. */
        LAVA_FIELD("lava_field", true, false),
        /** The caves of the hot rock round a volcano's magma (underground only). */
        THERMAL_CAVES("thermal_caves", false, true);

        /** The tag path under {@code deeptime:volcanic/}. */
        public final String id;
        /** The role's biome goes only where the climate point lies in that biome's own temperature and humidity ranges. */
        public final boolean climateGated;
        /** A cave role: underground, not at the surface. */
        public final boolean cave;

        Role(String id, boolean climateGated, boolean cave) {
            this.id = id;
            this.climateGated = climateGated;
            this.cave = cave;
        }
    }

    /** The smallest radius of any zone, blocks: three biome cells, so blending leaves no speck. */
    public static final int MIN_RADIUS = 12; // ledger: BIO-105 (game-choice)
    /** A ring narrower than this beyond the zone inside it is left out, blocks. */
    public static final int MIN_RING = 8; // ledger: BIO-105 (game-choice)
    /** The cone zone: summit and upper flanks, as a share of the basal radius (times the reach setting). */
    public static final double CONE_SHARE = 0.45; // ledger: BIO-106 (game-choice)
    /** The ash and lava fields reach this share of the basal radius (times the reach setting). */
    public static final double FIELD_SHARE = 1.0; // ledger: BIO-107 (game-choice)
    /** The least activity of a volcano with a fresh-ash field. */
    public static final double ASH_MIN_ACTIVITY = 0.75; // ledger: BIO-108 (game-choice)
    /** A caldera's floor zone, as a share of its rim radius. */
    public static final double CALDERA_FLOOR_SHARE = 0.9; // ledger: BIO-109 (game-choice)
    /** Blocks added beyond a summit crater's rim and beyond a hot-spring pool's outermost terrace. */
    public static final int CRATER_MARGIN = 4, SPRING_MARGIN = 4; // ledger: BIO-110 (game-choice)
    /** Thermal caves reach this multiple of a magma chamber's horizontal radius from its axis. */
    public static final double CAVE_SHARE = 1.5; // ledger: BIO-111 (game-choice)

    /**
     * One disc.
     *
     * @param source the volcano's id; for a hot-spring field, the pool site's id
     * @param x      its centre (x canonical, in [−C/2, C/2))
     */
    public record Zone(Role role, int source, int x, int z, double radius) {}

    public static final VolcanicZones NONE = new VolcanicZones(List.of(), 0);

    private static final int CELL = 64;

    private final List<Zone> zones;
    private final int circumference;
    private final int nx;
    private final double cellW;
    private final Map<Long, int[]> grid = new HashMap<>();

    private VolcanicZones(List<Zone> zones, int circumference) {
        List<Zone> sorted = new ArrayList<>(zones);
        sorted.sort(java.util.Comparator.comparingInt((Zone z) -> z.role.ordinal()));
        this.zones = List.copyOf(sorted);
        this.circumference = circumference;
        this.nx = circumference <= 0 ? 1 : Math.max(1, circumference / CELL);
        this.cellW = circumference <= 0 ? CELL : (double) circumference / nx;
        Map<Long, List<Integer>> cells = new HashMap<>();
        for (int i = 0; i < this.zones.size(); i++) {
            Zone zn = this.zones.get(i);
            int r = (int) Math.ceil(zn.radius) + 1;
            int cz0 = Math.floorDiv(zn.z - r, CELL), cz1 = Math.floorDiv(zn.z + r, CELL);
            java.util.Set<Integer> xs = new java.util.TreeSet<>();
            for (double bx = zn.x - r; bx < zn.x + r + cellW / 2; bx += cellW / 2)
                xs.add(cellX((int) Math.round(Math.min(bx, zn.x + r))));
            for (int cx : xs)
                for (int cz = cz0; cz <= cz1; cz++)
                    cells.computeIfAbsent(key(cx, cz), k -> new ArrayList<>()).add(i);
        }
        cells.forEach((k, v) -> grid.put(k, v.stream().mapToInt(Integer::intValue).toArray()));
    }

    private static long key(int cx, int cz) {
        return ((long) cx << 32) ^ (cz & 0xFFFFFFFFL);
    }

    private int wrap(int x) {
        return circumference <= 0 ? x : Math.floorMod(x + circumference / 2, circumference) - circumference / 2;
    }

    private int cellX(int x) {
        if (circumference <= 0)
            return Math.floorDiv(x, CELL);
        int i = (int) Math.floor((wrap(x) + circumference / 2) / cellW);
        return Math.min(nx - 1, Math.max(0, i));
    }

    /** x − zx folded into [−C/2, C/2). */
    private int dx(int x, int zx) {
        return circumference <= 0 ? x - zx : Math.floorMod(x - zx + circumference / 2, circumference) - circumference / 2;
    }

    public List<Zone> zones() {
        return zones;
    }

    /**
     * The roles whose discs hold block column (x, z), as a bit set over {@link Role#ordinal()} (bit i:
     * role i); 0 outside every zone. Surface and cave roles both.
     */
    public int rolesAt(int x, int z) {
        if (zones.isEmpty())
            return 0;
        int[] ids = grid.get(key(cellX(x), Math.floorDiv(z, CELL)));
        if (ids == null)
            return 0;
        int bits = 0;
        for (int i : ids) {
            Zone zn = zones.get(i);
            if ((bits & (1 << zn.role.ordinal())) != 0)
                continue;
            double ddx = dx(x, zn.x), ddz = z - zn.z;
            if (ddx * ddx + ddz * ddz <= zn.radius * zn.radius)
                bits |= 1 << zn.role.ordinal();
        }
        return bits;
    }

    /**
     * How far block column (x, z) lies outside every zone of a surface role, in blocks (the distance to
     * the nearest disc's edge); 0 or less inside one. {@code Double.POSITIVE_INFINITY} with no zone.
     */
    public double outsideSurfaceZones(int x, int z) {
        double best = Double.POSITIVE_INFINITY;
        for (Zone zn : zones) {
            if (zn.role.cave)
                continue;
            double d = Math.hypot(dx(x, zn.x), z - zn.z) - zn.radius;
            if (d < best)
                best = d;
        }
        return best;
    }

    /**
     * The world's zones.
     *
     * @param volcanoes     the active volcanoes
     * @param sites         the geothermal sites (the pools mark hot-spring fields)
     * @param bodies        the volcanoes' magma (a body with a chamber gets thermal caves)
     * @param mH            metres per block across
     * @param circumference blocks round the world (x wraps); 0: no wrap
     */
    public static VolcanicZones build(List<Volcanoes.Volcano> volcanoes, List<Geothermal.Site> sites, List<Magma.Body> bodies,
                                      VolcanicBiomeRules rules, double mH, int circumference) {
        if (rules.off())
            return new VolcanicZones(List.of(), circumference);
        double reach = rules.reach();
        List<Zone> out = new ArrayList<>();
        for (Volcanoes.Volcano v : volcanoes) {
            int x = circumference <= 0 ? (int) Math.round(v.x())
                    : Math.floorMod((int) Math.round(v.x()) + circumference / 2, circumference) - circumference / 2;
            int z = (int) Math.round(v.z());
            double basal = Math.max(6.0, v.radiusM() / mH);
            double crater = v.craterRadiusM() / mH;
            String kind = v.kind();
            boolean shield = "shield".equals(kind);
            boolean cone = shield || "stratovolcano".equals(kind);
            double inner = 0.0;
            if (crater > 0.0 && v.caldera()) {
                double r = Math.max(MIN_RADIUS, CALDERA_FLOOR_SHARE * crater);
                out.add(new Zone(Role.CALDERA, v.id(), x, z, r));
                inner = r;
            } else if (crater > 0.0) {
                double r = Math.max(MIN_RADIUS, crater + CRATER_MARGIN);
                out.add(new Zone(Role.CRATER, v.id(), x, z, r));
                inner = r;
            }
            if (cone) {
                double r = Math.max(MIN_RADIUS, CONE_SHARE * basal * reach);
                if (r > inner)
                    out.add(new Zone(Role.CONE, v.id(), x, z, r));
                inner = Math.max(inner, r);
            }
            double field = FIELD_SHARE * basal * reach;
            if (field - inner >= MIN_RING && field >= MIN_RADIUS) {
                if (shield)
                    out.add(new Zone(Role.LAVA_FIELD, v.id(), x, z, field));
                else if (v.activity() >= ASH_MIN_ACTIVITY)
                    out.add(new Zone(Role.ASH_FIELD, v.id(), x, z, field));
            }
        }
        for (Geothermal.Site s : sites)
            if (s.kind() == Geothermal.Kind.POOL)
                out.add(new Zone(Role.HOT_SPRING_FIELD, s.id(), s.x(), s.z(), Math.max(MIN_RADIUS, s.reach() + SPRING_MARGIN)));
        for (Magma.Body b : bodies)
            if (b.chamberRh() > 0)
                out.add(new Zone(Role.THERMAL_CAVES, b.id(), b.x(), b.z(), Math.max(MIN_RADIUS, CAVE_SHARE * b.chamberRh())));
        return new VolcanicZones(out, circumference);
    }
}
