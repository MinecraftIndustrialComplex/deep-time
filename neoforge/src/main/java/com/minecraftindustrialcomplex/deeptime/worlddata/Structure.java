// SPDX-License-Identifier: LGPL-3.0-or-later
//
// profileAt is Noddy's calcProfile (Loop3D/noddyverse, commit 6a21b23, src/proflib.c; MIT License,
// Copyright (c) 2021 Loop3D), as ported in crates/deeptime-noddy. See NOTICE.md.
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;

/**
 * {@code structure/events.dtst} (plan §8.5, P2-01; FORMAT.md "Structural events"; Rust
 * {@code deeptime_format::structure}): the world's folds and faults, and the deformed column rule
 * that reads a column's horizon stack through them. Rust's {@code Structure::deformed_stack} is its
 * twin, operation for operation ({@code ColumnParityTest}).
 *
 * <p>An event's strength at a block column is a window weight that depends on the map position
 * only; a fold shifts the column vertically by its profile, a fault moves the part of the column
 * above its plane by one vector, an intrusion replaces the rock inside its ellipsoid. Events are
 * undone youngest first; rock younger than an event (horizons whose unit key is below the event's
 * key) is not touched by it.
 *
 * <p>Arithmetic: IEEE doubles, {@code + − × /}, comparisons, {@code floor} and {@code ceil}, in the
 * order written; nothing here calls a library function whose result could differ from Rust's.
 * Immutable and thread-safe; the working memory is a {@link Scratch} per thread.
 */
public final class Structure {

    public static final String FILE = "structure/events.dtst";
    static final int EVENT_BYTES = 136;
    /** Samples of a fold profile over one wavelength. */
    static final int PROFILE_SAMPLES = 629;
    /** 2π as Noddy writes it: part of the contract. */
    static final double NODDY_TAU = 6.2831853; // ledger: DEF-066
    /** Side of the index's buckets, blocks. */
    private static final double BUCKET = 128.0;

    private final int n;
    private final int[] kind, profile, key;
    /** Window: centre (x, z), along (x, z), across (x, z), half-lengths. */
    private final double[] cx, cz, alx, alz, acx, acz, halfAlong, halfAcross;
    /** Fold: wavelength, amplitude, offset. Fault: y, pole x y z, slip x y z. Intrusion: y, half-height (its class is {@code profile}). */
    private final double[][] tail;
    private final float[][] profiles;
    private final double margin;
    private final double c;
    /** The buckets' origin (the world projection's index frame). */
    private final double ox, oz;
    private final int w, h;
    private final double[] radius;
    private final int[][] buckets;

    private Structure(WorldProjection frame, int n, int[] kind, int[] profile, int[] key, double[][] win, double[][] tail, float[][] profiles,
                      double margin, int circumference) {
        this.n = n;
        this.kind = kind;
        this.profile = profile;
        this.key = key;
        this.cx = win[0];
        this.cz = win[1];
        this.alx = win[2];
        this.alz = win[3];
        this.acx = win[4];
        this.acz = win[5];
        this.halfAlong = win[6];
        this.halfAcross = win[7];
        this.tail = tail;
        this.profiles = profiles;
        this.margin = margin;
        this.c = circumference;
        this.ox = frame.indexOriginBlockX();
        this.oz = frame.indexOriginBlockZ();
        this.w = Math.max(1, (int) Math.ceil(c / BUCKET));
        this.h = Math.max(1, (int) Math.ceil((double) frame.indexSpanBlockZ() / BUCKET));
        this.radius = new double[n];
        int[] counts = new int[w * h];
        int[][] lists = new int[w * h][];
        for (int pass = 0; pass < 2; pass++) {
            for (int e = 0; e < n; e++) {
                double r = Math.sqrt(halfAlong[e] * halfAlong[e] + halfAcross[e] * halfAcross[e]) + margin;
                radius[e] = r;
                // The buckets' frame is the world projection's index frame (fields ox, oz).
                long j0 = (long) Math.floor((cz[e] - r - oz) / BUCKET);
                long j1 = (long) Math.floor((cz[e] + r - oz) / BUCKET);
                long i0 = (long) Math.floor((cx[e] - r - ox) / BUCKET);
                long i1 = (long) Math.floor((cx[e] + r - ox) / BUCKET);
                long across = Math.min(i1 - i0 + 1, w);
                for (long j = Math.max(j0, 0); j <= Math.min(j1, h - 1L); j++) {
                    for (long di = 0; di < across; di++) {
                        int b = (int) (j * w + Math.floorMod(i0 + di, (long) w));
                        if (pass == 1)
                            lists[b][counts[b]] = e;
                        counts[b]++;
                    }
                }
            }
            if (pass == 0) {
                for (int b = 0; b < lists.length; b++)
                    lists[b] = new int[counts[b]];
                Arrays.fill(counts, 0);
            }
        }
        this.buckets = lists;
    }

