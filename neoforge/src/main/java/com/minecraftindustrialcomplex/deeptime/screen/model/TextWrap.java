// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import java.util.ArrayList;
import java.util.List;
import java.util.function.ToIntFunction;

/**
 * Text broken into a few lines of given widths, client-free so JUnit checks it with a model of the
 * font's widths (docs/design/create-world-ux.md: rows wrap to two lines rather than cut to one).
 * The screens pass the font's {@code width}. Lines break at spaces; a word wider than a line is cut
 * by character. Text that needs more lines than it has ends its last line in "…" at a word, and
 * says it was cut, so the screen shows it whole on hover.
 */
public final class TextWrap {

    private TextWrap() {}

    public static final String ELLIPSIS = "…";

    /**
     * The lines, and whether the text was cut to fit them.
     *
     * @param lines the lines, at most as many as there were widths, none empty
     * @param cut   true when words were left out (the last line ends in "…")
     */
    public record Wrapped(List<String> lines, boolean cut) {
        public Wrapped {
            lines = List.copyOf(lines);
        }
    }

    /**
     * {@code text} in lines of {@code lineWidths} (the first line's width first; at least one),
     * measured by {@code width}.
     */
    public static Wrapped wrap(String text, ToIntFunction<String> width, int... lineWidths) {
        String[] words = text == null ? new String[0] : text.trim().split("\\s+");
        List<String> lines = new ArrayList<>();
        if (words.length == 0 || words[0].isEmpty() || lineWidths.length == 0)
            return new Wrapped(lines, false);
        int i = 0;
        StringBuilder line = new StringBuilder();
        while (i < words.length) {
            int max = Math.max(1, lineWidths[lines.size()]);
            boolean last = lines.size() == lineWidths.length - 1;
            String w = words[i];
            String tried = line.isEmpty() ? w : line + " " + w;
            if (width.applyAsInt(tried) <= max) {
                line.setLength(0);
                line.append(tried);
                i++;
                continue;
            }
            if (line.isEmpty()) {
                // A word wider than the line: as much of it as fits, the rest on the next line.
                int n = fitting(w, width, last ? max - width.applyAsInt(ELLIPSIS) : max);
                if (last) {
                    lines.add(w.substring(0, n) + ELLIPSIS);
                    return new Wrapped(lines, true);
                }
                lines.add(w.substring(0, n));
                words[i] = w.substring(n);
                continue;
            }
            if (last)
                return new Wrapped(withEllipsis(lines, line.toString(), width, max), true);
            lines.add(line.toString());
            line.setLength(0);
        }
        lines.add(line.toString());
        return new Wrapped(lines, false);
    }

    /**
     * What a setting's hint says first, in the lines it has: as many of its sentences as fit
     * whole, else the first sentence cut with "…" ({@link #wrap}). Cut whenever any of the hint
     * is left out, so the screen keeps the whole hint for the tooltip.
     */
    public static Wrapped summary(String hint, ToIntFunction<String> width, int... lineWidths) {
        List<String> sentences = sentences(hint);
        if (sentences.isEmpty())
            return new Wrapped(List.of(), false);
        Wrapped best = null;
        StringBuilder text = new StringBuilder();
        for (int n = 0; n < sentences.size(); n++) {
            if (n > 0)
                text.append(' ');
            text.append(sentences.get(n));
            Wrapped w = wrap(text.toString(), width, lineWidths);
            if (w.cut())
                break;
            best = new Wrapped(w.lines(), n < sentences.size() - 1);
        }
        if (best != null)
            return best;
        return beforeOpenAside(wrap(sentences.get(0), width, lineWidths), width, lineWidths);
    }

    /**
     * A cut that would end inside a bracket ("as a multiple of the default sizes (at 1 a…") ends
     * before the bracket instead, when that keeps at least half the first line: the aside is the
     * part a short summary can drop. game-choice (P1-11d): the half line is a layout rule.
     */
    private static Wrapped beforeOpenAside(Wrapped cut, ToIntFunction<String> width, int... lineWidths) {
        if (!cut.cut() || cut.lines().isEmpty())
            return cut;
        String shown = String.join(" ", cut.lines());
        shown = shown.substring(0, shown.length() - ELLIPSIS.length());
        int open = shown.lastIndexOf('(');
        if (open <= 0 || shown.indexOf(')', open) >= 0)
            return cut;
        String prefix = stripTrailing(shown.substring(0, open).trim());
        if (width.applyAsInt(prefix) < lineWidths[0] / 2)
            return cut;
        Wrapped shorter = wrap(prefix + ELLIPSIS, width, lineWidths);
        return shorter.cut() || shorter.lines().isEmpty() ? cut : new Wrapped(shorter.lines(), true);
    }

    /** The sentences of {@code text}: split after ".", "!" or "?" followed by a space and a capital letter or a digit. */
    static List<String> sentences(String text) {
        List<String> out = new ArrayList<>();
        if (text == null || text.isBlank())
            return out;
        String t = text.trim();
        int start = 0;
        for (int i = 0; i + 2 < t.length(); i++) {
            char c = t.charAt(i);
            if ((c == '.' || c == '!' || c == '?') && t.charAt(i + 1) == ' '
                    && (Character.isUpperCase(t.charAt(i + 2)) || Character.isDigit(t.charAt(i + 2)))) {
                out.add(t.substring(start, i + 1).trim());
                start = i + 2;
            }
        }
        out.add(t.substring(start).trim());
        out.removeIf(String::isEmpty);
        return out;
    }

    /** {@code lines} with {@code line} as the last, ended in "…": its words while they fit beside it. */
    private static List<String> withEllipsis(List<String> lines, String line, ToIntFunction<String> width, int max) {
        String s = line;
        while (width.applyAsInt(s + ELLIPSIS) > max) {
            int sp = s.lastIndexOf(' ');
            if (sp <= 0) {
                s = s.substring(0, fitting(s, width, max - width.applyAsInt(ELLIPSIS)));
                break;
            }
            s = s.substring(0, sp);
        }
        s = stripTrailing(s);
        List<String> out = new ArrayList<>(lines);
        out.add(s + ELLIPSIS);
        return out;
    }

    /** Drops the punctuation a cut line would end on before its "…" (a comma, a colon). */
    private static String stripTrailing(String s) {
        int end = s.length();
        while (end > 0 && ",;:".indexOf(s.charAt(end - 1)) >= 0)
            end--;
        return s.substring(0, end);
    }

    /** How many leading characters of {@code s} fit {@code max} (at least one, so a line always moves on). */
    private static int fitting(String s, ToIntFunction<String> width, int max) {
        int n = 0;
        while (n < s.length() && width.applyAsInt(s.substring(0, n + 1)) <= max)
            n++;
        return Math.max(1, n);
    }
}
