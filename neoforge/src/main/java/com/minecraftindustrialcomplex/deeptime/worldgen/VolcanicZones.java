// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

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
 * <p>Every zone is an ellipse in blocks (a disc on the conformal cube world), each semi-axis at least
 * {@link #MIN_RADIUS} (two of the biome grid's 4-block cells), so a role never makes a speck of one or
 * two biome cells; a ring (the ash and lava fields round a cone) is a zone of lower priority than the
 * cone's, and is left out when it would be narrower than {@link #MIN_RING}. Where zones overlap,
 * {@link Role}'s order decides. The simulation draws a cone as a circle on the planet (since
 * 2026-10-23; a circle in its Lambert blocks before), which a Lambert-frame world shows wide east–west
 * and the conformal cube round; a volcano's zones take the projection's local stretch at the vent
 * ({@link #stretch}), so they lie on the cone, its crater and its flanks as the terrain draws them.
 * An oceanic volcano's zones are sized on the part of its edifice above the sea, not on its basal radius on the
 * sea floor ({@link #subaerialShare}; ledger BIO-198). Every number here is a game choice (ledger BIO-104 to
 * BIO-113, BIO-182).
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

    /** The smallest radius of any zone, blocks: two biome cells, so blending leaves no speck. */
    public static final int MIN_RADIUS = 8; // ledger: BIO-105 (game-choice)
    /** A ring narrower than this beyond the zone inside it is left out, blocks. */
    public static final int MIN_RING = 8; // ledger: BIO-105 (game-choice)
    /**
     * The cone zone: summit and upper flanks, as a share of the basal radius (times the reach setting); of an
     * oceanic volcano's radius above the sea ({@link #subaerialShare}, ledger: BIO-198).
     */
    public static final double CONE_SHARE = 0.45; // ledger: BIO-106 (game-choice)
    /** The ash and lava fields reach this share of the basal radius (an oceanic volcano's above the sea), times the reach setting. */
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
     * One zone: an ellipse in blocks, a disc when its semi-axes are equal.
     *
     * @param source the volcano's id; for a hot-spring field, the pool site's id
     * @param x      its centre (x canonical, in [−C/2, C/2))
     * @param radius its semi-major axis, blocks (a disc's radius; the zone lies within it)
     * @param minor  its semi-minor axis, blocks
     * @param ax     with {@code az}, the unit vector of the semi-major axis in block space
     */
    public record Zone(Role role, int source, int x, int z, double radius, double minor, double ax, double az) {

        /** A disc. */
        public Zone(Role role, int source, int x, int z, double radius) {
            this(role, source, x, z, radius, radius, 1.0, 0.0);
        }

        /** The squared reach of the offset (dx, dz) from the centre in units of the zone: at most 1 inside. */
        public double reach2(double dx, double dz) {
            double p = (dx * ax + dz * az) / radius, q = (dz * ax - dx * az) / minor;
            return p * p + q * q;
        }

        /** Whether the offset (dx, dz) from the centre lies in the zone (a disc's test exactly as before ellipses). */
        public boolean holds(double dx, double dz) {
            return radius == minor ? dx * dx + dz * dz <= radius * radius : reach2(dx, dz) <= 1.0;
        }

        /**
         * How far the offset (dx, dz) lies outside the zone, blocks: the distance to its nearest edge point (a
         * disc's edge distance exactly); 0 or less inside, scaled there by the semi-minor axis. Never less than
         * {@code hypot(dx, dz) − radius}, since the zone lies within its semi-major disc.
         */
        public double outside(double dx, double dz) {
            if (radius == minor)
                return Math.hypot(dx, dz) - radius;
            double u = Math.abs(dx * ax + dz * az), v = Math.abs(dz * ax - dx * az), a = radius, b = minor;
            double s2 = (u / a) * (u / a) + (v / b) * (v / b);
            if (s2 <= 1.0)
                return (Math.sqrt(s2) - 1.0) * b;
            // The nearest edge point is (a²u/(t + a²), b²v/(t + b²)) for the root t > 0 of
            // (au/(t + a²))² + (bv/(t + b²))² = 1, which falls from s2 − 1 at t = 0 to below 0 at √(a²u² + b²v²):
            // bisection. (The distance along the line from the centre, the old measure, overstates it between the
            // axes, by up to about 2x on a 3:1 zone.)
            double lo = 0.0, hi = Math.sqrt(a * a * u * u + b * b * v * v);
            for (int i = 0; i < 60; i++) {
                double t = 0.5 * (lo + hi), p = a * u / (t + a * a), q = b * v / (t + b * b);
                if (p * p + q * q > 1.0)
                    lo = t;
                else
                    hi = t;
            }
            double t = 0.5 * (lo + hi);
            return Math.hypot(u - a * a * u / (t + a * a), v - b * b * v / (t + b * b));
        }

        /** Its area, blocks². */
        public double area() {
            return Math.PI * radius * minor;
        }
    }

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
            if (zn.holds(dx(x, zn.x), z - zn.z))
                bits |= 1 << zn.role.ordinal();
        }
        return bits;
    }

    /**
     * How far block column (x, z) lies outside every zone of a surface role, in blocks ({@link Zone#outside});
     * 0 or less inside one. {@code Double.POSITIVE_INFINITY} with no zone.
     */
    public double outsideSurfaceZones(int x, int z) {
        double best = Double.POSITIVE_INFINITY;
        for (Zone zn : zones) {
            if (zn.role.cave)
                continue;
            double ox = dx(x, zn.x), oz = z - zn.z;
            if (Math.hypot(ox, oz) - zn.radius >= best)   // a lower bound of its distance: no nearer
                continue;
            double d = zn.outside(ox, oz);
            if (d < best)
                best = d;
        }
        return best;
    }

    /** The world's zones as discs: {@link #build(List, List, List, VolcanicBiomeRules, double, int, WorldProjection)} without a projection. */
    public static VolcanicZones build(List<Volcanoes.Volcano> volcanoes, List<Geothermal.Site> sites, List<Magma.Body> bodies,
                                      VolcanicBiomeRules rules, double mH, int circumference) {
        return build(volcanoes, sites, bodies, rules, mH, circumference, null);
    }

    /**
     * The world's zones.
     *
     * @param volcanoes     the active volcanoes
     * @param sites         the geothermal sites (the pools mark hot-spring fields)
     * @param bodies        the volcanoes' magma (a body with a chamber gets thermal caves)
     * @param mH            metres per block across
     * @param circumference blocks round the world (x wraps); 0: no wrap
     * @param projection    the world's projection: a volcano's zones take its local stretch at the vent
     *                      ({@link #stretch}); null: discs
     */
    public static VolcanicZones build(List<Volcanoes.Volcano> volcanoes, List<Geothermal.Site> sites, List<Magma.Body> bodies,
                                      VolcanicBiomeRules rules, double mH, int circumference, WorldProjection projection) {
        return build(volcanoes, sites, bodies, rules, mH, circumference, projection, null);
    }

    /**
     * What an oceanic volcano's zones are sized on: the world's ground at its vent and the sea.
     *
     * @param heights  the terrain height H(x, z), blocks (the generator's ground)
     * @param seaLevel the sea level, blocks
     * @param mV       metres per block up
     */
    public record Ground(Magma.Heights heights, int seaLevel, double mV) {}

    /**
     * As {@link #build(List, List, List, VolcanicBiomeRules, double, int, WorldProjection)} with the world's ground
     * given.
     *
     * @param ground the ground an oceanic volcano's zones are sized on (null: its basal radius, as before)
     */
    public static VolcanicZones build(List<Volcanoes.Volcano> volcanoes, List<Geothermal.Site> sites, List<Magma.Body> bodies,
                                      VolcanicBiomeRules rules, double mH, int circumference, WorldProjection projection,
                                      Ground ground) {
        if (rules.off())
            return new VolcanicZones(List.of(), circumference);
        double reach = rules.reach();
        List<Zone> out = new ArrayList<>();
        for (Volcanoes.Volcano v : volcanoes) {
            int x = circumference <= 0 ? (int) Math.round(v.x())
                    : Math.floorMod((int) Math.round(v.x()) + circumference / 2, circumference) - circumference / 2;
            int z = (int) Math.round(v.z());
            // World blocks per planet block along the stretch's two axes (equal on the conformal cube).
            Stretch st = stretch(projection, v.x(), v.z()); // ledger: BIO-182 (game-choice)
            double basal = Math.max(6.0, v.radiusM() / mH);
            // An oceanic volcano's biomes lie on the part of the edifice above the sea (ledger: BIO-198).
            if (v.oceanic() && ground != null)
                basal = Math.max(6.0, subaerialShare(v, x, z, ground) * v.radiusM() / mH);
            double crater = v.craterRadiusM() / mH;
            String kind = v.kind();
            boolean shield = "shield".equals(kind);
            boolean cone = shield || "stratovolcano".equals(kind);
            double innerA = 0.0, innerB = 0.0;
            if (crater > 0.0 && v.caldera()) {
                double a = Math.max(MIN_RADIUS, CALDERA_FLOOR_SHARE * crater * st.major);
                double b = Math.max(MIN_RADIUS, CALDERA_FLOOR_SHARE * crater * st.minor);
                out.add(st.zone(Role.CALDERA, v.id(), x, z, a, b));
                innerA = a;
                innerB = b;
            } else if (crater > 0.0) {
                double a = Math.max(MIN_RADIUS, crater * st.major + CRATER_MARGIN);
                double b = Math.max(MIN_RADIUS, crater * st.minor + CRATER_MARGIN);
                out.add(st.zone(Role.CRATER, v.id(), x, z, a, b));
                innerA = a;
                innerB = b;
            }
            if (cone) {
                double a = Math.max(MIN_RADIUS, CONE_SHARE * basal * reach * st.major);
                double b = Math.max(MIN_RADIUS, CONE_SHARE * basal * reach * st.minor);
                if (a > innerA || b > innerB)
                    out.add(st.zone(Role.CONE, v.id(), x, z, Math.max(a, innerA), Math.max(b, innerB)));
                innerA = Math.max(innerA, a);
                innerB = Math.max(innerB, b);
            }
            double fieldA = FIELD_SHARE * basal * reach * st.major, fieldB = FIELD_SHARE * basal * reach * st.minor;
            if (Math.min(fieldA - innerA, fieldB - innerB) >= MIN_RING && fieldB >= MIN_RADIUS) {
                if (shield)
                    out.add(st.zone(Role.LAVA_FIELD, v.id(), x, z, fieldA, fieldB));
                else if (v.activity() >= ASH_MIN_ACTIVITY)
                    out.add(st.zone(Role.ASH_FIELD, v.id(), x, z, fieldA, fieldB));
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

    /**
     * The share of an oceanic volcano's basal radius that stands above the sea (ledger: BIO-198): its radial profile
     * (the simulation's, {@code Shape::profile}: stratovolcano (1 - rho)^1.5, shield (1 - rho^2)^1.5, caldera complex
     * (1 - rho^2)^2; ledger VOL-016 to VOL-018) scaled so that its summit, the vent, stands {@code heightM} over the
     * foot, and cut at the sea. The vent's height over the sea is the world's ground there ({@code ground}); the sea
     * is the profile's value f = (H - h) / H at ρ, H the edifice's height and h the vent's over the sea, so ρ is the
     * profile inverted at f. 0 with the vent at or under the sea, 1 with the vent no lower than the whole edifice
     * (the foot lies at the sea) or with no height to the record. The crater's bowl, which truncates the summit, is
     * left out: the vent then stands lower than the profile's summit, and the share errs small (a smaller zone).
     */
    static double subaerialShare(Volcanoes.Volcano v, int x, int z, Ground ground) {
        double height = v.heightM() / ground.mV();
        if (!(height > 0.0))
            return 1.0;
        double above = ground.heights().at(x, z) - ground.seaLevel();
        if (!(above > 0.0))
            return 0.0;
        double f = (height - above) / height;
        if (!(f > 0.0))
            return 1.0;
        return switch (v.kind()) {
            case "stratovolcano" -> 1.0 - Math.pow(f, 1.0 / 1.5);
            case "shield" -> Math.sqrt(1.0 - Math.pow(f, 1.0 / 1.5));
            case "caldera_complex" -> Math.sqrt(1.0 - Math.sqrt(f));
            default -> 1.0;
        };
    }

    /**
     * A projection's local stretch at a point: the world blocks one planet block (a block of the
     * planet's surface, {@code mH} metres) spans along the semi-major axis ({@code major}, unit
     * direction {@code ax, az} in block space) and across it ({@code minor}).
     */
    record Stretch(double major, double minor, double ax, double az) {

        static final Stretch NONE = new Stretch(1.0, 1.0, 1.0, 0.0);

        /** A zone of semi-axes a (along the stretch) and b (across it), blocks. */
        Zone zone(Role role, int source, int x, int z, double a, double b) {
            return new Zone(role, source, x, z, a, b, ax, az);
        }
    }

    /**
     * The stretch at world point (x, z) (ledger: BIO-182, game-choice). The simulation draws a cone as a
     * circle on the planet (ledger SCL-230: its distances are great-circle ones, the azimuthal equidistant
     * frame at the vent; {@code deeptime volcano-census} measures in the same frame), which a world shows
     * as the ellipse its projection makes of it there: wide east–west by 1/cos φ and narrow north–south by
     * cos φ on a Lambert-frame world, round on the conformal cube (and by any other map's own stretch:
     * the retired Petroff–Guyou world's was 1.78 : 1 at the median). The local
     * Jacobian J of the map from world blocks to the planet's tangent plane at the vent (east and south,
     * central differences over one block, in {@code radius()} blocks a radian) gives it: a circle of
     * radius s there is the world ellipse of semi-axes s/σ, σ the singular values of J, its semi-major
     * axis along the eigenvector of JᵀJ's smaller eigenvalue. {@link Stretch#NONE} (a disc) with no
     * projection, at a pole and where J is not finite or is degenerate. Until 2026-10-23 the frame was
     * the simulation's Lambert blocks (east–west the radius times the longitude difference, north–south
     * the radius times the unit vector's z difference), and a Lambert world kept discs.
     */
    static Stretch stretch(WorldProjection p, double x, double z) {
        if (p == null)
            return Stretch.NONE;
        double rr = p.radius();
        double[] u0 = p.unit(x, z);
        double c0 = Math.sqrt(u0[0] * u0[0] + u0[1] * u0[1]);
        if (!(c0 > 1e-9))
            return Stretch.NONE;
        // The planet's east and north at the vent.
        double ex = -u0[1] / c0, ey = u0[0] / c0;
        double nx = -u0[2] * u0[0] / c0, ny = -u0[2] * u0[1] / c0, nz = c0;
        double[][] at = {{x + 1.0, z}, {x - 1.0, z}, {x, z + 1.0}, {x, z - 1.0}};
        double[] east = new double[4], south = new double[4];
        for (int i = 0; i < 4; i++) {
            double[] u = p.unit(at[i][0], at[i][1]);
            double dx = u[0] - u0[0], dy = u[1] - u0[1], dz = u[2] - u0[2];
            east[i] = rr * (ex * dx + ey * dy);
            south[i] = -rr * (nx * dx + ny * dy + nz * dz);
        }
        // J = [[a, b], [c, d]]: its columns are the tangent plane's (east, south) change per block of x and of z.
        double a = 0.5 * (east[0] - east[1]), c = 0.5 * (south[0] - south[1]);
        double b = 0.5 * (east[2] - east[3]), d = 0.5 * (south[2] - south[3]);
        double sxx = a * a + c * c, sxz = a * b + c * d, szz = b * b + d * d;
        double half = 0.5 * (sxx + szz), root = Math.sqrt(0.25 * (sxx - szz) * (sxx - szz) + sxz * sxz);
        double lmax = half + root, lmin = half - root;
        if (!Double.isFinite(lmax) || !(lmin > 1e-9 * lmax))
            return Stretch.NONE;
        // The eigenvector of the smaller eigenvalue: the way one simulation block spans the most world blocks.
        double vx, vz;
        if (Math.abs(sxz) <= 1e-12 * lmax) {
            vx = sxx <= szz ? 1.0 : 0.0;
            vz = sxx <= szz ? 0.0 : 1.0;
        } else {
            vx = sxz;
            vz = lmin - sxx;
        }
        double n = Math.hypot(vx, vz);
        return new Stretch(1.0 / Math.sqrt(lmin), 1.0 / Math.sqrt(lmax), vx / n, vz / n);
    }
}
