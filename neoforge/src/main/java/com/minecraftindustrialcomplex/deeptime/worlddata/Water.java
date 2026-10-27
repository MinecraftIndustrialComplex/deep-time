// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

/**
 * Where the water goes in one block column (P1-22; FORMAT.md "Water columns"; Rust
 * {@code deeptime_format::water}, operation for operation): the lake, sea, river or bank a column
 * is, its water top and bed, and a bank's limits. A function of (x, z) and the world data only, so
 * neighbouring chunks and the two sides of the seam agree without seeing each other; the game's
 * {@code worldgen/WaterBodies} pass makes every generated column match it. Plain Java, no
 * Minecraft, thread-safe.
 */
public final class Water {

    private Water() {}

    public enum Kind {
        NONE("none"), BANK("bank"), RIVER("river"), LAKE("lake"), SEA("sea");

        public final String id;

        Kind(String id) {
            this.id = id;
        }

        public boolean isWater() {
            return this == RIVER || this == LAKE || this == SEA;
        }
    }

    /**
     * The constants the generator fixes: the noise cell width, the shore rule's reach, sea level
     * and the build range.
     */
    public record Spec(int cell, int reach, int seaLevel, int minY, int maxY) {
        /** A Deep Time world's (cell 4, reach 4) with its manifest's sea level and range. */
        public static Spec of(WorldData wd, int cell) {
            Manifest m = wd.manifest();
            return new Spec(cell, cell, m.seaLevelY(), m.minY(), m.maxY());
        }
    }

    /**
     * Whether the world's carvers spare the water Deep Time places: {@code
     * notes.bake.water.params.protect_water_from_carvers}. True when the key is absent (a world baked
     * before the setting existed, or with {@code hydro.enabled} false, which has no water anyway).
     */
    public static boolean protectsFromCarvers(JsonObject notes) {
        JsonElement e = notes;
        for (String key : new String[]{"bake", "water", "params", "protect_water_from_carvers"}) {
            if (e == null || !e.isJsonObject())
                return true;
            e = e.getAsJsonObject().get(key);
        }
        return e == null || !e.isJsonPrimitive() || !e.getAsJsonPrimitive().isBoolean() || e.getAsBoolean();
    }

    /** A column's kind, surface, level, top and bed before its neighbours are looked at. */
    public record Core(Kind kind, double surface, int ground, double level, int top, int bed, Rivers.Sample rivers) {}

    /**
     * One column's water (Rust {@code WaterColumn}); boxed fields are null where Rust has None.
     */
    public record Column(Kind kind, double surface, int ground, Double level, Integer top, Integer bed,
                         Integer minGround, Integer maxGround, Integer sealFrom, boolean riverBiome, Rivers.Hit river) {}

    /** How far below the sea's top a coastal bank is sealed, blocks (Rust {@code SEA_SEAL}). */
    public static final int SEA_SEAL = 3;

    /**
     * A lake column a channel covers keeps the channel's bed where the lake's water over the
     * generated surface is less than this deep, blocks: the bake's own line between lake water a
     * river crosses and lake water it ends in (HYD-020, a block), read at the column (Rust
     * {@code SHALLOW_LAKE}; ledger: HYD-122).
     */
    public static final double SHALLOW_LAKE = 1.0;

    /**
     * How far a river column's water and bed go down where its banks do not stand above the water,
     * blocks: one, the freeboard the bake leaves under its filled surface (HYD-018), so a channel on
     * flat ground is cut a block into it rather than walled a block above it (Rust {@code INCISION};
     * game-choice: Ben's curb report, 2026-10-22; ledger: HYD-123).
     */
    public static final int INCISION = 1;

    /** The 4-neighbour offsets, in the order both readers visit them. */
    public static final int[][] NEIGHBOURS = {{1, 0}, {-1, 0}, {0, 1}, {0, -1}};

    /** ⌈h⌉ − 1. */
    public static int topBlock(double h) {
        long t = (long) Math.ceil(h) - 1;
        return (int) Math.max(Integer.MIN_VALUE, Math.min(Integer.MAX_VALUE, t));
    }

    /**
     * Step 3's test, the one place the sea's rule is written (Rust {@code water::sea_covers}, ledger
     * HYD-106): the open sea stands on a column whose top block {@code ground} is under sea level - 1,
     * unless the {@code closed_basin} layer keeps it off ({@code closed}, read only where the first
     * test passes). {@link #core} reads it; the bake's biome layers read the Rust twin.
     */
    public static boolean seaCovers(int ground, int seaLevel, java.util.function.BooleanSupplier closed) {
        return ground < seaLevel - 1 && !closed.getAsBoolean();
    }

