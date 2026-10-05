// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.preset.ShareCode;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/**
 * The world-shaping tuning knobs (settings audit; decisions log 2026-09-29: such knobs are settings with a
 * sensible default): the catalogue's terrain and Advanced sections against the real pipeline
 * presets, and their round trip through share codes, game and workbench. The Rust twin is
 * {@code watch/editor.rs} ({@code every_knob_is_in_every_pipeline_preset_and_both_ends_are_valid}).
 */
class AdvancedSettingsTest {

    @TempDir
    Path gameDir;

    static final long GIB = 1L << 30;

    static final List<String> PIPELINES = List.of("pipeline_quick_16k", "pipeline_full_16k", "pipeline_quick_32k", "pipeline_full_32k");

    /**
     * Fields no core parameter backs yet: canyons (Phase 2; rivers are P1-22's), the pack-defined
     * essentials lists (the datapack loader), and Phase 3's fantasy toggles.
     */
    static final Set<String> NO_PARAMETER = Set.of("feature_canyons", "fantasy_fields", "sky_islands",
            "exotic_landforms", "essentials");

    EditorModel model() {
        return new EditorModel(new EditorModel.Deps(json -> "{\"peak_bytes\":1000,\"stages\":[{\"bytes\":1,\"name\":\"plates\",\"seconds\":5.0}]}",
                (json, out, threads, cap, resume) -> new FakeRunHandle(out), gameDir, new MachineInfo(8 * GIB, 16 * GIB, 4 * GIB, 8),
                Runnable::run, Runnable::run, () -> true, List::of));
    }

    static Path repo() {
        return Path.of(System.getProperty("deeptime.repoRoot"));
    }

    static String pipelineText(String name) throws IOException {
        return Files.readString(repo().resolve("crates/deeptime-core/presets/pipeline/" + name + ".json"), StandardCharsets.UTF_8);
    }

    /** What the workbench hands over: the pipeline preset with a timeline, as a share code. */
    static String pipelineCode(String name) throws IOException {
        JsonObject o = Presets.parseObject(pipelineText(name));
        return ShareCode.encode(Presets.withTimeline(o, new Presets.Timeline("scattered_continents", 100)));
    }

    /** A model with a pipeline preset imported, as if pasted from the workbench. */
    EditorModel imported(String name) throws IOException {
        EditorModel m = model();
        assertTrue(m.importShareCode(pipelineCode(name)).isEmpty());
        assertTrue(m.isImported());
        return m;
    }

    /** Every knob is in every pipeline preset, at a value inside the range the editor allows. */
    @Test
    void pipelinePresetsCarryEveryKnob() throws IOException {
        int knobs = 0;
        for (String name : PIPELINES) {
            JsonObject o = Presets.parseObject(pipelineText(name));
            for (EditorFields.Field f : EditorFields.ALL) {
                if (NO_PARAMETER.contains(f.id())) {
                    assertFalse(f.availableIn(o), name + " unexpectedly carries " + f.path() + " (EditorFields." + f.id() + ")");
                    continue;
                }
                assertTrue(f.availableIn(o), name + " has no " + f.path() + " (EditorFields." + f.id() + ")");
                knobs++;
                JsonElement v = f.get(o).orElseThrow();
                switch (f.kind()) {
                    case NUMBER -> assertTrue(f.inRange(v.getAsDouble()), name + ": " + f.id() + " = " + v + " is outside " + f.min() + ".." + f.max());
                    case CHOICE -> assertTrue(f.options().contains(v.getAsString()), name + ": " + f.id() + " = " + v);
                    default -> { }
                }
            }
        }
        assertEquals(PIPELINES.size() * (EditorFields.ALL.size() - NO_PARAMETER.size()), knobs);
    }

    /** The game's built-ins are the pipeline presets under their names (P1-08): every knob is in each. */
    @Test
    void builtInPresetsCarryEveryKnob() {
        for (String name : Presets.BUILTIN) {
            JsonObject o = Presets.parseObject(Presets.builtin(name).orElseThrow());
            for (EditorFields.Field f : EditorFields.ALL) {
                boolean carried = f.availableIn(o);
                assertEquals(!NO_PARAMETER.contains(f.id()), carried, name + ": " + f.id() + " at " + f.path());
            }
        }
    }

