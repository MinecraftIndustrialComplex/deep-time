// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import java.io.ByteArrayOutputStream;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.zip.DataFormatException;
import java.util.zip.Deflater;
import java.util.zip.Inflater;

/**
 * A map picture on the wire: ABGR pixels as four bytes each (A, B, G, R), deflated, cut into
 * parts small enough for one custom payload packet each (a clientbound one carries at most 1 MiB).
 */
public final class ImageCodec {

    /** Bytes of deflated image a part carries at most. */
    public static final int PART_BYTES = 256 * 1024;

    /** The most pixels a decoded image may hold (4096 × 4096): a bound on what a server can make a client allocate. */
    public static final int MAX_PIXELS = 4096 * 4096;

    private ImageCodec() {}

    /** The deflated parts of an ABGR image. */
    public static List<byte[]> encode(int[] abgr) {
        byte[] raw = new byte[abgr.length * 4];
        for (int i = 0, o = 0; i < abgr.length; i++) {
            int c = abgr[i];
            raw[o++] = (byte) (c >>> 24);
            raw[o++] = (byte) (c >>> 16);
            raw[o++] = (byte) (c >>> 8);
            raw[o++] = (byte) c;
        }
        Deflater d = new Deflater(6);
        d.setInput(raw);
        d.finish();
        ByteArrayOutputStream bos = new ByteArrayOutputStream(raw.length / 4 + 64);
        byte[] buf = new byte[64 * 1024];
        while (!d.finished()) {
            int n = d.deflate(buf);
            bos.write(buf, 0, n);
        }
        d.end();
        byte[] all = bos.toByteArray();
        List<byte[]> parts = new ArrayList<>();
        for (int off = 0; off < all.length; off += PART_BYTES)
            parts.add(Arrays.copyOfRange(all, off, Math.min(all.length, off + PART_BYTES)));
        if (parts.isEmpty())
            parts.add(new byte[0]);
        return parts;
    }

    /**
     * The ABGR image of {@code w × h} pixels from its parts, in order.
     *
     * @throws IllegalArgumentException when the parts do not inflate to exactly that many pixels
     */
    public static int[] decode(List<byte[]> parts, int w, int h) {
        if (w <= 0 || h <= 0 || (long) w * h > MAX_PIXELS)
            throw new IllegalArgumentException("image size " + w + "x" + h + " out of range");
        Inflater inf = new Inflater();
        byte[] raw = new byte[w * h * 4];
        int at = 0;
        byte[] scratch = new byte[1];
        try {
            for (byte[] p : parts) {
                inf.setInput(p);
                while (!inf.needsInput() && !inf.finished()) {
                    if (inf.needsDictionary())
                        throw new IllegalArgumentException("bad image data: wants a dictionary");
                    if (at < raw.length) {
                        at += inf.inflate(raw, at, raw.length - at);
                    } else if (inf.inflate(scratch) > 0) {
                        throw new IllegalArgumentException("image inflates past " + raw.length + " bytes");
                    } else if (!inf.finished() && !inf.needsInput()) {
                        break;
                    }
                }
            }
        } catch (DataFormatException e) {
            throw new IllegalArgumentException("bad image data: " + e.getMessage(), e);
        } finally {
            inf.end();
        }
        if (at != raw.length)
            throw new IllegalArgumentException("image inflates to " + at + " bytes, expected " + raw.length);
        int[] out = new int[w * h];
        for (int i = 0, o = 0; i < out.length; i++, o += 4)
            out[i] = (raw[o] & 0xff) << 24 | (raw[o + 1] & 0xff) << 16 | (raw[o + 2] & 0xff) << 8 | (raw[o + 3] & 0xff);
        return out;
    }
}
