// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * What a map layer's colours mean, for the generation page's Map tab (docs/design/create-world-ux.md:
 * a legend wherever colour is used). Read from the same {@code ramps.json} the keyframes are coloured
 * with ({@link Ramps}), so the legend cannot drift from the picture:
 * <ul>
 *   <li>a ramp with {@code labels} (satellite, provinces, Köppen) lists its classes, under its
 *       {@code groups}' headings when it has them, then its {@code keys} (the satellite view's lakes);</li>
 *   <li>a categorical ramp without labels whose codes the game names ({@link #CODED}: volcanoes,
 *       impacts, tectonic regime, metamorphic grade) lists them by language key;</li>
 *   <li>any other categorical ramp (plates, rock classes) gets one line saying what the colours are;</li>
 *   <li>a continuous ramp is a colour bar from its first stop to its last, with the values in the
 *       layer's unit when its name carries one ({@code _c}, {@code _mm}, {@code _m}, {@code _m_myr},
 *       {@code _myr}), else "low" and "high";</li>
 *   <li>the rivers view lists its keys and a bar of river flow.</li>
 * </ul>
 * Client-free; the screen draws the parts in order.
 */
public final class Legend {

    private Legend() {}

    /** One line of the legend. */
    public sealed interface Part permits Heading, Swatch, Bar, Note {}

    /** A heading: a ramp group's name (literal, from ramps.json) or a phrase. */
    public record Heading(Phrase text) implements Part {}

    /** A colour and what it is. */
    public record Swatch(int rgb, Phrase text) implements Part {}

    /**
     * A colour bar.
     *
     * @param rgb  colours left to right, {@code 0xRRGGBB}
     * @param low  the left end's words
     * @param high the right end's words
     */
    public record Bar(int[] rgb, Phrase low, Phrase high) implements Part {}

    /** A line of explanation. */
    public record Note(Phrase text) implements Part {}

    /** How many colours a bar samples. */
    public static final int BAR_STEPS = 48;

    /** Categorical ramps whose codes have names in the language file ({@code legend.<ramp>.<code>}): the codes, in order. */
    public static final Map<String, int[]> CODED = Map.of(
            "volcanoes", new int[]{1, 2, 3, 4, 5, 6, 7},
            "impacts", new int[]{1, 2, 3, 4},
            "regime", new int[]{0, 1, 2, 3, 4, 5, 6, 7, 8, 9},
            "metamorphic_grade", new int[]{0, 1, 2, 3, 4});

    /** The legend of {@code layer}. */
    public static List<Part> of(Ramps ramps, String layer) {
        String name = ramps.rampName(layer);
        Ramps.Ramp r = ramps.ramp(name);
        List<Part> out = new ArrayList<>();
        if (!r.labels().isEmpty()) {
            labelled(r, out);
            return out;
        }
        if (name.equals("rivers")) {
            for (Map.Entry<String, Integer> k : r.keys().entrySet())
                out.add(new Swatch(k.getValue(), Phrase.of("legend.key." + k.getKey())));
            out.add(new Heading(Phrase.of("legend.rivers.flow")));
            // The ramp is log10 of the discharge, m³/s.
            out.add(new Bar(bar(r), Phrase.of("legend.value.flow", Formats.number(roundSignificant(Math.pow(10, r.min())))),
                    Phrase.of("legend.value.flow", Formats.number(roundSignificant(Math.pow(10, r.max()))))));
            return out;
        }
        if (categorical(r)) {
            int[] codes = CODED.get(name);
            if (codes != null) {
                for (int c : codes)
                    out.add(new Swatch(r.colour(c + 0.5), Phrase.of("legend." + name + "." + c)));
            } else {
                out.add(new Note(Phrase.of(name.equals("plates") ? "legend.note.plates" : "legend.note.categories")));
            }
            return out;
        }
        // The height layers are block heights, which players know as Y levels.
        String unit = name.equals("elevation") ? "y" : unit(layer);
        out.add(new Bar(bar(r), endWords(r.min(), unit, "legend.low"), endWords(r.max(), unit, "legend.high")));
        for (Map.Entry<String, Integer> k : r.keys().entrySet())
            out.add(new Swatch(k.getValue(), Phrase.of("legend.key." + k.getKey())));
        return out;
    }

