// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/**
 * The site registry of a world-data folder (plan §5.6): {@code sites/sites.dtsr} and
 * {@code sites/index.dtsi}, byte layout in FORMAT.md ("Sites"), Rust {@code deeptime_format::sites}.
 * Plain Java (no Minecraft types), immutable and thread-safe; the game-facing API is
 * {@code api.DeepTimeSites}.
 */
public final class SiteData {

    public static final String SITES = "sites/sites.dtsr";
    public static final String INDEX = "sites/index.dtsi";
    static final int RECORD_BYTES = 104;
    static final int PRODUCT_BYTES = 24;
    /** "None" for u32 references. */
    public static final int NONE = -1;
    public static final long NONE_U64 = -1L;

    /** Site kinds, by their on-disk code. */
    public static final List<String> KINDS = List.of("deposit", "reservoir", "placer", "supergene", "fantasy");

    /** One product of a site. */
    public record ProductRecord(String material, float grade, double tonnageT, String role) {}

    /**
     * One site record. {@code body} is the first body id or {@link #NONE}; {@code parent} likewise;
     * {@code cdgChargeMb} is {@link #NONE_U64} for none; {@code min}/{@code max} the inclusive AABB
     * whose x may leave the canonical span.
     */
    public record SiteRecord(int id, int kind, String depositType, int body, int[] center, float lat, float lon,
                             int[] min, int[] max, int parent, long formingEvent, List<ProductRecord> products,
                             double amountBase, String coeVein, long cdgChargeMb, boolean finite) {

        public String kindName() {
            return KINDS.get(kind);
        }

        /** True when block (x, z) lies in the AABB's footprint on a planet of circumference c (x wraps). */
        public boolean footprintContains(int c, int x, int z) {
            if (z < min[2] || z > max[2])
                return false;
            long span = (long) max[0] - min[0];
            if (span >= c - 1L)
                return true;
            return Math.floorMod((long) x - min[0], (long) c) <= span;
        }

        /** True when block (x, y, z) lies in the AABB (x wraps). */
        public boolean aabbContains(int c, int x, int y, int z) {
            return y >= min[1] && y <= max[1] && footprintContains(c, x, z);
        }
    }

    private final List<SiteRecord> sites;
    private final int circumference;
    private final int cellBlocks, cols, rows, originX, originZ;
    private final int[] offsets, ids;

    private SiteData(List<SiteRecord> sites, int circumference, int cellBlocks, int cols, int rows, int originX,
                     int originZ, int[] offsets, int[] ids) {
        this.sites = sites;
        this.circumference = circumference;
        this.cellBlocks = cellBlocks;
        this.cols = cols;
        this.rows = rows;
        this.originX = originX;
        this.originZ = originZ;
        this.offsets = offsets;
        this.ids = ids;
    }

    /** A registry with no sites (a world without {@code sites/}). */
    public static SiteData empty(int circumference) {
        return new SiteData(List.of(), circumference, 512, 1, 1,
                (int) Projection.indexOriginBlockX(circumference), (int) Projection.indexOriginBlockZ(circumference),
                new int[]{0, 0}, new int[0]);
    }

    /** Reads {@code sites/} of {@code folder}; an empty registry when the folder has none. */
    public static SiteData read(Path folder, int circumference) {
        Path recs = folder.resolve(SITES), idx = folder.resolve(INDEX);
        if (!Files.isRegularFile(recs))
            return empty(circumference);
        List<SiteRecord> sites = readRecords(recs);
        Sectioned ix = Sectioned.read(idx, "DTSI", 32, f -> 2);
        ByteBuffer f = ix.fixed();
        int c = f.getInt(0), cb = f.getInt(4), cols = f.getInt(8), rows = f.getInt(12);
        int ox = f.getInt(16), oz = f.getInt(20), entries = f.getInt(24);
        if (c != circumference)
            throw new WorldDataException(idx + ": index circumference " + c + " != world circumference " + circumference);
        if (cb <= 0 || cols <= 0 || rows <= 0)
            throw Sectioned.corrupt(idx, "empty index grid");
        int cells = cols * rows;
        ByteBuffer so = ix.section(0), si = ix.section(1);
        if (so.capacity() != (cells + 1) * 4 || si.capacity() != entries * 4)
            throw Sectioned.corrupt(idx, "index sections have the wrong length");
        int[] offsets = new int[cells + 1];
        for (int i = 0; i <= cells; i++) {
            offsets[i] = so.getInt(i * 4);
            if (i > 0 && Integer.compareUnsigned(offsets[i - 1], offsets[i]) > 0)
                throw Sectioned.corrupt(idx, "index offsets are not monotonic");
        }
        if (offsets[0] != 0 || offsets[cells] != entries)
            throw Sectioned.corrupt(idx, "index offsets are not monotonic");
        int[] ids = new int[entries];
        for (int i = 0; i < entries; i++)
            ids[i] = si.getInt(i * 4);
        return new SiteData(sites, circumference, cb, cols, rows, ox, oz, offsets, ids);
    }

