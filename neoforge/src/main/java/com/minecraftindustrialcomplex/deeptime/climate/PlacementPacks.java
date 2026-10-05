// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.Predicate;

/**
 * Pack placement tables (decisions log 2026-10-04, "Deep Time-side fixes (Recommended)"; FORMAT.md, "Pack placement
 * tables"): a biome mod whose biomes sit in other parameter slots than vanilla's gets a table of its own, a JSON merge
 * patch on the climate placement table, shipped as {@code data/<namespace>/deeptime/climate_mapping/packs/<id>.json}.
 * A pack table applies when every biome its {@code requires_biomes} names is registered; the ones that apply go into the
 * content bundle under {@value #KEY}, in order of {@code priority} then {@code id}, and the core merges them over the
 * base table ({@code bake::placement::Placement::with_packs}) and records their ids in
 * {@code notes.bake.climate.placement.patches}. Pure (no Minecraft classes), so it can be tested on its own;
 * {@link ClimatePlacement} finds the files and the registered biomes.
 *
 * <p>Terralith's is the first ({@code packs/terralith.json}); Biomes O' Plenty or Expanded Ecosphere get theirs the
 * same way, with their own required biomes. A datapack may add a table, or replace one by shipping a file of the same
 * id (the highest-priority pack's file wins).
 */
public final class PlacementPacks {

    /** The content bundle key the core reads ({@code bake::placement::PATCHES_KEY}). */
    public static final String KEY = "climate_placement_patches";

    /** The folder of pack tables under {@code data/<namespace>/}. */
    public static final String DIR = "deeptime/climate_mapping/packs";

    /** The schema of a pack table (the placement table's). */
    public static final int SCHEMA = 1;

    private PlacementPacks() {}

    /** One pack table: its id, order, the biomes it needs, the whole file and where it came from. */
    public record Pack(String id, int priority, List<String> requiresBiomes, JsonObject json, String source) {}

    /** Parses one pack table; throws {@link IllegalArgumentException} naming what is wrong. */
    public static Pack parse(JsonObject o, String source) {
        if (!o.has("schema") || !o.get("schema").isJsonPrimitive() || o.get("schema").getAsInt() != SCHEMA)
            throw new IllegalArgumentException(source + ": schema must be " + SCHEMA);
        if (!o.has("id") || !o.get("id").isJsonPrimitive() || o.get("id").getAsString().isBlank())
            throw new IllegalArgumentException(source + ": needs an id");
        if (!o.has("patch") || !o.get("patch").isJsonObject())
            throw new IllegalArgumentException(source + ": needs a patch object");
        if (!o.has("requires_biomes") || !o.get("requires_biomes").isJsonArray() || o.getAsJsonArray("requires_biomes").isEmpty())
            throw new IllegalArgumentException(source + ": needs a non-empty requires_biomes list");
        List<String> req = new ArrayList<>();
        for (JsonElement e : o.getAsJsonArray("requires_biomes")) {
            String id = e.getAsString();
            if (!id.contains(":"))
                throw new IllegalArgumentException(source + ": requires_biomes entry " + id + " is not a biome id");
            req.add(id);
        }
        int priority = o.has("priority") ? o.get("priority").getAsInt() : 0;
        return new Pack(o.get("id").getAsString(), priority, List.copyOf(req), o, source);
    }

    /** Parses a pack table's text. */
    public static Pack parse(String json, String source) {
        try {
            return parse(JsonParser.parseString(json).getAsJsonObject(), source);
        } catch (IllegalStateException | com.google.gson.JsonParseException e) {
            throw new IllegalArgumentException(source + ": not a JSON object: " + e.getMessage(), e);
        }
    }

    /**
     * The pack tables that apply: every one whose {@code requires_biomes} are all {@code registered}, one per id (the
     * last of {@code all} with that id, so list the files in pack order, lowest priority first), ordered by
     * {@code priority} then {@code id}.
     */
    public static List<Pack> select(Collection<Pack> all, Predicate<String> registered) {
        Map<String, Pack> byId = new LinkedHashMap<>();
        for (Pack p : all)
            byId.put(p.id(), p);
        List<Pack> out = new ArrayList<>();
        for (Pack p : byId.values())
            if (p.requiresBiomes().stream().allMatch(registered))
                out.add(p);
        out.sort(Comparator.comparingInt(Pack::priority).thenComparing(Pack::id));
        return out;
    }

    /** The ids of {@code packs}, in order. */
    public static List<String> ids(List<Pack> packs) {
        return packs.stream().map(Pack::id).toList();
    }

    /**
     * {@code contentJson} (a JSON object, or empty) with {@code packs} under {@value #KEY}; unchanged when there are
     * none.
     */
    public static String addTo(String contentJson, List<Pack> packs) {
        if (packs.isEmpty())
            return contentJson;
        JsonObject root = contentJson == null || contentJson.isBlank()
                ? new JsonObject()
                : JsonParser.parseString(contentJson).getAsJsonObject();
        JsonArray list = new JsonArray();
        for (Pack p : packs)
            list.add(p.json());
        root.add(KEY, list);
        return root.toString();
    }

    /** The pack ids a world's bake recorded ({@code notes.bake.climate.placement.patches}); empty when none. */
    public static List<String> recorded(JsonObject climateNote) {
        List<String> out = new ArrayList<>();
        if (climateNote == null || !climateNote.has("placement") || !climateNote.get("placement").isJsonObject())
            return out;
        JsonObject p = climateNote.getAsJsonObject("placement");
        if (p.has("patches") && p.get("patches").isJsonArray())
            for (JsonElement e : p.getAsJsonArray("patches"))
                out.add(e.getAsString());
        return out;
    }
}
