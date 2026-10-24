// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * Samples one layer (plan §5.3 {@code RasterRead}): cell lookups that wrap in x and clamp in z,
 * nearest and bilinear sampling in block space. The arithmetic is the Rust
 * {@code deeptime_format::RasterRead}'s, operation for operation, so both sides agree bit for bit
 * ({@code TilePackParityTest}). Thread-safe; a hit on one of the calling thread's last two tiles
 * allocates nothing.
 */
public final class RasterSampler {

    private final TilePack pack;
    private final LayerDesc desc;
    private final GridDesc grid;
    private final int tile;
    private final int tilesX;
    /**
     * The calling thread's last two tiles, the latest first: consecutive samples of a chunk nearly
     * always share one, and a chunk on a tile's edge goes back and forth between two (the column
     * rule's neighbourhood, and the source columns of a fault's hanging wall).
     */
    private final ThreadLocal<Tile[]> last = ThreadLocal.withInitial(() -> new Tile[2]);

    public RasterSampler(TilePack pack) {
        this.pack = pack;
        this.desc = pack.desc();
        this.grid = desc.grid();
        this.tile = desc.tile();
        this.tilesX = desc.tilesX();
    }

    public LayerDesc desc() {
        return desc;
    }

    public TilePack pack() {
        return pack;
    }

    private Tile tileFor(int i, int j) {
        int t = (j / tile) * tilesX + (i / tile);
        Tile[] slot = last.get();
        Tile hit = slot[0];
        if (hit != null && hit.index == t) return hit;
        Tile other = slot[1];
        if (other != null && other.index == t) {
            slot[1] = hit;
            slot[0] = other;
            return other;
        }
        Tile fresh = pack.tile(t);
        slot[1] = hit;
        slot[0] = fresh;
        return fresh;
    }

    /** The raw value of in-range cell (0 ≤ i < w, 0 ≤ j < h). */
    public double rawCell(int i, int j) {
        return tileFor(i, j).raw(i % tile, j % tile);
    }

    /** The raw value at cell (i, j), x wrapped, z clamped or folded through the edges ({@link GridDesc#cell}). */
    public double raw(long i, long j) {
        long c = grid.cell(i, j);
        return rawCell(GridDesc.cellI(c), GridDesc.cellJ(c));
    }

    /** The physical value at cell (i, j), x wrapped, z clamped; NaN for nodata. */
    public double get(long i, long j) {
        return desc.physical(raw(i, j));
    }

    /** The physical value of a cube grid's face {@code face}, local cell (i, j) resolved off the face (Rust {@code get_face}). */
    public double getFace(CubeLayout.Face face, long i, long j) {
        long c = grid.faceCell(face, i, j);
        return desc.physical(rawCell(GridDesc.cellI(c), GridDesc.cellJ(c)));
    }

    /** The physical value of the cell nearest block (bx, bz); on a cube grid the cell under the block's centre (NaN past the net's corners). */
    public double nearest(double bx, double bz) {
        if (grid.isCube()) {
            GridDesc.CubeSpot s = grid.cubeSpotOfBlock(bx, bz);
            return s == null ? Double.NaN : getFace(s.face(), (long) Math.floor(s.a() + 0.5), (long) Math.floor(s.b() + 0.5));
        }
        double u = (bx - (double) grid.originBlockX()) / (double) grid.r();
        double v = (bz - (double) grid.originBlockZ()) / (double) grid.r();
        return get((long) Math.floor(u + 0.5), (long) Math.floor(v + 0.5));
    }