    /**
     * P1-26's per-cell climate settings (design §5.7): each is an Advanced climate field whose
     * path is {@code climate.<key>}, every pipeline preset and built-in carries it at the calibrated
     * default {@code ClimateParams::default()} holds, and the default lies inside the editor's range.
     */
    @Test
    void perCellClimateSettingsCarryTheCalibratedDefaults() throws IOException {
        record Num(String id, String key, double value, boolean integer) {}
        List<Num> numbers = List.of(
                new Num("climate_grid", "grid_columns", 128, true),
                new Num("atmosphere_mixing", "atmosphere_diffusivity_scale", 1.3, false),
                new Num("zonal_mixing", "zonal_diffusivity_ratio", 0.2, false),
                new Num("wind_heat", "wind_heat_transport", 0.2, false),
                new Num("desert_albedo", "desert_albedo", 0.08, false),
                new Num("monsoon_strength", "thermal_wind_pa_k", 60, false),
                new Num("rain_efficiency", "precip_rh_slope", 11.4, false),
                new Num("soil_water", "bucket_mm", 150, false),
                new Num("rain_spread", "rain_diffusivity_m2_s", 500000, false),
                new Num("rain_wind", "rain_wind_share", 0.3, false),
                new Num("rain_recycling", "rain_recycling", 1.0, false),
                new Num("sea_humidity", "sea_surface_rh", 0.8, false),
                new Num("rain_humidity", "rain_rh_slope", 0, false),
                new Num("east_coast_rain", "west_flank_wet", 0.4, false),
                new Num("ocean_gyres", "gyre_heat_efficiency", 1.0, false),
                new Num("ocean_overturning", "overturning_sv", 30, false),
                new Num("ocean_mixing", "ocean_diffusivity_w_m2_k", 0.3, false),
                new Num("winter_inversions", "winter_inversion_c", 15, false),
                new Num("sea_ice_edges", "sea_ice_edge_sharpness", 4, false),
                new Num("snowball_clouds", "snowball_cloud_w_m2", 0, false));
        List<JsonObject> presets = new ArrayList<>();
        for (String name : PIPELINES)
            presets.add(Presets.parseObject(pipelineText(name)));
        for (String name : Presets.BUILTIN)
            presets.add(Presets.parseObject(Presets.builtin(name).orElseThrow()));
        for (Num n : numbers) {
            EditorFields.Field f = EditorFields.byId(n.id());
            assertEquals(EditorFields.Section.ADV_CLIMATE, f.section(), n.id());
            assertEquals(EditorFields.Kind.NUMBER, f.kind(), n.id());
            assertEquals("climate." + n.key(), f.path(), n.id());
            assertEquals(n.integer(), f.integer(), n.id());
            assertTrue(f.inRange(n.value()), n.id() + " default " + n.value() + " is outside " + f.min() + ".." + f.max());
            for (JsonObject o : presets)
                assertEquals(n.value(), f.get(o).orElseThrow(() -> new AssertionError(n.id())).getAsDouble(), 0.0, n.id());
        }
        EditorFields.Field model = EditorFields.byId("climate_model");
        assertEquals("climate.model", model.path());
        assertEquals(List.of("per_cell", "banded"), model.options());
        EditorFields.Field rain = EditorFields.byId("rain_model");
        assertEquals("climate.rain_model", rain.path());
        assertEquals(List.of("belts", "budget", "upwind"), rain.options());
        for (JsonObject o : presets) {
            assertEquals("per_cell", model.get(o).orElseThrow().getAsString());
            assertEquals("belts", rain.get(o).orElseThrow().getAsString());
        }
        // The two choices write the core's names back, and an invalid name is refused.
        EditorModel m = imported("pipeline_quick_16k");
        assertEquals(java.util.Optional.of("per_cell"), m.choice(model));
        m.setChoice(model, "banded");
        m.setChoice(rain, "budget");
        m.setFieldText(EditorFields.byId("climate_grid"), "180");
        String json = m.effectivePresetJson();
        assertTrue(json.contains("\"model\":\"banded\""), json);
        assertTrue(json.contains("\"rain_model\":\"budget\""), json);
        assertTrue(json.contains("\"grid_columns\":180,"), "an integer stays an integer: " + json);
        m.setFieldText(EditorFields.byId("climate_grid"), "128.5");
        assertFalse(m.fieldValid(EditorFields.byId("climate_grid")), "a whole number");
        m.setFieldText(EditorFields.byId("climate_grid"), "8");
        assertFalse(m.fieldValid(EditorFields.byId("climate_grid")), "below the core's 16");
    }

