// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.worlddata.Volcanoes;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldProjection;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Set;
import java.util.SplittableRandom;
import java.util.concurrent.atomic.AtomicReferenceArray;
import java.util.function.Consumer;

/**
 * Lava flows painted on and around the world's active volcanoes (review fix 8, 2026-10-22;
 * docs/science/volcanoes-and-magma.md, "Lava flows"). Paint only: a flow changes which block tops a
 * column ({@link LavaFlowPass} near, {@code farterrain.FarTerrain} far), never the terrain's height.
 *
 * <p><b>Which flows a cone shows.</b> A volcano's eruptions come as a Poisson process whose rate is its
 * group's (a basaltic shield's, from Mauna Loa's record; a viscous arc or silicic cone's) times its
 * activity. Going back in time from the present, each eruption leaves one flow of the age so drawn, until
 * the flows are older than the climate lets a flow stay visible, a cap on how much of the cone they cover
 * is reached, or {@link #MAX_FLOWS}. A flow's look follows its age against the rain where it lies: fresh
 * (aa, pahoehoe or blocky lava), then grey, then brown and patchy as soil and plants take it, then gone
 * under vegetation. Younger flows lie over older ones.
 *
 * <p><b>Where a flow runs.</b> From a vent a block past its crater's rim (or, on a shield, half the time
 * from a flank vent on one of two rift zones), downhill by steepest descent on the terrain the generator
 * draws (so it follows the gullies and craters of the drawn heightfield), turned from side to side by a
 * smooth per-flow wander, filling a hollow up to the flow's own thickness before it stops there, stopping
 * at water, until the length the flow was given is used up. Lengths are log-uniform between bounds read
 * from Earth's longest flows, basaltic flows 3.2 times as long as viscous ones at the median (Walker 1973).
 * Lengths, the wander, steps and slopes are in true metres on the ground through the world's own metric
 * ({@link #sphereMetric}: the projection's unit sphere), so nothing here is tied to the projection's frame.
 *
 * <p><b>What a flow paints.</b> One block wide, two where the ground is gentle, and a small lobe at its toe.
 *
 * <p>Deterministic: everything comes from the world data (the volcano list, the drawn height, the water,
 * the rain) and seeds of each volcano's id and position, so a column's answer is the same whatever asks it
 * and in whatever order; far terrain therefore paints what the chunks paint. A volcano's flows are traced on
 * the first question about any column within its reach, once. Thread-safe.
 */
public final class LavaFlows {

    /** What the paint asks of a world. */
    public interface Env {
        /** The terrain height the generator draws at (x, z), in blocks of y; x on any lap. */
        double height(double x, double z);

        /** Whether water (the sea, a lake, a river) lies over column (x, z); x on any lap. */
        boolean wet(int x, int z);

        /** Mean annual precipitation at (x, z), mm a year; NaN where the world has no climate. */
        double precipMm(double x, double z);

        /** The ground's metric at (x, z): {g_xx, g_xz, g_zz}, square metres of ground per square block. */
        double[] metric(double x, double z);
    }

    /** What a column of a flow shows. The code a query answers is {@code ordinal() + 1}. */
    public enum Paint {
        /** Fresh aa: rough, clinkery basalt. */
        AA("aa"),
        /** Fresh pahoehoe: smooth, ropy basalt. */
        PAHOEHOE("pahoehoe"),
        /** Fresh blocky lava of an andesite, dacite or rhyolite flow. */
        BLOCKY("blocky"),
        /** A flow grey with lichen and weathering. */
        WEATHERED("weathered"),
        /** An old flow, brown with oxidation and the first soil, in patches. */
        OXIDIZED("oxidized");

        public final String id;

        Paint(String id) {
            this.id = id;
        }

        public int code() {
            return ordinal() + 1;
        }

        /** The paint of a positive code, or null. */
        public static Paint of(int code) {
            Paint[] all = values();
            return code >= 1 && code <= all.length ? all[code - 1] : null;
        }
    }

    /** A volcano's lava. */
    public enum Group {
        /** Fluid basalt: a shield's. */
        BASALTIC,
        /** Viscous andesite, dacite or rhyolite: an arc stratovolcano's, a silicic centre's. */
        VISCOUS
    }

    // ---- lengths (ledger VOL-100 to VOL-104) ----
    /** The longest basaltic flow, m: Mauna Loa's 1859 flow. */
    static final double BASALT_MAX_M = 51_000.0;  // ledger: VOL-100 (abstract-only: Riker et al. 2009)
    /** The longest viscous flow, m: andesite flows are typically under 12 km. */
    static final double VISCOUS_MAX_M = 12_000.0; // ledger: VOL-101 (verified: USGS VHP, Medicine Lake)
    /** The shortest viscous flow, m. */
    static final double VISCOUS_MIN_M = 500.0;    // ledger: VOL-102 (game-choice)
    /** Basaltic flows' median length over that of andesite, trachyte and rhyolite flows. */
    static final double WALKER_RATIO = 3.2;       // ledger: VOL-103 (abstract-only: Walker 1973)
    /**
     * The shortest basaltic flow, m: lengths are log-uniform (median the geometric mean of the bounds), so
     * the medians' ratio is {@link #WALKER_RATIO} when this is 3.2² × 0.5 × 12 / 51 km = 1.2 km.
     */
    static final double BASALT_MIN_M = WALKER_RATIO * WALKER_RATIO * VISCOUS_MIN_M * VISCOUS_MAX_M / BASALT_MAX_M; // ledger: VOL-104 (game-choice)

