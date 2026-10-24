// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.minecraftindustrialcomplex.deeptime.wrapengine.CellIsometry;

import java.util.List;

/**
 * The cube world's geometry: six square faces of side s in a cross-shaped net (docs/plans/
 * cube-world-plan.md §1.2, work item W2). Pure integer and plane geometry: which face a block is on,
 * how the net's edges are glued, the corners and the face adjacency. The twin of Rust's
 * {@code deeptime_format::cube} (bit for bit, checked against the cube section of the parity
 * fixture by {@code ProjectionParityTest}); no latitude here, the map is {@link CubeMap} and the
 * turn {@link CubeTurn}.
 *
 * <p>+x is east and −z north. The window is x ∈ [−3s/2, 5s/2), z ∈ [−3s/2, 3s/2); x wraps with
 * period 4s. F3 is [−3s/2, −s/2) × [−s/2, s/2), F0 [−s/2, s/2), F1 [s/2, 3s/2), F2 [3s/2, 5s/2) (all
 * with z ∈ [−s/2, s/2)), N above F0 and S below it. Every face is drawn without a turn, so its local
 * axes are the net's: cell (i, j) is the net block (x0 + i, z0 + j) from the face's north-west corner
 * and the local point (u, v) ∈ [−1, 1]² is ((x − cx)/(s/2), (z − cz)/(s/2)).
 *
 * <p>Five cube edges are joined in the net; the other seven are cut into 14 net edges glued by an
 * {@link Iso} (the wrap engine's {@link CellIsometry}, with 64-bit translations): the slide of 4s,
 * four quarter turns about F0's corners and two half turns. {@link #fold} takes any block to the
 * planet block it shows.
 */
public final class CubeLayout {

    /**
     * The face side is a multiple of this many blocks: every grid's r (up to 256) divides s, the
     * east-west period 4s is a multiple of 512, and every turn pivot (s/2 from a face centre) sits on
     * a chunk corner.
     */
    public static final int FACE_QUANTUM = 256; // ledger: SCL-100 (game-choice)

    /**
     * The face side s for a planet {@code circumference} blocks round (an unsigned 32-bit count as in
     * Rust): the six faces' 6s² blocks cover the planet's 4πR² (R = C/2π, so s = C/√(6π)), rounded to
     * the nearest multiple of {@link #FACE_QUANTUM} (game choice, ledger SCL-100). 4k: 1,024; 16k:
     * 3,840; 32k: 7,424.
     */
    public static int faceFor(int circumference) {
        double exact = (double) Integer.toUnsignedLong(circumference) / Math.sqrt(6.0 * Math.PI);
        double q = FACE_QUANTUM; // ledger: SCL-100
        return (int) (long) (Math.max(roundHalfAway(exact / q), 1.0) * q);
    }

    /** Rust's {@code f64::round}: half away from zero, exact. */
    static double roundHalfAway(double x) {
        double a = Math.abs(x);
        double r = Math.floor(a);
        if (a - r >= 0.5)
            r += 1.0;
        return x < 0.0 ? -r : r;
    }

    /** The point turn of (x, z) by k clockwise quarter turns, on integers. */
    private static long turnX(int k, long x, long z) {
        return switch (k) {
            case 1 -> -z;
            case 2 -> -x;
            case 3 -> z;
            default -> x;
        };
    }

    private static long turnZ(int k, long x, long z) {
        return switch (k) {
            case 1 -> x;
            case 2 -> -z;
            case 3 -> -x;
            default -> z;
        };
    }

    // ------------------------------------------------------------------------------------------
    // Faces, sides
    // ------------------------------------------------------------------------------------------

    /** A face of the cube, in the format's order (rasters are stored face by face in this order). */
    public enum Face {
        F0, F1, F2, F3, N, S;

        private static final Face[] ALL = values();

        private static final double[][][] FRAMES = frames();

        private static double[][][] frames() {
            double[] x = {1.0, 0.0, 0.0}, y = {0.0, 1.0, 0.0}, z = {0.0, 0.0, 1.0};
            double[] mx = {-1.0, 0.0, 0.0}, my = {0.0, -1.0, 0.0}, mz = {0.0, 0.0, -1.0};
            return new double[][][]{
                    {x, y, mz},  // F0
                    {y, mx, mz}, // F1
                    {mx, my, mz}, // F2
                    {my, x, mz}, // F3
                    {z, y, x},   // N
                    {mz, y, mx}, // S
            };
        }

        /** The six faces in the format's order. */
        public static Face[] all() {
            return ALL.clone();
        }

        /** The face's index in the format's order. */
        public int index() {
            return ordinal();
        }