    /** Reads {@code structure/} of a Lambert world-data folder; null when the world has no structural events. */
    public static Structure read(Path folder, int circumference) {
        return read(folder, circumference, new WorldProjection.Lambert(circumference));
    }

    /**
     * Reads {@code structure/} of {@code folder}, its index over the frame of the world's projection
     * {@code frame} (Rust {@code Deformer::read_in}); null when the world has no structural events.
     */
    public static Structure read(Path folder, int circumference, WorldProjection frame) {
        Path path = folder.resolve(FILE);
        if (!Files.isRegularFile(path))
            return null;
        Sectioned s = Sectioned.read(path, "DTST", 16, f -> 2);
        ByteBuffer f = s.fixed();
        int n = f.getInt(0), np = f.getInt(4);
        double margin = f.getDouble(8);
        ByteBuffer recs = s.section(0), profs = s.section(1);
        if (n < 0 || np < 0 || recs.capacity() != n * EVENT_BYTES || profs.capacity() != np * PROFILE_SAMPLES * 4)
            throw Sectioned.corrupt(path, "event or profile section has the wrong length");
        if (!(margin >= 0.0) || Double.isInfinite(margin))
            throw Sectioned.corrupt(path, "margin " + margin);
        if (n == 0)
            return null;
        float[][] profiles = new float[np][PROFILE_SAMPLES];
        for (int k = 0; k < np; k++)
            for (int i = 0; i < PROFILE_SAMPLES; i++)
                profiles[k][i] = profs.getFloat((k * PROFILE_SAMPLES + i) * 4);
        int[] kind = new int[n], profile = new int[n], key = new int[n];
        double[][] win = new double[8][n];
        double[][] tail = new double[n][7];
        for (int e = 0; e < n; e++) {
            int a = e * EVENT_BYTES;
            int id = recs.getInt(a);
            kind[e] = recs.get(a + 4) & 0xFF;
            profile[e] = recs.get(a + 5) & 0xFF;
            key[e] = recs.getShort(a + 6) & 0xFFFF;
            for (int k = 0; k < 16; k++) {
                double v = recs.getDouble(a + 8 + 8 * k);
                if (Double.isNaN(v) || Double.isInfinite(v))
                    throw Sectioned.corrupt(path, "event " + id + ": a value is not finite");
                if (k >= 1 && k <= 8)
                    win[k - 1][e] = v;
                else if (k >= 9)
                    tail[e][k - 9] = v;
            }
            if (!(win[6][e] > 0.0 && win[7][e] > 0.0))
                throw Sectioned.corrupt(path, "event " + id + ": an empty window");
            if (kind[e] == 0) {
                if (profile[e] >= np || tail[e][0] == 0.0)
                    throw Sectioned.corrupt(path, "event " + id + ": fold profile or wavelength");
            } else if (kind[e] == 1) {
                if (!(tail[e][2] > 0.0))
                    throw Sectioned.corrupt(path, "event " + id + ": the pole must point up");
            } else if (kind[e] == 2) {
                if (!(tail[e][1] > 0.0))
                    throw Sectioned.corrupt(path, "event " + id + ": an intrusion of no height");
            } else {
                throw Sectioned.corrupt(path, "event " + id + ": kind " + kind[e]);
            }
            if (e > 0 && key[e - 1] > key[e])
                throw Sectioned.corrupt(path, "events are not youngest first");
        }
        return new Structure(frame, n, kind, profile, key, win, tail, profiles, margin, circumference);
    }

    /** Events in the world. */
    public int size() {
        return n;
    }

