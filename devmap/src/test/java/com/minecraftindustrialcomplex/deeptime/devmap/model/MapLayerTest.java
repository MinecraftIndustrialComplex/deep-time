// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.devmap.model;

import org.junit.jupiter.api.Test;

import java.util.EnumSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;

/** The dev map's two pictures: satellite by default, the switch, the ids and the fallback. */
class MapLayerTest {

    @Test
    void satelliteIsTheDefaultAndTheSwitchGoesBackAndForth() {
        assertEquals(MapLayer.SATELLITE, MapLayer.DEFAULT);
        assertEquals(MapLayer.ELEVATION, MapLayer.SATELLITE.other());
        assertEquals(MapLayer.SATELLITE, MapLayer.ELEVATION.other());
        for (MapLayer l : MapLayer.values())
            assertEquals(l, l.other().other());
        assertEquals("Satellite", MapLayer.SATELLITE.label());
        assertEquals("Elevation", MapLayer.ELEVATION.label());
    }

    @Test
    void idsRoundTripAndAnythingElseIsTheDefault() {
        assertEquals("satellite", MapLayer.SATELLITE.id());
        assertEquals("elevation", MapLayer.ELEVATION.id());
        for (MapLayer l : MapLayer.values())
            assertEquals(l, MapLayer.parse(l.id()));
        for (String odd : new String[]{"", "SATELLITE", "water", "sat", null})
            assertEquals(MapLayer.DEFAULT, MapLayer.parse(odd), "'" + odd + "'");
    }

    @Test
    void theChosenPictureIsShownWhenHereElseTheOtherElseNone() {
        Set<MapLayer> both = EnumSet.allOf(MapLayer.class);
        Set<MapLayer> none = EnumSet.noneOf(MapLayer.class);
        for (MapLayer wanted : MapLayer.values()) {
            assertEquals(wanted, MapLayer.shown(wanted, both::contains));
            assertEquals(wanted, MapLayer.shown(wanted, EnumSet.of(wanted)::contains));
            assertEquals(wanted.other(), MapLayer.shown(wanted, EnumSet.of(wanted.other())::contains),
                    "the other picture while " + wanted + " is not here");
            assertNull(MapLayer.shown(wanted, none::contains));
        }
    }
}