        /** The face of index {@code i} in the format's order, or null. */
        public static Face fromIndex(int i) {
            return i >= 0 && i < ALL.length ? ALL[i] : null;
        }

        /** The face's centre in the net, x in units of s. */
        public int centreUnitsX() {
            return switch (this) {
                case F1 -> 1;
                case F2 -> 2;
                case F3 -> -1;
                default -> 0;
            };
        }

        /** The face's centre in the net, z in units of s. */
        public int centreUnitsZ() {
            return switch (this) {
                case N -> -1;
                case S -> 1;
                default -> 0;
            };
        }

        /**
         * The face's frame in the cube frame: the unit vector of its centre, then e_u and e_v, the
         * cube-frame directions of local +u (net +x) and +v (net +z) at the centre. A copy.
         */
        public double[][] frame() {
            double[][] f = FRAMES[ordinal()];
            return new double[][]{f[0].clone(), f[1].clone(), f[2].clone()};
        }

        /** The frame without the copy (the map's evaluator); do not modify. */
        double[][] frameRef() {
            return FRAMES[ordinal()];
        }

        /**
         * The face whose centre is the largest component of the cube-frame vector {@code p} (any
         * length &gt; 0): ties go to x, then y, then z, and a zero component counts as positive.
         */
        public static Face ofVector(double[] p) {
            double ax = Math.abs(p[0]), ay = Math.abs(p[1]), az = Math.abs(p[2]);
            if (ax >= ay && ax >= az)
                return p[0] >= 0.0 ? F0 : F2;
            if (ay >= az)
                return p[1] >= 0.0 ? F1 : F3;
            return p[2] >= 0.0 ? N : S;
        }

        /** What lies across side {@code side} of this face ({@link #ADJACENCY}). */
        public Across across(FaceSide side) {
            return ADJACENCY[ordinal()][side.index()];
        }
    }

    /**
     * A side of a face in its own local frame: east is local +u (i = n), south +v (j = n). The order
     * is clockwise on the map.
     */
    public enum FaceSide {
        EAST("east"), SOUTH("south"), WEST("west"), NORTH("north");

        private static final FaceSide[] ALL = values();

        private final String id;

        FaceSide(String id) {
            this.id = id;
        }

        public static FaceSide[] all() {
            return ALL.clone();
        }

        public int index() {
            return ordinal();
        }

        /** {@code east}, {@code south}, {@code west}, {@code north}. */
        public String id() {
            return id;
        }
    }

    /** The axis an edge's normal runs along: x for an east or west edge. */
    public enum Axis {
        X, Z
    }

    /** Which side of an edge line is beyond the world. */
    public enum Side {
        /** Beyond is v &lt; coord (a west or north edge). */
        MIN,
        /** Beyond is v ≥ coord (an east or south edge). */
        MAX
    }

    // ------------------------------------------------------------------------------------------
    // Iso
    // ------------------------------------------------------------------------------------------

    /**
     * A motion of the block grid that keeps blocks on blocks: {@code k} clockwise quarter turns (seen
     * from above with north up, Minecraft's sense) about the origin, then the translation (tx, tz).
     * The wrap engine's {@link CellIsometry} with 64-bit translations, the format's {@code iso}
     * triple {@code [k, tx, tz]}. On block cells (the block (x, z) spans [x, x+1) × [z, z+1)) the
     * turns are k = 1: (x, z) → (−z − 1, x), k = 2: (−x − 1, −z − 1), k = 3: (z, −x − 1); on points
     * k = 1: (−z, x), k = 2: (−x, −z), k = 3: (z, −x).
     *
     * @param k clockwise quarter turns, taken mod 4
     */
    public record Iso(int k, long tx, long tz) {

        public static final Iso IDENTITY = new Iso(0, 0, 0);

        public Iso {
            k = Math.floorMod(k, 4);
        }

        public static Iso translation(long dx, long dz) {
            return new Iso(0, dx, dz);
        }

        /** The half turn about the block corner (a, b). */
        public static Iso halfTurnAbout(long a, long b) {
            return new Iso(2, 2 * a, 2 * b);
        }

        /** The quarter turn (k = 1 clockwise, 3 counter-clockwise) about the block corner (a, b). */
        public static Iso quarterTurnAbout(int k, long a, long b) {
            int kk = Math.floorMod(k, 4);
            return new Iso(k, a - turnX(kk, a, b), b - turnZ(kk, a, b));
        }

        /** x of the block cell (x, z) moved. */
        public long cellX(long x, long z) {
            long rx = switch (k) {
                case 1 -> -z - 1;
                case 2 -> -x - 1;
                case 3 -> z;
                default -> x;
            };
            return rx + tx;
        }

