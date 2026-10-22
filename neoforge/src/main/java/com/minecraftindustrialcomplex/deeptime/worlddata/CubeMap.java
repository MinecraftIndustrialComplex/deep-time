// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.io.IOException;
import java.io.InputStream;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import java.util.List;
import java.util.Locale;
import java.util.concurrent.ConcurrentHashMap;

/**
 * The cube world's map slot (docs/plans/cube-world-plan.md §1.6, work item M3): a face's local
 * point (u, v) ∈ [−1, 1]² to the unit sphere and back, from a table. The twin of Rust's
 * {@code deeptime_format::cube_map::CubeMap}, bit for bit (checked against the cube section of the
 * parity fixture by {@code ProjectionParityTest}).
 *
 * <p>Two tables ship in the jar ({@code /deeptime/cube-map/}), selected by name through the same
 * slot ({@link #builtin}): {@code conformal-n513} ({@link #WORLD_MAP}), the conformal cube the world
 * uses, which its manifest names by the table's sha256, and {@code moser-t1-n513}
 * ({@link #SIMULATION_MAP}), Moser's equal-area cube the simulation uses. The table holds one eighth
 * of a face, the triangle 0 ≤ v ≤ u ≤ 1, as the planet point in gnomonic coordinates of the face
 * frame, (a, b) = (y/x, z/x) with x along the face centre; the rest of the face is sign flips and a
 * swap.
 *
 * <h2>Arithmetic</h2>
 * Only +, −, ×, IEEE division and {@link Math#sqrt}, in the order written here and in Rust, with no
 * fused multiply-add ({@code Math.fma} is never called, and Java 17+ evaluates every double
 * expression strictly), no {@code Math.floor} in the evaluator ({@link #floorClamped}). Latitude and
 * longitude go through {@link PortableMath}. Do not reorder an expression "for speed": the bits are
 * the contract.
 *
 * <h2>The evaluator</h2>
 * The forward is a Catmull-Rom bicubic over the table's nodes, padded by ghost nodes (beyond u = 1
 * the mirror across the edge's plane, (a, b) → (1/a, b/a); beyond v = 1 its swap; past the corner a
 * linear extrapolation; beyond u = 0 and v = 0 the sign flips), with a = 1 written exactly on the
 * edge u = 1. The conformal map behaves like (z − z_c)^(4/3) at a cube corner, so its table carries
 * a corner patch (ledger SCL-104): a finer table over the K coarse cells next to the corner, blended
 * in by a smoothstep over the coarse cell next to it. The inverse starts from a second table over
 * the gnomonic triangle (bilinear) and takes {@link #NEWTON_STEPS} Newton steps on the forward
 * bicubic.
 */
public final class CubeMap {

    /** The table file's magic. */
    private static final byte[] MAGIC = {'D', 'T', 'C', 'M'};
    /** The table file's layout version (the header's second field). */
    public static final int LAYOUT_VERSION = 1;
    /** The header's size, bytes. */
    public static final int HEADER_BYTES = 112;

    /** The world's map: the conformal cube (decisions log 2026-10-22; ledger SCL-101). */
    public static final String WORLD_MAP = "conformal-n513"; // ledger: SCL-101
    /** The simulation's map: Moser's equal-area cube on the polar axis (ledger SCL-102). */
    public static final String SIMULATION_MAP = "moser-t1-n513"; // ledger: SCL-102

    /**
     * Newton steps of the inverse after the inverse table's start: 4 take the round trip to 3e-12
     * block at 32k on the triangle and 8e-10 within 100 blocks of a corner (the plan's 2 leave 0.005
     * and 0.029 block; ledger SCL-103).
     */
    public static final int NEWTON_STEPS = 4; // ledger: SCL-103

    /** The jar folder the shipped tables are copied into (neoforge/build.gradle, processResources). */
    private static final String RESOURCE_DIR = "/deeptime/cube-map/";

    private static final ConcurrentHashMap<String, CubeMap> LOADED = new ConcurrentHashMap<>();