    /** The Advanced settings are closed until asked for, and counted against what the preset carries. */
    @Test
    void advancedSettingsAreClosedByDefault() throws IOException {
        EditorModel m = model();
        List<EditorFields.Field> adv = EditorFields.advancedFields();
        assertTrue(adv.size() >= 25, "the audit's knobs: " + adv.size());
        assertFalse(m.advancedOpen(), "collapsed by default");
        assertEquals(adv.size(), m.advancedAvailable(), "the built-in Earth-like presets carry every one of them");
        for (EditorFields.Field f : adv)
            assertTrue(m.available(f), f.id());

        // A stand-in preset (the dummy pipeline) carries none.
        EditorModel none = model();
        assertTrue(none.importShareCode(ShareCode.encode("{\"name\":\"stand_in\",\"placeholder\":true,"
                + "\"scale\":{\"circumference_blocks\":16384},\"dummy\":{\"steps\":1,\"sleep_ms\":0}}")).isEmpty());
        assertEquals(0, none.advancedAvailable());

        EditorModel p = imported("pipeline_quick_16k");
        assertFalse(p.advancedOpen(), "a new preset does not open them");
        assertEquals(adv.size(), p.advancedAvailable());
        assertEquals(0, p.advancedEdited());
        p.setAdvancedOpen(true);
        assertTrue(p.advancedOpen());
        p.selectSize(p.circumference());
        assertTrue(p.advancedOpen(), "still open after a redraw's worth of model calls");
        p.setFieldText(EditorFields.byId("erosion_strength"), "5e-9");
        assertEquals(1, p.advancedEdited());
        p.setFieldText(EditorFields.byId("carving"), "10");
        assertEquals(1, p.advancedEdited(), "the terrain section is not one of the advanced ones");
        p.resetFields();
        assertEquals(0, p.advancedEdited());
    }

    /** The catalogue's shape, which both editors rely on. */
    @Test
    void theCatalogueGroupsTheAdvancedSectionsLast() {
        List<EditorFields.Section> adv = EditorFields.advancedSections();
        assertEquals(List.of(EditorFields.Section.ADV_PLATES, EditorFields.Section.ADV_SURFACE, EditorFields.Section.ADV_CLIMATE,
                EditorFields.Section.ADV_ROCKS, EditorFields.Section.ADV_VOLCANOES, EditorFields.Section.ADV_IMPACTS,
                EditorFields.Section.ADV_DEPOSITS,
                EditorFields.Section.ADV_BIOMES, EditorFields.Section.ADV_WATER, EditorFields.Section.ADV_UNDERGROUND,
                EditorFields.Section.ADV_GEOTHERMAL), adv);
        assertEquals(Set.copyOf(adv), EditorFields.ADVANCED);
        boolean seen = false;
        for (EditorFields.Field f : EditorFields.ALL) {
            assertTrue(f.hint().length() > 20, f.id() + " has a hint");
            if (f.advanced())
                seen = true;
            else
                assertFalse(seen, f.id() + " comes after an advanced field: the advanced sections must be last");
        }
        EditorFields.Field province = EditorFields.byId("province_strength");
        assertEquals(EditorFields.Kind.CHOICE, province.kind());
        assertEquals(List.of("off", "weak", "medium", "strong"), province.options());
        assertEquals("deeptime.editor.option.province_strength.weak", "deeptime." + province.optionKey("weak"));
        assertEquals("deposits.count_mult", EditorFields.byId("deposit_count").path(), "the key the core reads");
        assertEquals("amplify.incision_cap", EditorFields.byId("carving").path());
        assertEquals(EditorFields.Section.TERRAIN, EditorFields.byId("carving").section());
        assertEquals(EditorFields.Section.TERRAIN, EditorFields.byId("drama").section());
        assertTrue(EditorFields.byId("plate_count").integer());
        assertTrue(EditorFields.byId("continent_count").integer());
        assertFalse(EditorFields.byId("erosion_strength").integer());
    }