        /** z of the block cell (x, z) moved. */
        public long cellZ(long x, long z) {
            long rz = switch (k) {
                case 1 -> x;
                case 2 -> -z - 1;
                case 3 -> -x - 1;
                default -> z;
            };
            return rz + tz;
        }

        /** The block cell (x, z) moved: {x', z'}. */
        public long[] cell(long x, long z) {
            return new long[]{cellX(x, z), cellZ(x, z)};
        }

        /** The point (x, z) moved: {x', z'}. */
        public double[] point(double x, double z) {
            double[] r = vector(x, z);
            return new double[]{r[0] + (double) tx, r[1] + (double) tz};
        }

        /** A direction (dx, dz) turned (no translation): {dx', dz'}. */
        public double[] vector(double dx, double dz) {
            return switch (k) {
                case 1 -> new double[]{-dz, dx};
                case 2 -> new double[]{-dx, -dz};
                case 3 -> new double[]{dz, -dx};
                default -> new double[]{dx, dz};
            };
        }

        /** {@code this ∘ other}: first {@code other}, then this. */
        public Iso compose(Iso other) {
            return new Iso(k + other.k, turnX(k, other.tx, other.tz) + tx, turnZ(k, other.tx, other.tz) + tz);
        }

        /** The inverse motion. */
        public Iso inverse() {
            int ki = (4 - k) % 4;
            return new Iso(ki, turnX(ki, -tx, -tz), turnZ(ki, -tx, -tz));
        }

        /** {@code [k, tx, tz]}, the format's {@code iso}. */
        public long[] asArray() {
            return new long[]{k, tx, tz};
        }

        /** The wrap engine's motion (translations must fit an int). */
        public CellIsometry toCellIsometry() {
            return new CellIsometry(k, Math.toIntExact(tx), Math.toIntExact(tz));
        }
    }

    // ------------------------------------------------------------------------------------------
    // Adjacency
    // ------------------------------------------------------------------------------------------

    /**
     * What lies across one side of a face: the neighbour face, the side of it the step enters
     * through, and the motion of local cells. For an n × n grid on every face, the cell (i, j) of
     * this face one step beyond the side (one of i, j in {−1, n}, or further out up to a face width,
     * the other in [0, n)) is the neighbour's cell {@code R_k(i, j) + n · shift}, R_k the clockwise
     * cell turn of {@link Iso#cell}.
     *
     * @param quarterTurns clockwise quarter turns from this face's local frame to the neighbour's
     * @param shiftU       the translation in units of the grid's n, along u
     * @param shiftV       along v
     */
    public record Across(Face face, FaceSide side, int quarterTurns, int shiftU, int shiftV) {

        /** The neighbour's cell of this face's cell (i, j) of an n × n face grid: {i', j'}. */
        public long[] cell(long n, long i, long j) {
            return new Iso(quarterTurns, n * shiftU, n * shiftV).cell(i, j);
        }

        /**
         * The neighbour's local point of this face's local point (u, v) (continuous, [−1, 1]² on
         * each face, extended across the side): {u', v'}.
         */
        public double[] local(double u, double v) {
            double[] p = new Iso(quarterTurns, 0, 0).vector(u + 1.0, v + 1.0);
            return new double[]{
                    p[0] + 2.0 * (double) shiftU - 1.0,
                    p[1] + 2.0 * (double) shiftV - 1.0};
        }

        /** A direction (du, dv) in this face's local frame, in the neighbour's. */
        public double[] vector(double du, double dv) {
            return new Iso(quarterTurns, 0, 0).vector(du, dv);
        }
    }

    private static Across across(Face face, FaceSide side, int quarterTurns, int su, int sv) {
        return new Across(face, side, quarterTurns, su, sv);
    }

    /**
     * The face adjacency, {@code [face][side]} in the format's face order and {@link FaceSide} order
     * (east, south, west, north). Derived from the net's seams and checked against the fold and the
     * cube frames (CubeLayoutTest).
     */
    private static final Across[][] ADJACENCY = adjacency();

