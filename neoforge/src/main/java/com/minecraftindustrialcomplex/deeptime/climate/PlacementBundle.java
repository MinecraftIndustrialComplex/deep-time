// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.Reader;
import java.nio.charset.StandardCharsets;
import java.util.Optional;

/**
 * The content bundle side of the climate class placement table (no Minecraft classes, so it can be
 * tested on its own; {@link ClimatePlacement} finds the table in the resources): a datapack's
 * table goes into the bundle the core reads under {@value #KEY}.
 */
public final class PlacementBundle {

    /** The content bundle key the core reads ({@code bake::placement::CONTENT_KEY}). */
    public static final String KEY = "climate_placement";

    /** The datapack path of the table under {@code data/deeptime/}. */
    public static final String PATH = "deeptime/climate_mapping/placement.json";

    private PlacementBundle() {}

    /** A table found in the resources: the JSON and the pack it came from. */
    public record Found(JsonObject table, String packId) {}

    /** Parses a table; throws when it is not a JSON object. */
    public static JsonObject parse(InputStream in) throws IOException {
        try (Reader r = new InputStreamReader(in, StandardCharsets.UTF_8)) {
            return JsonParser.parseReader(r).getAsJsonObject();
        }
    }

    /**
     * {@code contentJson} (a JSON object, or empty) with the table added under {@value #KEY}; the
     * bundle is returned unchanged when there is no table.
     */
    public static String addTo(String contentJson, Optional<Found> table) {
        if (table.isEmpty())
            return contentJson;
        JsonObject root = contentJson == null || contentJson.isBlank()
                ? new JsonObject()
                : JsonParser.parseString(contentJson).getAsJsonObject();
        root.add(KEY, table.get().table());
        return root.toString();
    }
}
