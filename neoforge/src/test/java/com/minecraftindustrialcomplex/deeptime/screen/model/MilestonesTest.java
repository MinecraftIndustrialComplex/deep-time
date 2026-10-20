// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;

/** The History list: the event log grouped by time window and kind, counted, in plain words. */
class MilestonesTest {

    static GeoEvent ev(long id, double t, String kind) {
        return new GeoEvent(id, t, t, "deeptime:" + kind, "point", 10, 20, -1, 0.5);
    }

    @Test
    void windowsAreAFiftiethOfTheRun() {
        assertEquals(20e6, Milestones.bucketYears(1000e6));
        assertEquals(1e6, Milestones.bucketYears(10e6), "at least a million years");
        assertEquals(1e6, Milestones.bucketYears(Double.NaN));
    }

    @Test
    void groupsCountsAndOrders() {
        List<GeoEvent> events = List.of(
                ev(1, -519e6, "rift_onset"), ev(2, -505e6, "rift_onset"), ev(3, -501e6, "rift_onset"),
                ev(4, -510e6, "collision"),
                ev(5, -100e6, "ridge_birth"),
                ev(6, -50e6, "glaciation")); // after the cursor
        List<Milestones.Row> rows = Milestones.of(events, -60e6, 20e6);
        assertEquals(3, rows.size(), rows::toString);
        // Newest window first; within a window the newest row first, so the times read in order.
        assertEquals("deeptime:ridge_birth", rows.get(0).kind());
        assertEquals(1, rows.get(0).count());
        Milestones.Row rifts = rows.get(1);
        assertEquals("deeptime:rift_onset", rifts.kind());
        assertEquals(3, rifts.count());
        assertEquals(-501e6, rifts.tYears(), "the row's time is its newest event's");
        assertEquals("rift_onset", rifts.kindPath());
        assertEquals(Phrase.of("milestone.rift_onset.many", 3), rifts.phrase());
        assertEquals(EventStyle.colour("deeptime:rift_onset"), rifts.colour(), "coloured like the map's markers");
        Milestones.Row collision = rows.get(2);
        assertEquals(Phrase.of("milestone.collision"), collision.phrase(), "one event: the singular words, no count");
        // Same window: a newer single event comes before an older frequent one; ties go to the most frequent.
        List<Milestones.Row> mixed = Milestones.of(List.of(
                ev(1, -519e6, "rift_onset"), ev(2, -518e6, "rift_onset"), ev(3, -517e6, "rift_onset"),
                ev(4, -503e6, "collision"), ev(5, -503e6, "plate_birth"), ev(6, -503e6, "plate_birth")),
                Double.POSITIVE_INFINITY, 20e6);
        assertEquals(List.of("deeptime:plate_birth", "deeptime:collision", "deeptime:rift_onset"),
                mixed.stream().map(Milestones.Row::kind).toList());
        // Every event (the cursor at the end).
        assertEquals(4, Milestones.of(events, Double.POSITIVE_INFINITY, 20e6).size());
        assertEquals(List.of(), Milestones.of(events, -600e6, 20e6), "nothing before the first event");
    }

    @Test
    void theListKeepsTheNewestRows() {
        List<GeoEvent> events = new ArrayList<>();
        for (int i = 0; i < Milestones.MAX_ROWS + 50; i++)
            events.add(ev(i, -1e9 + i * 1e6, "plate_birth"));
        List<Milestones.Row> rows = Milestones.of(events, 0, 1e6);
        assertEquals(Milestones.MAX_ROWS, rows.size());
        assertEquals(-1e9 + (Milestones.MAX_ROWS + 49) * 1e6, rows.get(0).tYears(), "the newest first");
        assertTrue(rows.get(rows.size() - 1).tYears() > -1e9 + 49 * 1e6, "the oldest went");
    }
}