    private static Across[][] adjacency() {
        Face f0 = Face.F0, f1 = Face.F1, f2 = Face.F2, f3 = Face.F3, n = Face.N, s = Face.S;
        FaceSide e = FaceSide.EAST, so = FaceSide.SOUTH, w = FaceSide.WEST, no = FaceSide.NORTH;
        return new Across[][]{
                // F0
                {across(f1, w, 0, -1, 0), across(s, no, 0, 0, -1), across(f3, e, 0, 1, 0), across(n, so, 0, 0, 1)},
                // F1
                {across(f2, w, 0, -1, 0), across(s, e, 1, 2, 0), across(f0, e, 0, 1, 0), across(n, e, 3, 1, 1)},
                // F2
                {across(f3, w, 0, -1, 0), across(s, so, 2, 1, 2), across(f1, e, 0, 1, 0), across(n, no, 2, 1, 0)},
                // F3
                {across(f0, w, 0, -1, 0), across(s, w, 3, -1, 1), across(f2, e, 0, 1, 0), across(n, w, 1, 0, 0)},
                // N
                {across(f1, no, 1, 1, -1), across(f0, no, 0, 0, -1), across(f3, no, 3, 0, 0), across(f2, no, 2, 1, 0)},
                // S
                {across(f1, so, 3, 0, 2), across(f2, so, 2, 1, 2), across(f3, so, 1, 1, 1), across(f0, so, 0, 0, 1)},
        };
    }

    /** What lies across side {@code side} of {@code face}: a copy-free read of the adjacency table. */
    public static Across adjacency(Face face, FaceSide side) {
        return ADJACENCY[face.index()][side.index()];
    }

    /** {@link #cellOn}'s answer. */
    public record CellOn(Face face, long i, long j, int quarterTurns) {}

    /**
     * The cell (i, j) of face {@code face}'s n × n grid, or the neighbour's cell it is when it lies
     * beyond one side (at most a face width out). Null in the corner squares beyond two sides at
     * once, where three faces meet at a cube corner and no single neighbour holds the cell.
     */
    public static CellOn cellOn(Face face, long n, long i, long j) {
        boolean outI = i < 0 || i >= n;
        boolean outJ = j < 0 || j >= n;
        if (!outI && !outJ)
            return new CellOn(face, i, j, 0);
        if (outI && outJ)
            return null;
        if (i < -n || i >= 2 * n || j < -n || j >= 2 * n)
            return null;
        FaceSide side;
        if (i >= n)
            side = FaceSide.EAST;
        else if (i < 0)
            side = FaceSide.WEST;
        else if (j >= n)
            side = FaceSide.SOUTH;
        else
            side = FaceSide.NORTH;
        Across a = face.across(side);
        long[] c = a.cell(n, i, j);
        return new CellOn(a.face(), c[0], c[1], a.quarterTurns());
    }

    // ------------------------------------------------------------------------------------------
    // Edges, corners, folds
    // ------------------------------------------------------------------------------------------

    /** Which pair of net edges a seam is. */
    public enum Seam {
        /** F2 east and F3 west, the slide (Toroidal World). */
        EAST_WEST("east_west"),
        /** N east and F1 top, a quarter turn about F0's north-east corner. */
        NORTH_EAST("ne"),
        /** N west and F3 top, a quarter turn about F0's north-west corner. */
        NORTH_WEST("nw"),
        /** S east and F1 bottom, a quarter turn about F0's south-east corner. */
        SOUTH_EAST("se"),
        /** S west and F3 bottom, a quarter turn about F0's south-west corner. */
        SOUTH_WEST("sw"),
        /** N top and F2 top, a half turn about (s, −s). */
        NORTH_NORTH("nn"),
        /** S bottom and F2 bottom, a half turn about (s, s). */
        SOUTH_SOUTH("ss");

        private final String id;

        Seam(String id) {
            this.id = id;
        }

        /** The fixture's and the format's name of the seam. */
        public String id() {
            return id;
        }
    }

    /** How a net edge is glued. */
    public enum EdgeMotion {
        SLIDE("slide"), QUARTER_TURN("quarter_turn"), HALF_TURN("half_turn");

        private final String id;

        EdgeMotion(String id) {
            this.id = id;
        }

        public String id() {
            return id;
        }
    }

