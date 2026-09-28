// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;

/**
 * One decoded tile: row-major little-endian cells over a read-only buffer (a slice of the mapped
 * file for raw tiles, a heap buffer for LZ4 tiles). Immutable and thread-safe: only absolute reads.
 */
public final class Tile {

    final TilePack pack;
    final int index;
    final int width, height;
    private final ByteBuffer data;
    private final LayerDesc.DType dtype;
    private final int cellBytes;

    Tile(TilePack pack, int index, int width, int height, ByteBuffer data) {
        this.pack = pack;
        this.index = index;
        this.width = width;
        this.height = height;
        this.data = data.order(ByteOrder.LITTLE_ENDIAN);
        this.dtype = pack.desc().dtype();
        this.cellBytes = pack.desc().cellBytes();
    }

    /** Bytes of decoded cells (the cache weight). */
    public int byteSize() {
        return width * height * cellBytes;
    }

    /** The raw scalar at tile-local (ci, cj). */
    public double raw(int ci, int cj) {
        int at = (cj * width + ci) * cellBytes;
        return switch (dtype) {
            case U8 -> data.get(at) & 0xFF;
            case I8 -> data.get(at);
            case U16 -> data.getShort(at) & 0xFFFF;
            case I16 -> data.getShort(at);
            case U32 -> data.getInt(at) & 0xFFFFFFFFL;
            case F32 -> data.getFloat(at);
            case HORIZON_STACK -> Double.NaN;
        };
    }

    /** Horizon k's top (1/16 block) at tile-local (ci, cj) of a horizon-stack tile. */
    public short horizonTop(int ci, int cj, int k) {
        return data.getShort((cj * width + ci) * cellBytes + 2 * k);
    }

    /** Horizon k's class index (0–255) at tile-local (ci, cj) of a horizon-stack tile. */
    public int horizonClass(int ci, int cj, int k) {
        int depth = cellBytes / 3;
        return data.get((cj * width + ci) * cellBytes + 2 * depth + k) & 0xFF;
    }
}