    /**
     * Step 2's test, the one place the lake's rule is written (Rust {@code water::core}'s lake
     * branch): a lake stands on a column whose top block {@code ground} is under the top block of
     * the {@code level} the shore rule gives it ({@link #levelAt}). The shore rule gives a level to
     * the dry columns within its reach of a lake cell too; this says which columns hold the lake.
     * {@link #core} and {@link LakeIce} read it.
     */
    public static boolean lakeCovers(int ground, double level) {
        return Double.isFinite(level) && ground < topBlock(level);
    }

    static long wrap(WorldData wd, long x) {
        long c = wd.circumference();
        return Math.floorMod(x + c / 2, c) - c / 2;
    }

    /** The terrain height H at block (x, z), any lap (Rust {@code ColumnWorld::height}). */
    static double height(WorldData wd, long x, long z) {
        return wd.height(wrap(wd, x), z);
    }

    /** H bilinear between the corners of the noise cell holding canonical column (wx, z). */
    public static double cellSurface(WorldData wd, Spec spec, long wx, long z) {
        int c = spec.cell();
        long x0 = Math.floorDiv(wx, (long) c) * c;
        long z0 = Math.floorDiv(z, (long) c) * c;
        double fx = (double) (wx - x0) / (double) c;
        double fz = (double) (z - z0) / (double) c;
        double h00 = height(wd, x0, z0);
        double h10 = height(wd, x0 + c, z0);
        double h01 = height(wd, x0, z0 + c);
        double h11 = height(wd, x0 + c, z0 + c);
        double a = h00 + (h10 - h00) * fx;
        double b = h01 + (h11 - h01) * fx;
        return a + (b - a) * fz;
    }

    /**
     * The shore rule (Rust {@code column::water_level_at}; {@code worldgen/WaterLevels.levelAt}
     * delegates here): the {@code water_level} cell nearest block (x, z), else the value of the
     * nearest wet cell within {@code reach} blocks; NaN when there is none.
     */
    public static double levelAt(RasterSampler levels, long x, long z, int reach) {
        double own = levels.nearest(x, z);   // x wraps in the sampler
        if (!Double.isNaN(own) || reach <= 0)
            return own;
        GridDesc g = levels.desc().grid();
        if (g.isCube()) {
            // On the block centre's face, the cells round it read across the seams, each at its distance from the
            // block centre (Rust water_level_at's cube branch, the same operations).
            GridDesc.CubeSpot s = g.cubeSpotOfBlock((double) x, (double) z);
            if (s == null)
                return Double.NaN;
            double rr = g.r();
            double span = (double) reach / rr;
            long ci0 = (long) Math.floor(s.a() - span), ci1 = (long) Math.ceil(s.a() + span);
            long cj0 = (long) Math.floor(s.b() - span), cj1 = (long) Math.ceil(s.b() + span);
            double best = Double.NaN;
            double bestD2 = (double) ((long) reach * reach + 1);
            for (long i = ci0; i <= ci1; i++) {
                double dx = ((double) i - s.a()) * rr;
                for (long j = cj0; j <= cj1; j++) {
                    double dz = ((double) j - s.b()) * rr;
                    double d2 = dx * dx + dz * dz;
                    if (d2 >= bestD2)
                        continue;
                    double v = levels.getFace(s.face(), i, j);
                    if (!Double.isNaN(v)) {
                        best = v;
                        bestD2 = d2;
                    }
                }
            }
            return best;
        }
        int r = g.r();
        long i0 = Math.floorDiv(x - reach - g.originBlockX(), r), i1 = Math.floorDiv(x + reach - g.originBlockX(), r) + 1;
        long j0 = Math.floorDiv(z - reach - g.originBlockZ(), r), j1 = Math.floorDiv(z + reach - g.originBlockZ(), r) + 1;
        double best = Double.NaN;
        long bestD2 = (long) reach * reach + 1;
        for (long i = i0; i <= i1; i++) {
            long dx = g.originBlockX() + i * r - x;
            for (long j = j0; j <= j1; j++) {
                long dz = g.originBlockZ() + j * r - z;
                long d2 = dx * dx + dz * dz;
                if (d2 >= bestD2)
                    continue;
                double v = levels.get(i, j);
                if (!Double.isNaN(v)) {
                    best = v;
                    bestD2 = d2;
                }
            }
        }
        return best;
    }