    /**
     * The column rule's cubic B-spline of the physical values at block column (x, z) (FORMAT.md, "The lift"): the
     * 4 × 4 cells and weights {@link Columns#stack} reads a column's horizon tops with, summed as an offset from cell
     * (i0, j0)'s value as the tops are. Rust {@code RasterRead::column_spline}, the same operations.
     */
    public double columnSpline(long x, long z) {
        if (grid.isCube()) {
            GridDesc.CubeSpot s = grid.cubeSpotOfBlock((double) x, (double) z);
            if (s == null)
                return Double.NaN;
            long i0 = (long) Math.floor(s.a()), j0 = (long) Math.floor(s.b());
            double[] bu = Columns.bspline(s.a() - (double) i0), bv = Columns.bspline(s.b() - (double) j0);
            double base = getFace(s.face(), i0, j0);
            double sum = 0.0;
            for (int b = 0; b < 4; b++)
                for (int a = 0; a < 4; a++) {
                    double w = bu[a] * bv[b];
                    sum += w * (getFace(s.face(), i0 - 1 + a, j0 - 1 + b) - base);
                }
            return base + sum;
        }
        long c = (long) grid.w() * grid.r();
        long wx = Math.floorMod(x + c / 2, c) - c / 2;
        double u = (double) (wx - grid.originBlockX()) / grid.r();
        double v = (double) (z - grid.originBlockZ()) / grid.r();
        long i0 = (long) Math.floor(u), j0 = (long) Math.floor(v);
        double[] bu = Columns.bspline(u - i0), bv = Columns.bspline(v - j0);
        double base = get(i0, j0);
        double sum = 0.0;
        for (int b = 0; b < 4; b++)
            for (int a = 0; a < 4; a++) {
                double w = bu[a] * bv[b];
                sum += w * (get(i0 - 1 + a, j0 - 1 + b) - base);
            }
        return base + sum;
    }

    /**
     * The physical value of the strata cell {@link FarColumn#cellStack} reads at block column (x, z): the nearest
     * cell, x not folded first (the cell index wraps).
     */
    public double cellValue(long x, long z) {
        if (grid.isCube())
            return nearest((double) x, (double) z);
        long i = (long) Math.floor((double) (x - grid.originBlockX()) / grid.r() + 0.5);
        long j = (long) Math.floor((double) (z - grid.originBlockZ()) / grid.r() + 0.5);
        return get(i, j);
    }

    /** Bilinear interpolation of physical values at block (bx, bz); NaN if a corner is nodata. */
    public double bilinear(double bx, double bz) {
        if (grid.isCube()) {
            GridDesc.CubeSpot s = grid.cubeSpotOfBlock(bx, bz);
            if (s == null)
                return Double.NaN;
            double fi = Math.floor(s.a()), fj = Math.floor(s.b());
            double fu = s.a() - fi, fv = s.b() - fj;
            long i0 = (long) fi, j0 = (long) fj;
            double v00 = getFace(s.face(), i0, j0);
            double v10 = getFace(s.face(), i0 + 1, j0);
            double v01 = getFace(s.face(), i0, j0 + 1);
            double v11 = getFace(s.face(), i0 + 1, j0 + 1);
            double a = v00 + (v10 - v00) * fu;
            double b = v01 + (v11 - v01) * fu;
            return a + (b - a) * fv;
        }
        double u = (bx - (double) grid.originBlockX()) / (double) grid.r();
        double v = (bz - (double) grid.originBlockZ()) / (double) grid.r();
        double fi = Math.floor(u);
        double fj = Math.floor(v);
        double fu = u - fi;
        double fv = v - fj;
        long i0 = (long) fi, j0 = (long) fj;
        double v00 = get(i0, j0);
        double v10 = get(i0 + 1, j0);
        double v01 = get(i0, j0 + 1);
        double v11 = get(i0 + 1, j0 + 1);
        double a = v00 + (v10 - v00) * fu;
        double b = v01 + (v11 - v01) * fu;
        return a + (b - a) * fv;
    }