    static List<SiteRecord> readRecords(Path path) {
        Sectioned s = Sectioned.read(path, "DTSR", 16, f -> 3);
        ByteBuffer f = s.fixed();
        int n = f.getInt(0), ns = f.getInt(4), np = f.getInt(8);
        ByteBuffer recs = s.section(1), prods = s.section(2);
        if (recs.capacity() != n * RECORD_BYTES || prods.capacity() != np * PRODUCT_BYTES)
            throw Sectioned.corrupt(path, "record or product section has the wrong length");
        List<String> strings = s.strings(0, ns);
        List<SiteRecord> out = new ArrayList<>(n);
        for (int r = 0; r < n; r++) {
            int a = r * RECORD_BYTES;
            int id = recs.getInt(a);
            int kind = recs.get(a + 4) & 0xFF;
            if (kind >= KINDS.size())
                throw Sectioned.corrupt(path, "site " + Integer.toUnsignedString(id) + ": kind " + kind);
            int pf = recs.getInt(a + 72), pn = recs.getInt(a + 76);
            if (pf < 0 || pn < 0 || (long) (pf + pn) * PRODUCT_BYTES > prods.capacity())
                throw Sectioned.corrupt(path, "site " + id + ": products past the section");
            List<ProductRecord> products = new ArrayList<>(pn);
            for (int i = pf; i < pf + pn; i++) {
                int e = i * PRODUCT_BYTES;
                products.add(new ProductRecord(s.string(strings, prods.getInt(e), "material"), prods.getFloat(e + 4),
                        prods.getDouble(e + 8), s.string(strings, prods.getInt(e + 16), "role")));
            }
            int[] center = {recs.getInt(a + 16), recs.getInt(a + 20), recs.getInt(a + 24)};
            int[] min = {recs.getInt(a + 36), recs.getInt(a + 40), recs.getInt(a + 44)};
            int[] max = {recs.getInt(a + 48), recs.getInt(a + 52), recs.getInt(a + 56)};
            SiteRecord site = new SiteRecord(id, kind, s.string(strings, recs.getInt(a + 8), "deposit type"),
                    recs.getInt(a + 12), center, recs.getFloat(a + 28), recs.getFloat(a + 32), min, max,
                    recs.getInt(a + 60), recs.getLong(a + 64), List.copyOf(products), recs.getDouble(a + 80),
                    s.optString(strings, recs.getInt(a + 88), "coe vein"), recs.getLong(a + 96),
                    (recs.get(a + 5) & 1) != 0);
            if (!out.isEmpty() && Integer.compareUnsigned(out.getLast().id(), id) >= 0)
                throw Sectioned.corrupt(path, "site " + id + " out of order");
            out.add(site);
        }
        return List.copyOf(out);
    }

    public int circumference() {
        return circumference;
    }

    /** Every site, sorted by id. */
    public List<SiteRecord> sites() {
        return sites;
    }

    /** The site with {@code id}, or null. */
    public SiteRecord byId(int id) {
        int lo = 0, hi = sites.size() - 1;
        while (lo <= hi) {
            int mid = (lo + hi) >>> 1;
            int c = Integer.compareUnsigned(sites.get(mid).id(), id);
            if (c == 0) return sites.get(mid);
            if (c < 0) lo = mid + 1;
            else hi = mid - 1;
        }
        return null;
    }

    /** The ids of the index cell holding block (x, z), ascending: every site whose footprint touches it. */
    public int[] idsAt(int x, int z) {
        int col = (int) Math.floorMod(Math.floorDiv((long) x - originX, (long) cellBlocks), (long) cols);
        int row = (int) Math.max(0, Math.min(rows - 1, Math.floorDiv((long) z - originZ, (long) cellBlocks)));
        int c = row * cols + col;
        int[] out = new int[offsets[c + 1] - offsets[c]];
        System.arraycopy(ids, offsets[c], out, 0, out.length);
        return out;
    }

    /**
     * The sites whose footprint touches the block square [x0, x1] × [z0, z1] (inclusive; x may
     * leave the canonical span), each once, ascending by id.
     */
    public List<SiteRecord> touching(int x0, int z0, int x1, int z1) {
        if (sites.isEmpty())
            return List.of();
        java.util.TreeSet<Integer> found = new java.util.TreeSet<>(Integer::compareUnsigned);
        long width = (long) x1 - x0;
        int step = cellBlocks;
        // Every index cell the square touches (x wraps, so at most `cols` columns).
        long spanCells = Math.min(cols, width / step + 2);
        for (long zc = z0; ; zc += step) {
            long zz = Math.min(zc, z1);
            for (long k = 0; k < spanCells; k++) {
                long xx = Math.min((long) x0 + k * step, x1);
                for (int id : idsAt((int) xx, (int) zz))
                    found.add(id);
            }
            if (zz >= z1)
                break;
        }
        List<SiteRecord> out = new ArrayList<>();
        for (int id : found) {
            SiteRecord s = byId(id);
            if (s != null && overlaps(s, x0, z0, x1, z1))
                out.add(s);
        }
        return Collections.unmodifiableList(out);
    }

    /** True when the site's footprint overlaps the square (x wraps). */
    private boolean overlaps(SiteRecord s, int x0, int z0, int x1, int z1) {
        if (s.max()[2] < z0 || s.min()[2] > z1)
            return false;
        long c = circumference;
        long a0 = s.min()[0], a1 = s.max()[0];
        if (a1 - a0 >= c - 1 || (long) x1 - x0 >= c - 1)
            return true;
        // Shift the site's span to the copy nearest the square and compare intervals.
        long shift = Math.floorDiv((long) x0 - a0, c) * c;
        for (long k = -1; k <= 1; k++) {
            long b0 = a0 + shift + k * c, b1 = a1 + shift + k * c;
            if (b1 >= x0 && b0 <= x1)
                return true;
        }
        return false;
    }
}