    // ---- how often (VOL-105, VOL-106) ----
    /** A basaltic shield's flows a year at activity 1: Mauna Loa's 34 eruptions from 1843 to 2022. */
    static final double BASALT_RATE_PER_YR = 34.0 / 179.0; // ledger: VOL-105 (verified: USGS HVO, Mauna Loa FAQ)
    /** A viscous cone's lava flows a year at activity 1. */
    static final double VISCOUS_RATE_PER_YR = 0.01;        // ledger: VOL-106 (from-memory, game-choice)

    // ---- how long a flow shows, against the rain (VOL-107 to VOL-110) ----
    /** Mean annual precipitation of the three anchors, mm: arid, Hawaii's dry Kona side, its wet Hilo side. */
    static final double[] ANCHOR_P_MM = {150.0, 380.0, 3_800.0};      // ledger: VOL-107, VOL-108, VOL-109
    /** How long a flow stays fresh, black and bare, at each anchor, years. */
    static final double[] FRESH_YR = {15_000.0, 1_500.0, 12.0};       // ledger: VOL-107, VOL-108, VOL-109
    /** How long a flow shows at all before vegetation and soil hide it, at each anchor, years. */
    static final double[] VISIBLE_YR = {250_000.0, 3_000.0, 150.0};   // ledger: VOL-107, VOL-108, VOL-109
    /** The rain assumed where the world has no climate, mm. */
    static final double DEFAULT_P_MM = 1_000.0;                       // ledger: VOL-110 (game-choice)
    /** The share of an oxidised flow's columns still showing when it reaches the end of its visible time. */
    static final double OXIDIZED_KEEP_END = 0.25;                     // ledger: VOL-111 (game-choice)

    // ---- how many (VOL-112, VOL-113) ----
    /** The flows stop when the visible ones cover this share of the cone's basal disc. */
    static final double COVER_CAP = 0.35; // ledger: VOL-112 (game-choice on USGS HVO: 35 to 40 % of Mauna Loa in 1,000 yr)
    /** At most this many flows a volcano. */
    static final int MAX_FLOWS = 256;     // ledger: VOL-113 (game-choice)

    // ---- vents (VOL-114, VOL-115) ----
    /** The share of a shield's flows that start from a flank vent on its rift zones. */
    static final double FLANK_SHARE = 0.5;                          // ledger: VOL-114 (game-choice)
    /** A rift zone's flank vents lie within this angle of its axis (two axes, opposite). */
    static final double RIFT_SPREAD_RAD = Math.toRadians(25.0);     // ledger: VOL-114 (game-choice)
    /** Flank vents lie out to this share of the basal radius. */
    static final double FLANK_MAX_SHARE = 0.5;                      // ledger: VOL-114 (game-choice)
    /** A summit flow starts this many blocks past the crest of its crater's rim. */
    static final double RIM_PAST = 1.0;                             // ledger: VOL-115 (game-choice)

    // ---- the path (VOL-116 to VOL-119) ----
    /** The step of a path, blocks. */
    static final double STEP = 0.5;                                 // ledger: VOL-116 (game-choice)
    /** A path turns up to this far either side of steepest descent, smoothly with distance. */
    static final double WANDER_MAX_RAD = Math.toRadians(35.0);      // ledger: VOL-116 (game-choice)
    /** The wander's lattice, blocks. */
    static final double WANDER_CELL = 3.0;                          // ledger: VOL-116 (game-choice)
    /** Ground flatter than this, m/m, has no direction of its own: the path keeps its last one (at its start, out from the vent). */
    static final double FLAT_SLOPE = 1e-4;                          // ledger: VOL-116 (game-choice)
    /** How deep a flow fills a hollow before it stops there, m: aa 3 to 5 m; andesite 25 to 60 m. */
    static final double BASALT_THICK_M = 5.0, VISCOUS_THICK_M = 40.0; // ledger: VOL-117 (verified: USGS FS 144-02; NPS Yellowstone)
    /** Pahoehoe turns to aa for good on ground steeper than this. */
    static final double PAHOEHOE_TO_AA_DEG = 8.0;                   // ledger: VOL-118 (from-memory, game-choice)
    /** A flow paints two columns side by side on ground gentler than this. */
    static final double WIDE_BELOW_DEG = 5.0;                       // ledger: VOL-119 (game-choice)
    /** The second column lies this far to one side of the path, blocks. */
    static final double SIDE_OFFSET = 0.8;                          // ledger: VOL-119 (game-choice)
    /** The toe lobe: every column whose centre is this near the flow's end, blocks. */
    static final double LOBE_RADIUS = 1.25;                         // ledger: VOL-119 (game-choice)
    /** A metric's principal scales are kept within this factor of the world's nominal block. */
    static final double MAX_STRETCH = 4.0;                          // ledger: VOL-120 (game-choice)

