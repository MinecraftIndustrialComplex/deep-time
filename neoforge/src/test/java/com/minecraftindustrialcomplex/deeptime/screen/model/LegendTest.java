// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

/** The Map tab's legends, read from the ramps the frames are coloured with. */
class LegendTest {

    static final Ramps RAMPS = Ramps.builtin();

    static <T extends Legend.Part> List<T> parts(List<Legend.Part> l, Class<T> type) {
        return l.stream().filter(type::isInstance).map(type::cast).toList();
    }

    @Test
    void satelliteListsItsClassesUnderTheirGroups() {
        List<Legend.Part> l = Legend.of(RAMPS, "satellite");
        Ramps.Ramp r = RAMPS.ramp("satellite");
        List<Legend.Swatch> swatches = parts(l, Legend.Swatch.class);
        assertEquals(Phrase.literal(r.labels().get(0)), swatches.get(0).text());
        assertEquals(r.colour(0.5), swatches.get(0).rgb());
        assertEquals(List.of(Phrase.literal("Land"), Phrase.literal("Snow and ice"), Phrase.literal("Sea")),
                parts(l, Legend.Heading.class).stream().map(Legend.Heading::text).toList());
        assertInstanceOf(Legend.Heading.class, l.get(0), "the first group's heading comes first");
        // Every class once; the lake keys follow, the salt flat (a class of its own) not twice.
        long classes = swatches.stream().filter(s -> s.text().isLiteral()).count();
        assertEquals(r.labels().size(), classes);
        List<Phrase> keys = swatches.stream().filter(s -> !s.text().isLiteral()).map(Legend.Swatch::text).toList();
        assertTrue(keys.contains(Phrase.of("legend.key.lake")), keys::toString);
        assertTrue(keys.contains(Phrase.of("legend.key.frozen_lake")), keys::toString);
        assertFalse(keys.contains(Phrase.of("legend.key.salt_flat")), keys::toString);
    }

    @Test
    void codedCategoriesAreNamed() {
        List<Legend.Part> v = Legend.of(RAMPS, "volcanoes/map");
        assertEquals(7, v.size());
        assertEquals(Phrase.of("legend.volcanoes.1"), ((Legend.Swatch) v.get(0)).text());
        assertEquals(RAMPS.ramp("volcanoes").colour(1.5), ((Legend.Swatch) v.get(0)).rgb());
        assertEquals(10, Legend.of(RAMPS, "regime").size());
        assertEquals(4, Legend.of(RAMPS, "impacts/craters").size());
        assertEquals(List.of(new Legend.Note(Phrase.of("legend.note.plates"))), Legend.of(RAMPS, "plates"));
        assertEquals(List.of(new Legend.Note(Phrase.of("legend.note.categories"))), Legend.of(RAMPS, "structure/rock"),
                "rock classes: pairs with a single last stop");
        assertEquals(List.of(new Legend.Note(Phrase.of("legend.note.categories"))), Legend.of(RAMPS, "deposits/sites"),
                "deposit sites: one stop per code");
        List<Legend.Part> grade = Legend.of(RAMPS, "structure/log_grade");
        assertEquals(5, grade.size());
        assertEquals(Phrase.of("legend.metamorphic_grade.4"), ((Legend.Swatch) grade.get(4)).text());
    }

    @Test
    void continuousLayersGetABarInTheirUnit() {
        Legend.Bar t = (Legend.Bar) Legend.of(RAMPS, "climate/t_mean_c").get(0);
        Ramps.Ramp r = RAMPS.ramp("climate_t");
        assertEquals(Phrase.of("legend.value.c", Formats.number(r.min())), t.low());
        assertEquals(Phrase.of("legend.value.c", Formats.number(r.max())), t.high());
        assertEquals(Legend.BAR_STEPS, t.rgb().length);
        assertEquals(r.colour(r.min()), t.rgb()[0]);
        assertEquals(r.colour(r.max()), t.rgb()[Legend.BAR_STEPS - 1]);
        Legend.Bar e = (Legend.Bar) Legend.of(RAMPS, "elevation").get(0);
        assertEquals(Phrase.of("legend.value.y", "-352"), e.low(), "heights as Y levels");
        Legend.Bar u = (Legend.Bar) Legend.of(RAMPS, "climate/aridity").get(0);
        assertEquals(Phrase.of("legend.low"), u.low(), "no unit: low and high");
        assertEquals(Phrase.of("legend.high"), u.high());
    }

    @Test
    void riversShowTheirKeysAndAFlowBar() {
        List<Legend.Part> l = Legend.of(RAMPS, "rivers");
        assertEquals(Phrase.of("legend.key.land"), ((Legend.Swatch) l.get(0)).text());
        Legend.Bar flow = parts(l, Legend.Bar.class).get(0);
        assertEquals(Phrase.of("legend.value.flow", "2"), flow.low(), "10^0.3 m³/s to two figures");
        assertEquals(Phrase.of("legend.value.flow", "3200"), flow.high());
        assertTrue(l.contains(new Legend.Heading(Phrase.of("legend.rivers.flow"))));
    }

    @Test
    void unitsAndCategories() {
        assertEquals("m_myr", Legend.unit("surface/erosion_m_myr"));
        assertEquals("mm", Legend.unit("climate/precip_mm"));
        assertEquals("m", Legend.unit("surface/sediment_m"));
        assertEquals("c", Legend.unit("climate/t_cold_c"));
        assertEquals("myr", Legend.unit("meta/crust_age_myr"));
        assertEquals("", Legend.unit("climate/koppen"));
        assertEquals("", Legend.unit("_m"), "a bare suffix is not a unit");
        assertTrue(Legend.categorical(RAMPS.ramp("plates")));
        assertFalse(Legend.categorical(RAMPS.ramp("elevation")));
        assertFalse(Legend.categorical(RAMPS.ramp("diverging")), "-1, 0, 1 is a scale, not three codes");
        assertFalse(Legend.categorical(RAMPS.ramp("ice")), "one pair-like step does not make classes");
        assertEquals(2.0, Legend.roundSignificant(1.995), 1e-12);
        assertEquals(3200.0, Legend.roundSignificant(3162.3), 1e-9);
    }
}