    /** {@code (1 − t²)²} for |t| &lt; 1, 0 outside (Rust {@code structure::bump}). Ledger: DEF-039. */
    static double bump(double t) {
        if (t > -1.0 && t < 1.0) {
            double u = 1.0 - t * t;
            return u * u;
        }
        return 0.0;
    }

    /** Rust {@code structure::profile_at} (Noddy's {@code calcProfile}). */
    static double profileAt(float[] table, double position) {
        final long res = 628; // ledger: DEF-068
        double percent;
        long i1, i2;
        if (position < 0.0) {
            double d = (-position / NODDY_TAU) * 628.0;
            double f = Math.floor(d);
            percent = d - f;
            i1 = res - ((long) f) % res;
            i2 = res - ((long) Math.ceil(d)) % res;
        } else {
            double d = (position / NODDY_TAU) * 628.0;
            double f = Math.floor(d);
            percent = d - f;
            i1 = ((long) f) % res;
            i2 = ((long) Math.ceil(d)) % res;
        }
        i1 = Math.max(0, Math.min(res - 1, i1));
        i2 = Math.max(0, Math.min(res - 1, i2));
        double v1 = (double) table[(int) i1] / 100.0;
        double v2 = (double) table[(int) i2] / 100.0;
        return v1 + percent * (v2 - v1);
    }

    /** Per thread: the working memory of {@link #deformedStack}. */
    public static final class Scratch {
        int[] cands = new int[64];
        int nc;
        // Pieces, bottom-up.
        double[] lo = new double[16], hi = new double[16], sx = new double[16], sz = new double[16], dy = new double[16];
        int[] pkey = new int[16], src = new int[16];
        /** An intrusion's rock class, or -1: the piece reads its source stack. Its unit key is {@code pkey}. */
        int[] prock = new int[16];
        boolean[] done = new boolean[16];
        int np;
        // The stacks of the source columns met.
        /**
         * Whether source columns are read as their nearest strata cell's stored stack
         * ({@link FarColumn#cellStack}: far terrain's wide columns) instead of by the column rule.
         */
        boolean cell;
        Columns.Stack[] stacks = new Columns.Stack[4];
        int[] stackX = new int[4], stackZ = new int[4];
        int ns;

        private void grow() {
            int cap = 2 * lo.length;
            lo = Arrays.copyOf(lo, cap);
            hi = Arrays.copyOf(hi, cap);
            sx = Arrays.copyOf(sx, cap);
            sz = Arrays.copyOf(sz, cap);
            dy = Arrays.copyOf(dy, cap);
            pkey = Arrays.copyOf(pkey, cap);
            src = Arrays.copyOf(src, cap);
            prock = Arrays.copyOf(prock, cap);
            done = Arrays.copyOf(done, cap);
        }

        /** Inserts a copy of piece {@code from} at {@code at}. */
        private void insertCopy(int at, int from) {
            if (np == lo.length)
                grow();
            for (int k = np; k > at; k--)
                copy(k - 1, k);
            np++;
            if (from >= at)
                from++;
            copy(from, at);
        }

        private void copy(int a, int b) {
            lo[b] = lo[a];
            hi[b] = hi[a];
            sx[b] = sx[a];
            sz[b] = sz[a];
            dy[b] = dy[a];
            pkey[b] = pkey[a];
            src[b] = src[a];
            prock[b] = prock[a];
            done[b] = done[a];
        }

        /** The scratch index of the stack of block column (x, z), read when new. */
        private int source(WorldData wd, RasterSampler strata, int x, int z) {
            for (int i = 0; i < ns; i++)
                if (stackX[i] == x && stackZ[i] == z)
                    return i;
            if (ns == stacks.length) {
                stacks = Arrays.copyOf(stacks, 2 * ns);
                stackX = Arrays.copyOf(stackX, 2 * ns);
                stackZ = Arrays.copyOf(stackZ, 2 * ns);
            }
            if (stacks[ns] == null)
                stacks[ns] = new Columns.Stack(strata.stackDepth());
            if (cell)
                FarColumn.cellStack(strata, x, z, stacks[ns]);
            else
                Columns.stack(wd, strata, x, z, stacks[ns]);
            stackX[ns] = x;
            stackZ[ns] = z;
            return ns++;
        }
    }