    /** The spatial index's cell, blocks. */
    private static final int GRID = 256;

    private static final double TAN_AA = Math.tan(Math.toRadians(PAHOEHOE_TO_AA_DEG));
    private static final double TAN_WIDE = Math.tan(Math.toRadians(WIDE_BELOW_DEG));

    /** One traced flow (its path only when the flows were built to keep them, for the tests). */
    public record Flow(double ageYr, double lengthM, boolean flank, boolean aaFromVent, double startX, double startZ,
                       double endX, double endZ, double travelledM, int steps, String stop, List<double[]> path) {}

    /** What a volcano's flows came to. {@code cellsByCode[c]}: columns of code c (0: covered by vegetation). */
    public record Summary(int id, String kind, Group group, double precipMm, double ratePerYr, double windowYr, int flows,
                          int visible, int[] cellsByCode, double cover, String stops) {

        public String describe() {
            StringBuilder sb = new StringBuilder();
            for (Paint p : Paint.values())
                if (cellsByCode[p.code()] > 0)
                    sb.append(sb.length() == 0 ? "" : " ").append(p.id).append('=').append(cellsByCode[p.code()]);
            return String.format(Locale.ROOT,
                    "volcano %d (%s, %s): %d flows (rate %.4f/yr, window %.0f yr at %.0f mm), %d columns showing (%s), "
                            + "%d covered, %.1f %% of the basal disc; stops %s",
                    id, kind, group, flows, ratePerYr, windowYr, precipMm, visible, sb, cellsByCode[0], 100.0 * cover, stops);
        }
    }

    /** A volcano's reach: every column it can paint lies within {@code r} blocks of (cx, cz), x wrapped. */
    private record Reach(double cx, double cz, double r, Metric metric) {}

    /** A volcano's flows, as sorted column keys with their code and age. */
    private record Built(long[] keys, byte[] codes, float[] ages, Summary summary, List<Flow> flows) {}

    private final List<Volcanoes.Volcano> volcanoes;
    private final Env env;
    private final int circumference, half;
    private final double zMin, zMax, mH, mV;
    private final boolean keepPaths;
    private final Consumer<String> log;
    private final Reach[] reach;
    private final AtomicReferenceArray<Built> built;
    private final Object[] locks;
    private final Map<Long, int[]> grid;

    /**
     * @param volcanoes     the world's active volcanoes
     * @param circumference x period, blocks
     * @param zMin          the world's z range (a path stops at its edge)
     * @param mH            metres a block horizontally (the nominal block: the metric's bounds), {@code mV} vertically
     * @param log           receives one line per volcano when its flows are traced
     */
    public LavaFlows(List<Volcanoes.Volcano> volcanoes, Env env, int circumference, double zMin, double zMax, double mH, double mV,
                     Consumer<String> log) {
        this(volcanoes, env, circumference, zMin, zMax, mH, mV, log, false);
    }

    LavaFlows(List<Volcanoes.Volcano> volcanoes, Env env, int circumference, double zMin, double zMax, double mH, double mV,
              Consumer<String> log, boolean keepPaths) {
        this.volcanoes = List.copyOf(volcanoes);
        this.env = env;
        this.circumference = circumference;
        this.half = circumference / 2;
        this.zMin = zMin;
        this.zMax = zMax;
        this.mH = mH;
        this.mV = mV;
        this.log = log == null ? s -> {} : log;
        this.keepPaths = keepPaths;
        int n = this.volcanoes.size();
        this.reach = new Reach[n];
        this.built = new AtomicReferenceArray<>(n);
        this.locks = new Object[n];
        Map<Long, Set<Integer>> cells = new HashMap<>();
        for (int i = 0; i < n; i++) {
            locks[i] = new Object();
            Volcanoes.Volcano v = this.volcanoes.get(i);
            Metric m = new Metric(sanitize(env.metric(v.x(), v.z()), mH));
            Group g = group(v);
            double lMax = g == Group.BASALTIC ? BASALT_MAX_M : VISCOUS_MAX_M;
            double start = Math.max(rimCap(v) + RIM_PAST, g == Group.BASALTIC ? FLANK_MAX_SHARE * v.radiusM() / m.mMin : 0.0) + 1.0;
            double r = start + lMax / m.mMin + SIDE_OFFSET + LOBE_RADIUS + 2.0;
            double cx = wrap(v.x());
            reach[i] = new Reach(cx, v.z(), r, m);
            for (double x = cx - r; ; x = Math.min(x + GRID / 2.0, cx + r)) {
                for (double z = v.z() - r; ; z = Math.min(z + GRID / 2.0, v.z() + r)) {
                    cells.computeIfAbsent(gridKey(wrap((int) Math.floor(x)), (int) Math.floor(z)), k -> new LinkedHashSet<>()).add(i);
                    if (z >= v.z() + r)
                        break;
                }
                if (x >= cx + r)
                    break;
            }
        }
        this.grid = new HashMap<>();
        for (var e : cells.entrySet())
            grid.put(e.getKey(), e.getValue().stream().mapToInt(Integer::intValue).sorted().toArray());
    }

