// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.Set;

/**
 * The planet editor's parameter catalogue (plan §5.1, §7.12 setup page): each field names the
 * preset JSON path it edits. <b>A field is editable exactly when the selected preset carries its
 * path</b>, so the screen never adds keys a preset does not know: today's placeholder presets
 * (P1-13) carry only {@code scale.circumference_blocks}, and the Rust core rejects unknown keys, so
 * these fields show as unavailable; the pipeline presets of P1-03…P1-17 (a share code from the
 * workbench, until P1-08 makes them the built-ins) carry every path of {@code terrain} and the
 * {@code adv_*} sections at their calibrated values, so those fields light up without a screen
 * change. If the real schema names a path differently, the one line here changes
 * ({@code EditorModelTest.pipelinePresetsCarryEveryKnob} fails until it does).
 *
 * <p>The size (circumference) and the timeline are not in this list: every preset carries them
 * (the size picks the built-in preset; the timeline has no default, plan §5.1).
 *
 * <p>The list itself is {@code crates/deeptime-viewer/assets/editor_fields.json}, shipped in the jar
 * as {@code deeptime/editor_fields.json} (Gradle {@code processResources}, like {@code ramps.json}):
 * the browser workbench ({@code deeptime watch}, P1-20) builds its form from the same file, so the
 * two editors cannot drift apart. Its {@code advanced_sections} (settings audit) are the sections the setup
 * page keeps behind one collapsed Advanced header; the world-shaping tuning knobs live there
 * (decisions log 2026-09-29: such knobs are settings with a sensible default, not constants).
 */
public final class EditorFields {

    private EditorFields() {}

    public enum Section { PLANET, SCALE, TERRAIN, MULTIPLIERS, BIOMES, FANTASY, PLAYABILITY, ADV_PLATES, ADV_SURFACE, ADV_CLIMATE, ADV_ROCKS, ADV_VOLCANOES, ADV_DEPOSITS, ADV_BIOMES, ADV_WATER, ADV_UNDERGROUND, ADV_GEOTHERMAL }

    public enum Kind { NUMBER, TOGGLE, CHOICE }

    /**
     * One editable parameter.
     *
     * @param id      the field's id; its label is {@code deeptime.editor.field.<id>}
     * @param path    dotted JSON path inside the preset ({@code planet.gravity_m_s2}); a number
     *                indexes an array ({@code structure.pluton_length_km.0})
     * @param unit    shown after the value ({@code m/s²}); empty for none
     * @param phase3  a fantasy toggle: shown, always disabled until Phase 3 (decisions file, "Fantasy terrain")
     * @param integer a whole number, written as a JSON integer (the seed is a u64 in the core)
     * @param hint    one plain-language line on what the field does (the tooltip); may be empty
     * @param options the values of a choice with a fixed list ({@code off, weak, medium, strong});
     *                empty for a choice whose list comes from elsewhere (the essentials lists)
     */
    public record Field(String id, Section section, Kind kind, String path, double min, double max, String unit, boolean phase3,
                        boolean integer, String hint, List<String> options) {

        public String labelKey() {
            return "editor.field." + id;
        }

        /** The label key of one option of a fixed-list choice. */
        public String optionKey(String option) {
            return "editor.option." + id + "." + option;
        }

        /** True for a field in one of the {@link #ADVANCED} sections. */
        public boolean advanced() {
            return ADVANCED.contains(section);
        }

        /** The value at {@link #path} in {@code preset}, if present. */
        public Optional<JsonElement> get(JsonObject preset) {
            JsonElement e = preset;
            for (String part : path.split("\\.")) {
                e = step(e, part);
                if (e == null)
                    return Optional.empty();
            }
            return Optional.ofNullable(e).filter(v -> !v.isJsonNull());
        }

        /** Editable in this preset: not a Phase 3 placeholder, and the preset carries the path. */
        public boolean availableIn(JsonObject preset) {
            return !phase3 && get(preset).isPresent();
        }

        /** Checks a number against the range (and, for an integer field, that it is whole). */
        public boolean inRange(double v) {
            return !Double.isNaN(v) && v >= min && v <= max && (!integer || v == Math.rint(v));
        }

        /** The JSON a valid number becomes: an integer field's as a long, the others' as a double. */
        public JsonPrimitive number(double v) {
            return integer ? new JsonPrimitive((long) v) : new JsonPrimitive(v);
        }
    }

    /** One step down a path: an object's member, or an array's element when the step is a number. */
    private static JsonElement step(JsonElement e, String part) {
        if (e == null)
            return null;
        if (e.isJsonObject())
            return e.getAsJsonObject().get(part);
        if (e.isJsonArray()) {
            try {
                int i = Integer.parseInt(part);
                JsonArray a = e.getAsJsonArray();
                return i >= 0 && i < a.size() ? a.get(i) : null;
            } catch (NumberFormatException ex) {
                return null;
            }
        }
        return null;
    }

    /** Where the catalogue sits in the jar. */
    public static final String RESOURCE = "/deeptime/editor_fields.json";