    /**
     * floor(x) clamped to [0, last] (0 for NaN), without {@code Math.floor}: the cast truncates
     * toward zero (saturating, NaN to 0, as Rust's {@code as i64}) and a negative x is stepped down.
     */
    static int floorClamped(double x, int last) {
        long i = (long) x;
        if ((double) i > x)
            i -= 1;
        return (int) Math.max(0L, Math.min(i, (long) last));
    }

    // ------------------------------------------------------------------------------------------
    // Grids
    // ------------------------------------------------------------------------------------------

    /**
     * A padded node grid over [−1, n + 1]², node (i, j) at index (j + 1)·side + (i + 1), so the four
     * nodes a bicubic row reads along i are adjacent. Nodes are (a, b) pairs in a flat array.
     */
    private static final class Grid {
        /** Cells a side. */
        final int n;
        /** n + 3. */
        final int side;
        final double[] nodes;

        Grid(int n) {
            this.n = n;
            this.side = n + 3;
            this.nodes = new double[2 * side * side];
            java.util.Arrays.fill(nodes, Double.NaN);
        }

        int idx(int i, int j) {
            return (j + 1) * side + (i + 1);
        }

        void set(int i, int j, double a, double b) {
            int k = 2 * idx(i, j);
            nodes[k] = a;
            nodes[k + 1] = b;
        }

        double a(int i, int j) {
            return nodes[2 * idx(i, j)];
        }

        double b(int i, int j) {
            return nodes[2 * idx(i, j) + 1];
        }

        /** The bicubic at grid coordinates (x, y), node units: the cubic along x on each of the four rows, then along y. */
        double[] value(double x, double y) {
            int i0 = floorClamped(x, n - 1);
            double tu = x - (double) i0;
            int j0 = floorClamped(y, n - 1);
            double tv = y - (double) j0;
            double[] wu = weights(tu);
            double[] r = new double[8];
            for (int dj = 0; dj < 4; dj++) {
                int b = (j0 + dj) * side + i0;
                r[2 * dj] = mix(wu, nodes, b, 0);
                r[2 * dj + 1] = mix(wu, nodes, b, 1);
            }
            double[] wv = weights(tv);
            return new double[]{mix(wv, r, 0, 0), mix(wv, r, 0, 1)};
        }

        /** The bicubic at (x, y) and its derivatives along x and y, per node unit. */
        Eval valueD(double x, double y) {
            int i0 = floorClamped(x, n - 1);
            double tu = x - (double) i0;
            int j0 = floorClamped(y, n - 1);
            double tv = y - (double) j0;
            double[] wu = weights(tu), dwu = dweights(tu);
            double[] wv = weights(tv), dwv = dweights(tv);
            double[] r = new double[8], dr = new double[8];
            for (int dj = 0; dj < 4; dj++) {
                int b = (j0 + dj) * side + i0;
                r[2 * dj] = mix(wu, nodes, b, 0);
                r[2 * dj + 1] = mix(wu, nodes, b, 1);
                dr[2 * dj] = mix(dwu, nodes, b, 0);
                dr[2 * dj + 1] = mix(dwu, nodes, b, 1);
            }
            return new Eval(
                    new double[]{mix(wv, r, 0, 0), mix(wv, r, 0, 1)},
                    new double[]{mix(wv, dr, 0, 0), mix(wv, dr, 0, 1)},
                    new double[]{mix(dwv, r, 0, 0), mix(dwv, r, 0, 1)});
        }
    }

    /** The triangle's (a, b) at a point and its derivatives along u and v. */
    private record Eval(double[] val, double[] du, double[] dv) {
        Eval scaled(double k) {
            return new Eval(val, new double[]{du[0] * k, du[1] * k}, new double[]{dv[0] * k, dv[1] * k});
        }
    }

