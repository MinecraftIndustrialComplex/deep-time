// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.util.ArrayList;
import java.util.List;

/**
 * One timelapse keyframe's description (plan §6.1; {@code FORMAT.md}, "Keyframes"): the JSON of
 * {@code keyframeInfo} ({@code {"h","layers","n","stage","t","w"}}) or an entry of
 * {@code keyframes/index.json} (which adds {@code path}, {@code bytes}, {@code crc32}).
 *
 * @param n      frame number within the run, from 0
 * @param t      simulated time, years relative to world creation (negative = the past)
 * @param stage  the stage that produced it
 * @param layers the layer names, in section order (a layer index counts in this list)
 */
public record KeyframeInfo(int n, double t, String stage, int w, int h, List<String> layers) {

    public KeyframeInfo {
        layers = List.copyOf(layers);
        if (w <= 0 || h <= 0)
            throw new WorldDataException("keyframe " + n + ": size " + w + "×" + h);
    }

    public static KeyframeInfo fromJson(String json) {
        try {
            return fromJson(JsonParser.parseString(json).getAsJsonObject());
        } catch (IllegalStateException | com.google.gson.JsonParseException e) {
            throw new WorldDataException("keyframe info is not a JSON object: " + e.getMessage(), e);
        }
    }

    public static KeyframeInfo fromJson(JsonObject o) {
        try {
            List<String> layers = new ArrayList<>();
            for (JsonElement e : o.getAsJsonArray("layers"))
                layers.add(e.getAsString());
            return new KeyframeInfo(o.get("n").getAsInt(), o.get("t").getAsDouble(), o.get("stage").getAsString(),
                    o.get("w").getAsInt(), o.get("h").getAsInt(), layers);
        } catch (RuntimeException e) {
            if (e instanceof WorldDataException w)
                throw w;
            throw new WorldDataException("malformed keyframe info " + o + ": " + e.getMessage(), e);
        }
    }

    /** The index of {@code layer}, or −1. */
    public int layerIndex(String layer) {
        return layers.indexOf(layer);
    }

    /** Pixels per frame layer. */
    public int pixels() {
        return w * h;
    }
}
