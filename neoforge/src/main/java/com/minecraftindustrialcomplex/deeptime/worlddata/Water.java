// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

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

    /** A column's kind, surface, level, top and bed before its neighbours are looked at. */
    public record Core(Kind kind, double surface, int ground, double level, int top, int bed, Rivers.Sample rivers) {}

    /**
     * One column's water (Rust {@code WaterColumn}); boxed fields are null where Rust has None.
     */
    public record Column(Kind kind, double surface, int ground, Double level, Integer top, Integer bed,
                         Integer minGround, Integer maxGround, Integer sealFrom, boolean riverBiome, Rivers.Hit river) {}

    /** The 4-neighbour offsets, in the order both readers visit them. */
    public static final int[][] NEIGHBOURS = {{1, 0}, {-1, 0}, {0, 1}, {0, -1}};

    /** ⌈h⌉ − 1. */
    public static int topBlock(double h) {
        long t = (long) Math.ceil(h) - 1;
        return (int) Math.max(Integer.MIN_VALUE, Math.min(Integer.MAX_VALUE, t));
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

    /** Steps 1–4 for canonical column (wx, z) given its surface and channel sample. */
    public static Core core(WorldData wd, Spec spec, long wx, long z, double surface, Rivers.Sample rivers) {
        int ground = Double.isFinite(surface) ? Math.max(spec.minY() - 1, Math.min(spec.maxY(), topBlock(surface))) : spec.minY() - 1;
        double level = wd.has(WorldData.WATER_LEVEL) ? levelAt(wd.layer(WorldData.WATER_LEVEL), wx, z, spec.reach()) : Double.NaN;
        if (Double.isFinite(level)) {
            int t = topBlock(level);
            if (ground < t)
                return new Core(Kind.LAKE, surface, ground, level, t, ground, rivers);
        }
        int seaTop = spec.seaLevel() - 1;
        if (ground < seaTop)
            return new Core(Kind.SEA, surface, ground, level, seaTop, ground, rivers);
        Rivers.Hit h = rivers.hit();
        if (h != null) {
            int t = topBlock(h.w());
            double r = h.d() / h.hw();
            double dp = Math.max(h.depth() * (1.0 - r * r), 1.0);
            int bed = Math.min((int) Math.floor(h.w() - dp), t - 1);
            return new Core(Kind.RIVER, surface, ground, level, t, bed, rivers);
        }
        return new Core(Kind.NONE, surface, ground, level, Integer.MIN_VALUE, Integer.MIN_VALUE, rivers);
    }

    /** Steps 1–4 for column (x, z), any lap. */
    public static Core core(WorldData wd, Spec spec, long x, long z) {
        long wx = wrap(wd, x);
        Rivers r = wd.rivers();
        Rivers.Sample s = r == null ? Rivers.Sample.NONE : r.sample(wx, z);
        return core(wd, spec, wx, z, cellSurface(wd, spec, wx, z), s);
    }

    /** Step 5: the column from its own core and its neighbours' ({@link #NEIGHBOURS} order). */
    public static Column finish(Core own, Core[] n) {
        Double level = Double.isFinite(own.level()) ? own.level() : null;
        Rivers.Sample rs = own.rivers();
        if (own.kind().isWater())
            return new Column(own.kind(), own.surface(), own.ground(), level, own.top(), own.bed(), null, null, null,
                    rs.biome(), rs.hit());
        Integer minGround = null, seal = null;
        for (Core c : n) {
            if (!c.kind().isWater())
                continue;
            int need = c.top() + (c.kind() == Kind.RIVER ? 1 : 0);
            minGround = minGround == null ? need : Math.max(minGround, need);
            int from = c.bed() + 1;
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
        return finish(own, n);
    }

    /**
     * The cores of a block box [x0, x0 + w) × [z0, z0 + h) at once (the game's pass: one gather of
     * the river segments for the box); {@link #at} indexes it. x0 may be any lap.
     */
    public static final class Region {
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

        public Core at(int dx, int dz) {
            return cores[dz * w + dx];
        }

        /** The column at (x0 + dx, z0 + dz), which must not be on the region's edge. */
        public Column column(int dx, int dz) {
            Core[] n = new Core[4];
            for (int k = 0; k < 4; k++)
                n[k] = at(dx + NEIGHBOURS[k][0], dz + NEIGHBOURS[k][1]);
            return finish(at(dx, dz), n);
        }
    }

    /** The cores of block box [x0, x0 + w) × [z0, z0 + h) (x0 any lap). */
    public static Region region(WorldData wd, Spec spec, long x0, long z0, int w, int h) {
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
        return new Region(x0, z0, w, h, cores);
    }
}
