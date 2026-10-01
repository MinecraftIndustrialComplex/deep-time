// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules.Candidates;
import com.minecraftindustrialcomplex.deeptime.rock.PaletteRules.Source;
import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.function.Function;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/** P1-12: the block-resolution rules of plan §7.5 on string "blocks". */
class PaletteRulesTest {

    /** Only air is rejected: stone and deepslate are ordinary candidates (P1-28). */
    private static final PaletteRules.Kinds<String> KINDS = b -> b.equals("minecraft:air");

    private static PaletteRules.Result<String> resolve(String cls, List<String> memberTag, List<String> memberStates,
                                                       List<String> fallbackTag, List<String> fallbackStates, String vanilla) {
        return PaletteRules.resolve(cls, new Candidates<>(memberTag, memberStates, fallbackTag, fallbackStates,
                Optional.ofNullable(vanilla)), KINDS, Function.identity());
    }

    @Test
    void membersAreTheTagInOrderThenMemberStates() {
        var r = resolve("sand", List.of("minecraft:sand", "minecraft:red_sand"), List.of("mod:sand_layer[layers=8]"),
                List.of("minecraft:gravel"), List.of(), "minecraft:sandstone");
        assertEquals(Source.MEMBER, r.source());
        assertEquals(List.of("minecraft:sand", "minecraft:red_sand", "mod:sand_layer[layers=8]"), r.blocks());
        assertEquals("minecraft:sand", r.primary().orElseThrow());
    }

    @Test
    void memberStatesAloneAreMembers() {
        var r = resolve("loess", List.of(), List.of("projectatmosphere:dust[layers=8]"), List.of("minecraft:smooth_sandstone"),
                List.of(), "minecraft:smooth_sandstone");
        assertEquals(Source.MEMBER, r.source());
        assertEquals(List.of("projectatmosphere:dust[layers=8]"), r.blocks());
    }

    @Test
    void duplicatesCollapse() {
        var r = resolve("soil", List.of("minecraft:dirt"), List.of("minecraft:dirt", "minecraft:rooted_dirt"), List.of(), List.of(), null);
        assertEquals(List.of("minecraft:dirt", "minecraft:rooted_dirt"), r.blocks());
    }

    @Test
    void firstInstalledFallbackEntryThenStateFallbacks() {
        // Absent optional entries never reach the rules (the tag loader and RockPalette drop them).
        var r = resolve("shale", List.of(), List.of(), List.of("born_in_chaos_v1:black_argillite", "minecraft:polished_tuff"),
                List.of("minecraft:gravel"), "minecraft:polished_tuff");
        assertEquals(Source.FALLBACK, r.source());
        assertEquals(List.of("born_in_chaos_v1:black_argillite"), r.blocks());

        var states = resolve("x", List.of(), List.of(), List.of(), List.of("minecraft:polished_granite", "minecraft:gravel"), "minecraft:andesite");
        assertEquals(Source.FALLBACK, states.source());
        assertEquals(List.of("minecraft:polished_granite"), states.blocks());
    }

    @Test
    void vanillaFallbackIsTheLastResort() {
        var r = resolve("gabbro", List.of(), List.of(), List.of(), List.of(), "minecraft:smooth_basalt");
        assertEquals(Source.VANILLA_FALLBACK, r.source());
        assertEquals(List.of("minecraft:smooth_basalt"), r.blocks());
    }

    @Test
    void nothingLeavesUnclassifiedCrust() {
        var r = resolve("gabbro", List.of(), List.of(), List.of(), List.of(), null);
        assertEquals(Source.NONE, r.source());
        assertTrue(r.primary().isEmpty());
        assertNull(r.pick(42));
    }

    @Test
    void stoneAndDeepslateAreOrdinaryCandidates() {
        // P1-28 (Ben, 2026-10-01: "Use stone for common rocks"): a pack's fallback list may name stone or
        // deepslate for any class, and a class's vanilla_fallback may be either; they win by list order
        // like any other block.
        var shale = resolve("shale", List.of(), List.of(), List.of("minecraft:stone", "minecraft:polished_tuff"), List.of(), null);
        assertEquals(Source.FALLBACK, shale.source());
        assertEquals(List.of("minecraft:stone"), shale.blocks());
        assertTrue(shale.rejected().isEmpty());

        var slate = resolve("slate", List.of(), List.of(), List.of(), List.of(), "minecraft:deepslate");
        assertEquals(Source.VANILLA_FALLBACK, slate.source());
        assertEquals(List.of("minecraft:deepslate"), slate.blocks());

        // A pack that wants something else puts it first, ahead of the stone stand-in.
        var overridden = resolve("shale", List.of(), List.of(), List.of("born_in_chaos_v1:black_argillite", "minecraft:stone"),
                List.of(), "minecraft:stone");
        assertEquals(List.of("born_in_chaos_v1:black_argillite"), overridden.blocks());

        // Members win over a stone vanilla_fallback; stone as a member is allowed too.
        var member = resolve("mudstone", List.of("minecraft:stone"), List.of(), List.of(), List.of(), "minecraft:stone");
        assertEquals(Source.MEMBER, member.source());
    }

    @Test
    void airIsRejected() {
        var r = resolve("loess", List.of("minecraft:air"), List.of(), List.of("minecraft:air"), List.of(),
                "minecraft:smooth_sandstone");
        assertEquals(Source.VANILLA_FALLBACK, r.source());
        assertEquals(2, r.rejected().size());
    }

    @Test
    void oneMemberPerKeyStableAndSpread() {
        var r = resolve("soil", List.of("minecraft:dirt", "minecraft:rooted_dirt", "minecraft:coarse_dirt"), List.of(), List.of(),
                List.of(), null);
        Set<String> seen = new HashSet<>();
        for (long key = 0; key < 64; key++) {
            String a = r.pick(key);
            assertEquals(a, r.pick(key));
            seen.add(a);
        }
        assertEquals(Set.copyOf(r.blocks()), seen);
        var one = resolve("granite", List.of("minecraft:granite"), List.of(), List.of(), List.of(), null);
        assertEquals("minecraft:granite", one.pick(123456789L));
    }
}