    /**
     * One of the 14 net edges that face the void or the window's ends, and the motion that brings what
     * lies beyond it back onto the planet (the wrap engine's {@code WrapEdge}, the manifest's
     * {@code edges[]}).
     *
     * @param name     {@code F3-west}, {@code N-east}, {@code F1-top}, ...
     * @param axis     the axis of the edge's normal (x for an east or west edge)
     * @param beyond   which side of the line is beyond
     * @param coord    the edge line (x = coord or z = coord), a block boundary
     * @param spanFrom the first block along the edge
     * @param spanTo   one past the last block along the edge
     * @param iso      beyond → planet
     * @param pivot    the turn's centre (a block corner) for turning edges, else null
     * @param partner  the index in {@link CubeLayout#edges()} of the other net edge of the same seam
     * @param owner    who joins the edge in the game: {@code toroidal_world} (the slide) or {@code wrap_engine}
     */
    public record NetEdge(String name, Seam seam, EdgeMotion motion, Face face, FaceSide faceSide, Axis axis, Side beyond,
                          long coord, long spanFrom, long spanTo, Iso iso, long[] pivot, int partner, String owner) {

        /**
         * How deep the block (x, z) lies beyond the edge, in its band (0 touching the line), or -1
         * when it is not beyond the edge within its span.
         */
        public long depth(long x, long z) {
            long n = axis == Axis.X ? x : z;
            long t = axis == Axis.X ? z : x;
            if (t < spanFrom || t >= spanTo)
                return -1;
            long d = beyond == Side.MIN ? coord - 1 - n : n - coord;
            return d >= 0 ? d : -1;
        }

        /**
         * How far the point (x, z) lies beyond the edge line (above 0), or -1.0 when it is on the
         * planet's side (or on the line) or outside the span.
         */
        public double depthPoint(double x, double z) {
            double n = axis == Axis.X ? x : z;
            double t = axis == Axis.X ? z : x;
            if (t < (double) spanFrom || t > (double) spanTo)
                return -1.0;
            double d = beyond == Side.MIN ? (double) coord - n : n - (double) coord;
            return d > 0.0 ? d : -1.0;
        }
    }

    /**
     * A cube corner: a cone point of 270° where three faces meet.
     *
     * @param name   {@code n0} .. {@code n3}, the corners of face N (cube +Z) counter-clockwise from
     *               (+1, +1, +1), then {@code s0} .. {@code s3}, face S's
     * @param faces  the three faces that meet there
     * @param cube   the corner's direction in the cube frame, (±1, ±1, ±1) unnormalised
     * @param images every block corner of the closed window that is this corner (x ∈ [−3s/2, 5s/2],
     *               so the F2/F3 corners have a third image at x = 5s/2): {x, z} each
     */
    public record Corner(String name, Face[] faces, int[] cube, long[][] images) {}

    /** {@link #faceOfBlock}'s answer. */
    public record FaceCell(Face face, long i, long j) {}

    /** {@link #localOfPoint}'s answer, and {@link CubeMap#inverse}'s. */
    public record LocalPoint(Face face, double u, double v) {}

    /** Where {@link #fold} takes a block. */
    public sealed interface Fold permits OnFace, Outside, Ocean {
        Ocean OCEAN = new Ocean();
    }

    /** On the planet (after the slide): the face and the block's local cell in [0, s)². */
    public record OnFace(Face face, long i, long j) implements Fold {}

    /**
     * Beyond a net edge, in its band: the edge (an index into {@link CubeLayout#edges()}), the planet
     * block it shows (in the window) and the motion from the queried block to it (its {@code k} is the
     * quarter turns a block state read there is turned back by).
     */
    public record Outside(int edge, long x, long z, Iso iso) implements Fold {}

    /**
     * In no band, or more than a face width out: past the corners of the net (the missing quarter
     * round an outer cube corner). The game draws deep ocean there (plan §1.4).
     */
    public record Ocean() implements Fold {}

    /**
     * A point brought onto the planet by {@link #foldPoint}.
     *
     * @param u    local coordinates in [−1, 1]
     * @param x    the planet point in the window
     * @param iso  the motion from the queried point (identity on the planet, the slide included)
     * @param edge the net edge crossed, or -1
     */
    public record PointOnFace(Face face, double u, double v, double x, double z, Iso iso, int edge) {}

    // ------------------------------------------------------------------------------------------
    // The net
    // ------------------------------------------------------------------------------------------

    private final long s;
    private final List<NetEdge> edges;

    private CubeLayout(long s) {
        this.s = s;
        this.edges = netEdges(s);
    }

    /** The net of face side {@code s} (even). */
    public static CubeLayout of(int s) {
        if (s < 2 || s % 2 != 0)
            throw new IllegalArgumentException("cube face side " + s + " must be even");
        return new CubeLayout(s);
    }

    private static final java.util.concurrent.ConcurrentHashMap<Long, CubeLayout> SHARED = new java.util.concurrent.ConcurrentHashMap<>();

    /** The net of face side {@code s}, one per side for the process's life (Rust {@code Cube::shared}). */
    public static CubeLayout shared(long s) {
        CubeLayout c = SHARED.get(s);
        if (c != null)
            return c;
        return SHARED.computeIfAbsent(s, k -> of(Math.toIntExact(k)));
    }