    /**
     * The Catmull-Rom weights of four nodes p0 .. p3 at t ∈ [0, 1] (the cubic through p1 at t = 0 and
     * p2 at t = 1, slopes (p2 − p0)/2 and (p3 − p1)/2): exactly (0, 1, 0, 0) at t = 0 and (0, 0, 1, 0)
     * at t = 1, so the bicubic is exact at its nodes (ledger: SCL-105).
     */
    private static double[] weights(double t) {
        double t2 = t * t;
        double t3 = t2 * t;
        return new double[]{
                0.5 * ((2.0 * t2 - t) - t3),
                0.5 * ((2.0 - 5.0 * t2) + 3.0 * t3),
                0.5 * ((t + 4.0 * t2) - 3.0 * t3),
                0.5 * (t3 - t2)};
    }

    /** The weights' derivatives along t. */
    private static double[] dweights(double t) {
        double t2 = t * t;
        return new double[]{
                0.5 * ((4.0 * t - 1.0) - 3.0 * t2),
                0.5 * (9.0 * t2 - 10.0 * t),
                0.5 * ((1.0 + 8.0 * t) - 9.0 * t2),
                0.5 * (3.0 * t2 - 2.0 * t)};
    }

    /** Σ w_k p_k on component {@code c} of the four (a, b) nodes from node {@code node}, summed from k = 0. */
    private static double mix(double[] w, double[] p, int node, int c) {
        int at = 2 * node + c;
        return ((w[0] * p[at] + w[1] * p[at + 2]) + w[2] * p[at + 4]) + w[3] * p[at + 6];
    }

    /** The corner patch of a conformal table. */
    private static final class Patch {
        /** K: coarse cells from the corner the patch covers. */
        final int k;
        /** F: fine cells to a coarse cell. */
        final int f;
        /** 1 − K/m: where the patch starts along u and v. */
        final double u0;
        /** m·F: fine cells per unit of u. */
        final double mf;
        final Grid grid;

        Patch(int k, int f, double u0, double mf, Grid grid) {
            this.k = k;
            this.f = f;
            this.u0 = u0;
            this.mf = mf;
            this.grid = grid;
        }
    }

    /**
     * A point on the unit sphere and its derivatives per unit of a face's u and v, in the cube frame
     * ({@link #jacobian}).
     */
    public record FaceJacobian(double[] p, double[] du, double[] dv) {}

    // ------------------------------------------------------------------------------------------
    // The table
    // ------------------------------------------------------------------------------------------

    private final String name;
    private final String commit;
    private final double tau;
    private final int flowSteps;
    private final int m;
    private final double mf;
    private final int q;
    private final byte[] sha256;
    private final Grid coarse;
    private final Patch patch;
    /** The inverse table over the whole gnomonic square [0, 1]² (the triangle and its swap): node (k, l) at k·(q + 1) + l. */
    private final double[] inv;

    private CubeMap(String name, String commit, double tau, int flowSteps, int m, int q, byte[] sha256, Grid coarse,
                    Patch patch, double[] inv) {
        this.name = name;
        this.commit = commit;
        this.tau = tau;
        this.flowSteps = flowSteps;
        this.m = m;
        this.mf = (double) m;
        this.q = q;
        this.sha256 = sha256;
        this.coarse = coarse;
        this.patch = patch;
        this.inv = inv;
    }

    private static WorldDataException bad(String why) {
        return new WorldDataException("cube map table: " + why);
    }

    private static long u32At(ByteBuffer b, int at) {
        return b.getInt(at) & 0xFFFF_FFFFL;
    }

    private static String asciiField(byte[] b, int from, int to, String what) {
        int end = to;
        for (int i = from; i < to; i++)
            if (b[i] == 0) {
                end = i;
                break;
            }
        for (int i = from; i < end; i++)
            if (b[i] < 0x21 || b[i] > 0x7e)
                throw bad("the " + what + " is not printable ASCII");
        return new String(b, from, end - from, StandardCharsets.ISO_8859_1);
    }

    /**
     * Reads a triangle of (a, b) pairs stored row by row (for i, for j ≤ i) from {@code at}: pair
     * (hi, lo) of the result is at index hi·(hi + 1)/2 + lo.
     */
    private static double[] trianglePairs(ByteBuffer b, int at, int rows) {
        int count = rows * (rows + 1) / 2;
        double[] out = new double[2 * count];
        int p = at;
        for (int i = 0; i < 2 * count; i++) {
            out[i] = b.getDouble(p);
            p += 8;
        }
        return out;
    }

