// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.function.ToIntFunction;
import java.util.zip.CRC32;

/**
 * The binary container shared by event segments ({@code DTEV}), site records ({@code DTSR}), the
 * site index ({@code DTSI}) and bodies ({@code DTBD}): the 16-byte header, fixed fields, a section
 * table and 8-byte-aligned sections, each with its own CRC32 (FORMAT.md, "Sectioned files"; Rust
 * {@code deeptime_format::container}). Checked exactly as the Rust reader checks it: magic, minor
 * (the version message for another one), header length, header CRC, then each section's bounds
 * and CRC. The files are small (kilobytes to a few megabytes), so they are read whole.
 */
public final class Sectioned {

    static final int HEADER_BYTES = 16;
    static final int SECTION_ENTRY_BYTES = 16;

    private final Path path;
    private final ByteBuffer fixed;
    private final List<ByteBuffer> sections;

    private Sectioned(Path path, ByteBuffer fixed, List<ByteBuffer> sections) {
        this.path = path;
        this.fixed = fixed;
        this.sections = sections;
    }

    /** Reads and checks {@code path}; {@code sectionCount} gives K from the fixed fields. */
    public static Sectioned read(Path path, String magic, int fixedLen, ToIntFunction<ByteBuffer> sectionCount) {
        byte[] bytes;
        try {
            bytes = Files.readAllBytes(path);
        } catch (IOException e) {
            throw new WorldDataException(path + ": " + e.getMessage(), e);
        }
        return parse(path, bytes, magic, fixedLen, sectionCount);
    }

    public static Sectioned parse(Path path, byte[] bytes, String magic, int fixedLen, ToIntFunction<ByteBuffer> sectionCount) {
        ByteBuffer b = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN);
        if (bytes.length < HEADER_BYTES + fixedLen)
            throw corrupt(path, "shorter than its header");
        byte[] m = magic.getBytes(StandardCharsets.US_ASCII);
        for (int i = 0; i < 4; i++)
            if (bytes[i] != m[i])
                throw corrupt(path, "magic " + new String(bytes, 0, 4, StandardCharsets.ISO_8859_1) + ", expected " + magic);
        int minor = b.getShort(8) & 0xFFFF;
        if (minor != FormatVersion.MINOR)
            throw new WorldDataException(path + ": " + FormatVersion.refusal("0." + minor));
        ByteBuffer fixed = b.slice(HEADER_BYTES, fixedLen).order(ByteOrder.LITTLE_ENDIAN);
        int k = sectionCount.applyAsInt(fixed.duplicate().order(ByteOrder.LITTLE_ENDIAN));
        long headerLen = b.getInt(4) & 0xFFFFFFFFL;
        long expected = HEADER_BYTES + fixedLen + (long) k * SECTION_ENTRY_BYTES;
        if (headerLen != expected || headerLen > bytes.length)
            throw corrupt(path, "header length " + headerLen + ", expected " + expected);
        CRC32 crc = new CRC32();
        crc.update(bytes, 0, 12);
        crc.update(bytes, HEADER_BYTES, (int) headerLen - HEADER_BYTES);
        if ((int) crc.getValue() != b.getInt(12))
            throw new WorldDataException(String.format("%s: header CRC32 %08x, expected %08x", path, (int) crc.getValue(), b.getInt(12)));
        List<ByteBuffer> sections = new ArrayList<>(k);
        for (int s = 0; s < k; s++) {
            int e = HEADER_BYTES + fixedLen + s * SECTION_ENTRY_BYTES;
            long offset = b.getLong(e);
            long length = b.getInt(e + 8) & 0xFFFFFFFFL;
            if (offset < 0 || offset + length > bytes.length)
                throw corrupt(path, "section " + s + " past end of file");
            CRC32 sc = new CRC32();
            sc.update(bytes, (int) offset, (int) length);
            if ((int) sc.getValue() != b.getInt(e + 12))
                throw new WorldDataException(String.format("%s: section %d CRC32 %08x, expected %08x", path, s,
                        (int) sc.getValue(), b.getInt(e + 12)));
            sections.add(b.slice((int) offset, (int) length).order(ByteOrder.LITTLE_ENDIAN));
        }
        return new Sectioned(path, fixed, List.copyOf(sections));
    }

    static WorldDataException corrupt(Path path, String why) {
        return new WorldDataException(path + ": corrupt world data: " + why);
    }

    public Path path() {
        return path;
    }

    /** The fixed fields, little-endian, absolute indexing from 0. */
    public ByteBuffer fixed() {
        return fixed.duplicate().order(ByteOrder.LITTLE_ENDIAN);
    }

    /** Section {@code s}, little-endian, absolute indexing from 0. */
    public ByteBuffer section(int s) {
        return sections.get(s).duplicate().order(ByteOrder.LITTLE_ENDIAN);
    }

    /** A string table: {@code count} × (u32 byte length, UTF-8 bytes), back to back. */
    public List<String> strings(int s, int count) {
        ByteBuffer b = section(s);
        List<String> out = new ArrayList<>(count);
        int at = 0;
        for (int i = 0; i < count; i++) {
            if (at + 4 > b.capacity())
                throw corrupt(path, "string table ends early");
            int len = b.getInt(at);
            if (len < 0 || at + 4 + len > b.capacity())
                throw corrupt(path, "string " + i + " past its section");
            byte[] s8 = new byte[len];
            b.get(at + 4, s8);
            out.add(new String(s8, StandardCharsets.UTF_8));
            at += 4 + len;
        }
        return List.copyOf(out);
    }

    /** String {@code i} of {@code table}, or a corrupt-data error naming {@code what}. */
    public String string(List<String> table, int i, String what) {
        if (i < 0 || i >= table.size())
            throw corrupt(path, what + " string " + Integer.toUnsignedString(i) + " out of range");
        return table.get(i);
    }

    /** As {@link #string} for an optional reference ({@code u32::MAX} = none). */
    public String optString(List<String> table, int i, String what) {
        return i == -1 ? null : string(table, i, what);
    }
}
