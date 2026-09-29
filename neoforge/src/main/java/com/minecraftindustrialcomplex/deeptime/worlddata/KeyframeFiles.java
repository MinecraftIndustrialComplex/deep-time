// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.jpountz.lz4.LZ4Exception;
import net.jpountz.lz4.LZ4Factory;
import net.jpountz.lz4.LZ4SafeDecompressor;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.NoSuchFileException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * Reads timelapse keyframes from disk without natives ({@code FORMAT.md}, "Keyframes"; Rust
 * {@code deeptime_format::keyframes}): {@code keyframes/index.json} and {@code keyframes/kf-<n>.dtk}.
 * The core writes each frame as it is produced, so this works while a run goes, after its session
 * is destroyed, and on a staged or finished world-data folder.
 */
public final class KeyframeFiles {

    public static final String DIR = "keyframes";
    public static final String INDEX = "keyframes/index.json";
    static final String MAGIC = "DTKF";
    static final int FIXED_BYTES = 24;

    // lz4-java's pure-Java decoder, as the tile packs use.
    private static final LZ4SafeDecompressor LZ4 = LZ4Factory.safeInstance().safeDecompressor();

    private KeyframeFiles() {}

    /** One {@code index.json} entry: the frame's description and its file. */
    public record Entry(KeyframeInfo info, String path, long bytes, long crc32) {}

    /** The frames listed in {@code <folder>/keyframes/index.json}; empty when there is none. */
    public static List<Entry> index(Path folder) {
        Path file = folder.resolve(INDEX);
        String text;
        try {
            text = Files.readString(file, StandardCharsets.UTF_8);
        } catch (NoSuchFileException e) {
            return List.of();
        } catch (IOException e) {
            throw new WorldDataException(file + ": " + e.getMessage(), e);
        }
        JsonObject o;
        try {
            o = JsonParser.parseString(text).getAsJsonObject();
        } catch (RuntimeException e) {
            throw new WorldDataException(file + ": not JSON: " + e.getMessage(), e);
        }
        JsonElement fv = o.get("format_version");
        if (fv == null || !fv.isJsonPrimitive())
            throw new WorldDataException(file + ": no format_version");
        FormatVersion.require(fv.getAsString());
        List<Entry> out = new ArrayList<>();
        try {
            for (JsonElement e : o.getAsJsonArray("frames")) {
                JsonObject f = e.getAsJsonObject();
                out.add(new Entry(KeyframeInfo.fromJson(f), f.get("path").getAsString(), f.get("bytes").getAsLong(),
                        f.get("crc32").getAsLong()));
            }
        } catch (RuntimeException e) {
            if (e instanceof WorldDataException w)
                throw w;
            throw new WorldDataException(file + ": malformed: " + e.getMessage(), e);
        }
        return List.copyOf(out);
    }

    /** {@code keyframes/kf-<n>.dtk}. */
    public static String path(int n) {
        return DIR + "/kf-" + n + ".dtk";
    }

    /**
     * Layer {@code layer} of frame {@code n} as ABGR ints ({@code NativeImage}'s pixel order: the
     * RGBA8 bytes read as little-endian ints), row-major from the north edge.
     */
    public static int[] readAbgr(Path folder, int n, int layer) {
        Path file = folder.resolve(path(n));
        byte[] b;
        try {
            b = Files.readAllBytes(file);
        } catch (IOException e) {
            throw new WorldDataException(file + ": " + e.getMessage(), e);
        }
        Container c = Container.parse(file, b, MAGIC, FIXED_BYTES, f -> 1 + f.getInt(12));
        ByteBuffer f = c.fixed();
        int fn = f.getInt(0), w = f.getInt(4), h = f.getInt(8), count = f.getInt(12);
        if (fn != n)
            throw Container.corrupt(file, "holds frame " + fn + ", expected " + n);
        if (layer < 0 || layer >= count)
            throw new WorldDataException(file + ": frame " + n + " has " + count + " layers, not " + (layer + 1));
        long size = (long) w * h * 4;
        if (w <= 0 || h <= 0 || size > Integer.MAX_VALUE)
            throw Container.corrupt(file, "size " + w + "×" + h);
        ByteBuffer src = c.section(1 + layer);
        byte[] in = new byte[src.remaining()];
        src.get(in);
        byte[] rgba = new byte[(int) size];
        int decoded;
        try {
            decoded = LZ4.decompress(in, 0, in.length, rgba, 0, rgba.length);
        } catch (LZ4Exception e) {
            throw Container.corrupt(file, "layer " + layer + ": LZ4: " + e.getMessage());
        }
        if (decoded != size)
            throw Container.corrupt(file, "layer " + layer + ": " + decoded + " bytes decoded, expected " + size);
        return abgr(ByteBuffer.wrap(rgba), w * h);
    }

    /** The description a {@code .dtk} file carries (its stage and layer names from the string table). */
    public static KeyframeInfo readInfo(Path folder, int n) {
        Path file = folder.resolve(path(n));
        byte[] b;
        try {
            b = Files.readAllBytes(file);
        } catch (IOException e) {
            throw new WorldDataException(file + ": " + e.getMessage(), e);
        }
        Container c = Container.parse(file, b, MAGIC, FIXED_BYTES, f -> 1 + f.getInt(12));
        ByteBuffer f = c.fixed();
        int count = f.getInt(12);
        List<String> strings = Container.strings(file, c.section(0), 1 + count);
        return new KeyframeInfo(f.getInt(0), f.getDouble(16), strings.get(0), f.getInt(4), f.getInt(8),
                strings.subList(1, strings.size()));
    }

    /** RGBA8 bytes (from the buffer's position) as {@code pixels} ABGR ints. */
    public static int[] abgr(ByteBuffer rgba, int pixels) {
        ByteBuffer b = rgba.duplicate().order(ByteOrder.LITTLE_ENDIAN);
        int[] out = new int[pixels];
        b.asIntBuffer().get(out);
        return out;
    }
}