    private static int tri(int hi, int lo) {
        return hi * (hi + 1) / 2 + lo;
    }

    /** A table from its file's bytes (layout in FORMAT.md "The cube map table"). */
    public static CubeMap load(byte[] bytes) {
        if (bytes.length < HEADER_BYTES || bytes[0] != MAGIC[0] || bytes[1] != MAGIC[1] || bytes[2] != MAGIC[2] || bytes[3] != MAGIC[3])
            throw bad("not a cube map table (magic DTCM)");
        ByteBuffer bb = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN);
        long version = u32At(bb, 4);
        if (version != LAYOUT_VERSION)
            throw bad("layout version " + version + ", this reader reads " + LAYOUT_VERSION);
        long ml = u32At(bb, 8);
        long ql = u32At(bb, 12);
        double tau = bb.getDouble(16);
        long flowSteps = u32At(bb, 24);
        long kl = u32At(bb, 28);
        long fl = u32At(bb, 32);
        if (!(ml >= 2 && ml <= 65_536) || !(ql >= 1 && ql <= 65_536))
            throw bad("m " + ml + ", q " + ql);
        if ((kl == 0) != (fl == 0) || kl >= ml || fl > 4_096)
            throw bad("corner patch K " + kl + ", F " + fl + " with m " + ml);
        String name = asciiField(bytes, 40, 72, "name");
        String commit = asciiField(bytes, 72, 112, "generator commit");
        long nf = (ml + 1) * (ml + 2) / 2;
        long ni = (ql + 1) * (ql + 2) / 2;
        long ps = kl > 0 ? kl * fl + 3 : 0;
        long np = ps * (ps + 1) / 2;
        long want = HEADER_BYTES + 16 * (nf + ni + np);
        if ((long) bytes.length != want)
            throw bad(bytes.length + " bytes, want " + want);
        byte[] sha256 = sha256Of(bytes);
        int m = (int) ml, q = (int) ql, k = (int) kl, f = (int) fl;

        // The coarse table and its ghosts (cube_map.py `padded`).
        double[] tr = trianglePairs(bb, HEADER_BYTES, m + 1);
        Grid g = new Grid(m);
        for (int i = 0; i <= m; i++) {
            for (int j = 0; j <= m; j++) {
                int hi = Math.max(i, j), lo = Math.min(i, j);
                double pa = tr[2 * tri(hi, lo)], pb = tr[2 * tri(hi, lo) + 1];
                if (j <= i)
                    g.set(i, j, pa, pb);
                else
                    g.set(i, j, pb, pa);
            }
        }
        padEdgeAndCorner(g, m);
        // Beyond u = 0: (−a, b) of column 1, rows j ∈ [0, m + 1]; beyond v = 0: (a, −b) of row 1,
        // i ∈ [−1, m + 1].
        for (int j = 0; j <= m + 1; j++)
            g.set(-1, j, -g.a(1, j), g.b(1, j));
        for (int i = -1; i <= m + 1; i++)
            g.set(i, -1, g.a(i, 1), -g.b(i, 1));
        if (anyNotFinite(g.nodes))
            throw bad("forward nodes not finite");

        // The inverse table, as a square.
        double[] itri = trianglePairs(bb, HEADER_BYTES + 16 * (int) nf, q + 1);
        double[] inv = new double[2 * (q + 1) * (q + 1)];
        for (int kk = 0; kk <= q; kk++) {
            for (int ll = 0; ll <= q; ll++) {
                int at = 2 * tri(Math.max(kk, ll), Math.min(kk, ll));
                double pa = itri[at], pb = itri[at + 1];
                int o = 2 * (kk * (q + 1) + ll);
                if (ll <= kk) {
                    inv[o] = pa;
                    inv[o + 1] = pb;
                } else {
                    inv[o] = pb;
                    inv[o + 1] = pa;
                }
            }
        }

