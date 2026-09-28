// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

/**
 * Samples one layer (plan §5.3 {@code RasterRead}): cell lookups that wrap in x and clamp in z,
 * nearest and bilinear sampling in block space. The arithmetic is the Rust
 * {@code deeptime_format::RasterRead}'s, operation for operation, so both sides agree bit for bit
 * ({@code TilePackParityTest}). Thread-safe; a hit on the calling thread's last tile allocates
 * nothing.
 */
public final class RasterSampler {

    private final TilePack pack;
    private final LayerDesc desc;
    private final GridDesc grid;
    private final int tile;
    private final int tilesX;
    /** The calling thread's last tile: consecutive samples of a chunk nearly always share one. */
    private final ThreadLocal<Tile[]> last = ThreadLocal.withInitial(() -> new Tile[1]);

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
        Tile fresh = pack.tile(t);
        slot[0] = fresh;
        return fresh;
    }

    /** The raw value of in-range cell (0 ≤ i < w, 0 ≤ j < h). */
    public double rawCell(int i, int j) {
        return tileFor(i, j).raw(i % tile, j % tile);
    }

    /** The raw value at cell (i, j), x wrapped, z clamped. */
    public double raw(long i, long j) {
        return rawCell(grid.wrapI(i), grid.clampJ(j));
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

    /** Horizons in each stack (K); 0 for scalar layers. */
    public int stackDepth() {
        return desc.stackDepth();
    }

    /** Horizon k's top in blocks at cell (i, j) (x wrapped, z clamped); NaN if unused. */
    public double horizonTop(long i, long j, int k) {
        int ci = grid.wrapI(i), cj = grid.clampJ(j);
        short top = tileFor(ci, cj).horizonTop(ci % tile, cj % tile, k);
        return top == Short.MIN_VALUE ? Double.NaN : top * desc.scale() + desc.offset();
    }

    /** Horizon k's raw top (1/16 block) at cell (i, j). */
    public short horizonTopRaw(long i, long j, int k) {
        int ci = grid.wrapI(i), cj = grid.clampJ(j);
        return tileFor(ci, cj).horizonTop(ci % tile, cj % tile, k);
    }

    /** Horizon k's class index at cell (i, j); 255 if unused. */
    public int horizonClass(long i, long j, int k) {
        int ci = grid.wrapI(i), cj = grid.clampJ(j);
        return tileFor(ci, cj).horizonClass(ci % tile, cj % tile, k);
    }
}