    /**
     * A cover layer's value at block (bx, bz) ({@code climate/ice_cover}, FORMAT.md "The ice cover"): {@link #coverOf}
     * with the {@link #nearest} cell deciding whether the block is covered and the four cells of the bilinear blend its
     * depth, so the cover's edge lies half-way between a covered cell and a bare one, where the nearest-cell Köppen class
     * changes too. The arithmetic of {@code RasterRead::cover} in deeptime-format (ledger BIO-159); a plain bilinear read
     * rounded to blocks covered every block where the blend passed half a block, most of a cell past a thick margin.
     */
    public double cover(double bx, double bz) {
        if (grid.isCube()) {
            GridDesc.CubeSpot s = grid.cubeSpotOfBlock(bx, bz);
            if (s == null)
                return 0;
            double fi = Math.floor(s.a()), fj = Math.floor(s.b());
            double fu = s.a() - fi, fv = s.b() - fj;
            long i0 = (long) fi, j0 = (long) fj;
            CubeLayout.Face f = s.face();
            return coverOf(nearest(bx, bz),
                    new double[] {getFace(f, i0, j0), getFace(f, i0 + 1, j0), getFace(f, i0, j0 + 1), getFace(f, i0 + 1, j0 + 1)},
                    new double[] {(1 - fu) * (1 - fv), fu * (1 - fv), (1 - fu) * fv, fu * fv});
        }
        double u = (bx - (double) grid.originBlockX()) / (double) grid.r();
        double v = (bz - (double) grid.originBlockZ()) / (double) grid.r();
        double fi = Math.floor(u);
        double fj = Math.floor(v);
        double fu = u - fi;
        double fv = v - fj;
        long i0 = (long) fi, j0 = (long) fj;
        return coverOf(nearest(bx, bz), new double[] {get(i0, j0), get(i0 + 1, j0), get(i0, j0 + 1), get(i0 + 1, j0 + 1)},
                new double[] {(1 - fu) * (1 - fv), fu * (1 - fv), (1 - fu) * fv, fu * fv});
    }

    /**
     * The cover rule: covered where {@code nearest}, the nearest cell's value, is above 0 (NaN is none), and then at the
     * weighted mean of the samples with a value above 0, or at {@code nearest} where none is; 0 elsewhere. The nearest
     * cell decides the extent as it decides a block's Köppen class. The arithmetic of {@code cover_of} in
     * deeptime-format (ledger BIO-159).
     */
    public static double coverOf(double nearest, double[] values, double[] weights) {
        if (!(nearest > 0))
            return 0;
        double held = 0, sum = 0;
        for (int k = 0; k < values.length; k++) {
            if (values[k] > 0) {
                held += weights[k];
                sum += weights[k] * values[k];
            }
        }
        return held > 0 ? sum / held : nearest;
    }

    /**
     * An odd field at block (bx, bz): the magnitude bilinear over the corners' absolute values, the
     * sign of the nearest corner. A sign change then is a sharp edge instead of a ramp through zero
     * (P1-09: the weirdness input, whose sign picks vanilla's variant biomes and whose |w| near 0 is
     * the river-valley slice).
     */
    public double bilinearOdd(double bx, double bz) {
        double fu, fv, v00, v10, v01, v11;
        if (grid.isCube()) {
            GridDesc.CubeSpot s = grid.cubeSpotOfBlock(bx, bz);
            if (s == null)
                return Double.NaN;
            double fi = Math.floor(s.a()), fj = Math.floor(s.b());
            fu = s.a() - fi;
            fv = s.b() - fj;
            long i0 = (long) fi, j0 = (long) fj;
            v00 = getFace(s.face(), i0, j0);
            v10 = getFace(s.face(), i0 + 1, j0);
            v01 = getFace(s.face(), i0, j0 + 1);
            v11 = getFace(s.face(), i0 + 1, j0 + 1);
        } else {
            double u = (bx - (double) grid.originBlockX()) / (double) grid.r();
            double v = (bz - (double) grid.originBlockZ()) / (double) grid.r();
            double fi = Math.floor(u);
            double fj = Math.floor(v);
            fu = u - fi;
            fv = v - fj;
            long i0 = (long) fi, j0 = (long) fj;
            v00 = get(i0, j0);
            v10 = get(i0 + 1, j0);
            v01 = get(i0, j0 + 1);
            v11 = get(i0 + 1, j0 + 1);
        }
        double a = Math.abs(v00) + (Math.abs(v10) - Math.abs(v00)) * fu;
        double b = Math.abs(v01) + (Math.abs(v11) - Math.abs(v01)) * fu;
        double m = a + (b - a) * fv;
        double near = fv < 0.5 ? (fu < 0.5 ? v00 : v10) : (fu < 0.5 ? v01 : v11);
        return Math.copySign(m, near);
    }

