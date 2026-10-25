// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.minecraftindustrialcomplex.deeptime.screen.model.CustomizeTopics.Topic;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;

/** The Customize page's topics over the shared editor catalogue (docs/design/create-world-ux.md, "Customize"). */
class CustomizeTopicsTest {

    @Test
    void everySettingIsInExactlyOneTopic() {
        List<EditorFields.Field> listed = new ArrayList<>();
        for (Topic t : Topic.values())
            listed.addAll(CustomizeTopics.fields(t));
        assertEquals(EditorFields.ALL.size(), listed.size(), "no setting twice");
        assertEquals(new HashSet<>(EditorFields.ALL), new HashSet<>(listed), "no setting left out");
        for (Topic t : Topic.values())
            assertFalse(CustomizeTopics.fields(t).isEmpty(), t + " has settings");
    }

    @Test
    void topicsReadAsAPlayerWould() {
        assertEquals(Topic.PLANET, CustomizeTopics.of(EditorFields.byId("gravity")));
        assertEquals(Topic.PLATES, CustomizeTopics.of(EditorFields.byId("plate_count")));
        assertEquals(Topic.CLIMATE, CustomizeTopics.of(EditorFields.byId("steering")));
        assertEquals(Topic.CLIMATE, CustomizeTopics.of(EditorFields.byId("biome_temperature")));
        assertEquals(Topic.SURFACE, CustomizeTopics.of(EditorFields.byId("rivers_lakes")));
        assertEquals(Topic.SURFACE, CustomizeTopics.of(EditorFields.byId("volcano_density")));
        assertEquals(Topic.ROCKS, CustomizeTopics.of(EditorFields.byId("deposit_count")));
        assertEquals(Topic.ROCKS, CustomizeTopics.of(EditorFields.byId("essentials")), "guaranteed materials are about rocks");
        assertEquals("customize.section.essentials", CustomizeTopics.heading(EditorFields.byId("essentials")));
        assertEquals("customize.section.adv_climate", CustomizeTopics.heading(EditorFields.byId(
                CustomizeTopics.fields(Topic.CLIMATE).stream().filter(EditorFields.Field::advanced).findFirst().orElseThrow().id())));
        assertEquals(Topic.ADVANCED, CustomizeTopics.of(EditorFields.byId("sky_islands")));
        assertEquals("customize.topic.surface", Topic.SURFACE.key());
        Set<String> headings = new HashSet<>(CustomizeTopics.headingKeys());
        for (EditorFields.Field f : EditorFields.ALL)
            assertTrue(headings.contains(CustomizeTopics.heading(f)), f.id());
    }

    @Test
    void everydaySettingsComeBeforeTheFineTuning() {
        for (Topic t : Topic.values()) {
            List<EditorFields.Field> fs = CustomizeTopics.fields(t);
            boolean fine = false;
            for (EditorFields.Field f : fs) {
                if (f.advanced())
                    fine = true;
                else
                    assertFalse(fine, t + ": " + f.id() + " after a fine-tuning setting");
            }
        }
        List<EditorFields.Field> surface = CustomizeTopics.fields(Topic.SURFACE);
        assertEquals(EditorFields.Section.TERRAIN, surface.get(0).section());
    }

    @Test
    void searchMatchesNamesPathsAndHintsWordByWord() {
        assertEquals(List.of(), CustomizeTopics.search("", f -> f.id()));
        assertEquals(List.of(), CustomizeTopics.search("   ", f -> f.id()));
        assertEquals(List.of(EditorFields.byId("gravity")), CustomizeTopics.search("gravity_m_s2", f -> f.id()), "by path");
        assertTrue(CustomizeTopics.search("GRAVITY", f -> f.id()).contains(EditorFields.byId("gravity")), "any case");
        assertTrue(CustomizeTopics.search("Pull", f -> f.id().equals("gravity") ? "Pull of the planet" : f.id())
                .contains(EditorFields.byId("gravity")), "by the label the player reads");
        List<EditorFields.Field> both = CustomizeTopics.search("deposits count", f -> f.id());
        assertTrue(both.contains(EditorFields.byId("deposit_count")), both.toString());
        assertFalse(both.contains(EditorFields.byId("deposit_size")), "every word must match");
        List<EditorFields.Field> rivers = CustomizeTopics.search("river", f -> f.id());
        assertTrue(rivers.size() > 1);
        assertEquals(EditorFields.ALL.stream().filter(rivers::contains).toList(), rivers, "catalogue order");
    }

    @Test
    void rangesAndTheSlowWarning() {
        assertEquals(Phrase.of("customize.range.unit", "0.1", "99.9", "m/s²"), CustomizeTopics.range(EditorFields.byId("gravity")));
        assertEquals(Phrase.of("customize.range.toggle"), CustomizeTopics.range(EditorFields.byId("rivers_lakes")));
        EditorFields.Field steeringShape = EditorFields.byId("steering_shape");
        assertEquals(Phrase.of("customize.range.choices", steeringShape.options().size()), CustomizeTopics.range(steeringShape));
        assertEquals(Phrase.of("customize.range.list"), CustomizeTopics.range(EditorFields.byId("essentials")));
        EditorFields.Field ocean = EditorFields.byId("ocean_fraction");
        assertEquals("customize.range", CustomizeTopics.range(ocean).key(), "no unit");
        assertEquals(Phrase.of("customize.range.min", "0", ""), CustomizeTopics.range(EditorFields.byId("seed")),
                "the seed's 2^53 - 1 is no limit a player meets: its range says only where it starts");
        assertFalse(CustomizeTopics.slow(null));
        assertFalse(CustomizeTopics.slow(new Estimate(1L << 30, List.of(new Estimate.Stage("plates", 1L << 30, 600)))));
        assertTrue(CustomizeTopics.slow(new Estimate(1L << 30, List.of(new Estimate.Stage("plates", 1L << 30, 1200),
                new Estimate.Stage("climate", 1L << 29, 900)))));
    }
}