    /** The volcanoes indexed. */
    public int size() {
        return volcanoes.size();
    }

    /**
     * What column (x, z), x on any lap, shows: a {@link Paint}'s code, 0 where a flow lies covered by
     * vegetation, −1 where no flow lies. Where flows of several volcanoes meet, the youngest answers.
     */
    public int code(int x, int z) {
        int wx = wrap(x);
        int[] list = grid.get(gridKey(wx, z));
        if (list == null)
            return -1;
        long key = key(wx, z);
        int best = -1;
        float bestAge = Float.POSITIVE_INFINITY;
        for (int i : list) {
            Reach r = reach[i];
            if (Math.abs(delta(wx + 0.5 - r.cx)) > r.r + 1.0 || Math.abs(z + 0.5 - r.cz) > r.r + 1.0)
                continue;
            Built b = built(i);
            int k = Arrays.binarySearch(b.keys, key);
            if (k >= 0 && b.ages[k] < bestAge) {
                bestAge = b.ages[k];
                best = b.codes[k];
            }
        }
        return best;
    }

    /** The paint at column (x, z), or null where none shows. */
    public Paint at(int x, int z) {
        return Paint.of(code(x, z));
    }

    /** Whether any volcano's reach meets block box [x0, x1] × [z0, z1] (a chunk; at most 256 blocks a side), x on any lap. */
    public boolean touches(int x0, int z0, int x1, int z1) {
        double cxBox = 0.5 * (x0 + x1 + 1), czBox = 0.5 * (z0 + z1 + 1), hx = 0.5 * (x1 - x0 + 1), hz = 0.5 * (z1 - z0 + 1);
        for (int x : new int[]{x0, x1})
            for (int z : new int[]{z0, z1}) {
                int[] list = grid.get(gridKey(wrap(x), z));
                if (list == null)
                    continue;
                for (int i : list) {
                    Reach r = reach[i];
                    if (Math.abs(delta(wrap(cxBox) - r.cx)) <= r.r + hx + 1.0 && Math.abs(czBox - r.cz) <= r.r + hz + 1.0)
                        return true;
                }
            }
        return false;
    }

    /** The summary of volcano {@code index} (in the list's order), tracing its flows if that has not happened yet. */
    public Summary summary(int index) {
        return built(index).summary;
    }

    /** The flows of volcano {@code index}, youngest first. */
    public List<Flow> flows(int index) {
        return built(index).flows;
    }

    private Built built(int i) {
        Built b = built.get(i);
        if (b == null) {
            synchronized (locks[i]) {
                b = built.get(i);
                if (b == null) {
                    b = build(i);
                    built.set(i, b);
                    log.accept(b.summary.describe());
                }
            }
        }
        return b;
    }

    // ---- building one volcano ----

    /** The mutable state of one volcano's build. */
    private static final class Ctx {
        final Volcanoes.Volcano v;
        final Group group;
        final Metric met;
        final double riftAz;
        final Map<Long, Long> cells = new HashMap<>();
        final int[] byCode = new int[Paint.values().length + 1];
        int visible;
        final Map<String, Integer> stops = new java.util.TreeMap<>();

        Ctx(Volcanoes.Volcano v, Group group, Metric met, double riftAz) {
            this.v = v;
            this.group = group;
            this.met = met;
            this.riftAz = riftAz;
        }
    }

    private Built build(int index) {
        Volcanoes.Volcano v = volcanoes.get(index);
        Group group = group(v);
        Metric met = reach[index].metric;
        SplittableRandom rng = new SplittableRandom(seed(v));
        Ctx c = new Ctx(v, group, met, rng.nextDouble() * 2.0 * Math.PI);
        double rate = rate(group) * Math.max(0.0, Math.min(1.0, v.activity()));
        double pDry = driestPrecip(v, met);
        double window = interpolate(VISIBLE_YR, pDry);
        double capCells = COVER_CAP * Math.PI * v.radiusM() * v.radiusM() / met.area;
        List<Flow> flows = new ArrayList<>();
        double age = 0.0;
        if (rate > 0.0) {
            for (int k = 0; k < MAX_FLOWS; k++) {
                age += -StrictMath.log(1.0 - rng.nextDouble()) / rate;
                long flowSeed = rng.nextLong();
                if (age >= window || c.visible >= capCells)
                    break;
                Flow f = flow(c, age, flowSeed);
                flows.add(f);
                c.stops.merge(f.stop(), 1, Integer::sum);
            }
        }
        int n = c.cells.size();
        long[] keys = new long[n];
        int i = 0;
        for (long k : c.cells.keySet())
            keys[i++] = k;
        Arrays.sort(keys);
        byte[] codes = new byte[n];
        float[] ages = new float[n];
        for (i = 0; i < n; i++) {
            long packed = c.cells.get(keys[i]);
            codes[i] = (byte) (packed & 0xff);
            ages[i] = Float.intBitsToFloat((int) (packed >>> 32));
        }
        double cover = v.radiusM() > 0.0 ? c.visible * met.area / (Math.PI * v.radiusM() * v.radiusM()) : 0.0;
        Summary s = new Summary(v.id(), v.kind(), group, pDry, rate, window, flows.size(), c.visible, c.byCode.clone(), cover,
                c.stops.toString());
        return new Built(keys, codes, ages, s, List.copyOf(flows));
    }

