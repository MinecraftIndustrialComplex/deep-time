// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;

/**
 * A raster in block space, laid out by the world's projection (plan §5.2, FORMAT.md "Grids"). Cell
 * (i, j) is the point sample at block (originBlockX + i·r, originBlockZ + j·r); x wraps with period
 * w·r. Rows past the first or the last clamp ({@link ZEdges#CLAMP}: the simulation's grids and the
 * Lambert-frame synthetic worlds). Every reader samples through {@link #cellI}/{@link #cellJ} (Rust
 * {@code GridDesc::cell}).
 *
 * <p>A cube world's grid ({@link #forCube}, {@link ZEdges#CUBE}) is six planes of n × n cells, one per
 * face in the format's order, stacked down the rows (w = n, h = 6n, no x wrap, origin F0's north-west
 * corner (−s/2, −s/2)); a face's cell (i, j) is the r × r square of net blocks from the face's origin
 * + (i·r, j·r), its value at the square's centre. Its JSON form is {@code {"name", "kind": "cube",
 * "n", "r"}} (FORMAT.md "Cube grids").
 */
public record GridDesc(String name, int w, int h, int r, boolean wrapsX, long originBlockX, long originBlockZ, ZEdges zEdges) {

    /** {@code z_edges}: what a row past the first or the last means. */
    public enum ZEdges {
        CLAMP,
        /** A cube world's six stacked faces: an index off a face resolves through the face adjacency ({@link #faceCell}). */
        CUBE
    }

    /** A clamping grid (the simulation's grids, the Lambert-frame synthetic worlds). */
    public GridDesc(String name, int w, int h, int r, boolean wrapsX, long originBlockX, long originBlockZ) {
        this(name, w, h, r, wrapsX, originBlockX, originBlockZ, ZEdges.CLAMP);
    }

    /**
     * The grid of a cube world of face side {@code s} blocks at {@code r} blocks a cell (n = s/r;
     * Rust {@code GridDesc::for_cube}).
     */
    public static GridDesc forCube(String name, long s, int r) {
        if (r <= 0 || s < 2 || s % 2 != 0 || s % r != 0)
            throw new IllegalArgumentException("cube face side " + s + " is not even or not a multiple of r = " + r);
        int n = Math.toIntExact(s / r);
        return new GridDesc(name, n, 6 * n, r, false, -(s / 2), -(s / 2), ZEdges.CUBE);
    }

    static GridDesc fromJson(JsonObject o) {
        if (o.has("kind") && !o.get("kind").isJsonNull()) {
            String kind = o.get("kind").getAsString();
            if (!"cube".equals(kind))
                throw new IllegalArgumentException("grid " + o.get("name").getAsString() + ": unknown kind " + kind);
            long n = o.get("n").getAsLong(), r = o.get("r").getAsLong();
            if (n <= 0 || r <= 0 || (n * r) % 2 != 0)
                throw new IllegalArgumentException("cube grid " + o.get("name").getAsString() + ": n " + n + " × r " + r
                        + " is not an even face side");
            return forCube(o.get("name").getAsString(), n * r, (int) r);
        }
        ZEdges z = ZEdges.CLAMP;
        if (o.has("z_edges")) {
            String s = o.get("z_edges").getAsString();
            z = switch (s) {
                case "clamp" -> ZEdges.CLAMP;
                default -> throw new IllegalArgumentException("unknown z_edges '" + s + "'");
            };
        }
        return new GridDesc(
                o.get("name").getAsString(),
                o.get("w").getAsInt(),
                o.get("h").getAsInt(),
                o.get("r").getAsInt(),
                o.get("wraps_x").getAsBoolean(),
                o.get("origin_block_x").getAsLong(),
                o.get("origin_block_z").getAsLong(),
                z);
    }

    /** Whether this is a cube world's grid. */
    public boolean isCube() {
        return zEdges == ZEdges.CUBE;
    }

    /** A cube grid's face side in blocks, s = n·r. */
    public long faceBlocks() {
        return (long) w * r;
    }

    /** A cube grid's net ({@link CubeLayout#shared}). */
    public CubeLayout cube() {
        return CubeLayout.shared(faceBlocks());
    }

    /** The x period w·r (C on a Lambert-frame world, 4s on a cube grid). */
    public long circumference() {
        return isCube() ? 4L * w * r : (long) w * r;
    }

    /**
     * The tile edge a layer on this grid is written with: 256, or on a cube grid the largest divisor of
     * n that divides 256 too, so no tile holds two faces (Rust {@code GridDesc::tile}).
     */
    public int tile() {
        if (!isCube())
            return 256;
        int a = w, b = 256;
        while (b != 0) {
            int t = a % b;
            a = b;
            b = t;
        }
        return a;
    }

    /** Wraps a column index into [0, w). */
    public int wrapI(long i) {
        return (int) Math.floorMod(i, (long) w);
    }

    /** Clamps a row index into [0, h − 1]. */
    public int clampJ(long j) {
        return (int) Math.max(0, Math.min(h - 1L, j));
    }

