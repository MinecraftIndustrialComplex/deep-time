// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The generation page's History list (docs/design/create-world-ux.md): the event log grouped and
 * counted in plain words instead of one row per event. Events are cut into time windows of
 * {@link #bucketYears} (a fiftieth of the run, at least a million years); within a window the
 * events of one kind make one row, "8 new ocean ridges formed". Newest window first, and within a
 * window the most frequent kind first. Each row carries its kind's colour ({@link EventStyle}, the
 * map markers' and the workbench's), so the list is also the markers' legend.
 */
public final class Milestones {

    private Milestones() {}

    /** The most rows the list keeps (the oldest go). */
    public static final int MAX_ROWS = 400;

    /**
     * One row.
     *
     * @param tYears the newest event's time in the row (years relative to world creation)
     * @param kind   the namespaced event kind
     * @param count  how many events of the kind fell in the window
     * @param colour the kind's colour, opaque ARGB
     */
    public record Row(double tYears, String kind, int count, int colour) {

        /** The kind without its namespace. */
        public String kindPath() {
            return EventStyle.shortKind(kind);
        }

        /** The row's words: {@code milestone.<kind>} for one event, {@code milestone.<kind>.many} with the count for more. */
        public Phrase phrase() {
            return count == 1 ? Phrase.of("milestone." + kindPath()) : Phrase.of("milestone." + kindPath() + ".many", count);
        }
    }

    /** The window length for a run of {@code lengthYears}: a fiftieth of it, at least a million years. */
    public static double bucketYears(double lengthYears) {
        return Double.isFinite(lengthYears) && lengthYears > 0 ? Math.max(1e6, lengthYears / 50) : 1e6;
    }

    /**
     * The rows of {@code events} (any order) up to {@code cursorYears} (inclusive; positive infinity
     * for all), windows of {@code bucketYears}.
     */
    public static List<Row> of(List<GeoEvent> events, double cursorYears, double bucketYears) {
        double size = bucketYears > 0 ? bucketYears : 1e6;
        // window index → kind → [count, newest time]
        Map<Long, Map<String, double[]>> windows = new LinkedHashMap<>();
        for (GeoEvent e : events) {
            if (!(e.tStart() <= cursorYears) || !Double.isFinite(e.tStart()))
                continue;
            long w = (long) Math.floor(e.tStart() / size);
            double[] c = windows.computeIfAbsent(w, k -> new LinkedHashMap<>()).computeIfAbsent(e.kind(), k -> new double[]{0, Double.NEGATIVE_INFINITY});
            c[0]++;
            c[1] = Math.max(c[1], e.tStart());
        }
        List<Long> order = new ArrayList<>(windows.keySet());
        order.sort(Comparator.reverseOrder());
        List<Row> out = new ArrayList<>();
        for (long w : order) {
            List<Row> rows = new ArrayList<>();
            for (Map.Entry<String, double[]> k : windows.get(w).entrySet())
                rows.add(new Row(k.getValue()[1], k.getKey(), (int) k.getValue()[0], EventStyle.colour(k.getKey())));
            rows.sort(Comparator.comparingInt(Row::count).reversed().thenComparing(Row::kind));
            for (Row r : rows) {
                if (out.size() >= MAX_ROWS)
                    return out;
                out.add(r);
            }
        }
        return out;
    }
}