    /** One flow of age {@code age}: its vent, its length, then its path. */
    private Flow flow(Ctx c, double age, long seed) {
        SplittableRandom fr = new SplittableRandom(seed);
        boolean basaltic = c.group == Group.BASALTIC;
        boolean flank = basaltic && fr.nextDouble() < FLANK_SHARE;
        double lo = basaltic ? BASALT_MIN_M : VISCOUS_MIN_M, hi = basaltic ? BASALT_MAX_M : VISCOUS_MAX_M;
        double length = lo * StrictMath.exp(fr.nextDouble() * StrictMath.log(hi / lo));
        // A long flow is one fed fast, and a flow fed fast is aa from its vent (ledger VOL-118).
        boolean aaFromVent = basaltic && length > Math.sqrt(lo * hi);
        int side = fr.nextBoolean() ? 1 : -1;
        double az = flank ? c.riftAz + (fr.nextBoolean() ? Math.PI : 0.0) + (2.0 * fr.nextDouble() - 1.0) * RIFT_SPREAD_RAD
                : fr.nextDouble() * 2.0 * Math.PI;
        double flankU = fr.nextDouble();
        double[] b = c.met.perMetre(az);           // blocks per metre of ground along the azimuth
        double bl = Math.hypot(b[0], b[1]);
        double ex = b[0] / bl, ez = b[1] / bl;
        double rim = rim(c.v, ex, ez);
        double s = rim + RIM_PAST;
        if (flank) {
            double rimM = rim / bl, maxM = Math.max(rimM, FLANK_MAX_SHARE * c.v.radiusM());
            s = Math.max(s, (rimM + flankU * (maxM - rimM)) * bl);
        }
        return trace(c, c.v.x() + s * ex, c.v.z() + s * ez, ex, ez, length, aaFromVent, side, age, seed, flank);
    }

    /**
     * The distance in blocks from the vent, along block direction (ex, ez), to the crest of its crater's rim:
     * the highest ground of the walk out, which ends where the ground has dropped well below that crest. 0
     * on a cone without a crater (the ground falls from the vent).
     */
    private double rim(Volcanoes.Volcano v, double ex, double ez) {
        double cap = rimCap(v), drop = Math.max(1.0, 0.25 * v.craterDepthM() / mV);
        double best = Double.NEGATIVE_INFINITY, at = 0.0;
        for (double s = 0.0; s <= cap; s += STEP) {
            double h = env.height(v.x() + s * ex, v.z() + s * ez);
            if (h > best) {
                best = h;
                at = s;
            } else if (h < best - drop) {
                break;
            }
        }
        return at;
    }

    /** How far out a rim walk goes at most, blocks: four crater radii at the nominal block, for any projection's stretch. */
    private double rimCap(Volcanoes.Volcano v) {
        return 4.0 * v.craterRadiusM() / mH + 4.0;
    }