    /** A number in a path indexes an array (the pluton size is the median of [median, min, max]). */
    @Test
    void pathsIndexArrays() throws IOException {
        EditorModel m = imported("pipeline_quick_16k");
        EditorFields.Field size = EditorFields.byId("pluton_size");
        assertEquals("structure.pluton_length_km.0", size.path());
        assertEquals("25", m.fieldText(size), "the median of [25, 8, 80]");
        m.setFieldText(size, "44");
        assertTrue(m.effectivePresetJson().contains("\"pluton_length_km\":[44.0,8.0,80.0]"), m.effectivePresetJson());
        m.setFieldText(size, "7");
        assertFalse(m.fieldValid(size), "below the shortest body (8 km)");
        assertTrue(m.effectivePresetJson().contains("\"pluton_length_km\":[25.0,8.0,80.0]"), "an invalid value is not used");

        EditorFields.Field swing = EditorFields.byId("sea_level_swing");
        assertEquals("80", m.fieldText(swing), "the first sea-level cycle");
        m.setFieldText(swing, "120");
        String json = m.effectivePresetJson();
        assertTrue(json.contains("\"cycles\":[{\"amplitude_m\":120.0,\"period_myr\":250.0,\"phase_deg\":0.0},{\"amplitude_m\":25.0,"), json);

        // A preset without the element leaves the field unavailable, and an edit is ignored.
        JsonObject bare = Presets.parseObject(pipelineText("pipeline_quick_16k"));
        bare.getAsJsonObject("surface").getAsJsonObject("sea_level").add("cycles", new com.google.gson.JsonArray());
        EditorModel n = model();
        assertTrue(n.importShareCode(ShareCode.encode(bare.toString())).isEmpty());
        assertFalse(n.available(swing));
        n.setFieldText(swing, "120");
        assertFalse(n.effectivePresetJson().contains("\"amplitude_m\":120"), "no element, no edit");
    }

    /** Integers are written as JSON integers (the core reads u32), the other numbers as doubles. */
    @Test
    void integerKnobsStayIntegers() throws IOException {
        EditorModel m = imported("pipeline_quick_16k");
        EditorFields.Field plates = EditorFields.byId("plate_count");
        m.setFieldText(plates, "22");
        assertTrue(m.effectivePresetJson().contains("\"initial_plates\":22,"), m.effectivePresetJson());
        m.setFieldText(plates, "22.5");
        assertFalse(m.fieldValid(plates), "a whole number");
        m.setFieldText(plates, "3");
        assertFalse(m.fieldValid(plates), "below the editor's range");
        m.setFieldText(plates, "41");
        assertFalse(m.fieldValid(plates), "above it: max_plates is 40");
        assertTrue(m.effectivePresetJson().contains("\"initial_plates\":14,"), "an invalid value is not used");
    }

    /** The province choice: a fixed list, shown and set through the choice API. */
    @Test
    void provinceIsAFixedChoice() throws IOException {
        EditorModel m = imported("pipeline_quick_16k");
        EditorFields.Field f = EditorFields.byId("province_strength");
        assertTrue(m.available(f));
        assertEquals(List.of("off", "weak", "medium", "strong"), m.choices(f));
        assertEquals(java.util.Optional.of("strong"), m.choice(f), "the calibrated default");
        m.setChoice(f, "weak");
        assertEquals(java.util.Optional.of("weak"), m.choice(f));
        assertTrue(m.effectivePresetJson().contains("\"province\":\"weak\""));
        m.setChoice(f, "everything");
        assertEquals(java.util.Optional.of("weak"), m.choice(f), "a value outside the list is ignored");
        m.setChoice(EditorFields.byId("essentials"), "x");
        assertTrue(m.effectivePresetJson().contains("\"province\":\"weak\""));
    }

