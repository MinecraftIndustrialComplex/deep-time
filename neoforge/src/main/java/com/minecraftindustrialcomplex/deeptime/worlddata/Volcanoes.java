// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * {@code volcanoes/active.json} (FORMAT.md, "Lava"): the volcanoes the simulation leaves active at
 * the present, which is where a Deep Time world has magma. Plain Java; a world without the file (a
 * synthetic terrain, a preset without the volcano stage) has none.
 */
public final class Volcanoes {

    public static final String FILE = "volcanoes/active.json";
    public static final int VERSION = 1;
    public static final Volcanoes NONE = new Volcanoes(List.of());

    /**
     * One active volcano.
     *
     * @param x             block position of its summit, x in [−C/2, C/2)
     * @param radiusM       basal radius, m
     * @param heightM       height of the edifice above its surroundings, m
     * @param activity      0–1: an arc volcano's magma flux, a hotspot volcano's nearness to its plume
     * @param craterRadiusM rim radius of its crater or caldera, m; 0 without one
     */
    public record Volcano(int id, String kind, double x, double z, double radiusM, double heightM, double activity,
                          boolean oceanic, boolean silicic, double craterRadiusM, double craterDepthM, boolean caldera) {}

    private final List<Volcano> list;

    public Volcanoes(List<Volcano> list) {
        this.list = List.copyOf(list);
    }

    public List<Volcano> list() {
        return list;
    }

    /** The world's active volcanoes; {@link #NONE} when the folder has no list. */
    public static Volcanoes read(Path folder) {
        Path path = folder.resolve(FILE);
        if (!Files.isRegularFile(path))
            return NONE;
        JsonObject root;
        try {
            root = JsonParser.parseString(Files.readString(path, StandardCharsets.UTF_8)).getAsJsonObject();
        } catch (IOException | RuntimeException e) {
            throw new WorldDataException(path + ": cannot read the active volcanoes: " + e);
        }
        return parse(root, path.toString());
    }

    /** Parses the file's JSON; {@code what} names it in errors. */
    public static Volcanoes parse(JsonObject root, String what) {
        try {
            int version = root.get("version").getAsInt();
            if (version != VERSION)
                throw new WorldDataException(what + ": version " + version + ", this build reads " + VERSION);
            List<Volcano> out = new ArrayList<>();
            for (JsonElement e : root.getAsJsonArray("volcanoes")) {
                JsonObject v = e.getAsJsonObject();
                JsonElement c = v.get("crater");
                JsonObject crater = c != null && c.isJsonObject() ? c.getAsJsonObject() : null;
                out.add(new Volcano(v.get("id").getAsInt(), v.get("kind").getAsString(), v.get("x").getAsDouble(),
                        v.get("z").getAsDouble(), v.get("radius_m").getAsDouble(), v.get("height_m").getAsDouble(),
                        v.get("activity").getAsDouble(), v.get("oceanic").getAsBoolean(), v.get("silicic").getAsBoolean(),
                        crater == null ? 0.0 : crater.get("radius_m").getAsDouble(),
                        crater == null ? 0.0 : crater.get("depth_m").getAsDouble(),
                        crater != null && crater.get("caldera").getAsBoolean()));
            }
            return new Volcanoes(out);
        } catch (WorldDataException e) {
            throw e;
        } catch (RuntimeException e) {
            throw new WorldDataException(what + ": not a list of active volcanoes: " + e);
        }
    }
}