    /**
     * Traces a flow from (x, z) and paints its columns. (ox, oz), a unit block direction, is the way out from the
     * vent: a flow that starts on ground with no slope of its own (a gentle summit, where the elevation raster
     * holds a level patch) runs that way until the ground falls.
     */
    private Flow trace(Ctx c, double x0, double z0, double ox, double oz, double length, boolean aaFromVent, int side, double age,
                       long seed, boolean flank) {
        Metric met = c.met;
        double thick = (c.group == Group.BASALTIC ? BASALT_THICK_M : VISCOUS_THICK_M) / mV;
        double x = x0, z = z0;
        List<double[]> path = keepPaths ? new ArrayList<>() : null;
        if (z < zMin || z > zMax || env.wet(floor(x), floor(z)))
            return new Flow(age, length, flank, aaFromVent, x0, z0, x, z, 0.0, 0, "wet-start", path == null ? List.of() : path);
        boolean aa = aaFromVent;
        double travelled = 0.0;
        double px = Double.NaN, pz = Double.NaN;   // the last step's direction
        boolean pond = false;
        double pondFloor = 0.0, pondX = 0.0, pondZ = 0.0;
        int steps = 0, maxSteps = (int) Math.ceil(length / (STEP * met.mMin)) + 8;
        long cell = key(wrap(floor(x)), floor(z));
        int sameCell = 0;
        Map<Long, Integer> entered = new HashMap<>();
        entered.put(cell, 1);
        double slope0 = slopeAt(met, x, z);
        aa |= slope0 > TAN_AA;
        paintStep(c, x, z, slope0, 0.0, 0.0, side, aa, age, seed);
        double low = env.height(x, z);   // the lowest ground the flow has reached: a hollow's floor
        if (path != null)
            path.add(new double[]{x, z, low});
        String stop;
        while (true) {
            if (travelled >= length) {
                stop = "length";
                break;
            }
            if (steps >= maxSteps) {
                stop = "steps";
                break;
            }
            double gx = gradX(x, z), gz = gradZ(x, z);
            double[] d = met.descent(gx, gz);
            double slope = mV * Math.sqrt(Math.max(0.0, -(gx * d[0] + gz * d[1])));
            double dx, dz;
            if (!(slope > FLAT_SLOPE)) {
                // level ground: keep going the way the flow went, or straight out from the vent at its start
                dx = Double.isNaN(px) ? ox : px;
                dz = Double.isNaN(px) ? oz : pz;
            } else {
                double dn = Math.hypot(d[0], d[1]);
                dx = d[0] / dn;
                dz = d[1] / dn;
                double phi = WANDER_MAX_RAD * wander(seed, x, z);
                double cs = StrictMath.cos(phi), sn = StrictMath.sin(phi);
                double rx = dx * cs - dz * sn, rz = dx * sn + dz * cs;
                if (rx * gx + rz * gz < 0.0) {   // still downhill
                    dx = rx;
                    dz = rz;
                }
            }
            if (!Double.isNaN(px) && dx * px + dz * pz < 0.0) {
                // The way down turns back on itself: across a gully's floor (follow the floor, the two ways'
                // mean) or in a closed hollow (fill it up to the flow's thickness along the way it came).
                double mx = dx + px, mz = dz + pz, mn = Math.hypot(mx, mz);
                if (!pond && mn > 0.3) {
                    dx = mx / mn;
                    dz = mz / mn;
                } else {
                    if (!pond) {
                        pond = true;
                        pondFloor = low;
                        pondX = px;
                        pondZ = pz;
                    }
                    dx = pondX;
                    dz = pondZ;
                }
            } else {
                pond = false;
            }
            double nx = x + STEP * dx, nz = z + STEP * dz;
            if (nz < zMin || nz > zMax) {
                stop = "edge";
                break;
            }
            int ix = floor(nx), iz = floor(nz);
            if (env.wet(ix, iz)) {
                stop = "water";
                break;
            }
            double hn = env.height(nx, nz);
            if (pond && hn > pondFloor + thick) {
                stop = "pond";
                break;
            }
            long next = key(wrap(ix), iz);
            if (next != cell) {
                int times = entered.merge(next, 1, Integer::sum);
                if (times > 3) {
                    stop = "loop";
                    break;
                }
                cell = next;
                sameCell = 0;
            } else if (++sameCell > 8) {
                stop = "stuck";
                break;
            }
            travelled += STEP * met.length(dx, dz);
            x = nx;
            z = nz;
            px = dx;
            pz = dz;
            steps++;
            double s = slope > FLAT_SLOPE ? slope : 0.0;
            aa |= s > TAN_AA;
            paintStep(c, x, z, s, dx, dz, side, aa, age, seed);
            low = Math.min(low, hn);
            if (path != null)
                path.add(new double[]{x, z, hn});
        }
        // The toe: a small lobe round the end.
        int r = (int) Math.ceil(LOBE_RADIUS);
        for (int dzc = -r; dzc <= r; dzc++)
            for (int dxc = -r; dxc <= r; dxc++) {
                int cx = floor(x) + dxc, cz = floor(z) + dzc;
                if (Math.hypot(cx + 0.5 - x, cz + 0.5 - z) <= LOBE_RADIUS && cz >= zMin && cz <= zMax && !env.wet(cx, cz))
                    mark(c, cx, cz, paint(c, aa, age, seed, cx, cz), age);
            }
        return new Flow(age, length, flank, aaFromVent, x0, z0, x, z, travelled, steps, stop, path == null ? List.of() : path);
    }

    /** Paints the column under (x, z) and, on gentle ground, the one beside it. */
    private void paintStep(Ctx c, double x, double z, double slope, double dx, double dz, int side, boolean aa, double age, long seed) {
        int cx = floor(x), cz = floor(z);
        mark(c, cx, cz, paint(c, aa, age, seed, cx, cz), age);
        if (slope < TAN_WIDE && (dx != 0.0 || dz != 0.0)) {
            int sx = floor(x - side * SIDE_OFFSET * dz), sz = floor(z + side * SIDE_OFFSET * dx);
            if ((sx != cx || sz != cz) && sz >= zMin && sz <= zMax && !env.wet(sx, sz))
                mark(c, sx, sz, paint(c, aa, age, seed, sx, sz), age);
        }
    }

    /** Puts a column's code unless a younger flow (or this one) already lies there. */
    private void mark(Ctx c, int x, int z, int code, double age) {
        long k = key(wrap(x), z);
        if (c.cells.containsKey(k))
            return;
        c.cells.put(k, ((long) Float.floatToIntBits((float) age) << 32) | (code & 0xffL));
        c.byCode[code]++;
        if (code > 0)
            c.visible++;
    }