    /** The net for a planet {@code circumference} blocks round ({@link #faceFor}). */
    public static CubeLayout forCircumference(int circumference) {
        return of(faceFor(circumference));
    }

    /** s, blocks. */
    public long side() {
        return s;
    }

    /** Whether s is a multiple of {@link #FACE_QUANTUM} (a real world's face). */
    public boolean quantised() {
        return s % FACE_QUANTUM == 0;
    }

    /** The east-west period 4s, blocks (Toroidal World's loop). */
    public long period() {
        return 4 * s;
    }

    /** The radius of the sphere the six faces stand for at equal area, blocks: 6s² = 4πR². */
    public double radius() {
        // ledger: SCL-078 (6 s² = 4π R²)
        return (double) s * Math.sqrt(1.5 / Math.PI);
    }

    /** The window: {x_min, x_max, z_min, z_max} = x ∈ [−3s/2, 5s/2), z ∈ [−3s/2, 3s/2]. */
    public double[] window() {
        double h = (double) s / 2.0;
        return new double[]{-3.0 * h, 5.0 * h, -3.0 * h, 3.0 * h};
    }

    /** x of the net block of the face's local cell (0, 0) (its north-west corner). */
    public long faceOriginX(Face face) {
        return face.centreUnitsX() * s - s / 2;
    }

    /** z of the net block of the face's local cell (0, 0). */
    public long faceOriginZ(Face face) {
        return face.centreUnitsZ() * s - s / 2;
    }

    /** x of the face's centre, a block corner. */
    public long faceCentreX(Face face) {
        return face.centreUnitsX() * s;
    }

    /** z of the face's centre, a block corner. */
    public long faceCentreZ(Face face) {
        return face.centreUnitsZ() * s;
    }

    /** The block x wrapped into the window [−3s/2, 5s/2). */
    public long wrapX(long x) {
        long lo = -3 * s / 2;
        return Math.floorMod(x - lo, 4 * s) + lo;
    }

    /** The point x wrapped into the window [−3s/2, 5s/2). */
    public double wrapXPoint(double x) {
        double p = (double) (4 * s);
        double lo = -1.5 * (double) s;
        double t = PortableMath.remEuclid(x - lo, p);
        return (t >= p ? 0.0 : t) + lo;
    }

    /** The face and local cell of the block (x, z) (x wraps), or null off the planet. */
    public FaceCell faceOfBlock(long x, long z) {
        x = wrapX(x);
        long h = s / 2;
        // Columns of s from x = −3s/2: 0 F3, 1 F0, 2 F1, 3 F2.
        int col = (int) Math.floorDiv(x + 3 * h, s);
        Face face;
        if (z >= -h && z < h)
            face = new Face[]{Face.F3, Face.F0, Face.F1, Face.F2}[col];
        else if (col == 1 && z >= -3 * h && z < -h)
            face = Face.N;
        else if (col == 1 && z >= h && z < 3 * h)
            face = Face.S;
        else
            return null;
        return new FaceCell(face, x - faceOriginX(face), z - faceOriginZ(face));
    }

    /** x of the net block of a face's local cell (i, j) (in the window when (i, j) ∈ [0, s)²). */
    public long blockX(Face face, long i) {
        return faceOriginX(face) + i;
    }

    /** z of the net block of a face's local cell (i, j). */
    public long blockZ(Face face, long j) {
        return faceOriginZ(face) + j;
    }

    /**
     * The face and local point (u, v) ∈ [−1, 1]² of the point (x, z) (x wraps), or null off the
     * planet. Faces are closed squares here: a point on a line two faces share is given to the first
     * in the format's order (both faces map it to the same planet point).
     */
    public LocalPoint localOfPoint(double x, double z) {
        x = wrapXPoint(x);
        double h = (double) s / 2.0;
        for (Face face : Face.ALL) {
            double u = (x - (double) faceCentreX(face)) / h;
            double v = (z - (double) faceCentreZ(face)) / h;
            if (u >= -1.0 && u <= 1.0 && v >= -1.0 && v <= 1.0)
                return new LocalPoint(face, u, v);
        }
        return null;
    }

    /** The net point of a face's local point (u, v): {x, z}. */
    public double[] pointOfLocal(Face face, double u, double v) {
        double h = (double) s / 2.0;
        return new double[]{(double) faceCentreX(face) + u * h, (double) faceCentreZ(face) + v * h};
    }

    /**
     * The 14 net edges: the two slide edges, then the turning edges in the wrap engine's order
     * ({@code WrapLayouts.cube} builds its edges from these): N-east, F1-top, N-west, F3-top, N-top,
     * F2-top, S-east, F1-bottom, S-west, F3-bottom, S-bottom, F2-bottom.
     */
    public List<NetEdge> edges() {
        return edges;
    }

