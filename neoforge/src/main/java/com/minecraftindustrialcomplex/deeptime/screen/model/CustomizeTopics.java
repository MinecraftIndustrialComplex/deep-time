// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.function.Function;

/**
 * The Customize page's topics (docs/design/create-world-ux.md, "Customize"; Ben, 2026-10-19: "a much
 * better customization screen for any power users"): the {@link EditorFields} catalogue, the same
 * parameter set the browser workbench builds its form from, sorted into six tabs a player can read.
 * Each catalogue section belongs to one topic; inside a topic the everyday sections come first and the
 * fine-tuning ({@code adv_*}) sections after them, each under its own heading. Advanced holds what is
 * not about the planet itself (the memory cap, the Phase 3 placeholders) and a search over every
 * setting, the only place raw preset paths are shown.
 */
public final class CustomizeTopics {

    private CustomizeTopics() {}

    public enum Topic {
        PLANET, PLATES, CLIMATE, SURFACE, ROCKS, ADVANCED;

        /** The tab's language key, under {@code deeptime.}. */
        public String key() {
            return "customize.topic." + name().toLowerCase(Locale.ROOT);
        }
    }

    /** Fields that sit in another topic than their section's (ore guarantees are about rocks, not playability). */
    private static final java.util.Map<String, Topic> MOVED = java.util.Map.of("essentials", Topic.ROCKS);

    /** The topic a catalogue section belongs to. */
    public static Topic of(EditorFields.Section s) {
        return switch (s) {
            case PLANET, SCALE -> Topic.PLANET;
            case ADV_PLATES -> Topic.PLATES;
            case BIOMES, PLAYABILITY, ADV_CLIMATE, ADV_BIOMES -> Topic.CLIMATE;
            case TERRAIN, ADV_SURFACE, ADV_WATER, ADV_VOLCANOES, ADV_IMPACTS -> Topic.SURFACE;
            case MULTIPLIERS, ADV_ROCKS, ADV_DEPOSITS, ADV_UNDERGROUND, ADV_GEOTHERMAL -> Topic.ROCKS;
            case FANTASY -> Topic.ADVANCED;
        };
    }

    /** The topic a field is shown under. */
    public static Topic of(EditorFields.Field f) {
        Topic moved = MOVED.get(f.id());
        return moved != null ? moved : of(f.section());
    }

    /**
     * The heading a field is listed under, a language key under {@code deeptime.}: its section's
     * ({@code customize.section.adv_climate}), or its own for a field that moved topic
     * ({@code customize.section.essentials}).
     */
    public static String heading(EditorFields.Field f) {
        return "customize.section." + (MOVED.containsKey(f.id()) ? f.id() : f.section().name().toLowerCase(Locale.ROOT));
    }

    /** Every heading key {@link #heading} can return. */
    public static List<String> headingKeys() {
        List<String> out = new ArrayList<>();
        for (EditorFields.Section s : EditorFields.Section.values())
            out.add("customize.section." + s.name().toLowerCase(Locale.ROOT));
        for (String id : MOVED.keySet())
            out.add("customize.section." + id);
        return out;
    }

    /** A topic's fields: the everyday sections first, then the fine-tuning ones, each in catalogue order. */
    public static List<EditorFields.Field> fields(Topic t) {
        return fields(EditorFields.ALL, t);
    }

    static List<EditorFields.Field> fields(List<EditorFields.Field> all, Topic t) {
        List<EditorFields.Field> everyday = new ArrayList<>(), fine = new ArrayList<>();
        for (EditorFields.Field f : all)
            if (of(f) == t)
                (f.advanced() ? fine : everyday).add(f);
        everyday.addAll(fine);
        return List.copyOf(everyday);
    }

    /** How many settings a search lists at most (the rest: "N more, type more of the name"). */
    public static final int MAX_RESULTS = 80;

    /**
     * Every field whose label (as the player reads it), id, preset path or hint contains each word of
     * {@code query}, ignoring case; catalogue order. An empty query matches nothing.
     */
    public static List<EditorFields.Field> search(String query, Function<EditorFields.Field, String> label) {
        return search(EditorFields.ALL, query, label);
    }

    static List<EditorFields.Field> search(List<EditorFields.Field> all, String query, Function<EditorFields.Field, String> label) {
        String[] words = query == null ? new String[0] : query.toLowerCase(Locale.ROOT).trim().split("\\s+");
        if (words.length == 0 || words[0].isEmpty())
            return List.of();
        List<EditorFields.Field> out = new ArrayList<>();
        for (EditorFields.Field f : all) {
            String text = (label.apply(f) + " " + f.id() + " " + f.path() + " " + f.hint()).toLowerCase(Locale.ROOT);
            boolean hit = true;
            for (String w : words)
                if (!text.contains(w)) {
                    hit = false;
                    break;
                }
            if (hit)
                out.add(f);
        }
        return List.copyOf(out);
    }

    /** Above this many seconds the Customize page says the run will take a long time. */
    public static final double SLOW_SECONDS = 1800;

    /** True when a run of this estimate is slow enough to say so before it starts. */
    public static boolean slow(Estimate e) {
        return e != null && e.seconds() > SLOW_SECONDS;
    }

    /**
     * A maximum this large is no limit the player meets (the seed's is 2^53 - 1, the most a
     * double holds exactly), so the range says only its minimum. game-choice (P1-11d): a display
     * rule, not a physical value; 1e15 is where {@link Formats#number} stops writing whole numbers.
     */
    static final double NO_MAX = 1e15;

    /**
     * A field's range and unit as the player reads it: {@code 0.1 to 99.9 m/s²}, {@code 0 or more},
     * {@code on or off}, {@code 4 choices}, {@code from the pack's lists}.
     */
    public static Phrase range(EditorFields.Field f) {
        return switch (f.kind()) {
            case NUMBER -> f.max() >= NO_MAX
                    ? Phrase.of(f.unit().isEmpty() ? "customize.range.min" : "customize.range.min.unit", Formats.number(f.min()), f.unit())
                    : Phrase.of(f.unit().isEmpty() ? "customize.range" : "customize.range.unit",
                            Formats.number(f.min()), Formats.number(f.max()), f.unit());
            case TOGGLE -> Phrase.of("customize.range.toggle");
            case CHOICE -> f.options().isEmpty() ? Phrase.of("customize.range.list") : Phrase.of("customize.range.choices", f.options().size());
        };
    }
}