    /**
     * The stored cell of any index pair, packed: {@code (long) row << 32 | column}. Columns wrap and
     * rows clamp; on a cube grid the row's plane (clamped to the six) is the face and the local cell
     * resolves off it ({@link #faceCell}). Use {@link #cellI}/{@link #cellJ} to unpack.
     */
    public long cell(long i, long j) {
        if (zEdges == ZEdges.CUBE) {
            long n = w;
            int k = (int) Math.max(0, Math.min(5, Math.floorDiv(j, Math.max(n, 1))));
            return faceCell(CubeLayout.Face.fromIndex(k), i, j - k * n);
        }
        return ((long) clampJ(j) << 32) | wrapI(i);
    }

    /**
     * The stored cell of a cube grid's face {@code face}, local cell (i, j), resolved off the face
     * ({@link #resolveFaceCell}), packed as {@link #cell} packs (Rust {@code GridDesc::face_cell}).
     */
    public long faceCell(CubeLayout.Face face, long i, long j) {
        long n = w;
        long[] c = resolveFaceCell(face, n, i, j);
        long row = c[0] * n + c[2];
        return (row << 32) | c[1];
    }

    /**
     * The cell a face's local cell (i, j) of an n × n face grid is, {face index, i, j}: itself on the
     * face; the neighbour's beyond one side ({@link CubeLayout#cellOn}); beyond two sides at once (the
     * corner squares where three faces meet) the neighbour across the side it is further out of, ties
     * east or west, the other index clamped onto the face; more than a face out, clamped to a face out.
     * Rust {@code resolve_face_cell}, line for line.
     */
    public static long[] resolveFaceCell(CubeLayout.Face face, long n, long i, long j) {
        i = Math.max(-n, Math.min(2 * n - 1, i));
        j = Math.max(-n, Math.min(2 * n - 1, j));
        long oi = out(i, n), oj = out(j, n);
        if (oi > 0 && oj > 0) {
            if (oi >= oj)
                j = Math.max(0, Math.min(n - 1, j));
            else
                i = Math.max(0, Math.min(n - 1, i));
        }
        CubeLayout.CellOn on = CubeLayout.cellOn(face, n, i, j);
        if (on != null)
            return new long[]{on.face().index(), Math.max(0, Math.min(n - 1, on.i())), Math.max(0, Math.min(n - 1, on.j()))};
        return new long[]{face.index(), Math.max(0, Math.min(n - 1, i)), Math.max(0, Math.min(n - 1, j))};
    }

    private static long out(long k, long n) {
        if (k < 0)
            return -k;
        if (k >= n)
            return k - n + 1;
        return 0;
    }

    /** The face of a cube grid's stacked row {@code j} (in range). */
    public CubeLayout.Face faceOfRow(long j) {
        return CubeLayout.Face.fromIndex((int) (j / w));
    }

    /**
     * Where the net point (x, z) (any point: folded onto the planet) lies on a cube grid: its face and
     * fractional local cell (cell (i, j)'s centre is at (i, j)), or null past the net's corners (Rust
     * {@code GridDesc::cube_spot}).
     */
    public CubeSpot cubeSpot(double x, double z) {
        CubeLayout cube = cube();
        CubeLayout.PointOnFace p = cube.foldPoint(x, z);
        if (p == null)
            return null;
        double rr = r;
        return new CubeSpot(p.face(), (p.x() - (double) cube.faceOriginX(p.face())) / rr - 0.5,
                (p.z() - (double) cube.faceOriginZ(p.face())) / rr - 0.5);
    }

    /** {@link #cubeSpot} of block (bx, bz)'s centre (bx + ½, bz + ½): every sampler takes blocks this way on a cube. */
    public CubeSpot cubeSpotOfBlock(double bx, double bz) {
        return cubeSpot(bx + 0.5, bz + 0.5);
    }

    /** The net point at the centre of a cube grid's stacked cell (i, j) (in range), {x, z} (Rust {@code GridDesc::cube_point}). */
    public double[] cubePoint(long i, long j) {
        CubeLayout.Face face = faceOfRow(j);
        long b = j - (long) face.index() * w;
        CubeLayout cube = cube();
        double rr = r;
        return new double[]{(double) cube.faceOriginX(face) + ((double) i + 0.5) * rr,
                (double) cube.faceOriginZ(face) + ((double) b + 0.5) * rr};
    }

    /** Where a reader of a cube grid stands: the face, and the fractional local cell (a, b). */
    public record CubeSpot(CubeLayout.Face face, double a, double b) {}

    /** The column of a packed {@link #cell}. */
    public static int cellI(long cell) {
        return (int) (cell & 0xFFFF_FFFFL);
    }

    /** The row of a packed {@link #cell}. */
    public static int cellJ(long cell) {
        return (int) (cell >>> 32);
    }

    /** Block x of column i. */
    public long blockX(long i) {
        return originBlockX + i * r;
    }

    /** Block z of row j. */
    public long blockZ(long j) {
        return originBlockZ + j * r;
    }
}
