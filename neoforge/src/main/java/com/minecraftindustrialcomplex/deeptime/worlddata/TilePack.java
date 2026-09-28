// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.jpountz.lz4.LZ4Exception;
import net.jpountz.lz4.LZ4Factory;
import net.jpountz.lz4.LZ4SafeDecompressor;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.MappedByteBuffer;
import java.nio.channels.FileChannel;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.concurrent.atomic.AtomicReferenceArray;
import java.util.zip.CRC32;

/**
 * A memory-mapped tile pack ({@code .dtr}, plan §5.7.3; byte layout in
 * {@code crates/deeptime-format/FORMAT.md}). The whole file is mapped read-only once; raw tiles are
 * read in place, LZ4 tiles are decoded on first touch through a {@link TileCache}. Every tile's
 * CRC32 is checked before its cells are used. Thread-safe.
 */
public final class TilePack {

    static final int HEADER_BYTES = 16;
    static final int INDEX_ENTRY_BYTES = 16;
    private static final byte[] MAGIC = {'D', 'T', 'R', '1'};
    // lz4-java's pure-Java decoder: no natives, as vanilla's own LZ4 region support uses.
    private static final LZ4SafeDecompressor LZ4 = LZ4Factory.safeInstance().safeDecompressor();

    private final Path path;
    private final MappedByteBuffer map;
    private final LayerDesc desc;
    private final long[] offsets;
    private final int[] lengths;
    private final int[] crcs;
    private final TileCache cache;
    /** Raw tiles, created (and CRC-checked) on first use. */
    private final AtomicReferenceArray<Tile> rawTiles;

    private TilePack(Path path, MappedByteBuffer map, LayerDesc desc, long[] offsets, int[] lengths, int[] crcs,
                     TileCache cache) {
        this.path = path;
        this.map = map;
        this.desc = desc;
        this.offsets = offsets;
        this.lengths = lengths;
        this.crcs = crcs;
        this.cache = cache;
        this.rawTiles = new AtomicReferenceArray<>(desc.codec() == LayerDesc.Codec.RAW ? offsets.length : 0);
    }

    /** Maps {@code path} and checks its header, descriptor and tile index. */
    public static TilePack open(Path path, TileCache cache) {
        MappedByteBuffer map;
        try (FileChannel ch = FileChannel.open(path, StandardOpenOption.READ)) {
            long size = ch.size();
            if (size > Integer.MAX_VALUE)
                throw new WorldDataException(path + ": tile packs over 2 GiB are not supported by format 0.0");
            map = ch.map(FileChannel.MapMode.READ_ONLY, 0, size);
        } catch (IOException e) {
            throw new WorldDataException(path + ": " + e.getMessage(), e);
        }
        map.order(ByteOrder.LITTLE_ENDIAN);
        int size = map.capacity();
        if (size < HEADER_BYTES)
            throw corrupt(path, "shorter than the 16-byte header");
        for (int i = 0; i < 4; i++)
            if (map.get(i) != MAGIC[i])
                throw corrupt(path, "bad magic (expected DTR1)");
        long headerLen = map.getInt(4) & 0xFFFFFFFFL;
        int minor = map.getShort(8) & 0xFFFF;
        if (minor != FormatVersion.MINOR)
            throw new WorldDataException(path + ": " + FormatVersion.refusal("0." + minor));
        if (headerLen < HEADER_BYTES || headerLen > size)
            throw corrupt(path, "header length " + headerLen + " out of range");

        byte[] head = new byte[(int) headerLen];
        map.get(0, head);
        CRC32 crc = new CRC32();
        crc.update(head, 0, 12);
        crc.update(head, HEADER_BYTES, head.length - HEADER_BYTES);
        int expected = map.getInt(12);
        if ((int) crc.getValue() != expected)
            throw new WorldDataException(String.format("%s: header CRC32 %08x, expected %08x", path, (int) crc.getValue(), expected));
        String json = new String(head, HEADER_BYTES, head.length - HEADER_BYTES, StandardCharsets.UTF_8);
        LayerDesc desc;
        try {
            JsonObject o = JsonParser.parseString(json).getAsJsonObject();
            desc = LayerDesc.fromJson(o);
        } catch (RuntimeException e) {
            if (e instanceof WorldDataException wde) throw wde;
            throw new WorldDataException(path + ": bad layer descriptor: " + e.getMessage(), e);
        }

        int n = desc.tileCount();
        long indexAt = align8(headerLen);
        if (indexAt + (long) n * INDEX_ENTRY_BYTES > size)
            throw corrupt(path, "tile index past end of file");
        long[] offsets = new long[n];
        int[] lengths = new int[n];
        int[] crcs = new int[n];
        for (int t = 0; t < n; t++) {
            int e = (int) (indexAt + (long) t * INDEX_ENTRY_BYTES);
            offsets[t] = map.getLong(e);
            lengths[t] = map.getInt(e + 8);
            crcs[t] = map.getInt(e + 12);
            if (offsets[t] < 0 || lengths[t] < 0 || offsets[t] + lengths[t] > size)
                throw corrupt(path, "tile " + t + " past end of file");
        }
        return new TilePack(path, map, desc, offsets, lengths, crcs, cache);
    }