    /**
     * Whether a river's channel lowers the bed of a sea or lake column under its ground: a channel
     * run on into the sea (HYD-121) or across lake water less than {@link #SHALLOW_LAKE} deep
     * (HYD-122). The game cuts such a bed and lays the channel's sediment on it.
     */
    public static boolean cutByChannel(Column c) {
        return (c.kind() == Kind.SEA || c.kind() == Kind.LAKE) && c.bed() != null && c.bed() < c.ground();
    }

    /** {@link #cutByChannel(Column)} for a column's core. */
    public static boolean cutByChannel(Core c) {
        return (c.kind() == Kind.SEA || c.kind() == Kind.LAKE) && c.bed() < c.ground();
    }

    /** The top block of a generated surface, as step 1 clamps it. */
    static int groundOf(Spec spec, double surface) {
        return Double.isFinite(surface) ? Math.max(spec.minY() - 1, Math.min(spec.maxY(), topBlock(surface))) : spec.minY() - 1;
    }

    /** Steps 1–4 for canonical column (wx, z) given its surface and channel sample. */
    public static Core core(WorldData wd, Spec spec, long wx, long z, double surface, Rivers.Sample rivers) {
        int ground = groundOf(spec, surface);
        double level = wd.has(WorldData.WATER_LEVEL) ? levelAt(wd.layer(WorldData.WATER_LEVEL), wx, z, spec.reach()) : Double.NaN;
        // The channel's floor under the column, as a block: the lowest of its covering segments'
        // parabolas (ledger: HYD-022, HYD-120).
        Rivers.Hit deep = rivers.deep();
        boolean channel = deep != null;
        int f = channel ? (int) Math.floor(deep.floor()) : 0;
        if (lakeCovers(ground, level)) {
            // A channel crossing lake water less than a block deep keeps its bed (ledger: HYD-122).
            int bed = channel && level - surface < SHALLOW_LAKE ? Math.min(ground, f) : ground;
            return new Core(Kind.LAKE, surface, ground, level, topBlock(level), bed, rivers);
        }
        // No sea in a closed basin below sea level (HYD-100; Rust WaterSource::closed).
        if (seaCovers(ground, spec.seaLevel(), () -> wd.closedBasin(wx, z))) {
            // A channel into the sea keeps its bed across the shallows (ledger: HYD-121).
            int bed = channel ? Math.min(ground, f) : ground;
            return new Core(Kind.SEA, surface, ground, level, spec.seaLevel() - 1, bed, rivers);
        }
        Rivers.Hit h = rivers.hit();
        if (h != null && channel) {
            int t = topBlock(h.w());
            // Incised, not perched: where the ground at the channel's banks does not stand above the
            // water's top block, the water and the bed go INCISION down; never under the sea's top
            // (ledger: HYD-123).
            int seaTop = spec.seaLevel() - 1;
            int drop = t > seaTop && bankGround(wd, spec, ground, deep) <= t ? Math.min(INCISION, t - seaTop) : 0;
            int top = t - drop;
            return new Core(Kind.RIVER, surface, ground, level, top, Math.min(f - drop, top - 1), rivers);
        }
        return new Core(Kind.NONE, surface, ground, level, Integer.MIN_VALUE, Integer.MIN_VALUE, rivers);
    }

    /**
     * The lower of the generated ground's top blocks at the two points a block beyond the edges of
     * channel {@code d} (on its segment's normal through the foot at {@code d.t()}, {@code d.hw()} + 1
     * out each side): where the channel's banks stand before the pass touches them (Rust
     * {@code bank_ground}; ledger: HYD-123). A segment of no length has no normal: the column's own
     * {@code ground}.
     */
    public static int bankGround(WorldData wd, Spec spec, int ground, Rivers.Hit d) {
        Rivers r = wd.rivers();
        if (r == null)
            return ground;
        double ax = r.vertexX(d.seg()), az = r.vertexZ(d.seg());
        double ex = (double) r.vertexX(d.seg() + 1) - ax, ez = (double) r.vertexZ(d.seg() + 1) - az;
        double l = Math.sqrt(ex * ex + ez * ez);
        if (l <= 0.0)
            return ground;
        double cx = ax + d.t() * ex, cz = az + d.t() * ez;
        double nx = -ez / l, nz = ex / l;
        double off = d.hw() + 1.0;
        return Math.min(groundAt(wd, spec, cx + nx * off, cz + nz * off), groundAt(wd, spec, cx - nx * off, cz - nz * off));
    }