    /**
     * The physical values of the four cells a {@link #bilinear} read at block (bx, bz) blends, {v00, v10, v01, v11}
     * (on a cube grid on the block centre's face, resolved across its seams; NaN past the net's corners).
     */
    public double[] bilinearCells(double bx, double bz) {
        if (grid.isCube()) {
            GridDesc.CubeSpot s = grid.cubeSpotOfBlock(bx, bz);
            if (s == null)
                return new double[]{Double.NaN, Double.NaN, Double.NaN, Double.NaN};
            long i0 = (long) Math.floor(s.a()), j0 = (long) Math.floor(s.b());
            CubeLayout.Face f = s.face();
            return new double[]{getFace(f, i0, j0), getFace(f, i0 + 1, j0), getFace(f, i0, j0 + 1), getFace(f, i0 + 1, j0 + 1)};
        }
        double u = (bx - (double) grid.originBlockX()) / (double) grid.r();
        double v = (bz - (double) grid.originBlockZ()) / (double) grid.r();
        long i0 = (long) Math.floor(u), j0 = (long) Math.floor(v);
        return new double[]{get(i0, j0), get(i0 + 1, j0), get(i0, j0 + 1), get(i0 + 1, j0 + 1)};
    }

    /**
     * Whether any cell a read in the block box [x0, x1] × [z0, z1] can touch holds a value {@code test} accepts: the
     * chunk passes' quick "is there anything here" checks. On the polar grids the cells floorDiv(x0 − ox, r) to
     * floorDiv(x1 − ox, r) + 1 (and so in z); on a cube grid the box is carried by its centre's fold onto the centre's
     * face and its cells, one more each way, read there across the seams ({@link #getFace}); false past the net's
     * corners.
     */
    public boolean anyInBox(long x0, long z0, long x1, long z1, java.util.function.DoublePredicate test) {
        if (grid.isCube()) {
            long[] box = cubeCellBox(x0, z0, x1, z1, 0);
            if (box == null)
                return false;
            CubeLayout.Face f = CubeLayout.Face.fromIndex((int) box[0]);
            for (long i = box[1]; i <= box[2]; i++)
                for (long j = box[3]; j <= box[4]; j++)
                    if (test.test(getFace(f, i, j)))
                        return true;
            return false;
        }
        int r = grid.r();
        long i0 = Math.floorDiv(x0 - grid.originBlockX(), r), i1 = Math.floorDiv(x1 - grid.originBlockX(), r) + 1;
        long j0 = Math.floorDiv(z0 - grid.originBlockZ(), r), j1 = Math.floorDiv(z1 - grid.originBlockZ(), r) + 1;
        for (long i = i0; i <= i1; i++)
            for (long j = j0; j <= j1; j++)
                if (test.test(get(i, j)))
                    return true;
        return false;
    }