    /** The candidates of the column at block position (x, z), youngest first (Rust {@code StructIndex::candidates}). */
    private void candidates(double x, double z, Scratch s) {
        s.nc = 0;
        int i = (int) Math.floorMod((long) Math.floor((x - ox) / BUCKET), (long) w);
        long jl = (long) Math.floor((z - oz) / BUCKET);
        int j = (int) Math.max(0, Math.min(h - 1L, jl));
        for (int e : buckets[j * w + i]) {
            double dx = x - cx[e];
            dx -= Math.floor(dx / c + 0.5) * c;
            double dz = z - cz[e];
            double r = radius[e];
            if (dx >= -r && dx <= r && dz >= -r && dz <= r) {
                if (s.nc == s.cands.length)
                    s.cands = Arrays.copyOf(s.cands, 2 * s.nc);
                s.cands[s.nc++] = e;
            }
        }
    }

    /**
     * Fills {@code out} with the horizon stack of block column (x, z) read through the world's folds
     * and faults: Rust {@code Structure::deformed_stack}, which documents the rule.
     */
    public void deformedStack(WorldData wd, RasterSampler strata, int x, int z, Columns.Stack out, Scratch s) {
        deformedStack(wd, strata, x, z, out, s, false);
    }

    /**
     * As {@link #deformedStack(WorldData, RasterSampler, int, int, Columns.Stack, Scratch)}; with
     * {@code cell} the undeformed stacks the rule reads (the column's own and every fault's source
     * column) are their nearest strata cells' stored stacks, not the column rule's: the same events,
     * undone the same way, over the cheap stacks far terrain's wide columns use.
     */
    public void deformedStack(WorldData wd, RasterSampler strata, int x, int z, Columns.Stack out, Scratch s,
                              boolean cell) {
        s.cell = cell;
        out.count = 0;
        s.ns = 0;
        s.np = 0;
        int minY = wd.manifest().minY();
        double fx = x, fz = z;
        candidates(fx, fz, s);
        int own = s.source(wd, strata, x, z);
        Columns.Stack ownStack = s.stacks[own];
        if (s.nc == 0 || ownStack.count == 0) {
            for (int h = 0; h < ownStack.count; h++)
                out.push(ownStack.tops[h], ownStack.classes[h], ownStack.units[h]);
            return;
        }
        s.lo[0] = Double.NEGATIVE_INFINITY;
        s.hi[0] = Double.POSITIVE_INFINITY;
        s.sx[0] = fx;
        s.sz[0] = fz;
        s.dy[0] = 0.0;
        s.pkey[0] = 0;
        s.done[0] = false;
        s.src[0] = own;
        s.prock[0] = -1;
        s.np = 1;
        for (int ci = 0; ci < s.nc; ci++) {
            int e = s.cands[ci];
            int k = 0;
            while (k < s.np) {
                if (s.done[k]) {
                    k++;
                    continue;
                }
                double dx0 = s.sx[k] - cx[e];
                dx0 -= Math.floor(dx0 / c + 0.5) * c;
                double dz0 = s.sz[k] - cz[e];
                double a = dx0 * alx[e] + dz0 * alz[e];
                double b = dx0 * acx[e] + dz0 * acz[e];
                double wa = bump(a / halfAlong[e]);
                double wgt = wa == 0.0 ? 0.0 : wa * bump(b / halfAcross[e]);
                if (wgt == 0.0) {
                    k++;
                    continue;
                }
                // Cover: what the event is too old to have moved.
                Columns.Stack st = s.stacks[s.src[k]];
                int hc = -1;
                for (int q = 0; q < st.count; q++)
                    if (st.units[q] >= key[e]) {
                        hc = q;
                        break;
                    }
                if (hc < 0) {
                    s.done[k] = true;
                    k++;
                    continue;
                }
                int next = k + 1;
                if (hc > 0) {
                    double cut = (double) (st.tops[hc] + 1) - s.dy[k];
                    if (cut <= s.lo[k]) {
                        s.done[k] = true;
                        k++;
                        continue;
                    }
                    if (cut < s.hi[k]) {
                        s.insertCopy(k + 1, k);
                        s.hi[k] = cut;
                        s.lo[k + 1] = cut;
                        s.done[k + 1] = true;
                        next++;
                    }
                }
                double[] t = tail[e];
                if (kind[e] == 0) {
                    double phase = (b - t[2]) * NODDY_TAU / t[0];
                    double v = profileAt(profiles[profile[e]], phase);
                    s.dy[k] = s.dy[k] - wgt * (t[1] * v);
                    s.pkey[k] = key[e];
                } else if (kind[e] == 2) {
                    double ua = a / halfAlong[e], ub = b / halfAcross[e];
                    double r2 = ua * ua + ub * ub;
                    if (r2 < 1.0) {
                        double hh = t[1] * Math.sqrt(1.0 - r2);
                        double plo = s.lo[k], phi = s.hi[k];
                        double lo = (t[0] - hh) - s.dy[k];
                        double hi = (t[0] + hh) - s.dy[k];
                        if (!(lo > plo))
                            lo = plo;
                        if (!(hi < phi))
                            hi = phi;
                        if (lo < hi) {
                            int at = k;
                            if (lo > plo) {
                                s.insertCopy(k + 1, k);
                                s.hi[k] = lo;
                                at = k + 1;
                                next++;
                            }
                            if (hi < phi) {
                                // The rest of the piece above the body, as it was.
                                s.insertCopy(at + 1, at);
                                s.lo[at + 1] = hi;
                                s.hi[at + 1] = phi;
                                next++;
                            }
                            s.lo[at] = lo;
                            s.hi[at] = hi;
                            s.done[at] = true;
                            s.prock[at] = profile[e];
                            s.pkey[at] = key[e];
                        }
                    }
                } else {
                    double dx = a * alx[e] + b * acx[e];
                    double dz = a * alz[e] + b * acz[e];
                    double plane = t[0] - (dx * t[1] + dz * t[3]) / t[2];
                    double cut = plane - s.dy[k];
                    if (cut < s.hi[k]) {
                        double nsx = s.sx[k] - wgt * t[4], nsz = s.sz[k] - wgt * t[6];
                        double ndy = s.dy[k] - wgt * t[5];
                        int nsrc = s.source(wd, strata, (int) (long) Math.floor(nsx), (int) (long) Math.floor(nsz));
                        int at = k;
                        if (cut > s.lo[k]) {
                            s.insertCopy(k + 1, k);
                            s.hi[k] = cut;
                            at = k + 1;
                            s.lo[at] = cut;
                            next++;
                        }
                        s.sx[at] = nsx;
                        s.sz[at] = nsz;
                        s.dy[at] = ndy;
                        s.pkey[at] = key[e];
                        s.done[at] = false;
                        s.src[at] = nsrc;
                    }
                }
                k = next;
            }
        }
        ownStack = s.stacks[own];
        int topLimit = ownStack.tops[0];
        for (int k = s.np - 1; k >= 0; k--) {
            int yMax = s.hi[k] == Double.POSITIVE_INFINITY ? topLimit
                    : Math.min(topLimit, (int) Math.ceil(s.hi[k] - 0.5) - 1);
            int yMin = s.lo[k] == Double.NEGATIVE_INFINITY ? minY : Math.max(minY, (int) Math.ceil(s.lo[k] - 0.5));
            if (yMax < yMin)
                continue;
            if (s.prock[k] >= 0) {
                if (out.count == 0 || out.classes[out.count - 1] != s.prock[k])
                    out.push(yMax, s.prock[k], s.pkey[k]);
                continue;
            }
            int d = (int) Math.floor(s.dy[k] + 0.5);
            Columns.Stack st = s.stacks[s.src[k]];
            int cnt = st.count;
            for (int hh = 0; hh < cnt; hh++) {
                int top = hh == 0 ? yMax : Math.min(yMax, st.tops[hh] - d);
                int bottom = hh + 1 < cnt ? Math.max(yMin, st.tops[hh + 1] + 1 - d) : yMin;
                if (top < bottom)
                    continue;
                // Rock younger than the event that moved the piece: the cover's lowest bed stands in.
                int g = hh;
                if (st.units[g] < s.pkey[k])
                    while (g + 1 < cnt && st.units[g + 1] < s.pkey[k])
                        g++;
                if (out.count == 0 || out.classes[out.count - 1] != st.classes[g])
                    out.push(top, st.classes[g], st.units[g]);
            }
        }
    }
}
