// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import org.junit.jupiter.api.Test;

/** The generation page's words for time (docs/design/create-world-ux.md: one convention, "N million years ago"). */
class PhraseTest {

    static Phrase p(String key, Object... args) {
        return Phrase.of(key, args);
    }

    @Test
    void yearsAgoPicksTheUnitAndRounds() {
        assertEquals(p("time.million.ago", "640"), Phrase.yearsAgo(-640e6));
        assertEquals(p("time.million.ago", "3.5"), Phrase.yearsAgo(-3.5e6));
        assertEquals(p("time.million.ago", "10"), Phrase.yearsAgo(-9.96e6), "whole millions from 10 on");
        assertEquals(p("time.million.ago", "1"), Phrase.yearsAgo(-1.0e6));
        assertEquals(p("time.million.ago", "999"), Phrase.yearsAgo(-999.4e6));
        assertEquals(p("time.billion.ago", "1"), Phrase.yearsAgo(-999.6e6), "rounds up into billions");
        assertEquals(p("time.billion.ago", "1.2"), Phrase.yearsAgo(-1.2e9));
        assertEquals(p("time.billion.ago", "4.6"), Phrase.yearsAgo(-4.6e9));
        assertEquals(p("time.thousand.ago", "800"), Phrase.yearsAgo(-800e3));
        assertEquals(p("time.today"), Phrase.yearsAgo(0));
        assertEquals(p("time.today"), Phrase.yearsAgo(-120));
        assertEquals(p("time.million.ahead", "50"), Phrase.yearsAgo(50e6), "a run continued past the present");
        assertEquals(p("time.unknown"), Phrase.yearsAgo(Double.NaN));
    }

    @Test
    void myrAgoIsABareNumberForTheHistoryColumn() {
        assertEquals("640", Phrase.myrAgo(-640e6));
        assertEquals("3.5", Phrase.myrAgo(-3.5e6));
        assertEquals("12", Phrase.myrAgo(-12.34e6));
        assertEquals("1,200", Phrase.myrAgo(-1.2e9));
        assertEquals("0", Phrase.myrAgo(-10_000));
        assertEquals("-50", Phrase.myrAgo(50e6));
        assertEquals("", Phrase.myrAgo(Double.NaN));
    }

    @Test
    void timeLeftAndDurations() {
        assertEquals(p("time.left.unknown"), Phrase.timeLeft(-1));
        assertEquals(p("time.left.under_minute"), Phrase.timeLeft(30_000));
        assertEquals(p("time.left.minutes", 1), Phrase.timeLeft(59_500), "rounded up: never promises less than is left");
        assertEquals(p("time.left.minutes", 4), Phrase.timeLeft(240_000));
        assertEquals(p("time.left.minutes", 89), Phrase.timeLeft(89 * 60_000));
        assertEquals(p("time.left.hours", 1, 30), Phrase.timeLeft(90 * 60_000));
        assertEquals(p("time.left.hours", 2, 5), Phrase.timeLeft(125 * 60_000));
        assertEquals(p("time.duration.under_minute"), Phrase.duration(12));
        assertEquals(p("time.duration.minutes", 3), Phrase.duration(150));
        assertEquals(p("time.duration.hours", 1, 40), Phrase.duration(100 * 60));
        assertEquals(p("time.unknown"), Phrase.duration(Double.NaN));
    }

    @Test
    void argumentsAndLiterals() {
        Phrase nested = Phrase.of("gen.status.stage_at", GenerationStatus.stage("plates"), Phrase.yearsAgo(-5e8));
        assertEquals("deeptime.gen.status.stage_at", nested.translationKey());
        assertEquals(List.of(p("stage.plates"), p("time.million.ago", "500")), nested.args(), "phrases stay phrases");
        assertEquals(List.of("5", "2.5"), Phrase.of("x", 5, 2.5).args(), "anything else becomes text");
        Phrase lit = Phrase.literal("Snow and ice");
        assertTrue(lit.isLiteral());
        assertEquals(List.of("Snow and ice"), lit.args());
        assertFalse(p("time.today").isLiteral());
    }

    @Test
    void stagesWithoutANameReadAsSimulating() {
        assertEquals(p("stage.climate"), GenerationStatus.stage("climate"));
        assertEquals(p("stage.other"), GenerationStatus.stage("stage 2/3"), "TimelapseModel's fallback name");
        assertEquals(p("stage.other"), GenerationStatus.stage(null));
    }
}