    /** What a flow of this age shows at column (x, z), by the rain there (ledger VOL-107 to VOL-111). */
    private int paint(Ctx c, boolean aa, double age, long seed, int x, int z) {
        double p = precip(x + 0.5, z + 0.5);
        double fresh = interpolate(FRESH_YR, p), visible = interpolate(VISIBLE_YR, p);
        if (age < fresh)
            return (c.group == Group.VISCOUS ? Paint.BLOCKY : aa ? Paint.AA : Paint.PAHOEHOE).code();
        if (age >= visible)
            return 0;
        double mid = Math.sqrt(fresh * visible);
        if (age < mid)
            return Paint.WEATHERED.code();
        double keep = 1.0 - (1.0 - OXIDIZED_KEEP_END) * StrictMath.log(age / mid) / StrictMath.log(visible / mid);
        return unit(mix(seed ^ 0x6F78696469736564L, wrap(x), z)) < keep ? Paint.OXIDIZED.code() : 0;
    }

    /** The rain at (x, z), mm, or the default where the world has none. */
    private double precip(double x, double z) {
        double p = env.precipMm(x, z);
        return Double.isFinite(p) && p >= 0.0 ? p : DEFAULT_P_MM;
    }

    /** The driest rain on the cone: its summit and two rings (half the basal radius and the basal radius). */
    private double driestPrecip(Volcanoes.Volcano v, Metric met) {
        double p = precip(v.x(), v.z());
        for (double share : new double[]{0.5, 1.0})
            for (int k = 0; k < 8; k++) {
                double[] b = met.perMetre(k * Math.PI / 4.0);
                p = Math.min(p, precip(v.x() + b[0] * share * v.radiusM(), v.z() + b[1] * share * v.radiusM()));
            }
        return p;
    }

    /**
     * A time against the rain: linear in log time against log rain between the anchors
     * ({@link #ANCHOR_P_MM}), the end anchors' times outside them (ledger VOL-110).
     */
    static double interpolate(double[] years, double precipMm) {
        double[] p = ANCHOR_P_MM;
        if (!(precipMm > p[0]))
            return years[0];
        if (precipMm >= p[p.length - 1])
            return years[years.length - 1];
        int i = 0;
        while (precipMm > p[i + 1])
            i++;
        double t = StrictMath.log(precipMm / p[i]) / StrictMath.log(p[i + 1] / p[i]);
        return StrictMath.exp(StrictMath.log(years[i]) + t * StrictMath.log(years[i + 1] / years[i]));
    }

    /** The ground's slope at (x, z), m/m, through the metric. */
    private double slopeAt(Metric met, double x, double z) {
        double gx = gradX(x, z), gz = gradZ(x, z);
        double[] d = met.descent(gx, gz);
        return mV * Math.sqrt(Math.max(0.0, -(gx * d[0] + gz * d[1])));
    }

    private double gradX(double x, double z) {
        return (env.height(x + 0.5, z) - env.height(x - 0.5, z));
    }

    private double gradZ(double x, double z) {
        return (env.height(x, z + 0.5) - env.height(x, z - 0.5));
    }

    /** A smooth per-flow field in [−1, 1]: a lattice of random values, smoothstep-interpolated. */
    static double wander(long seed, double x, double z) {
        double u = x / WANDER_CELL, w = z / WANDER_CELL;
        long i = (long) Math.floor(u), j = (long) Math.floor(w);
        double fu = smooth(u - i), fw = smooth(w - j);
        double a = lattice(seed, i, j), b = lattice(seed, i + 1, j), c = lattice(seed, i, j + 1), d = lattice(seed, i + 1, j + 1);
        double top = a + (b - a) * fu, bottom = c + (d - c) * fu;
        return top + (bottom - top) * fw;
    }

    private static double smooth(double t) {
        return t * t * (3.0 - 2.0 * t);
    }

    private static double lattice(long seed, long i, long j) {
        return 2.0 * unit(mix(seed, i, j)) - 1.0;
    }

    // ---- the volcano's numbers ----

    static Group group(Volcanoes.Volcano v) {
        return "shield".equals(v.kind()) ? Group.BASALTIC : Group.VISCOUS;  // ledger: VOL-121 (game-choice)
    }

    static double rate(Group g) {
        return g == Group.BASALTIC ? BASALT_RATE_PER_YR : VISCOUS_RATE_PER_YR;
    }

    /** The seed of a volcano's flows: its id and position. */
    static long seed(Volcanoes.Volcano v) {
        return mix(0x4C617661466C6F77L ^ v.id(), Double.doubleToLongBits(v.x()), Double.doubleToLongBits(v.z()));
    }

    // ---- the metric ----

    /**
     * The ground's metric at world point (x, z) from the projection's unit sphere: the Jacobian J of the map from
     * world blocks to the sphere (central differences over a block, the sphere's radius in metres
     * {@code radius() × mH}), G = JᵀJ. With no projection, the nominal block: G = mH² I (ledger VOL-120).
     */
    public static double[] sphereMetric(WorldProjection p, double x, double z, double mH) {
        if (p == null)
            return new double[]{mH * mH, 0.0, mH * mH};
        double rr = p.radius() * mH;
        double[] xp = p.unit(x + 1.0, z), xm = p.unit(x - 1.0, z), zp = p.unit(x, z + 1.0), zm = p.unit(x, z - 1.0);
        double gxx = 0.0, gxz = 0.0, gzz = 0.0;
        for (int k = 0; k < 3; k++) {
            double jx = 0.5 * (xp[k] - xm[k]) * rr, jz = 0.5 * (zp[k] - zm[k]) * rr;
            gxx += jx * jx;
            gxz += jx * jz;
            gzz += jz * jz;
        }
        return new double[]{gxx, gxz, gzz};
    }