    /**
     * The block (x, z) brought onto the planet ({@link Fold}). x wraps (the slide); a block off the
     * planet takes the motion of the net edge whose band holds it, and where two bands meet (the void
     * squares by F0's corners) the nearer edge's, ties to the earlier edge in {@link #edges()} (so
     * the square splits on its diagonal through F0's corner, plan §1.4). Bands reach a face width
     * out; a block in none is {@link Fold#OCEAN}.
     */
    public Fold fold(long x, long z) {
        long xw = wrapX(x);
        FaceCell fc = faceOfBlock(xw, z);
        if (fc != null)
            return new OnFace(fc.face(), fc.i(), fc.j());
        int bestK = -1;
        long bestD = 0;
        for (int k = 2; k < edges.size(); k++) {
            long d = edges.get(k).depth(xw, z);
            // Bands a face width deep, the nearer edge first (ledger: SCL-108).
            if (d >= 0 && d < s && (bestK < 0 || d < bestD)) {
                bestK = k;
                bestD = d;
            }
        }
        if (bestK < 0)
            return Fold.OCEAN;
        Iso iso = edges.get(bestK).iso().compose(Iso.translation(xw - x, 0));
        long px = iso.cellX(x, z), pz = iso.cellZ(x, z);
        if (faceOfBlock(px, pz) != null && px == wrapX(px))
            return new Outside(bestK, px, pz, iso);
        return Fold.OCEAN;
    }

    /**
     * The point (x, z) brought onto the planet, as {@link #fold} for blocks: x wraps; a point off the
     * planet takes the motion of the nearest net edge whose band (a face width deep) holds it. Null
     * past the net's corners.
     */
    public PointOnFace foldPoint(double x, double z) {
        double xw = wrapXPoint(x);
        long slide = (long) roundHalfAway(xw - x);
        LocalPoint lp = localOfPoint(xw, z);
        if (lp != null)
            return new PointOnFace(lp.face(), lp.u(), lp.v(), xw, z, Iso.translation(slide, 0), -1);
        int bestK = -1;
        double bestD = 0.0;
        for (int k = 2; k < edges.size(); k++) {
            double d = edges.get(k).depthPoint(xw, z);
            if (d > 0.0 && d <= (double) s && (bestK < 0 || d < bestD)) {
                bestK = k;
                bestD = d;
            }
        }
        if (bestK < 0)
            return null;
        Iso iso = edges.get(bestK).iso().compose(Iso.translation(slide, 0));
        double[] p = iso.point(x, z);
        LocalPoint q = localOfPoint(p[0], p[1]);
        if (q == null)
            return null;
        return new PointOnFace(q.face(), q.u(), q.v(), wrapXPoint(p[0]), p[1], iso, bestK);
    }

    /**
     * The eight cube corners: {@code n0} .. {@code n3} round face N, then {@code s0} .. {@code s3}
     * round face S, each with its cube-frame direction and its images in the net.
     */
    public List<Corner> corners() {
        long h = s / 2;
        Face f0 = Face.F0, f1 = Face.F1, f2 = Face.F2, f3 = Face.F3, n = Face.N, so = Face.S;
        return List.of(
                corner("n0", new Face[]{f0, f1, n}, new int[]{1, 1, 1}, new long[][]{{h, -h}}),
                corner("n1", new Face[]{f1, f2, n}, new int[]{-1, 1, 1}, new long[][]{{h, -3 * h}, {3 * h, -h}}),
                corner("n2", new Face[]{f2, f3, n}, new int[]{-1, -1, 1}, new long[][]{{-h, -3 * h}, {-3 * h, -h}, {5 * h, -h}}),
                corner("n3", new Face[]{f3, f0, n}, new int[]{1, -1, 1}, new long[][]{{-h, -h}}),
                corner("s0", new Face[]{f0, f1, so}, new int[]{1, 1, -1}, new long[][]{{h, h}}),
                corner("s1", new Face[]{f1, f2, so}, new int[]{-1, 1, -1}, new long[][]{{h, 3 * h}, {3 * h, h}}),
                corner("s2", new Face[]{f2, f3, so}, new int[]{-1, -1, -1}, new long[][]{{-h, 3 * h}, {-3 * h, h}, {5 * h, h}}),
                corner("s3", new Face[]{f3, f0, so}, new int[]{1, -1, -1}, new long[][]{{-h, h}}));
    }

    private static Corner corner(String name, Face[] faces, int[] cube, long[][] images) {
        return new Corner(name, faces, cube, images);
    }