    private static int groundAt(WorldData wd, Spec spec, double px, double pz) {
        long x = wrap(wd, (long) Math.floor(px));
        return groundOf(spec, cellSurface(wd, spec, x, (long) Math.floor(pz)));
    }

    /** Steps 1–4 for column (x, z), any lap. */
    public static Core core(WorldData wd, Spec spec, long x, long z) {
        long wx = wrap(wd, x);
        Rivers r = wd.rivers();
        Rivers.Sample s = r == null ? Rivers.Sample.NONE : r.sample(wx, z);
        return core(wd, spec, wx, z, cellSurface(wd, spec, wx, z), s);
    }

    /**
     * The highest water block a water column holds once the pass has put in its steps: its own top,
     * or a higher water 4-neighbour's, whose flowing water the pass spreads into it ({@code n} in
     * {@link #NEIGHBOURS} order; Rust {@code held}; ledger: HYD-124). Not water:
     * {@link Integer#MIN_VALUE}.
     */
    public static int held(Core own, Core[] n) {
        if (!own.kind().isWater())
            return Integer.MIN_VALUE;
        int m = own.top();
        for (Core c : n)
            if (c.kind().isWater())
                m = Math.max(m, c.top());
        return m;
    }

    /**
     * Step 5: the column from its own core and its neighbours' ({@link #NEIGHBOURS} order), with
     * what each neighbour holds ({@link #held}).
     */
    public static Column finish(Core own, Core[] n, int[] holds) {
        Double level = Double.isFinite(own.level()) ? own.level() : null;
        Rivers.Sample rs = own.rivers();
        if (own.kind().isWater())
            return new Column(own.kind(), own.surface(), own.ground(), level, own.top(), own.bed(), null, null, null,
                    rs.biome(), rs.hit());
        // A one-block seam between two river channels is channel (Rust finish; ledger: HYD-182): where two
        // segments' corridors meet edge to edge, a column between them can fall outside both, and as a bank
        // it kept its ground above the water, a dotted line of single dry blocks across the river. Between
        // river columns on opposite sides it holds the lower of their waters on the shallower of their beds.
        for (int k = 0; k < 4; k += 2) {
            Core a = n[k], b = n[k + 1];
            if (a.kind() != Kind.RIVER || b.kind() != Kind.RIVER)
                continue;
            Core w = b.top() < a.top() ? b : a;
            int bed = Math.min(Math.max(a.bed(), b.bed()), w.top() - 1);
            return new Column(Kind.RIVER, own.surface(), own.ground(), level, w.top(), bed, null, null, null,
                    a.rivers().biome() || b.rivers().biome(), w.rivers().hit());
        }
        Integer minGround = null, seal = null;
        for (int k = 0; k < 4; k++) {
            Core c = n[k];
            if (!c.kind().isWater())
                continue;
            // Beside a river, the water it holds with its steps (ledger: HYD-124); beside a lake or
            // the sea, its top (HYD-077).
            int need = c.kind() == Kind.RIVER ? holds[k] : c.top();
            minGround = minGround == null ? need : Math.max(minGround, need);
            // Beside the sea only the waterline is sealed (cave mouths at the coast).
            int from = c.kind() == Kind.SEA ? Math.max(c.bed() + 1, c.top() - SEA_SEAL) : c.bed() + 1;
            seal = seal == null ? from : Math.min(seal, from);
        }
        Integer maxGround = rs.bankCap();
        Kind kind = minGround != null || maxGround != null ? Kind.BANK : own.kind();
        return new Column(kind, own.surface(), own.ground(), level, null, null, minGround, maxGround, seal,
                rs.biome(), rs.hit());
    }

    /** The whole evaluation of column (x, z), any lap. */
    public static Column column(WorldData wd, Spec spec, long x, long z) {
        Core own = core(wd, spec, x, z);
        Core[] n = new Core[4];
        for (int k = 0; k < 4; k++)
            n[k] = core(wd, spec, x + NEIGHBOURS[k][0], z + NEIGHBOURS[k][1]);
        int[] holds = {Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE};
        if (!own.kind().isWater()) {
            // A river neighbour's own neighbours: the ring two blocks out, and this column.
            for (int k = 0; k < 4; k++) {
                if (n[k].kind() != Kind.RIVER)
                    continue;
                long nx = x + NEIGHBOURS[k][0], nz = z + NEIGHBOURS[k][1];
                Core[] nn = new Core[4];
                for (int e = 0; e < 4; e++) {
                    long ex = nx + NEIGHBOURS[e][0], ez = nz + NEIGHBOURS[e][1];
                    nn[e] = ex == x && ez == z ? own : core(wd, spec, ex, ez);
                }
                holds[k] = held(n[k], nn);
            }
        }
        return finish(own, n, holds);
    }