    /**
     * A usable metric: {@code g} with its principal scales kept within {@link #MAX_STRETCH} of the nominal
     * block either way (a pole's or a fold's degenerate metric would give a flow an unbounded reach); the
     * nominal block when {@code g} is not finite or not positive definite.
     */
    static double[] sanitize(double[] g, double mH) {
        double[] iso = {mH * mH, 0.0, mH * mH};
        if (g == null || g.length != 3 || !Double.isFinite(g[0]) || !Double.isFinite(g[1]) || !Double.isFinite(g[2]))
            return iso;
        double a = g[0], b = g[1], c = g[2];
        double halfSum = 0.5 * (a + c), root = Math.sqrt(0.25 * (a - c) * (a - c) + b * b);
        double l1 = halfSum + root, l2 = halfSum - root;
        if (!(l2 > 0.0))
            return iso;
        double lo = mH * mH / (MAX_STRETCH * MAX_STRETCH), hi = mH * mH * MAX_STRETCH * MAX_STRETCH;
        double c1 = Math.max(lo, Math.min(hi, l1)), c2 = Math.max(lo, Math.min(hi, l2));
        if (c1 == l1 && c2 == l2)
            return g.clone();
        double vx, vz;
        if (Math.abs(b) <= 1e-12 * l1) {
            vx = a >= c ? 1.0 : 0.0;
            vz = a >= c ? 0.0 : 1.0;
        } else {
            vx = b;
            vz = l1 - a;
        }
        double n = Math.hypot(vx, vz);
        vx /= n;
        vz /= n;
        // G = c1 v vᵀ + c2 w wᵀ, w = (−vz, vx).
        return new double[]{c1 * vx * vx + c2 * vz * vz, (c1 - c2) * vx * vz, c1 * vz * vz + c2 * vx * vx};
    }

    /** A metric G (m² per block²) and what the paths need of it. */
    static final class Metric {
        final double gxx, gxz, gzz, ixx, ixz, izz, area, mMin;

        Metric(double[] g) {
            gxx = g[0];
            gxz = g[1];
            gzz = g[2];
            double det = gxx * gzz - gxz * gxz;
            ixx = gzz / det;
            ixz = -gxz / det;
            izz = gxx / det;
            area = Math.sqrt(det);
            double halfSum = 0.5 * (gxx + gzz), root = Math.sqrt(0.25 * (gxx - gzz) * (gxx - gzz) + gxz * gxz);
            mMin = Math.sqrt(halfSum - root);
        }

        /** Steepest descent of a height whose gradient is (gx, gz), in blocks: −G⁻¹∇H. */
        double[] descent(double gx, double gz) {
            return new double[]{-(ixx * gx + ixz * gz), -(ixz * gx + izz * gz)};
        }

        /** Metres of ground a block-space displacement (dx, dz) spans. */
        double length(double dx, double dz) {
            return Math.sqrt(gxx * dx * dx + 2.0 * gxz * dx * dz + gzz * dz * dz);
        }

        /**
         * The block displacement of one metre of ground at azimuth {@code az}, measured in the frame G makes
         * orthonormal (G = L Lᵀ, Cholesky; b = L⁻ᵀ (cos az, sin az)).
         */
        double[] perMetre(double az) {
            double l11 = Math.sqrt(gxx), l21 = gxz / l11, l22 = Math.sqrt(gzz - l21 * l21);
            double qx = StrictMath.cos(az), qz = StrictMath.sin(az);
            double bz = qz / l22;
            return new double[]{(qx - l21 * bz) / l11, bz};
        }
    }

    // ---- keys, wrapping, hashing ----

    /** x folded into [−C/2, C/2). */
    public int wrap(int x) {
        return Math.floorMod(x + half, circumference) - half;
    }

    double wrap(double x) {
        double w = (x + half) % circumference;
        if (w < 0)
            w += circumference;
        return w - half;
    }

    /** A difference of x folded into [−C/2, C/2). */
    private double delta(double dx) {
        return wrap(dx);
    }

    static long key(int wx, int z) {
        return ((long) wx << 32) | (z & 0xffffffffL);
    }

    private long gridKey(int wx, int z) {
        return ((long) Math.floorDiv(wx + half, GRID) << 32) | (Math.floorDiv(z, GRID) & 0xffffffffL);
    }

    private static int floor(double v) {
        return (int) Math.floor(v);
    }

    static long mix(long seed, long a, long b) {
        long h = seed ^ (a * 0x9E3779B97F4A7C15L) ^ (b * 0xC2B2AE3D27D4EB4FL);
        h = (h ^ (h >>> 30)) * 0xBF58476D1CE4E5B9L;
        h = (h ^ (h >>> 27)) * 0x94D049BB133111EBL;
        return h ^ (h >>> 31);
    }

    /** A hash as a double in [0, 1). */
    static double unit(long h) {
        return (h >>> 11) * 0x1.0p-53;
    }
}
