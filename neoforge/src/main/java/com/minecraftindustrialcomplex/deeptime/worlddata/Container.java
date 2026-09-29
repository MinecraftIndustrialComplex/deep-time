// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.function.ToIntFunction;
import java.util.zip.CRC32;

/**
 * The sectioned-file container every binary world-data file but a tile pack shares
 * ({@code FORMAT.md}, "Sectioned files"; Rust {@code deeptime_format::container}): a 16-byte header
 * (magic, header length, format minor, flags, header CRC32), the type's fixed fields, a section
 * table ({@code u64 offset, u32 length, u32 CRC32} per section) and the sections. Every check the
 * Rust reader makes is made here: magic, minor, header length, header CRC and each section's bounds
 * and CRC.
 */
public final class Container {

    public static final int HEADER_BYTES = 16;
    public static final int SECTION_ENTRY_BYTES = 16;

    private final ByteBuffer fixed;
    private final List<ByteBuffer> sections;

    private Container(ByteBuffer fixed, List<ByteBuffer> sections) {
        this.fixed = fixed;
        this.sections = sections;
    }

    /** The fixed fields, little-endian, position 0. */
    public ByteBuffer fixed() {
        return fixed.duplicate().order(ByteOrder.LITTLE_ENDIAN);
    }

    public int sectionCount() {
        return sections.size();
    }

    /** Section {@code k}, little-endian, position 0. */
    public ByteBuffer section(int k) {
        return sections.get(k).duplicate().order(ByteOrder.LITTLE_ENDIAN);
    }

    /**
     * Parses {@code b}.
     *
     * @param sectionCount the number of sections, from the fixed fields
     */
    public static Container parse(Path path, byte[] b, String magic, int fixedLen, ToIntFunction<ByteBuffer> sectionCount) {
        ByteBuffer all = ByteBuffer.wrap(b).order(ByteOrder.LITTLE_ENDIAN);
        if (b.length < HEADER_BYTES + fixedLen)
            throw corrupt(path, "shorter than its header");
        String m = new String(b, 0, 4, StandardCharsets.ISO_8859_1);
        if (!m.equals(magic))
            throw corrupt(path, "magic " + m + ", expected " + magic);
        int minor = Short.toUnsignedInt(all.getShort(8));
        if (minor != FormatVersion.MINOR)
            throw new WorldDataException(path + ": " + FormatVersion.refusal("0." + minor));
        ByteBuffer fixed = ByteBuffer.wrap(b, HEADER_BYTES, fixedLen).slice().order(ByteOrder.LITTLE_ENDIAN);
        int k = sectionCount.applyAsInt(fixed.duplicate().order(ByteOrder.LITTLE_ENDIAN));
        long headerLen = Integer.toUnsignedLong(all.getInt(4));
        long expectedLen = HEADER_BYTES + fixedLen + (long) k * SECTION_ENTRY_BYTES;
        if (k < 0 || headerLen != expectedLen || headerLen > b.length)
            throw corrupt(path, "header length " + headerLen + ", expected " + expectedLen);
        CRC32 h = new CRC32();
        h.update(b, 0, 12);
        h.update(b, HEADER_BYTES, (int) headerLen - HEADER_BYTES);
        long expectedCrc = Integer.toUnsignedLong(all.getInt(12));
        if (h.getValue() != expectedCrc)
            throw corrupt(path, String.format("header CRC32 %08x, expected %08x", h.getValue(), expectedCrc));
        List<ByteBuffer> sections = new ArrayList<>(k);
        for (int s = 0; s < k; s++) {
            int e = HEADER_BYTES + fixedLen + s * SECTION_ENTRY_BYTES;
            long offset = all.getLong(e);
            long length = Integer.toUnsignedLong(all.getInt(e + 8));
            long crc = Integer.toUnsignedLong(all.getInt(e + 12));
            if (offset < headerLen || offset + length > b.length)
                throw corrupt(path, "section " + s + " (" + offset + "+" + length + ") outside the file");
            CRC32 c = new CRC32();
            c.update(b, (int) offset, (int) length);
            if (c.getValue() != crc)
                throw corrupt(path, String.format("section %d CRC32 %08x, expected %08x", s, c.getValue(), crc));
            sections.add(ByteBuffer.wrap(b, (int) offset, (int) length).slice().order(ByteOrder.LITTLE_ENDIAN));
        }
        return new Container(fixed, List.copyOf(sections));
    }

    /** A string table: {@code count} × (u32 byte length, UTF-8 bytes), back to back. */
    public static List<String> strings(Path path, ByteBuffer section, int count) {
        ByteBuffer b = section.duplicate().order(ByteOrder.LITTLE_ENDIAN);
        List<String> out = new ArrayList<>(count);
        for (int i = 0; i < count; i++) {
            if (b.remaining() < 4)
                throw corrupt(path, "string table ends after " + i + " of " + count + " strings");
            int len = b.getInt();
            if (len < 0 || len > b.remaining())
                throw corrupt(path, "string " + i + " is " + Integer.toUnsignedLong(len) + " bytes, past the table");
            byte[] s = new byte[len];
            b.get(s);
            out.add(new String(s, StandardCharsets.UTF_8));
        }
        return out;
    }

    static WorldDataException corrupt(Path path, String what) {
        return new WorldDataException(path + ": corrupt: " + what);
    }
}
