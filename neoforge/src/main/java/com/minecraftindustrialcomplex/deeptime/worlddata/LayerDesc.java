// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.Locale;

/**
 * A layer's descriptor, the JSON inside every tile pack (plan §5.3, FORMAT.md "LayerDesc JSON").
 *
 * @param nodata     the raw cell value meaning "no data", or NaN when the layer has none
 * @param stackDepth K for {@link DType#HORIZON_STACK} layers, else 0
 * @param tongueBandKeys  horizon stacks: keys per band of the column rule's tongues ({@code tongue_band_keys},
 *                        default {@link Columns#BAND_KEYS})
 * @param tongueMinorShare horizon stacks: the share below which a rock takes no part in the tongues
 *                        ({@code tongue_minor_share}, default {@link Columns#MINOR_SHARE})
 */
public record LayerDesc(String name, DType dtype, double scale, double offset, String units, GridDesc grid,
                        int tile, Codec codec, double nodata, int stackDepth, int tongueBandKeys,
                        double tongueMinorShare) {

    /** Cell types (FORMAT.md). */
    public enum DType {
        U8(1), I8(1), U16(2), I16(2), U32(4), F32(4), HORIZON_STACK(0);

        /** Bytes per cell of a scalar type; 0 for the horizon stack (7K, format 0.2). */
        public final int bytes;

        DType(int bytes) {
            this.bytes = bytes;
        }

        static DType parse(String s) {
            return valueOf(s.toUpperCase(Locale.ROOT));
        }
    }

    /** Tile encodings. */
    public enum Codec {
        RAW, LZ4_BLOCK;

        static Codec parse(String s) {
            return valueOf(s.toUpperCase(Locale.ROOT));
        }
    }

    static LayerDesc fromJson(JsonObject o) {
        JsonElement nd = o.get("nodata");
        JsonElement k = o.get("stack_depth");
        JsonElement band = o.get("tongue_band_keys");
        JsonElement minor = o.get("tongue_minor_share");
        LayerDesc d = new LayerDesc(
                o.get("name").getAsString(),
                DType.parse(o.get("dtype").getAsString()),
                o.get("scale").getAsDouble(),
                o.get("offset").getAsDouble(),
                o.has("units") ? o.get("units").getAsString() : "",
                GridDesc.fromJson(o.getAsJsonObject("grid")),
                o.get("tile").getAsInt(),
                Codec.parse(o.get("codec").getAsString()),
                nd == null || nd.isJsonNull() ? Double.NaN : nd.getAsDouble(),
                k == null || k.isJsonNull() ? 0 : k.getAsInt(),
                band == null || band.isJsonNull() ? Columns.BAND_KEYS : band.getAsInt(),
                minor == null || minor.isJsonNull() ? Columns.MINOR_SHARE : minor.getAsDouble());
        d.validate();
        return d;
    }

    private void validate() {
        if (grid.w() <= 0 || grid.h() <= 0 || grid.r() <= 0 || tile <= 0)
            throw new WorldDataException("layer " + name + ": empty grid or tile");
        if ((dtype == DType.HORIZON_STACK) != (stackDepth > 0))
            throw new WorldDataException("layer " + name + ": stack_depth " + stackDepth + " with dtype " + dtype);
        if (tongueBandKeys <= 0 || !(tongueMinorShare >= 0 && tongueMinorShare < 0.5))
            throw new WorldDataException("layer " + name + ": tongue_band_keys " + tongueBandKeys
                    + ", tongue_minor_share " + tongueMinorShare);
    }

    public boolean hasNodata() {
        return !Double.isNaN(nodata);
    }

    public int cellBytes() {
        return dtype == DType.HORIZON_STACK ? 7 * stackDepth : dtype.bytes;
    }

    public int tilesX() {
        return (grid.w() + tile - 1) / tile;
    }

    public int tilesZ() {
        return (grid.h() + tile - 1) / tile;
    }

    public int tileCount() {
        return tilesX() * tilesZ();
    }

    /** Width in cells of tile column tx (edge tiles are cropped). */
    public int tileWidth(int tx) {
        return Math.min(tile, grid.w() - tx * tile);
    }

    /** Height in cells of tile row tz. */
    public int tileHeight(int tz) {
        return Math.min(tile, grid.h() - tz * tile);
    }

    /** Raw → physical; NaN for nodata. */
    public double physical(double raw) {
        return raw == nodata || Double.isNaN(raw) ? Double.NaN : raw * scale + offset;
    }
}
