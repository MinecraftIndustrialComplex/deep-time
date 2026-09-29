// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.file.Files;
import java.nio.file.NoSuchFileException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * A minimal pure-Java reader of the site registry's records, {@code sites/sites.dtsr}
 * ({@code FORMAT.md}, "Sites"; plan §5.6): what the Create World screen's review page needs for
 * its ore-province hints (type, kind, centre, latitude and longitude). Products, bodies and the
 * spatial index are left to the site API (P1-15).
 */
public final class SiteRecords {

    public static final String FILE = "sites/sites.dtsr";
    static final String MAGIC = "DTSR";
    static final int FIXED_BYTES = 16;
    static final int RECORD_BYTES = 104;

    /** A site kind ({@code u8 kind}). */
    public enum Kind { DEPOSIT, RESERVOIR, PLACER, SUPERGENE, FANTASY, UNKNOWN }

    /**
     * One site.
     *
     * @param depositType the deposit type id ({@code porphyry_cu_mo_au}, …)
     * @param x           centre block x in the canonical span
     * @param lat         latitude, degrees, north +
     * @param lon         longitude, degrees east of the seam, [0, 360)
     */
    public record Site(long id, Kind kind, String depositType, int x, int y, int z, double lat, double lon) {}

    private SiteRecords() {}

    /** The sites of {@code <folder>/sites/sites.dtsr}; empty when there is no such file. */
    public static List<Site> read(Path folder) {
        Path file = folder.resolve(FILE);
        byte[] b;
        try {
            b = Files.readAllBytes(file);
        } catch (NoSuchFileException e) {
            return List.of();
        } catch (IOException e) {
            throw new WorldDataException(file + ": " + e.getMessage(), e);
        }
        Container c = Container.parse(file, b, MAGIC, FIXED_BYTES, f -> 3);
        ByteBuffer f = c.fixed();
        int n = f.getInt(0), s = f.getInt(4);
        if (n < 0 || s < 0)
            throw Container.corrupt(file, "negative counts");
        List<String> strings = Container.strings(file, c.section(0), s);
        ByteBuffer rec = c.section(1);
        if (rec.remaining() != (long) n * RECORD_BYTES)
            throw Container.corrupt(file, "records section is " + rec.remaining() + " bytes, expected " + n + " × " + RECORD_BYTES);
        List<Site> out = new ArrayList<>(n);
        Kind[] kinds = Kind.values();
        for (int i = 0; i < n; i++) {
            int at = i * RECORD_BYTES;
            long id = Integer.toUnsignedLong(rec.getInt(at));
            int k = Byte.toUnsignedInt(rec.get(at + 4));
            int type = rec.getInt(at + 8);
            if (type < 0 || type >= strings.size())
                throw Container.corrupt(file, "site " + id + ": deposit type string " + type + " of " + strings.size());
            out.add(new Site(id, k < kinds.length - 1 ? kinds[k] : Kind.UNKNOWN, strings.get(type),
                    rec.getInt(at + 16), rec.getInt(at + 20), rec.getInt(at + 24),
                    rec.getFloat(at + 28), rec.getFloat(at + 32)));
        }
        return List.copyOf(out);
    }
}