    /**
     * A cube grid's face cells under the block box [x0, x1] × [z0, z1] carried by the motion of its centre's fold,
     * widened by {@code extra} cells: {face index, i0, i1, j0, j1} in the face's local cells (one cell past the box's
     * corners each way, as a bilinear read reaches); null past the net's corners.
     */
    public long[] cubeCellBox(long x0, long z0, long x1, long z1, long extra) {
        CubeLayout cube = grid.cube();
        double cx = ((double) x0 + (double) x1 + 1.0) * 0.5, cz = ((double) z0 + (double) z1 + 1.0) * 0.5;
        CubeLayout.PointOnFace p = cube.foldPoint(cx, cz);
        if (p == null)
            return null;
        double[] c = p.iso().point(cx, cz);
        double shift = p.x() - c[0];
        double[] a = p.iso().point((double) x0, (double) z0), b = p.iso().point((double) x1 + 1.0, (double) z1 + 1.0);
        double ox = (double) cube.faceOriginX(p.face()) - shift, oz = (double) cube.faceOriginZ(p.face());
        double rr = grid.r();
        double amin = (Math.min(a[0], b[0]) - ox) / rr - 0.5, amax = (Math.max(a[0], b[0]) - ox) / rr - 0.5;
        double bmin = (Math.min(a[1], b[1]) - oz) / rr - 0.5, bmax = (Math.max(a[1], b[1]) - oz) / rr - 0.5;
        return new long[]{p.face().index(), (long) Math.floor(amin) - extra, (long) Math.floor(amax) + 1 + extra,
                (long) Math.floor(bmin) - extra, (long) Math.floor(bmax) + 1 + extra};
    }

    /** Horizons in each stack (K); 0 for scalar layers. */
    public int stackDepth() {
        return desc.stackDepth();
    }

    /** Horizon k's top in blocks at cell (i, j) (x wrapped, z clamped or folded); NaN if unused. */
    public double horizonTop(long i, long j, int k) {
        return horizonTopAt(grid.cell(i, j), k);
    }

    /** Horizon k's raw top (1/16 block) at cell (i, j). */
    public short horizonTopRaw(long i, long j, int k) {
        return horizonTopRawAt(grid.cell(i, j), k);
    }

    /** Horizon k's class index at cell (i, j); 255 if unused. */
    public int horizonClass(long i, long j, int k) {
        return horizonClassAt(grid.cell(i, j), k);
    }

    /** Horizon k's stratigraphic key top at cell (i, j) (format 0.2); 65535 if unused. */
    public int horizonKeyTop(long i, long j, int k) {
        return horizonKeyTopAt(grid.cell(i, j), k);
    }

    /** Horizon k's stratigraphic key base at cell (i, j) (format 0.2); 65535 if unused. */
    public int horizonKeyBase(long i, long j, int k) {
        return horizonKeyBaseAt(grid.cell(i, j), k);
    }

    /**
     * The stored cell a stack reader reads for cell (i, j) of {@code face} (a cube grid: the face's local cell
     * resolved off it, {@link GridDesc#faceCell}) or of the grid (face null: {@link GridDesc#cell}), packed. Rust
     * {@code strata::spline_cell_at}.
     */
    public long cellAt(CubeLayout.Face face, long i, long j) {
        return face != null ? grid.faceCell(face, i, j) : grid.cell(i, j);
    }

    /** Horizon k's top in blocks at a packed stored cell ({@link #cellAt}); NaN if unused. */
    public double horizonTopAt(long cell, int k) {
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        short top = tileFor(ci, cj).horizonTop(ci % tile, cj % tile, k);
        return top == Short.MIN_VALUE ? Double.NaN : top * desc.scale() + desc.offset();
    }

    /** Horizon k's raw top (1/16 block) at a packed stored cell. */
    public short horizonTopRawAt(long cell, int k) {
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        return tileFor(ci, cj).horizonTop(ci % tile, cj % tile, k);
    }

    /** Horizon k's class index at a packed stored cell; 255 if unused. */
    public int horizonClassAt(long cell, int k) {
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        return tileFor(ci, cj).horizonClass(ci % tile, cj % tile, k);
    }

    /** Horizon k's stratigraphic key top at a packed stored cell; 65535 if unused. */
    public int horizonKeyTopAt(long cell, int k) {
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        return tileFor(ci, cj).horizonKeyTop(ci % tile, cj % tile, k);
    }

    /** Horizon k's stratigraphic key base at a packed stored cell; 65535 if unused. */
    public int horizonKeyBaseAt(long cell, int k) {
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        return tileFor(ci, cj).horizonKeyBase(ci % tile, cj % tile, k);
    }
}
