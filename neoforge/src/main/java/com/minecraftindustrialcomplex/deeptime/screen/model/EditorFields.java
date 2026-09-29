// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;

import java.util.List;
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
 */
public final class EditorFields {

    private EditorFields() {}

    public enum Section { PLANET, SCALE, MULTIPLIERS, FANTASY, PLAYABILITY }

    public enum Kind { NUMBER, TOGGLE, CHOICE }

    /**
     * One editable parameter.
     *
     * @param id      the field's id; its label is {@code deeptime.editor.field.<id>}
     * @param path    dotted JSON path inside the preset ({@code planet.gravity_m_s2})
     * @param unit    shown after the value ({@code m/s²}); empty for none
     * @param phase3  a fantasy toggle: shown, always disabled until Phase 3 (decisions file, "Fantasy terrain")
     */
    public record Field(String id, Section section, Kind kind, String path, double min, double max, String unit, boolean phase3) {

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

        /** Checks a number against the range. */
        public boolean inRange(double v) {
            return !Double.isNaN(v) && v >= min && v <= max;
        }
    }

    private static Field number(String id, Section s, String path, double min, double max, String unit) {
        return new Field(id, s, Kind.NUMBER, path, min, max, unit, false);
    }

    /** Every field, in the setup page's order. Ranges are the editor's sanity bounds, not physics. */
    public static final List<Field> ALL = List.of(
            number("gravity", Section.PLANET, "planet.gravity_m_s2", 0.1, 100, "m/s²"),
            number("day_length", Section.PLANET, "planet.day_length_h", 1, 10_000, "h"),
            number("axial_tilt", Section.PLANET, "planet.axial_tilt_deg", 0, 90, "°"),
            number("star_flux", Section.PLANET, "planet.star_flux_rel", 0.1, 10, "×"),
            number("ocean_fraction", Section.PLANET, "planet.ocean_fraction", 0, 1, ""),
            number("m_per_block_h", Section.SCALE, "scale.m_per_block_h", 10, 10_000, "m"),
            number("m_per_block_v", Section.SCALE, "scale.m_per_block_v", 1, 200, "m"),
            number("deposit_count", Section.MULTIPLIERS, "deposits.count_multiplier", 0, 10, "×"),
            number("deposit_size", Section.MULTIPLIERS, "deposits.size_multiplier", 0.1, 10, "×"),
            number("deposit_richness", Section.MULTIPLIERS, "deposits.richness_multiplier", 0.1, 10, "×"),
            number("province_strength", Section.MULTIPLIERS, "deposits.province_strength", 0, 1, ""),
            number("feature_deposits", Section.MULTIPLIERS, "scale.feature_scale.deposits", 0.1, 10, "×"),
            number("feature_rivers", Section.MULTIPLIERS, "scale.feature_scale.rivers", 0.1, 10, "×"),
            number("feature_canyons", Section.MULTIPLIERS, "scale.feature_scale.canyons", 0.1, 10, "×"),
            number("feature_strata", Section.MULTIPLIERS, "scale.feature_scale.strata", 0.1, 10, "×"),
            number("drama", Section.MULTIPLIERS, "surface.drama", 0, 2, ""),
            new Field("fantasy_fields", Section.FANTASY, Kind.TOGGLE, "fantasy.fields", 0, 1, "", true),
            new Field("sky_islands", Section.FANTASY, Kind.TOGGLE, "fantasy.sky_islands", 0, 1, "", true),
            new Field("exotic_landforms", Section.FANTASY, Kind.TOGGLE, "fantasy.exotic_landforms", 0, 1, "", true),
            new Field("essentials", Section.PLAYABILITY, Kind.CHOICE, "playability.essentials", 0, 0, "", false));

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