    private static void labelled(Ramps.Ramp r, List<Part> out) {
        List<String> labels = r.labels();
        List<Ramps.Group> groups = r.groups();
        int next = 0;
        for (Ramps.Group g : groups) {
            for (; next < g.from() && next < labels.size(); next++)
                out.add(swatch(r, next));
            out.add(new Heading(Phrase.literal(g.name())));
            for (; next <= g.to() && next < labels.size(); next++)
                out.add(swatch(r, next));
        }
        for (; next < labels.size(); next++)
            out.add(swatch(r, next));
        for (Map.Entry<String, Integer> k : r.keys().entrySet()) {
            // A key that repeats a labelled class's colour (the satellite view's salt flat) adds nothing.
            String keyLabel = k.getKey();
            boolean dup = false;
            for (int i = 0; i < labels.size(); i++)
                if (r.colour(i + 0.5) == k.getValue() || Formats.humanize(keyLabel).equalsIgnoreCase(labels.get(i)))
                    dup = true;
            if (!dup)
                out.add(new Swatch(k.getValue(), Phrase.of("legend.key." + keyLabel)));
        }
    }

    private static Swatch swatch(Ramps.Ramp r, int i) {
        return new Swatch(r.colour(r.min() + i + 0.5), Phrase.literal(r.labels().get(i)));
    }

    /**
     * True for a ramp of classes, as ramps.json writes them: stops in pairs {@code [k, c], [k + 0.999, c]}
     * (one flat colour per whole number; the last class may be a single stop), or one stop per whole
     * number, each its own colour (the deposit sites).
     */
    public static boolean categorical(Ramps.Ramp r) {
        double[] v = r.values();
        int[] c = r.rgb();
        if (v.length < 2)
            return false;
        boolean pairs = v.length % 2 == 0 || Math.abs(v[v.length - 1] - v[v.length - 2] - 0.001) < 1e-6;
        for (int i = 0; pairs && i + 1 < v.length; i += 2)
            if (c[i] != c[i + 1] || Math.abs(v[i + 1] - v[i] - 0.999) > 1e-6)
                pairs = false;
        if (pairs)
            return true;
        // Codes from 0 or 1 up, five or more (a three-stop -1, 0, 1 ramp is a diverging scale).
        if (v.length < 5 || v[0] < 0)
            return false;
        for (int i = 0; i < v.length; i++)
            if (v[i] != Math.rint(v[i]) || (i > 0 && (v[i] - v[i - 1] != 1 || c[i] == c[i - 1])))
                return false;
        return true;
    }

    /** The unit a layer's values are in, from its name: {@code c}, {@code mm}, {@code m}, {@code m_myr}, {@code myr}, or empty. */
    public static String unit(String layer) {
        String base = layer.substring(layer.lastIndexOf('/') + 1).toLowerCase(Locale.ROOT);
        for (String u : UNITS)
            if (base.endsWith("_" + u) && base.length() > u.length() + 1)
                return u;
        return "";
    }

    /**
     * The units a legend can show values in ({@code legend.value.<unit>}): those a layer name can end
     * in, longest suffix first, then {@code y} (block heights, the elevation ramp's).
     */
    public static final List<String> UNITS = List.of("m_myr", "myr", "mm", "km", "m", "c", "y");

    private static Phrase endWords(double v, String unit, String fallback) {
        if (unit.isEmpty())
            return Phrase.of(fallback);
        return Phrase.of("legend.value." + unit, Formats.number(v));
    }

    private static int[] bar(Ramps.Ramp r) {
        int[] out = new int[BAR_STEPS];
        double a = r.min(), b = r.max();
        for (int i = 0; i < BAR_STEPS; i++)
            out[i] = r.colour(a + (b - a) * i / (BAR_STEPS - 1));
        return out;
    }

    /** Two significant figures (2.0 for 1.995, 3,200 for 3,162). */
    static double roundSignificant(double v) {
        if (!(v > 0) || !Double.isFinite(v))
            return v;
        double mag = Math.pow(10, Math.floor(Math.log10(v)) - 1);
        return Math.round(v / mag) * mag;
    }
}
