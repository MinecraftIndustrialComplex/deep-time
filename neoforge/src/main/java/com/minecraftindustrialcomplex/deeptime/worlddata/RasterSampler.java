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

    /** The physical value of the cell nearest block (bx, bz). */
    public double nearest(double bx, double bz) {
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
        long i = (long) Math.floor((double) (x - grid.originBlockX()) / grid.r() + 0.5);
        long j = (long) Math.floor((double) (z - grid.originBlockZ()) / grid.r() + 0.5);
        return get(i, j);
    }

    /** Bilinear interpolation of physical values at block (bx, bz); NaN if a corner is nodata. */
    public double bilinear(double bx, double bz) {
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
        double a = Math.abs(v00) + (Math.abs(v10) - Math.abs(v00)) * fu;
        double b = Math.abs(v01) + (Math.abs(v11) - Math.abs(v01)) * fu;
        double m = a + (b - a) * fv;
        double near = fv < 0.5 ? (fu < 0.5 ? v00 : v10) : (fu < 0.5 ? v01 : v11);
        return Math.copySign(m, near);
    }

    /** Horizons in each stack (K); 0 for scalar layers. */
    public int stackDepth() {
        return desc.stackDepth();
    }

    /** Horizon k's top in blocks at cell (i, j) (x wrapped, z clamped or folded); NaN if unused. */
    public double horizonTop(long i, long j, int k) {
        long cell = grid.cell(i, j);
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        short top = tileFor(ci, cj).horizonTop(ci % tile, cj % tile, k);
        return top == Short.MIN_VALUE ? Double.NaN : top * desc.scale() + desc.offset();
    }

    /** Horizon k's raw top (1/16 block) at cell (i, j). */
    public short horizonTopRaw(long i, long j, int k) {
        long cell = grid.cell(i, j);
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        return tileFor(ci, cj).horizonTop(ci % tile, cj % tile, k);
    }

    /** Horizon k's class index at cell (i, j); 255 if unused. */
    public int horizonClass(long i, long j, int k) {
        long cell = grid.cell(i, j);
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        return tileFor(ci, cj).horizonClass(ci % tile, cj % tile, k);
    }

    /** Horizon k's stratigraphic key top at cell (i, j) (format 0.2); 65535 if unused. */
    public int horizonKeyTop(long i, long j, int k) {
        long cell = grid.cell(i, j);
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        return tileFor(ci, cj).horizonKeyTop(ci % tile, cj % tile, k);
    }

    /** Horizon k's stratigraphic key base at cell (i, j) (format 0.2); 65535 if unused. */
    public int horizonKeyBase(long i, long j, int k) {
        long cell = grid.cell(i, j);
        int ci = GridDesc.cellI(cell), cj = GridDesc.cellJ(cell);
        return tileFor(ci, cj).horizonKeyBase(ci % tile, cj % tile, k);
    }
}