    // ------------------------------------------------------------------------------------------
    // The 14 net edges for face side s
    // ------------------------------------------------------------------------------------------

    private static NetEdge edge(Face face, FaceSide faceSide, Seam seam, EdgeMotion motion, long coord, long spanFrom, long spanTo,
                                Iso iso, long[] pivot, int partner) {
        Axis axis = faceSide == FaceSide.EAST || faceSide == FaceSide.WEST ? Axis.X : Axis.Z;
        Side beyond = faceSide == FaceSide.EAST || faceSide == FaceSide.SOUTH ? Side.MAX : Side.MIN;
        String sideName = faceSide.id();
        if (face != Face.F0) {
            if (faceSide == FaceSide.NORTH)
                sideName = "top";
            else if (faceSide == FaceSide.SOUTH)
                sideName = "bottom";
        }
        return new NetEdge(face.name() + "-" + sideName, seam, motion, face, faceSide, axis, beyond, coord, spanFrom, spanTo,
                iso, pivot, partner, motion == EdgeMotion.SLIDE ? "toroidal_world" : "wrap_engine");
    }

    private static List<NetEdge> netEdges(long s) {
        long h = s / 2;
        Face f1 = Face.F1, f2 = Face.F2, f3 = Face.F3, n = Face.N, so = Face.S;
        FaceSide east = FaceSide.EAST, south = FaceSide.SOUTH, west = FaceSide.WEST, north = FaceSide.NORTH;
        EdgeMotion q = EdgeMotion.QUARTER_TURN, ht = EdgeMotion.HALF_TURN;
        Iso halfN = Iso.halfTurnAbout(s, -s);
        Iso halfS = Iso.halfTurnAbout(s, s);
        return List.of(
                // 0, 1: the slide.
                edge(f3, west, Seam.EAST_WEST, EdgeMotion.SLIDE, -3 * h, -h, h, Iso.translation(4 * s, 0), null, 1),
                edge(f2, east, Seam.EAST_WEST, EdgeMotion.SLIDE, 5 * h, -h, h, Iso.translation(-4 * s, 0), null, 0),
                // 2, 3: NE, a quarter turn about (s/2, −s/2), clockwise from N to F1.
                edge(n, east, Seam.NORTH_EAST, q, h, -3 * h, -h, Iso.quarterTurnAbout(1, h, -h), new long[]{h, -h}, 3),
                edge(f1, north, Seam.NORTH_EAST, q, -h, h, 3 * h, Iso.quarterTurnAbout(3, h, -h), new long[]{h, -h}, 2),
                // 4, 5: NW, about (−s/2, −s/2), counter-clockwise from N to F3.
                edge(n, west, Seam.NORTH_WEST, q, -h, -3 * h, -h, Iso.quarterTurnAbout(3, -h, -h), new long[]{-h, -h}, 5),
                edge(f3, north, Seam.NORTH_WEST, q, -h, -3 * h, -h, Iso.quarterTurnAbout(1, -h, -h), new long[]{-h, -h}, 4),
                // 6, 7: NN, the half turn about (s, −s).
                edge(n, north, Seam.NORTH_NORTH, ht, -3 * h, -h, h, halfN, new long[]{s, -s}, 7),
                edge(f2, north, Seam.NORTH_NORTH, ht, -h, 3 * h, 5 * h, halfN, new long[]{s, -s}, 6),
                // 8, 9: SE, about (s/2, s/2), counter-clockwise from S to F1.
                edge(so, east, Seam.SOUTH_EAST, q, h, h, 3 * h, Iso.quarterTurnAbout(3, h, h), new long[]{h, h}, 9),
                edge(f1, south, Seam.SOUTH_EAST, q, h, h, 3 * h, Iso.quarterTurnAbout(1, h, h), new long[]{h, h}, 8),
                // 10, 11: SW, about (−s/2, s/2), clockwise from S to F3.
                edge(so, west, Seam.SOUTH_WEST, q, -h, h, 3 * h, Iso.quarterTurnAbout(1, -h, h), new long[]{-h, h}, 11),
                edge(f3, south, Seam.SOUTH_WEST, q, h, -3 * h, -h, Iso.quarterTurnAbout(3, -h, h), new long[]{-h, h}, 10),
                // 12, 13: SS, the half turn about (s, s).
                edge(so, south, Seam.SOUTH_SOUTH, ht, 3 * h, -h, h, halfS, new long[]{s, s}, 13),
                edge(f2, south, Seam.SOUTH_SOUTH, ht, h, 3 * h, 5 * h, halfS, new long[]{s, s}, 12));
    }
}
