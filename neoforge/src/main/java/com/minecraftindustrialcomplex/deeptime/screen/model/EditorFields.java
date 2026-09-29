// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

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

/**
 * The planet editor's parameter catalogue (plan §5.1, §7.12 setup page): each field names the
 * preset JSON path it edits. <b>A field is editable exactly when the selected preset carries its
 * path</b>, so the screen never adds keys a preset does not know: today's placeholder presets
 * (P1-13) carry only {@code scale.circumference_blocks}, and the Rust core rejects unknown keys, so
 * these fields show as unavailable; when the real presets of P1-03…P1-17 land with their sections,
 * the fields light up without a screen change. If the real schema names a path differently, the
 * one line here changes ({@code ModelHelpersTest.realPresetsCarryEveryField} fails until it does).
 *
 * <p>The size (circumference) and the timeline are not in this list: every preset carries them
 * (the size picks the built-in preset; the timeline has no default, plan §5.1).
 *
 * <p>The list itself is {@code crates/deeptime-viewer/assets/editor_fields.json}, shipped in the jar
 * as {@code deeptime/editor_fields.json} (Gradle {@code processResources}, like {@code ramps.json}):
 * the browser workbench ({@code deeptime watch}, P1-20) builds its form from the same file, so the
 * two editors cannot drift apart.
 */
public final class EditorFields {

    private EditorFields() {}

    public enum Section { PLANET, SCALE, MULTIPLIERS, BIOMES, FANTASY, PLAYABILITY }

    public enum Kind { NUMBER, TOGGLE, CHOICE }

    /**
     * One editable parameter.
     *
     * @param id      the field's id; its label is {@code deeptime.editor.field.<id>}
     * @param path    dotted JSON path inside the preset ({@code planet.gravity_m_s2})
     * @param unit    shown after the value ({@code m/s²}); empty for none
     * @param phase3  a fantasy toggle: shown, always disabled until Phase 3 (decisions file, "Fantasy terrain")
     * @param integer a whole number, written as a JSON integer (the seed is a u64 in the core)
     */
    public record Field(String id, Section section, Kind kind, String path, double min, double max, String unit, boolean phase3,
                        boolean integer) {

        public String labelKey() {
            return "editor.field." + id;
        }

        /** The value at {@link #path} in {@code preset}, if present. */
        public Optional<JsonElement> get(JsonObject preset) {
            JsonElement e = preset;
            for (String part : path.split("\\.")) {
                if (e == null || !e.isJsonObject())
                    return Optional.empty();
                e = e.getAsJsonObject().get(part);
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

    /** Where the catalogue sits in the jar. */
    public static final String RESOURCE = "/deeptime/editor_fields.json";

    /**
     * Every field, in the setup page's order, from {@link #RESOURCE}. The planet, scale and seed
     * paths and ranges are P1-03's {@code params.rs} ({@code PlanetParams}, {@code ScaleParams},
     * {@code PlanetPreset::validate}); the others are proposals for the stages that have not landed
     * (their ranges are the editor's sanity bounds; the core validates in the end).
     */
    public static final List<Field> ALL = load();

    /** Reads the catalogue; a jar without it is broken, so this throws. */
    static List<Field> load() {
        try (InputStream in = EditorFields.class.getResourceAsStream(RESOURCE)) {
            if (in == null)
                throw new IllegalStateException("the editor catalogue " + RESOURCE + " is missing from the jar; reinstall Deep Time");
            return parse(JsonParser.parseReader(new InputStreamReader(in, StandardCharsets.UTF_8)).getAsJsonObject());
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }

    /** The fields of a catalogue ({@code {"fields": [...]}}); throws on a malformed entry. */
    static List<Field> parse(JsonObject catalogue) {
        List<Field> out = new ArrayList<>();
        for (JsonElement e : catalogue.getAsJsonArray("fields")) {
            JsonObject o = e.getAsJsonObject();
            String id = o.get("id").getAsString();
            try {
                out.add(new Field(id,
                        Section.valueOf(o.get("section").getAsString().toUpperCase(Locale.ROOT)),
                        Kind.valueOf(o.get("kind").getAsString().toUpperCase(Locale.ROOT)),
                        o.get("path").getAsString(),
                        o.get("min").getAsDouble(),
                        o.get("max").getAsDouble(),
                        o.has("unit") ? o.get("unit").getAsString() : "",
                        o.has("phase3") && o.get("phase3").getAsBoolean(),
                        o.has("integer") && o.get("integer").getAsBoolean()));
            } catch (RuntimeException ex) {
                throw new IllegalStateException("editor field '" + id + "' in " + RESOURCE + ": " + ex.getMessage(), ex);
            }
        }
        return List.copyOf(out);
    }

    /** The field with this id. */
    public static Field byId(String id) {
        for (Field f : ALL)
            if (f.id().equals(id))
                return f;
        throw new IllegalArgumentException("no editor field '" + id + "'");
    }

    /** Sets {@code path} in {@code preset} (in place), creating objects on the way. */
    static void set(JsonObject preset, String path, JsonElement value) {
        String[] parts = path.split("\\.");
        JsonObject o = preset;
        for (int i = 0; i < parts.length - 1; i++) {
            JsonElement next = o.get(parts[i]);
            if (next == null || !next.isJsonObject()) {
                JsonObject created = new JsonObject();
                o.add(parts[i], created);
                o = created;
            } else {
                o = next.getAsJsonObject();
            }
        }
        o.add(parts[parts.length - 1], value);
    }
}