    /**
     * The cores of a block box [x0, x0 + w) × [z0, z0 + h) at once (the game's pass: one gather of
     * the river segments for the box); {@link #at} indexes it. x0 may be any lap. It holds a ring of
     * {@link #MARGIN} more (what a bank's river neighbour holds reads the neighbour's neighbours), so
     * {@link #at} also answers one block outside the box.
     */
    public static final class Region {
        /** The hidden ring of cores round the box, blocks. */
        static final int MARGIN = 1;
        public final long x0, z0;
        public final int w, h;
        private final Core[] cores;

        Region(long x0, long z0, int w, int h, Core[] cores) {
            this.x0 = x0;
            this.z0 = z0;
            this.w = w;
            this.h = h;
            this.cores = cores;
        }

        /** The core at (x0 + dx, z0 + dz), dx in [−1, w], dz in [−1, h]. */
        public Core at(int dx, int dz) {
            return cores[(dz + MARGIN) * (w + 2 * MARGIN) + dx + MARGIN];
        }

        private Core[] around(int dx, int dz) {
            Core[] n = new Core[4];
            for (int k = 0; k < 4; k++)
                n[k] = at(dx + NEIGHBOURS[k][0], dz + NEIGHBOURS[k][1]);
            return n;
        }

        /** The column at (x0 + dx, z0 + dz), which must not be on the region's edge. */
        public Column column(int dx, int dz) {
            Core own = at(dx, dz);
            Core[] n = around(dx, dz);
            int[] holds = {Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE, Integer.MIN_VALUE};
            if (!own.kind().isWater())
                for (int k = 0; k < 4; k++)
                    if (n[k].kind() == Kind.RIVER)
                        holds[k] = held(n[k], around(dx + NEIGHBOURS[k][0], dz + NEIGHBOURS[k][1]));
            return finish(own, n, holds);
        }
    }

    /** The cores of block box [x0, x0 + w) × [z0, z0 + h) (x0 any lap), and its hidden ring. */
    public static Region region(WorldData wd, Spec spec, long bx0, long bz0, int bw, int bh) {
        int m = Region.MARGIN;
        long x0 = bx0 - m, z0 = bz0 - m;
        int w = bw + 2 * m, h = bh + 2 * m;
        Rivers r = wd.rivers();
        int[] segs = r == null ? new int[0] : r.entriesIn(x0, z0, x0 + w - 1, z0 + h - 1);
        Core[] cores = new Core[w * h];
        // H at the noise-cell corners of the box, once each.
        int c = spec.cell();
        long cx0 = Math.floorDiv(x0, (long) c) * c, cz0 = Math.floorDiv(z0, (long) c) * c;
        int nx = (int) ((x0 + w - 1 - cx0) / c) + 2, nz = (int) ((z0 + h - 1 - cz0) / c) + 2;
        double[] corner = new double[nx * nz];
        for (int j = 0; j < nz; j++)
            for (int i = 0; i < nx; i++)
                corner[j * nx + i] = height(wd, cx0 + (long) i * c, cz0 + (long) j * c);
        for (int dz = 0; dz < h; dz++) {
            for (int dx = 0; dx < w; dx++) {
                long x = x0 + dx, z = z0 + dz;
                long wx = wrap(wd, x);
                int i = (int) ((Math.floorDiv(x, (long) c) * c - cx0) / c), j = (int) ((Math.floorDiv(z, (long) c) * c - cz0) / c);
                double fx = (double) (x - (cx0 + (long) i * c)) / (double) c;
                double fz = (double) (z - (cz0 + (long) j * c)) / (double) c;
                double h00 = corner[j * nx + i], h10 = corner[j * nx + i + 1];
                double h01 = corner[(j + 1) * nx + i], h11 = corner[(j + 1) * nx + i + 1];
                double a = h00 + (h10 - h00) * fx;
                double b = h01 + (h11 - h01) * fx;
                double surface = a + (b - a) * fz;
                Rivers.Sample s = r == null || segs.length == 0 ? Rivers.Sample.NONE : r.sampleAmong(segs, wx, z);
                cores[dz * w + dx] = core(wd, spec, wx, z, surface, s);
            }
        }
        return new Region(bx0, bz0, bw, bh, cores);
    }
}