        // The corner patch: every node of [−1, n + 1]² stored, the upper triangle by the swap.
        Patch patch = null;
        if (k > 0) {
            int n = k * f;
            double[] ptri = trianglePairs(bb, HEADER_BYTES + 16 * (int) (nf + ni), (int) ps);
            Grid pg = new Grid(n);
            for (int i = -1; i <= n + 1; i++) {
                for (int j = -1; j <= n + 1; j++) {
                    int hi = Math.max(i, j) + 1, lo = Math.min(i, j) + 1;
                    double pa = ptri[2 * tri(hi, lo)], pb = ptri[2 * tri(hi, lo) + 1];
                    if (j <= i)
                        pg.set(i, j, pa, pb);
                    else
                        pg.set(i, j, pb, pa);
                }
            }
            if (anyNotFinite(pg.nodes))
                throw bad("corner patch nodes not finite");
            patch = new Patch(k, f, 1.0 - (double) k / (double) m, (double) (m * f), pg);
        }
        return new CubeMap(name, commit, tau, (int) flowSteps, m, q, sha256, g, patch, inv);
    }

    private static boolean anyNotFinite(double[] nodes) {
        for (double v : nodes)
            if (!Double.isFinite(v))
                return true;
        return false;
    }

    private static byte[] sha256Of(byte[] bytes) {
        try {
            return MessageDigest.getInstance("SHA-256").digest(bytes);
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException("SHA-256 is part of every Java platform", e);
        }
    }

    /** A table from its bytes, checked against the sha256 a manifest names (hex, any case). */
    public static CubeMap loadChecked(byte[] bytes, String sha256Hex) {
        CubeMap map = load(bytes);
        String want = sha256Hex.strip().toLowerCase(Locale.ROOT);
        if (!map.sha256Hex().equals(want))
            throw bad("cube map table " + map.name + ": sha256 " + map.sha256Hex() + ", the manifest names " + sha256Hex.strip());
        return map;
    }

    /** The names of the tables that ship in the jar. */
    public static List<String> builtinNames() {
        return List.of(WORLD_MAP, SIMULATION_MAP);
    }

    /**
     * A table that ships in the jar, by name ({@link #WORLD_MAP}, {@link #SIMULATION_MAP}), loaded
     * once from {@code /deeptime/cube-map/cube-map-<name>.bin} and checked against the
     * {@code .sha256} file beside it. Null for another name.
     *
     * @throws WorldDataException when the shipped table is missing or fails its checksum (a broken jar)
     */
    public static CubeMap builtin(String name) {
        if (!builtinNames().contains(name))
            return null;
        return LOADED.computeIfAbsent(name, n -> {
            String base = RESOURCE_DIR + "cube-map-" + n + ".bin";
            byte[] bytes = resource(base);
            String sum = new String(resource(base + ".sha256"), StandardCharsets.US_ASCII).strip();
            int sp = 0;
            while (sp < sum.length() && !Character.isWhitespace(sum.charAt(sp)))
                sp++;
            CubeMap map = loadChecked(bytes, sum.substring(0, sp));
            if (!map.name.equals(n))
                throw bad("the jar's table " + n + " names itself " + map.name);
            return map;
        });
    }

    private static byte[] resource(String path) {
        try (InputStream in = CubeMap.class.getResourceAsStream(path)) {
            if (in == null)
                throw new WorldDataException("the jar's cube map table " + path + " is missing");
            return in.readAllBytes();
        } catch (IOException e) {
            throw new WorldDataException("reading the jar's cube map table " + path, e);
        }
    }

    /** The world's map ({@link #WORLD_MAP}). */
    public static CubeMap world() {
        return builtin(WORLD_MAP);
    }

    /** The simulation's map ({@link #SIMULATION_MAP}). */
    public static CubeMap simulation() {
        return builtin(SIMULATION_MAP);
    }

    /** The table's name ({@code conformal-n513}, {@code moser-t1-n513}). */
    public String name() {
        return name;
    }

    /** The generator's commit. */
    public String commit() {
        return commit;
    }

    /** τ: 0 the conformal cube, 1 Moser's equal-area cube. */
    public double tau() {
        return tau;
    }

    /** The RK4 steps of Moser's flow the generator took (0 for the conformal cube). */
    public int flowSteps() {
        return flowSteps;
    }

    /** m: forward cells along the triangle's side (the face has 2m + 1 nodes a side). */
    public int cells() {
        return m;
    }

    /** q: the inverse table's cells along the gnomonic triangle's side. */
    public int inverseCells() {
        return q;
    }

    /** The corner patch's {K, F}, or null when the table has none. */
    public int[] cornerPatch() {
        return patch == null ? null : new int[]{patch.k, patch.f};
    }

    /** The file's sha256. */
    public byte[] sha256() {
        return sha256.clone();
    }

    /** The file's sha256, lowercase hex. */
    public String sha256Hex() {
        return HexFormat.of().formatHex(sha256);
    }

    // ------------------------------------------------------------------------------------------
    // The evaluator
    // ------------------------------------------------------------------------------------------

    /**
     * (a, b) on the triangle at (u, v), 0 ≤ v ≤ u ≤ 1, without the edge's exact a = 1 (a Newton
     * iterate may step a hair outside the triangle).
     */
    private double[] triangleValueRaw(double u, double v) {
        if (patch == null)
            return coarse.value(u * mf, v * mf);
        // Coarse cells inward from the patch's edge (v is the smaller coordinate).
        double w = (v - patch.u0) * mf;
        if (w <= 0.0)
            return coarse.value(u * mf, v * mf);
        double[] fv = patch.grid.value((u - patch.u0) * patch.mf, (v - patch.u0) * patch.mf);
        if (w >= 1.0)
            return fv;
        double[] c = coarse.value(u * mf, v * mf);
        double s = w * w * (3.0 - 2.0 * w); // ledger: SCL-106
        return new double[]{c[0] + s * (fv[0] - c[0]), c[1] + s * (fv[1] - c[1])};
    }

    /** (a, b) and its derivatives along u and v on the triangle, as {@link #triangleValueRaw}. */
    private Eval triangleRaw(double u, double v) {
        if (patch == null)
            return coarse.valueD(u * mf, v * mf).scaled(mf);
        double w = (v - patch.u0) * mf;
        if (w <= 0.0)
            return coarse.valueD(u * mf, v * mf).scaled(mf);
        Eval fe = patch.grid.valueD((u - patch.u0) * patch.mf, (v - patch.u0) * patch.mf).scaled(patch.mf);
        if (w >= 1.0)
            return fe;
        Eval ce = coarse.valueD(u * mf, v * mf).scaled(mf);
        double s = w * w * (3.0 - 2.0 * w); // ledger: SCL-106
        double ds = 6.0 * w * (1.0 - w) * mf;
        return new Eval(
                new double[]{blend(ce.val[0], fe.val[0], s), blend(ce.val[1], fe.val[1], s)},
                new double[]{blend(ce.du[0], fe.du[0], s), blend(ce.du[1], fe.du[1], s)},
                new double[]{
                        blend(ce.dv[0], fe.dv[0], s) + (fe.val[0] - ce.val[0]) * ds,
                        blend(ce.dv[1], fe.dv[1], s) + (fe.val[1] - ce.val[1]) * ds});
    }

    private static double blend(double c, double f, double s) {
        return c + s * (f - c);
    }

    /** (a, b) on the triangle at (u, v), with a = 1 exactly on the edge u = 1. */
    public double[] triangle(double u, double v) {
        double[] ab = triangleValueRaw(u, v);
        if (u == 1.0)
            ab[0] = 1.0;
        return ab;
    }

    /** The unit vector, cube frame, of face {@code face}'s local point (u, v) ∈ [−1, 1]². */
    public double[] forward(CubeLayout.Face face, double u, double v) {
        double au = Math.abs(u), av = Math.abs(v);
        boolean swap = av > au;
        double tu = swap ? av : au;
        double tv = swap ? au : av;
        double[] ab = triangle(tu, tv);
        double a = swap ? ab[1] : ab[0];
        double b = swap ? ab[0] : ab[1];
        double ga = u < 0.0 ? -a : a;
        double gb = v < 0.0 ? -b : b;
        // One division: the reciprocal of r, then products (Rust and JS do the same, bit for bit).
        double ir = 1.0 / Math.sqrt((1.0 + ga * ga) + gb * gb);
        return toCubeFrame(face, new double[]{ir, ga * ir, gb * ir});
    }

    /** The unit vector of (u, v) and its derivatives per unit of u and v, cube frame. */
    public FaceJacobian jacobian(CubeLayout.Face face, double u, double v) {
        double au = Math.abs(u), av = Math.abs(v);
        boolean swap = av > au;
        double tu = swap ? av : au;
        double tv = swap ? au : av;
        Eval e = triangleRaw(tu, tv);
        if (tu == 1.0)
            e.val[0] = 1.0;
        double su = u < 0.0 ? -1.0 : 1.0;
        double sv = v < 0.0 ? -1.0 : 1.0;
        double sw = su * sv;
        // Through the fold: |u|, |v| and the swap.
        double a, b, daDu, daDv, dbDu, dbDv;
        if (swap) {
            a = e.val[1];
            b = e.val[0];
            daDu = e.dv[1];
            daDv = sw * e.du[1];
            dbDu = sw * e.dv[0];
            dbDv = e.du[0];
        } else {
            a = e.val[0];
            b = e.val[1];
            daDu = e.du[0];
            daDv = sw * e.dv[0];
            dbDu = sw * e.du[1];
            dbDv = e.dv[1];
        }
        double ga = su * a;
        double gb = sv * b;
        double ir = 1.0 / Math.sqrt((1.0 + ga * ga) + gb * gb);
        double[] n = {ir, ga * ir, gb * ir};
        return new FaceJacobian(
                toCubeFrame(face, n),
                toCubeFrame(face, dn(n, ga, gb, ir, daDu, dbDu)),
                toCubeFrame(face, dn(n, ga, gb, ir, daDv, dbDv)));
    }

    /** d(g/r) = (dg − n (n·dg)) / r, with n·dg = (ga dga + gb dgb) / r. */
    private static double[] dn(double[] n, double ga, double gb, double ir, double dga, double dgb) {
        double k = (ga * dga + gb * dgb) * ir;
        return new double[]{
                -(n[0] * k) * ir,
                (dga - n[1] * k) * ir,
                (dgb - n[2] * k) * ir};
    }

    /** The inverse table's bilinear start at (a, b) on the gnomonic triangle: {u, v}. */
    private double[] inverseStart(double a, double b) {
        double x = a * (double) q, y = b * (double) q;
        int k = floorClamped(x, q - 1), l = floorClamped(y, q - 1);
        double tx = x - (double) k, ty = y - (double) l;
        int row = q + 1;
        int i00 = 2 * (k * row + l);
        int i10 = 2 * ((k + 1) * row + l);
        int i01 = 2 * (k * row + l + 1);
        int i11 = 2 * ((k + 1) * row + l + 1);
        double u0 = inv[i00] + tx * (inv[i10] - inv[i00]);
        double u1 = inv[i01] + tx * (inv[i11] - inv[i01]);
        double w0 = inv[i00 + 1] + tx * (inv[i10 + 1] - inv[i00 + 1]);
        double w1 = inv[i01 + 1] + tx * (inv[i11 + 1] - inv[i01 + 1]);
        return new double[]{u0 + ty * (u1 - u0), w0 + ty * (w1 - w0)};
    }

    /**
     * (u, v) on the triangle whose forward is (a, b), 0 ≤ b ≤ a ≤ 1: the inverse table's start,
     * {@link #NEWTON_STEPS} Newton steps, then clamped into the triangle and put back on its lines
     * (a = 1 gives u = 1, b = 0 gives v = 0, a = b gives v = u). Plain comparisons, not a clamp
     * call: the order Rust and JS follow.
     */
    public double[] triangleInverse(double a, double b) {
        double[] start = inverseStart(a, b);
        double u = start[0], v = start[1];
        for (int i = 0; i < NEWTON_STEPS; i++) {
            Eval e = triangleRaw(u, v);
            double ra = a - e.val[0], rb = b - e.val[1];
            double det = e.du[0] * e.dv[1] - e.dv[0] * e.du[1];
            double nu = u + (e.dv[1] * ra - e.dv[0] * rb) / det;
            double nv = v + (e.du[0] * rb - e.du[1] * ra) / det;
            u = nu;
            v = nv;
        }
        if (u > 1.0)
            u = 1.0;
        if (u < 0.0)
            u = 0.0;
        if (v > u)
            v = u;
        if (v < 0.0)
            v = 0.0;
        if (a == 1.0)
            u = 1.0;
        if (b == 0.0)
            v = 0.0;
        if (a == b)
            v = u;
        return new double[]{u, v};
    }

    /**
     * The face and local point (u, v) ∈ [−1, 1]² of the cube-frame vector {@code p} (any length &gt;
     * 0). The face is {@link CubeLayout.Face#ofVector}'s (ties to x, then y, then z).
     */
    public CubeLayout.LocalPoint inverse(double[] p) {
        CubeLayout.Face face = CubeLayout.Face.ofVector(p);
        double[][] fr = face.frameRef();
        double[] c = fr[0], eu = fr[1], ev = fr[2];
        double x = (c[0] * p[0] + c[1] * p[1]) + c[2] * p[2];
        double y = (eu[0] * p[0] + eu[1] * p[1]) + eu[2] * p[2];
        double z = (ev[0] * p[0] + ev[1] * p[1]) + ev[2] * p[2];
        double a = y / x, b = z / x;
        double aa = Math.abs(a), bb = Math.abs(b);
        boolean swap = bb > aa;
        double ta = swap ? bb : aa;
        double tb = swap ? aa : bb;
        double[] t = triangleInverse(ta, tb);
        double u = swap ? t[1] : t[0];
        double v = swap ? t[0] : t[1];
        return new CubeLayout.LocalPoint(face, a < 0.0 ? -u : u, b < 0.0 ? -v : v);
    }

    /**
     * The ghosts beyond u = 1 and v = 1 and past the corner (cube_map.py {@code padded}, in its order;
     * ledger: SCL-105).
     */
    private static void padEdgeAndCorner(Grid g, int m) {
        // Beyond u = 1 (j < m): the mirror of node (m − 1, j) across the edge's plane.
        for (int j = 0; j < m; j++) {
            double pa = g.a(m - 1, j), pb = g.b(m - 1, j);
            g.set(m + 1, j, 1.0 / pa, pb / pa);
        }
        // Beyond v = 1 (i < m): the swap of the above.
        for (int i = 0; i < m; i++) {
            double pa = g.a(m + 1, i), pb = g.b(m + 1, i);
            g.set(i, m + 1, pb, pa);
        }
        // Past the corner: linear along the edge, its swap, and linear along the diagonal.
        double mmA = g.a(m, m), mmB = g.b(m, m);
        double m1A = g.a(m, m - 1), m1B = g.b(m, m - 1);
        double dA = g.a(m - 1, m - 1), dB = g.b(m - 1, m - 1);
        double ea = 2.0 * mmA - m1A, eb = 2.0 * mmB - m1B;
        g.set(m, m + 1, ea, eb);
        g.set(m + 1, m, eb, ea);
        g.set(m + 1, m + 1, 2.0 * mmA - dA, 2.0 * mmB - dB);
    }

    /** A face-frame vector (along the centre, e_u, e_v) in the cube frame. */
    private static double[] toCubeFrame(CubeLayout.Face face, double[] w) {
        double[][] fr = face.frameRef();
        double[] c = fr[0], eu = fr[1], ev = fr[2];
        return new double[]{
                (c[0] * w[0] + eu[0] * w[1]) + ev[0] * w[2],
                (c[1] * w[0] + eu[1] * w[1]) + ev[1] * w[2],
                (c[2] * w[0] + eu[2] * w[1]) + ev[2] * w[2]};
    }
}
