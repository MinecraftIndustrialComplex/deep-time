// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.NoSuchFileException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * {@code manifest.json} (plan §5.7.1; FORMAT.md). The format version is checked before anything
 * else is parsed, so a newer layout still produces the version message.
 */
public record Manifest(String formatVersion, String source, String createdUtc, int circumference, int seaLevelY,
                       int minY, int maxY, Map<String, GridDesc> grids, List<LayerEntry> layers,
                       List<FileEntry> files, JsonObject notes) {

    public static final String FILE = "manifest.json";

    /** One tile pack of the folder. */
    public record LayerEntry(String name, String path, String grid, long bytes, long crc32) {}

    /** Any other listed file. */
    public record FileEntry(String path, long bytes, long crc32) {}

    /** Reads {@code <folder>/manifest.json}. */
    public static Manifest read(Path folder) {
        Path file = folder.resolve(FILE);
        String text;
        try {
            text = Files.readString(file, StandardCharsets.UTF_8);
        } catch (NoSuchFileException e) {
            throw new WorldDataException(file + ": no world-data manifest (missing or incompletely written)", e);
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
        try {
            JsonObject scale = o.getAsJsonObject("scale");
            Map<String, GridDesc> grids = new LinkedHashMap<>();
            for (Map.Entry<String, JsonElement> g : o.getAsJsonObject("grids").entrySet())
                grids.put(g.getKey(), GridDesc.fromJson(g.getValue().getAsJsonObject()));
            List<LayerEntry> layers = new ArrayList<>();
            for (JsonElement e : o.getAsJsonArray("layers")) {
                JsonObject l = e.getAsJsonObject();
                layers.add(new LayerEntry(l.get("name").getAsString(), l.get("path").getAsString(),
                        l.get("grid").getAsString(), l.get("bytes").getAsLong(), l.get("crc32").getAsLong()));
            }
            List<FileEntry> files = new ArrayList<>();
            if (o.has("files"))
                for (JsonElement e : o.getAsJsonArray("files")) {
                    JsonObject f = e.getAsJsonObject();
                    files.add(new FileEntry(f.get("path").getAsString(), f.get("bytes").getAsLong(),
                            f.get("crc32").getAsLong()));
                }
            return new Manifest(
                    fv.getAsString(),
                    o.get("source").getAsString(),
                    o.get("created_utc").getAsString(),
                    scale.get("circumference_blocks").getAsInt(),
                    scale.get("sea_level_y").getAsInt(),
                    scale.get("min_y").getAsInt(),
                    scale.get("max_y").getAsInt(),
                    Map.copyOf(grids),
                    List.copyOf(layers),
                    List.copyOf(files),
                    o.has("notes") ? o.getAsJsonObject("notes") : new JsonObject());
        } catch (RuntimeException e) {
            throw new WorldDataException(file + ": malformed manifest: " + e.getMessage(), e);
        }
    }

    /** The entry of layer {@code name}, or null. */
    public LayerEntry layer(String name) {
        for (LayerEntry l : layers)
            if (l.name().equals(name)) return l;
        return null;
    }
}