    /** A parsed catalogue: the fields in the setup page's order and the sections kept collapsed. */
    record Catalogue(List<Field> fields, Set<Section> advanced) {}

    private static final Catalogue LOADED = load();

    /**
     * Every field, in the setup page's order, from {@link #RESOURCE}. The planet, scale and seed
     * paths and ranges are P1-03's {@code params.rs} ({@code PlanetParams}, {@code ScaleParams},
     * {@code PlanetPreset::validate}); the terrain and advanced sections' paths are keys the
     * pipeline presets carry (their ranges were run at both ends, STATUS.md settings audit); the others are
     * proposals for the stages that have not landed (the core validates in the end).
     */
    public static final List<Field> ALL = LOADED.fields();

    /** The sections behind the Advanced header (the catalogue's {@code advanced_sections}). */
    public static final Set<Section> ADVANCED = LOADED.advanced();

    /** Reads the catalogue; a jar without it is broken, so this throws. */
    static Catalogue load() {
        try (InputStream in = EditorFields.class.getResourceAsStream(RESOURCE)) {
            if (in == null)
                throw new IllegalStateException("the editor catalogue " + RESOURCE + " is missing from the jar; reinstall Deep Time");
            return parseCatalogue(JsonParser.parseReader(new InputStreamReader(in, StandardCharsets.UTF_8)).getAsJsonObject());
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    /** The fields of a catalogue ({@code {"fields": [...]}}); throws on a malformed entry. */
    static List<Field> parse(JsonObject catalogue) {
        return parseCatalogue(catalogue).fields();
    }

    static Catalogue parseCatalogue(JsonObject catalogue) {
        List<Field> out = new ArrayList<>();
        for (JsonElement e : catalogue.getAsJsonArray("fields")) {
            JsonObject o = e.getAsJsonObject();
            String id = o.get("id").getAsString();
            try {
                List<String> options = new ArrayList<>();
                if (o.has("options"))
                    for (JsonElement opt : o.getAsJsonArray("options"))
                        options.add(opt.getAsString());
                out.add(new Field(id,
                        section(o.get("section").getAsString()),
                        Kind.valueOf(o.get("kind").getAsString().toUpperCase(Locale.ROOT)),
                        o.get("path").getAsString(),
                        o.get("min").getAsDouble(),
                        o.get("max").getAsDouble(),
                        o.has("unit") ? o.get("unit").getAsString() : "",
                        o.has("phase3") && o.get("phase3").getAsBoolean(),
                        o.has("integer") && o.get("integer").getAsBoolean(),
                        o.has("hint") ? o.get("hint").getAsString() : "",
                        List.copyOf(options)));
            } catch (RuntimeException ex) {
                throw new IllegalStateException("editor field '" + id + "' in " + RESOURCE + ": " + ex.getMessage(), ex);
            }
        }
        Set<Section> advanced = new java.util.LinkedHashSet<>();
        if (catalogue.has("advanced_sections"))
            for (JsonElement s : catalogue.getAsJsonArray("advanced_sections"))
                advanced.add(section(s.getAsString()));
        return new Catalogue(List.copyOf(out), Set.copyOf(advanced));
    }

    private static Section section(String id) {
        return Section.valueOf(id.toUpperCase(Locale.ROOT));
    }

    /** The field with this id. */
    public static Field byId(String id) {
        for (Field f : ALL)
            if (f.id().equals(id))
                return f;
        throw new IllegalArgumentException("no editor field '" + id + "'");
    }

    /** The fields of the advanced sections, in the setup page's order. */
    public static List<Field> advancedFields() {
        return ALL.stream().filter(Field::advanced).toList();
    }

    /** The advanced sections, in the setup page's order. */
    public static List<Section> advancedSections() {
        List<Section> out = new ArrayList<>();
        for (Field f : ALL)
            if (f.advanced() && !out.contains(f.section()))
                out.add(f.section());
        return out;
    }

    /**
     * Sets {@code path} in {@code preset} (in place), creating objects on the way; an array's
     * element ({@code structure.pluton_length_km.0}) must exist already.
     */
    static void set(JsonObject preset, String path, JsonElement value) {
        String[] parts = path.split("\\.");
        JsonElement e = preset;
        for (int i = 0; i < parts.length - 1; i++) {
            JsonElement next = step(e, parts[i]);
            if (next == null || (!next.isJsonObject() && !next.isJsonArray())) {
                if (!e.isJsonObject())
                    return;
                JsonObject created = new JsonObject();
                e.getAsJsonObject().add(parts[i], created);
                next = created;
            }
            e = next;
        }
        String last = parts[parts.length - 1];
        if (e.isJsonObject()) {
            e.getAsJsonObject().add(last, value);
        } else if (e.isJsonArray()) {
            try {
                int i = Integer.parseInt(last);
                JsonArray a = e.getAsJsonArray();
                if (i >= 0 && i < a.size())
                    a.set(i, value);
            } catch (NumberFormatException ex) {
                // not an index: nothing to set
            }
        }
    }
}
