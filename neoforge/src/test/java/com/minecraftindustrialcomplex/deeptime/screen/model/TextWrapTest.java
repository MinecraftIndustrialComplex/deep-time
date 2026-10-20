// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import org.junit.jupiter.api.Test;

/**
 * Rows wrap to two lines instead of being cut to one (P1-11d): the wrap itself, and every History
 * phrase fits the History column's two lines at the smallest layout.
 */
class TextWrapTest {

    /**
     * A model of the vanilla font's advance widths that is never narrower than the real one
     * (the font's glyphs are 6 px with 1 px spacing, narrower for a few). Greedy wrapping is
     * monotone in the widths, so text that fits under this model fits on screen.
     */
    static int width(String s) {
        int w = 0;
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            w += switch (c) {
                case 'i', '!', '.', ',', ':', ';', '\'', '|' -> 2;
                case 'l', '`' -> 3;
                case ' ', 't', 'I', '[', ']' -> 4;
                case 'f', 'k', '(', ')', '"', '*', '<', '>', '{', '}' -> 5;
                case '…' -> 8;
                default -> 6;
            };
        }
        return w;
    }

    /**
     * The History column's text width at 1920x1080 with GUI scale auto (480x270 scaled; the side
     * panel's rows are 141 wide, the time column 26), the narrowest layout the screen has.
     */
    static final int HISTORY_TEXT_W = 141 - 26 - 4 - 11;

    @Test
    void shortTextIsOneLine() {
        TextWrap.Wrapped w = TextWrap.wrap("A rift opened", TextWrapTest::width, 100, 100);
        assertEquals(List.of("A rift opened"), w.lines());
        assertFalse(w.cut());
    }

    @Test
    void longTextBreaksAtSpacesAndEveryLineFits() {
        String text = "Subduction began at 999 plate edges";
        TextWrap.Wrapped w = TextWrap.wrap(text, TextWrapTest::width, 100, 100);
        assertEquals(2, w.lines().size());
        assertFalse(w.cut());
        assertEquals(text, String.join(" ", w.lines()), "no word lost or split");
        for (String line : w.lines())
            assertTrue(width(line) <= 100, line);
    }

    @Test
    void textBeyondTheLinesEndsInAnEllipsisAtAWord() {
        String text = "one two three four five six seven eight nine ten eleven twelve thirteen";
        TextWrap.Wrapped w = TextWrap.wrap(text, TextWrapTest::width, 60, 60);
        assertTrue(w.cut());
        assertEquals(2, w.lines().size());
        String last = w.lines().get(1);
        assertTrue(last.endsWith(TextWrap.ELLIPSIS), last);
        String words = last.substring(0, last.length() - 1);
        assertTrue(text.contains(String.join(" ", w.lines().get(0), words)), "cut at a word: " + last);
        for (String line : w.lines())
            assertTrue(width(line) <= 60, line);
    }

    @Test
    void aCutLineDropsTheCommaBeforeItsEllipsis() {
        TextWrap.Wrapped w = TextWrap.wrap("alpha beta, gamma delta epsilon", TextWrapTest::width, 70);
        assertTrue(w.cut());
        assertEquals("alpha beta" + TextWrap.ELLIPSIS, w.lines().get(0));
    }

    @Test
    void aWordWiderThanALineIsBrokenByCharacter() {
        TextWrap.Wrapped w = TextWrap.wrap("abcdefghijklmnopqrstuvwxyz", TextWrapTest::width, 30, 30, 30, 30, 30, 30);
        assertFalse(w.cut());
        assertEquals("abcdefghijklmnopqrstuvwxyz", String.join("", w.lines()));
        for (String line : w.lines())
            assertTrue(width(line) <= 30, line);
        TextWrap.Wrapped cut = TextWrap.wrap("abcdefghijklmnopqrstuvwxyz", TextWrapTest::width, 30);
        assertTrue(cut.cut());
        assertTrue(width(cut.lines().get(0)) <= 30);
    }

    @Test
    void emptyTextHasNoLines() {
        assertEquals(List.of(), TextWrap.wrap("", TextWrapTest::width, 100).lines());
        assertEquals(List.of(), TextWrap.wrap(null, TextWrapTest::width, 100).lines());
        assertEquals(List.of(), TextWrap.summary(" ", TextWrapTest::width, 100).lines());
    }

    @Test
    void sentencesSplitAtAFullStopBeforeACapitalOrADigit() {
        assertEquals(List.of("Higher is drier.", "Zero turns it off.", "3 is the most."),
                TextWrap.sentences("Higher is drier. Zero turns it off. 3 is the most."));
        assertEquals(List.of("About 0.5 of the plates, e.g. most."), TextWrap.sentences("About 0.5 of the plates, e.g. most."),
                "a decimal point or an abbreviation before a small letter does not end a sentence");
    }

    @Test
    void summaryKeepsWholeSentencesThatFit() {
        String hint = "How fast plates move. Faster plates build more mountains and close oceans sooner, so the map changes more.";
        TextWrap.Wrapped two = TextWrap.summary(hint, TextWrapTest::width, 360, 360);
        assertFalse(two.cut(), "both sentences fit two 360 px lines: " + two.lines());
        assertEquals(hint, String.join(" ", two.lines()));

        TextWrap.Wrapped one = TextWrap.summary(hint, TextWrapTest::width, 200, 200);
        assertTrue(one.cut(), "the second sentence is left out, so the tooltip shows the hint");
        assertEquals("How fast plates move.", String.join(" ", one.lines()));
        assertFalse(one.lines().get(one.lines().size() - 1).endsWith(TextWrap.ELLIPSIS), "a whole sentence needs no ellipsis");
    }

    @Test
    void summaryCutsTheFirstSentenceWhenEvenThatIsTooLong() {
        String hint = "A very long first sentence that cannot fit in the two short lines it is given here. Then more.";
        TextWrap.Wrapped w = TextWrap.summary(hint, TextWrapTest::width, 80, 80);
        assertTrue(w.cut());
        assertEquals(2, w.lines().size());
        assertTrue(w.lines().get(1).endsWith(TextWrap.ELLIPSIS));
    }

    @Test
    void aCutSummaryEndsBeforeAnOpenBracket() {
        String hint = "Alpha beta gamma delta epsilon (zeta eta theta iota kappa lambda mu) nu.";
        TextWrap.Wrapped w = TextWrap.summary(hint, TextWrapTest::width, 100, 130);
        assertTrue(w.cut());
        assertEquals(List.of("Alpha beta gamma", "delta epsilon" + TextWrap.ELLIPSIS), w.lines(),
                "the plain cut would end in \"(zeta eta…\"");
    }

    /**
     * Every Customize setting's summary fits its two lines at 1920x1080 (GUI scale auto: rows 452
     * wide, the range at the right of the second line), and most say at least their first sentence
     * whole. The catalogue is the browser workbench's.
     */
    @Test
    void customizeSummariesFitTheirLines() throws IOException {
        JsonObject lang = LayerCatalogTest.lang();
        int rowW = 452;
        int whole = 0, n = 0;
        for (EditorFields.Field f : EditorFields.ALL) {
            if (f.hint().isEmpty() || f.phase3())
                continue;
            n++;
            Phrase p = CustomizeTopics.range(f);
            String range = String.format(Locale.ROOT, lang.get("deeptime." + p.key()).getAsString(), p.args().toArray());
            int first = rowW - 6 - width(range) - 8, second = rowW - 6;
            TextWrap.Wrapped w = TextWrap.summary(f.hint(), TextWrapTest::width, first, second);
            assertTrue(w.lines().size() <= 2 && !w.lines().isEmpty(), f.id());
            assertTrue(width(w.lines().get(0)) <= first, f.id() + ": " + w.lines());
            if (w.lines().size() > 1)
                assertTrue(width(w.lines().get(1)) <= second, f.id() + ": " + w.lines());
            if (!w.lines().get(w.lines().size() - 1).endsWith(TextWrap.ELLIPSIS))
                whole++;
        }
        assertTrue(whole >= n * 4 / 5, "at least four in five settings say a whole sentence: " + whole + " of " + n);
    }

    /**
     * Every History phrase, with a three-digit count, fits the History column's two lines at
     * 1920x1080 (GUI scale auto) without an ellipsis.
     */
    @Test
    void everyMilestonePhraseFitsTwoHistoryLines() throws IOException {
        JsonObject lang = LayerCatalogTest.lang();
        List<String> cut = new ArrayList<>();
        int checked = 0;
        for (Map.Entry<String, com.google.gson.JsonElement> e : lang.entrySet()) {
            if (!e.getKey().startsWith("deeptime.milestone."))
                continue;
            String text = e.getValue().getAsString().replace("%s", "999");
            checked++;
            TextWrap.Wrapped w = TextWrap.wrap(text, TextWrapTest::width, HISTORY_TEXT_W, HISTORY_TEXT_W);
            if (w.cut())
                cut.add(e.getKey() + ": " + text + " -> " + w.lines());
        }
        assertTrue(checked > 40, "the milestone phrases were read: " + checked);
        assertEquals(List.of(), cut, "History phrases that would be cut");
    }
}