    /**
     * Every setting survives export → import, game to game and (the fixture) game to workbench:
     * the top of every range and the last option, through the code and back, changes nothing.
     */
    @Test
    void everySettingRoundTripsThroughAShareCode() throws IOException {
        EditorModel a = imported("pipeline_quick_16k");
        a.setStart("scattered_continents");
        a.setLengthText("100");
        List<String> edited = new ArrayList<>();
        for (EditorFields.Field f : EditorFields.ALL) {
            if (!a.available(f))
                continue;
            switch (f.kind()) {
                case NUMBER -> {
                    a.setFieldText(f, Formats.number(f.max()));
                    assertTrue(a.fieldValid(f), f.id() + " = " + Formats.number(f.max()));
                }
                case CHOICE -> a.setChoice(f, f.options().get(f.options().size() - 1));
                // Toggles (the deep-time climate's carbon cycle, brightening star): flipped.
                case TOGGLE -> a.setToggle(f, !a.toggle(f));
            }
            edited.add(f.id());
        }
        assertEquals(EditorFields.ALL.size() - NO_PARAMETER.size(), edited.size(), edited.toString());
        String code = a.exportShareCode();
        JsonObject out = Presets.parseObject(ShareCode.decode(code));
        for (EditorFields.Field f : EditorFields.ALL) {
            if (!a.available(f))
                continue;
            JsonElement v = f.get(out).orElseThrow(() -> new AssertionError(f.path()));
            switch (f.kind()) {
                case NUMBER -> assertEquals(f.max(), v.getAsDouble(), 0.0, f.id());
                case TOGGLE -> assertEquals(a.toggle(f), v.getAsBoolean(), f.id());
                case CHOICE -> assertEquals(f.options().get(f.options().size() - 1), v.getAsString(), f.id());
            }
        }
        EditorModel b = model();
        assertTrue(b.importShareCode(code).isEmpty());
        assertTrue(b.isImported(), "edits make it a custom preset");
        assertEquals(a.start(), b.start());
        assertEquals(a.lengthText(), b.lengthText());
        for (EditorFields.Field f : EditorFields.ALL) {
            assertEquals(a.available(f), b.available(f), f.id());
            if (f.kind() == EditorFields.Kind.NUMBER && a.available(f))
                assertEquals(a.fieldText(f), b.fieldText(f), f.id());
            if (f.kind() == EditorFields.Kind.TOGGLE && a.available(f))
                assertEquals(a.toggle(f), b.toggle(f), f.id());
        }
        assertEquals(a.effectivePresetJson(), b.effectivePresetJson());
        assertEquals(code, b.exportShareCode(), "one preset, one code");
        assertNotEquals(pipelineCode("pipeline_quick_16k"), code);
    }

    /**
     * The workbench's side of the round trip: a tuned pipeline preset the Rust side encoded (and
     * the code this side writes for it) loads with every tuned value in its field, and this side
     * writes back exactly the JSON that Rust wrote. {@code crates/deeptime-core/tests/share-codes.json},
     * case {@code pipeline_quick_16k_tuned}, which the Rust tests decode too.
     */
    @Test
    void aTunedPipelineCodeLoadsUnchanged() throws IOException {
        JsonObject fixture = Presets.parseObject(Files.readString(repo().resolve("crates/deeptime-core/tests/share-codes.json"), StandardCharsets.UTF_8));
        JsonObject tuned = null;
        for (JsonElement e : fixture.getAsJsonArray("cases"))
            if (e.getAsJsonObject().get("name").getAsString().equals("pipeline_quick_16k_tuned"))
                tuned = e.getAsJsonObject();
        assertTrue(tuned != null, "the fixture has the tuned pipeline case");
        String json = tuned.get("json").getAsString();
        for (String code : List.of(tuned.get("rust").getAsString(), tuned.get("java").getAsString())) {
            EditorModel m = model();
            assertTrue(m.importShareCode(code).isEmpty());
            assertEquals("scattered_continents", m.start().orElseThrow());
            assertEquals("100", m.lengthText());
            assertEquals("10", m.fieldText(EditorFields.byId("carving")));
            assertEquals("22", m.fieldText(EditorFields.byId("plate_count")));
            assertEquals("5e-9", m.fieldText(EditorFields.byId("erosion_strength")));
            assertEquals("-3000", m.fieldText(EditorFields.byId("ridge_depth")));
            assertEquals("40", m.fieldText(EditorFields.byId("pluton_size")));
            assertEquals("120", m.fieldText(EditorFields.byId("sea_level_swing")));
            assertEquals("2.5", m.fieldText(EditorFields.byId("deposit_size")));
            assertEquals("1.5", m.fieldText(EditorFields.byId("deposit_richness")));
            assertEquals("2", m.fieldText(EditorFields.byId("feature_deposits")));
            assertEquals("0.5", m.fieldText(EditorFields.byId("feature_strata")));
            assertEquals(java.util.Optional.of("medium"), m.choice(EditorFields.byId("province_strength")));
            // The biome placement settings.
            assertEquals("40", m.fieldText(EditorFields.byId("beach_width")));
            assertEquals("300", m.fieldText(EditorFields.byId("inland_mid")));
            assertEquals("1500", m.fieldText(EditorFields.byId("inland_far")));
            assertEquals("8000", m.fieldText(EditorFields.byId("inland_interior")));
            assertEquals("3", m.fieldText(EditorFields.byId("wetland_humidity")));
            assertEquals("0.5", m.fieldText(EditorFields.byId("wetland_flat_share")));
            assertEquals("250", m.fieldText(EditorFields.byId("wetland_lowland")));
            assertEquals(json, m.effectivePresetJson(), "nothing is added, dropped or rounded");
            assertEquals(json, ShareCode.decode(m.exportShareCode()));
        }
    }
}
