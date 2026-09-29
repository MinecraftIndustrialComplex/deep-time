// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * The colour ramps the core's keyframes, the HTML viewer and the in-game screens share (plan §3.3):
 * {@code crates/deeptime-viewer/assets/ramps.json}, copied into the jar as {@code deeptime/ramps.json}.
 * The same rules as Rust's {@code deeptime_format::Ramps}: stops {@code [value, "#rrggbb"]} linearly
 * interpolated per channel and rounded, clamped at the ends; a layer's ramp is its own entry, else
 * the longest {@code <prefix>*} pattern it starts with, else the default. Keyframes arrive already
 * coloured; the game colours only what it renders itself (the review page's final map, legends).
 */
public final class Ramps {

    /** Where the ramps sit in the jar. */
    public static final String RESOURCE = "/deeptime/ramps.json";

    private static volatile Ramps builtin;

    private final Map<String, Ramp> ramps;
    private final Map<String, String> layers;
    private final String defaultRamp;

    private Ramps(Map<String, Ramp> ramps, Map<String, String> layers, String defaultRamp) {
        this.ramps = ramps;
        this.layers = layers;
        this.defaultRamp = defaultRamp;
    }

    /** One ramp: stop values ascending, colours as {@code 0xRRGGBB}. */
    public record Ramp(double[] values, int[] rgb) {

        /** The colour of {@code v} as {@code 0xRRGGBB}. */
        public int colour(double v) {
            if (values.length == 0)
                return 0x808080;
            if (v <= values[0])
                return rgb[0];
            for (int i = 1; i < values.length; i++) {
                if (v <= values[i]) {
                    double a = values[i - 1], b = values[i];
                    double t = b > a ? (v - a) / (b - a) : 1.0;
                    int ca = rgb[i - 1], cb = rgb[i];
                    int out = 0;
                    for (int shift = 16; shift >= 0; shift -= 8) {
                        double x = (ca >> shift) & 0xff, y = (cb >> shift) & 0xff;
                        out |= ((int) Math.round(x + (y - x) * t) & 0xff) << shift;
                    }
                    return out;
                }
            }
            return rgb[rgb.length - 1];
        }

        /** {@code NativeImage}'s ABGR int: opaque colour, or transparent black for NaN (nodata). */
        public int abgr(double v) {
            if (Double.isNaN(v))
                return 0;
            int c = colour(v);
            return 0xff000000 | (c & 0xff) << 16 | (c & 0xff00) | (c >> 16) & 0xff;
        }

        public double min() {
            return values.length == 0 ? 0 : values[0];
        }

        public double max() {
            return values.length == 0 ? 0 : values[values.length - 1];
        }
    }

    /** The ramps shipped in the jar. */
    public static Ramps builtin() {
        Ramps r = builtin;
        if (r == null) {
            try (InputStream in = Ramps.class.getResourceAsStream(RESOURCE)) {
                if (in == null)
                    throw new IllegalStateException(RESOURCE + " is missing from the jar; reinstall Deep Time");
                r = parse(new String(in.readAllBytes(), StandardCharsets.UTF_8));
            } catch (IOException e) {
                throw new UncheckedIOException(e);
            }
            builtin = r;
        }
        return r;
    }

    public static Ramps parse(String json) {
        JsonObject o = JsonParser.parseString(json).getAsJsonObject();
        Map<String, Ramp> ramps = new LinkedHashMap<>();
        for (Map.Entry<String, JsonElement> e : o.getAsJsonObject("ramps").entrySet()) {
            var arr = e.getValue().getAsJsonObject().getAsJsonArray("stops");
            double[] values = new double[arr.size()];
            int[] rgb = new int[arr.size()];
            for (int i = 0; i < arr.size(); i++) {
                var stop = arr.get(i).getAsJsonArray();
                values[i] = stop.get(0).getAsDouble();
                rgb[i] = hex(stop.get(1).getAsString());
            }
            ramps.put(e.getKey(), new Ramp(values, rgb));
        }
        Map<String, String> layers = new LinkedHashMap<>();
        for (Map.Entry<String, JsonElement> e : o.getAsJsonObject("layers").entrySet())
            layers.put(e.getKey(), e.getValue().getAsString());
        String def = o.get("default").getAsString();
        if (!ramps.containsKey(def))
            throw new IllegalArgumentException("ramps.json: default ramp '" + def + "' is not defined");
        return new Ramps(Map.copyOf(ramps), Map.copyOf(layers), def);
    }

    /** {@code #rrggbb} → {@code 0xRRGGBB} (missing digits read as 0, as Rust's parser does). */
    static int hex(String s) {
        String h = s.startsWith("#") ? s.substring(1) : s;
        int out = 0;
        for (int k = 0; k < 3; k++) {
            int v;
            try {
                v = h.length() >= 2 * k + 2 ? Integer.parseInt(h.substring(2 * k, 2 * k + 2), 16) : 0;
            } catch (NumberFormatException e) {
                v = 0;
            }
            out = out << 8 | v;
        }
        return out;
    }

    /** The name of {@code layer}'s ramp. */
    public String rampName(String layer) {
        String name = layers.get(layer);
        if (name == null) {
            int best = -1;
            for (Map.Entry<String, String> e : layers.entrySet()) {
                String k = e.getKey();
                if (!k.endsWith("*"))
                    continue;
                String prefix = k.substring(0, k.length() - 1);
                if (layer.startsWith(prefix) && prefix.length() > best) {
                    best = prefix.length();
                    name = e.getValue();
                }
            }
        }
        return name != null && ramps.containsKey(name) ? name : defaultRamp;
    }

    /** The ramp of {@code layer}. */
    public Ramp forLayer(String layer) {
        return ramps.get(rampName(layer));
    }

    /** A ramp by name, or the default. */
    public Ramp ramp(String name) {
        return ramps.getOrDefault(name, ramps.get(defaultRamp));
    }
}
