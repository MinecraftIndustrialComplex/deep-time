// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/**
 * The river network, {@code rivers/rivers.dtrv} (P1-22; FORMAT.md "Rivers"; Rust
 * {@code deeptime_format::rivers}): chains of vertices (x, z, water surface, half-width, depth,
 * discharge) and a tile index. {@link #sample} answers what the channels decide at one column
 * with Rust's arithmetic, operation for operation. Plain Java, immutable, thread-safe.
 */
public final class Rivers {

    public static final String FILE = "rivers/rivers.dtrv";
    static final int FIXED_BYTES = 48;
    static final int VERTEX_BYTES = 24;
    static final int CHAIN_BYTES = 16;
    /** The smallest half-width a channel has. */
    public static final double MIN_HALF_WIDTH = 0.71;

    public static final int MOUTH_SEA = 0, MOUTH_LAKE = 1, MOUTH_RIVER = 2;

    private final int c, tile, tilesX, tilesZ;
    /** Block x and z of the index's first tile: the world projection's index frame (format 0.5). */
    private final int originX, originZ;
    private final float biomeMinWidth, bankZone, bankRatio;
    /** x, z, w, hw, depth, q per vertex. */
    private final float[] x, z, w, hw, depth, q;
    /** first, count, mouth, into per chain. */
    private final int[] chainFirst, chainCount, chainMouth, chainInto;
    private final int[] offsets, entries;

    private Rivers(int c, int tile, int tilesX, int tilesZ, int originX, int originZ, float biomeMinWidth, float bankZone, float bankRatio,
                   float[] x, float[] z, float[] w, float[] hw, float[] depth, float[] q,
                   int[] chainFirst, int[] chainCount, int[] chainMouth, int[] chainInto, int[] offsets, int[] entries) {
        this.c = c;
        this.tile = tile;
        this.tilesX = tilesX;
        this.tilesZ = tilesZ;
        this.originX = originX;
        this.originZ = originZ;
        this.biomeMinWidth = biomeMinWidth;
        this.bankZone = bankZone;
        this.bankRatio = bankRatio;
        this.x = x;
        this.z = z;
        this.w = w;
        this.hw = hw;
        this.depth = depth;
        this.q = q;
        this.chainFirst = chainFirst;
        this.chainCount = chainCount;
        this.chainMouth = chainMouth;
        this.chainInto = chainInto;
        this.offsets = offsets;
        this.entries = entries;
    }

    /** Reads {@code rivers/} of {@code folder}; null when the folder has none (worlds before P1-22). */
    public static Rivers read(Path folder) {
        Path path = folder.resolve(FILE);
        if (!Files.isRegularFile(path))
            return null;
        Sectioned s = Sectioned.read(path, "DTRV", FIXED_BYTES, f -> 4);
        ByteBuffer f = s.fixed();
        int c = f.getInt(0), tile = f.getInt(4), tilesX = f.getInt(8), tilesZ = f.getInt(12);
        int nv = f.getInt(16), nc = f.getInt(20), ne = f.getInt(24);
        ByteBuffer vs = s.section(0), cs = s.section(1), os = s.section(2), es = s.section(3);
        long tiles = (long) tilesX * tilesZ;
        if (tile <= 0 || tilesX <= 0 || tilesZ <= 0 || nv < 0 || nc < 0 || ne < 0
                || vs.capacity() != (long) nv * VERTEX_BYTES || cs.capacity() != (long) nc * CHAIN_BYTES
                || os.capacity() != (tiles + 1) * 4 || es.capacity() != (long) ne * 4)
            throw Sectioned.corrupt(path, "a section has the wrong length");
        float[] x = new float[nv], z = new float[nv], w = new float[nv], hw = new float[nv], d = new float[nv], q = new float[nv];
        for (int i = 0; i < nv; i++) {
            int a = i * VERTEX_BYTES;
            x[i] = vs.getFloat(a);
            z[i] = vs.getFloat(a + 4);
            w[i] = vs.getFloat(a + 8);
            hw[i] = vs.getFloat(a + 12);
            d[i] = vs.getFloat(a + 16);
            q[i] = vs.getFloat(a + 20);
        }
        int[] first = new int[nc], count = new int[nc], mouth = new int[nc], into = new int[nc];
        int next = 0;
        for (int k = 0; k < nc; k++) {
            int a = k * CHAIN_BYTES;
            first[k] = cs.getInt(a);
            count[k] = cs.getInt(a + 4);
            mouth[k] = cs.get(a + 8) & 0xFF;
            into[k] = cs.getInt(a + 12);
            if (first[k] != next || count[k] < 2 || mouth[k] > MOUTH_RIVER)
                throw Sectioned.corrupt(path, "chain " + k + " is out of order, short or has an unknown mouth");
            next = first[k] + count[k];
        }
        if (next != nv)
            throw Sectioned.corrupt(path, "the chains do not cover the vertices");
        int[] offsets = new int[(int) tiles + 1];
        for (int t = 0; t <= tiles; t++)
            offsets[t] = os.getInt(t * 4);
        int[] entries = new int[ne];
        for (int i = 0; i < ne; i++) {
            entries[i] = es.getInt(i * 4);
            if (entries[i] < 0 || entries[i] + 1 >= nv)
                throw Sectioned.corrupt(path, "an index entry is past the vertices");
        }
        if (offsets[0] != 0 || offsets[(int) tiles] != ne)
            throw Sectioned.corrupt(path, "the tile offsets do not cover the entries");
        for (int t = 0; t < tiles; t++)
            if (offsets[t] > offsets[t + 1])
                throw Sectioned.corrupt(path, "the tile offsets decrease");
        return new Rivers(c, tile, tilesX, tilesZ, f.getInt(40), f.getInt(44), f.getFloat(28), f.getFloat(32), f.getFloat(36),
                x, z, w, hw, d, q, first, count, mouth, into, offsets, entries);
    }

    public int circumference() {
        return c;
    }

    public int vertexCount() {
        return x.length;
    }

    public int chainCount() {
        return chainFirst.length;
    }

    public float vertexX(int i) { return x[i]; }
    public float vertexZ(int i) { return z[i]; }
    public float vertexW(int i) { return w[i]; }
    public float vertexHalfWidth(int i) { return hw[i]; }
    public int chainFirst(int k) { return chainFirst[k]; }
    public int chainLength(int k) { return chainCount[k]; }
    public int chainMouth(int k) { return chainMouth[k]; }
    public int chainInto(int k) { return chainInto[k]; }

    /** The tile of canonical column (wx, z). */
    int tileOf(long wx, long z) {
        // The tiles' frame is the world projection's index frame, as the file states it.
        long tx = Math.floorMod(Math.floorDiv(wx - originX, (long) tile), (long) tilesX);
        long tz = Math.max(0, Math.min(tilesZ - 1L, Math.floorDiv(z - originZ, (long) tile)));
        return (int) (tz * tilesX + tx);
    }

    /** The segments listed in the tile of canonical column (wx, z). */
    int[] tileEntries(long wx, long z) {
        int t = tileOf(wx, z);
        return Arrays.copyOfRange(entries, offsets[t], offsets[t + 1]);
    }

    /**
     * The segment ids of every tile touching block box [x0, x1] × [z0, z1], ascending, without
     * repeats (Rust {@code Rivers::entries_in}).
     */
    public int[] entriesIn(long x0, long z0, long x1, long z1) {
        long ox = originX, oz = originZ;
        long tx0 = Math.floorDiv(x0 - ox, (long) tile), tx1 = Math.floorDiv(x1 - ox, (long) tile);
        long tz0 = Math.max(0, Math.min(tilesZ - 1L, Math.floorDiv(z0 - oz, (long) tile)));
        long tz1 = Math.max(0, Math.min(tilesZ - 1L, Math.floorDiv(z1 - oz, (long) tile)));
        long span = Math.min(tx1 - tx0, tilesX - 1L);
        int n = 0;
        for (long tz = tz0; tz <= tz1; tz++)
            for (long dx = 0; dx <= span; dx++) {
                int t = (int) (tz * tilesX + Math.floorMod(tx0 + dx, (long) tilesX));
                n += offsets[t + 1] - offsets[t];
            }
        int[] out = new int[n];
        int at = 0;
        for (long tz = tz0; tz <= tz1; tz++)
            for (long dx = 0; dx <= span; dx++) {
                int t = (int) (tz * tilesX + Math.floorMod(tx0 + dx, (long) tilesX));
                System.arraycopy(entries, offsets[t], out, at, offsets[t + 1] - offsets[t]);
                at += offsets[t + 1] - offsets[t];
            }
        Arrays.sort(out);
        int k = 0;
        for (int i = 0; i < out.length; i++)
            if (k == 0 || out[i] != out[k - 1])
                out[k++] = out[i];
        return Arrays.copyOf(out, k);
    }

    /** A channel over one column (Rust {@code RiverHit}). */
    public record Hit(int seg, double t, double d, double w, double hw, double depth, double q) {}

    /**
     * What the channels decide at one column (Rust {@code RiverSample}): the covering segment
     * with the lowest surface (ties: lowest id), the lowest bank cap of the segments whose bank
     * zone (but not channel) holds it, and whether a covering channel is wide enough for a river
     * biome.
     */
    public record Sample(Hit hit, Integer bankCap, boolean biome) {
        public static final Sample NONE = new Sample(null, null, false);
    }

    /** What the channels decide at column (x, z), any lap, from its tile's segments. */
    public Sample sample(long x, long z) {
        long wx = Math.floorMod(x + c / 2, (long) c) - c / 2;
        int t = tileOf(wx, z);
        return sampleRange(entries, offsets[t], offsets[t + 1], wx, z);
    }

    private static double lerp(float a, float b, double t) {
        return (double) a + ((double) b - (double) a) * t;
    }

    /** As {@link #sample} over any superset of the column's tile entries; {@code wx} canonical. */
    public Sample sampleAmong(int[] segs, long wx, long z) {
        return sampleRange(segs, 0, segs.length, wx, z);
    }

    /**
     * How far column (qx, qz), any lap, lies outside the nearest channel that comes within
     * {@code reach} blocks of it: the distance to the segment's centreline less its half-width (at
     * least {@link #MIN_HALF_WIDTH}, as {@link #sample} counts it), so 0 or less inside a channel;
     * {@code +∞} when no channel comes that close. A game-side rule ({@code WaterCaveShield}), not
     * part of the column rule Rust mirrors.
     */
    public double edgeDistance(long qx, long qz, double reach) {
        long wx = Math.floorMod(qx + c / 2, (long) c) - c / 2;
        long r = (long) Math.ceil(Math.max(0.0, reach));
        // A channel edge within reach lies in the box; the tiles there list the segment (the index
        // lists a segment in every tile its channel covers, or sample() would miss it).
        int[] segs = entriesIn(wx - r, qz - r, wx + r, qz + r);
        double cc = c, best = Double.POSITIVE_INFINITY;
        for (int s : segs) {
            double ax = x[s], az = this.z[s], bx = x[s + 1], bz = this.z[s + 1];
            double k = Math.floor((ax - wx) / cc + 0.5);
            double px = wx + k * cc, pz = qz;
            double ex = bx - ax, ez = bz - az;
            double l2 = ex * ex + ez * ez;
            double t = l2 > 0.0 ? ((px - ax) * ex + (pz - az) * ez) / l2 : 0.0;
            t = t < 0.0 ? 0.0 : (t > 1.0 ? 1.0 : t);
            double dx = px - (ax + t * ex), dz = pz - (az + t * ez);
            double e = Math.sqrt(dx * dx + dz * dz) - Math.max(lerp(hw[s], hw[s + 1], t), MIN_HALF_WIDTH);
            if (e < best)
                best = e;
        }
        return best <= reach ? best : Double.POSITIVE_INFINITY;
    }

    private Sample sampleRange(int[] segs, int from, int to, long wx, long z) {
        double cc = c;
        double px0 = wx, pz = z;
        Hit best = null;
        Integer cap = null;
        boolean biome = false;
        for (int i = from; i < to; i++) {
            int s = segs[i];
            double ax = x[s], az = this.z[s], bx = x[s + 1], bz = this.z[s + 1];
            double k = Math.floor((ax - px0) / cc + 0.5);
            double px = px0 + k * cc;
            double ex = bx - ax, ez = bz - az;
            double l2 = ex * ex + ez * ez;
            double t = l2 > 0.0 ? ((px - ax) * ex + (pz - az) * ez) / l2 : 0.0;
            t = t < 0.0 ? 0.0 : (t > 1.0 ? 1.0 : t);
            double dx = px - (ax + t * ex);
            double dz = pz - (az + t * ez);
            double d = Math.sqrt(dx * dx + dz * dz);
            double h = lerp(hw[s], hw[s + 1], t);
            double hEff = Math.max(h, MIN_HALF_WIDTH);
            if (d <= hEff) {
                double wv = lerp(w[s], w[s + 1], t);
                if (best == null || wv < best.w() || (wv == best.w() && s < best.seg()))
                    best = new Hit(s, t, d, wv, hEff, lerp(depth[s], depth[s + 1], t), lerp(q[s], q[s + 1], t));
                if (2.0 * h >= (double) biomeMinWidth)
                    biome = true;
            } else if (d <= hEff + (double) bankZone + (double) bankRatio * h) {
                double wv = lerp(w[s], w[s + 1], t);
                int b = (int) Math.ceil(wv) + (int) Math.floor(d - hEff);
                cap = cap == null ? b : Math.min(cap, b);
            }
        }
        return best == null && cap == null && !biome ? Sample.NONE : new Sample(best, cap, biome);
    }
}