    private static long align8(long n) {
        return (n + 7) & ~7L;
    }

    private static WorldDataException corrupt(Path path, String why) {
        return new WorldDataException(path + ": corrupt world data: " + why);
    }

    public Path path() {
        return path;
    }

    public LayerDesc desc() {
        return desc;
    }

    public int tileCount() {
        return offsets.length;
    }

    /** The stored (compressed, for LZ4) bytes of tile t, as a read-only slice of the mapping. */
    public ByteBuffer storedBytes(int t) {
        return map.slice((int) offsets[t], lengths[t]).order(ByteOrder.LITTLE_ENDIAN);
    }

    /** Tile t, decoded: raw tiles in place, LZ4 tiles through the cache. */
    public Tile tile(int t) {
        if (desc.codec() == LayerDesc.Codec.RAW) {
            Tile tile = rawTiles.get(t);
            if (tile == null) {
                tile = decode(t);
                rawTiles.compareAndSet(t, null, tile);
            }
            return tile;
        }
        return cache.get(this, t);
    }

    /** Decodes tile t now, bypassing the cache (checks its CRC32). */
    Tile decode(int t) {
        ByteBuffer stored = storedBytes(t);
        CRC32 crc = new CRC32();
        crc.update(stored.duplicate());
        if ((int) crc.getValue() != crcs[t])
            throw new WorldDataException(String.format("%s: tile %d CRC32 %08x, expected %08x",
                    path, t, (int) crc.getValue(), crcs[t]));
        int tx = t % desc.tilesX(), tz = t / desc.tilesX();
        int w = desc.tileWidth(tx), h = desc.tileHeight(tz);
        int size = w * h * desc.cellBytes();
        if (desc.codec() == LayerDesc.Codec.RAW) {
            if (lengths[t] != size)
                throw corrupt(path, "raw tile " + t + " is " + lengths[t] + " bytes, expected " + size);
            return new Tile(this, t, w, h, stored.asReadOnlyBuffer());
        }
        byte[] src = new byte[lengths[t]];
        stored.get(0, src);
        byte[] dst = new byte[size];
        int n;
        try {
            n = LZ4.decompress(src, 0, src.length, dst, 0, size);
        } catch (LZ4Exception e) {
            throw corrupt(path, "tile " + t + ": LZ4: " + e.getMessage());
        }
        if (n != size)
            throw corrupt(path, "tile " + t + ": " + n + " bytes decoded, expected " + size);
        return new Tile(this, t, w, h, ByteBuffer.wrap(dst).asReadOnlyBuffer());
    }

    /** CRC32 of the whole file (for {@link WorldData#verify()}). */
    long fileCrc32() {
        CRC32 crc = new CRC32();
        crc.update(map.duplicate().position(0));
        return crc.getValue();
    }

    long fileSize() {
        return map.capacity();
    }
}
