// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import org.junit.jupiter.api.Test;

/**
 * The timelapse's layer groups and names and its event colours follow the workbench, and every
 * layer a real planet produces and every event kind EVENTS.md lists has a name in the game's
 * language file (P1-11 on the real stages).
 */
class LayerCatalogTest {

    /**
     * Every layer of a real {@code earthlike_quick_16k} run (the workbench's run 1a103bb2d2b, master
     * 458a89a5, 2026-10-04): the coupled stages' 24, then the deposits, young and amplify frames.
     */
    static final List<String> REAL_LAYERS = List.of("satellite", "elevation", "plates", "crust_age", "regime",
            "climate/t_mean_c", "climate/precip_mm", "climate/koppen", "climate/ice_thickness_m", "climate/sea_ice",
            "surface/erosion_m_myr", "surface/deposition_m_myr", "surface/sediment_m", "surface/class", "surface/drainage",
            "surface/regolith_m", "surface/ice_thickness_m", "volcanoes/map", "impacts/craters", "structure/rock",
            "structure/log_grade", "structure/trap_structure", "provinces", "rivers",
            "deposits/favourability/porphyry_cu_mo_au", "deposits/favourability/bif_fe", "deposits/favourability/coal_seam",
            "deposits/favourability/mvt_zn_pb", "deposits/favourability/sedex_zn_pb", "deposits/favourability/petroleum_trap",
            "deposits/favourability/impact_ni_cu_pge", "deposits/favourability/impact_hydrothermal_cu_zn_au",
            "deposits/favourability/impact_diamond", "deposits/favourability/impact_basin_u", "deposits/sites",
            "input/elevation_m", "input/arc_magma_flux", "input/carbonate_platform", "input/euxinic_basin", "input/coal_facies",
            "input/erosion_rate_m_myr", "input/basin_brine", "input/regional_head_drop_m_km",
            "young/elevation", "young/drainage", "young/ice_thickness_m",
            "amplify/elevation", "amplify/detail", "amplify/drainage", "amplify/youth", "amplify/fold");

    static JsonObject lang() throws IOException {
        Path p = Path.of(System.getProperty("deeptime.repoRoot"), "neoforge/src/main/resources/assets/deeptime/lang/en_us.json");
        return JsonParser.parseString(Files.readString(p, StandardCharsets.UTF_8)).getAsJsonObject();
    }

    @Test
    void groupsFollowTheWorkbench() {
        assertEquals("", LayerCatalog.group("satellite"));
        assertEquals("deposits", LayerCatalog.group("deposits/favourability/bif_fe"));
        assertEquals(List.of("", "climate", "surface", "structure", "volcanoes", "impacts", "deposits", "input", "young", "amplify"),
                LayerCatalog.groups(REAL_LAYERS));
        Map<String, List<String>> by = LayerCatalog.byGroup(REAL_LAYERS);
        assertEquals(REAL_LAYERS.size(), by.values().stream().mapToInt(List::size).sum(), "every layer in one group");
        assertEquals(List.of("satellite", "elevation", "plates", "crust_age", "regime", "provinces", "rivers"), by.get(""));
        assertEquals(List.of("", "climate", "zz"), LayerCatalog.groups(List.of("zz/a", "climate/b", "c")), "unknown groups last");
        assertEquals("satellite", LayerCatalog.defaultLayer(REAL_LAYERS));
        assertEquals("elevation", LayerCatalog.defaultLayer(List.of("plates", "elevation")));
        assertEquals("plates", LayerCatalog.defaultLayer(List.of("plates")));
    }

    @Test
    void wordKeysDropUnitSuffixes() {
        assertEquals("deeptime.layer.word.erosion", LayerCatalog.wordKey("surface/erosion_m_myr"));
        assertEquals("deeptime.layer.word.precip", LayerCatalog.wordKey("climate/precip_mm"));
        assertEquals("deeptime.layer.word.t_mean", LayerCatalog.wordKey("climate/t_mean_c"));
        assertEquals("deeptime.layer.word.regional_head_drop_m", LayerCatalog.wordKey("input/regional_head_drop_m_km"),
                "one suffix only, as the workbench strips it");
        assertEquals("deeptime.layer.climate.koppen", LayerCatalog.key("climate/koppen"));
        assertEquals("deeptime.layer_group.planet", LayerCatalog.groupKey(""));
    }

    /** Every real layer has a name (its own key or its word), every group too: no humanised ids on screen. */
    @Test
    void everyRealLayerHasAName() throws IOException {
        JsonObject lang = lang();
        List<String> missing = new ArrayList<>();
        for (String l : REAL_LAYERS)
            if (!lang.has(LayerCatalog.key(l)) && !lang.has(LayerCatalog.wordKey(l)))
                missing.add(l + " (" + LayerCatalog.wordKey(l) + ")");
        for (String g : LayerCatalog.groups(REAL_LAYERS))
            if (!lang.has(LayerCatalog.groupKey(g)))
                missing.add("group " + LayerCatalog.groupKey(g));
        assertTrue(missing.isEmpty(), "no name for " + missing);
    }

    /** Every kind EVENTS.md lists in its kinds table has a name in the ticker. */
    @Test
    void everyEventKindHasAName() throws IOException {
        JsonObject lang = lang();
        String events = Files.readString(Path.of(System.getProperty("deeptime.repoRoot"), "crates/deeptime-format/EVENTS.md"));
        Matcher m = Pattern.compile("(?m)^\\| `deeptime:([a-z_]+)` \\|").matcher(events);
        List<String> kinds = new ArrayList<>(), missing = new ArrayList<>();
        while (m.find()) {
            kinds.add(m.group(1));
            if (!lang.has("deeptime.event." + m.group(1)))
                missing.add(m.group(1));
        }
        assertTrue(kinds.size() >= 30, "EVENTS.md lists " + kinds.size() + " kinds");
        assertTrue(missing.isEmpty(), "no ticker name for " + missing);
    }

    @Test
    void eventColoursFollowTheWorkbench() {
        assertEquals(0xffe6194b, EventStyle.colour("deeptime:collision"));
        assertEquals(0xffffd700, EventStyle.colour("deeptime:deposit_formation"));
        // An unlisted kind: a hue from its name, opaque, stable.
        int c = EventStyle.colour("deeptime:fold_belt");
        assertEquals(0xff, c >>> 24);
        assertEquals(c, EventStyle.colour("deeptime:fold_belt"));
        assertNotEquals(c, EventStyle.colour("deeptime:fault_system"));
        // CSS hsl(0, 70%, 60%) = rgb(224, 82, 82); hsl(120, 70%, 60%) = rgb(82, 224, 82).
        assertEquals(0xe05252, EventStyle.hsl(0, 0.70, 0.60));
        assertEquals(0x52e052, EventStyle.hsl(120, 0.70, 0.60));
    }
}
