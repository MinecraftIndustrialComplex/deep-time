// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

/**
 * {@code bodies/bodies.dtb} (plan §5.7.4; FORMAT.md "Bodies"; Rust {@code deeptime_format::bodies}):
 * every body sorted by id, and each site's bodies. Plain Java, immutable, thread-safe.
 */
public final class BodyData {

    public static final String FILE = "bodies/bodies.dtb";
    static final int BODY_BYTES = 112;
    static final int ZONE_BYTES = 40;

    private final List<Body> bodies;
    private final Map<Integer, List<Body>> bySite;

    private BodyData(List<Body> bodies) {
        this.bodies = bodies;
        Map<Integer, List<Body>> m = new TreeMap<>();
        for (Body b : bodies)
            if (b.site() != SiteData.NONE)
                m.computeIfAbsent(b.site(), k -> new ArrayList<>()).add(b);
        m.replaceAll((k, v) -> List.copyOf(v));
        this.bySite = Map.copyOf(m);
    }

    public static BodyData empty() {
        return new BodyData(List.of());
    }

    /** Reads {@code bodies/} of {@code folder}; empty when the folder has none. */
    public static BodyData read(Path folder) {
        Path path = folder.resolve(FILE);
        if (!Files.isRegularFile(path))
            return empty();
        Sectioned s = Sectioned.read(path, "DTBD", 16, f -> 3);
        ByteBuffer f = s.fixed();
        int n = f.getInt(0), ns = f.getInt(4), nz = f.getInt(8);
        ByteBuffer recs = s.section(1), zones = s.section(2);
        if (recs.capacity() != n * BODY_BYTES || zones.capacity() != nz * ZONE_BYTES)
            throw Sectioned.corrupt(path, "body or zone section has the wrong length");
        List<String> strings = s.strings(0, ns);
        List<Body> out = new ArrayList<>(n);
        for (int r = 0; r < n; r++) {
            int a = r * BODY_BYTES;
            int id = recs.getInt(a);
            int shape = recs.get(a + 8) & 0xFF;
            if (shape >= Body.SHAPES.size())
                throw Sectioned.corrupt(path, "body " + id + ": shape " + shape);
            int zf = recs.getInt(a + 88), zn = recs.getInt(a + 92);
            if (zf < 0 || zn < 0 || (long) (zf + zn) * ZONE_BYTES > zones.capacity())
                throw Sectioned.corrupt(path, "body " + id + ": zones past the section");
            List<Body.Zone> zs = new ArrayList<>(zn);
            for (int i = zf; i < zf + zn; i++) {
                int e = i * ZONE_BYTES;
                zs.add(new Body.Zone(s.string(strings, zones.getInt(e), "zone name"),
                        s.optString(strings, zones.getInt(e + 4), "zone material"),
                        s.optString(strings, zones.getInt(e + 8), "zone rock class"),
                        zones.getFloat(e + 12), zones.getFloat(e + 16), zones.getFloat(e + 20), zones.getFloat(e + 24),
                        zones.getFloat(e + 28), zones.getFloat(e + 32)));
            }
            Body b = new Body(id, recs.getInt(a + 4), shape, s.string(strings, recs.getInt(a + 12), "role"),
                    new double[]{recs.getDouble(a + 16), recs.getDouble(a + 24), recs.getDouble(a + 32)},
                    new float[]{recs.getFloat(a + 40), recs.getFloat(a + 44), recs.getFloat(a + 48)},
                    recs.getFloat(a + 52), recs.getFloat(a + 56), recs.getInt(a + 60),
                    new int[]{recs.getInt(a + 64), recs.getInt(a + 68), recs.getInt(a + 72)},
                    new int[]{recs.getInt(a + 76), recs.getInt(a + 80), recs.getInt(a + 84)},
                    s.optString(strings, recs.getInt(a + 96), "body rock class"), List.copyOf(zs));
            for (float ax : b.axes())
                if (!(ax > 0) || Float.isInfinite(ax))
                    throw Sectioned.corrupt(path, "body " + id + ": semi-axes must be finite and > 0");
            if (!out.isEmpty() && Integer.compareUnsigned(out.getLast().id(), id) >= 0)
                throw Sectioned.corrupt(path, "body " + id + " out of order");
            out.add(b);
        }
        return new BodyData(List.copyOf(out));
    }

    /** Every body, sorted by id. */
    public List<Body> bodies() {
        return bodies;
    }

    /** The bodies of site {@code id} (the ore body first), or none. */
    public List<Body> ofSite(int id) {
        return bySite.getOrDefault(id, List.of());
    }
}
