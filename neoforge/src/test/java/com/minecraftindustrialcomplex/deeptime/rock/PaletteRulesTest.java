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

    /** Stone and deepslate are filler; air is rejected. */
    private static final PaletteRules.Kinds<String> KINDS = new PaletteRules.Kinds<>() {
        @Override
        public boolean isStone(String b) {
            return b.equals("minecraft:stone");
        }

        @Override
        public boolean isFiller(String b) {
            return b.equals("minecraft:stone") || b.equals("minecraft:deepslate");
        }

        @Override
        public boolean isRejected(String b) {
            return b.equals("minecraft:air");
        }
    };

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
    void fillerIsRejectedExceptStoneForCoal() {
        var shale = resolve("shale", List.of("minecraft:stone"), List.of(), List.of("minecraft:deepslate", "minecraft:polished_tuff"),
                List.of(), null);
        assertEquals(Source.FALLBACK, shale.source());
        assertEquals(List.of("minecraft:polished_tuff"), shale.blocks());
        assertEquals(2, shale.rejected().size());

        // The decisions file: black argillite, then stone. Without Born in Chaos the tag holds stone only.
        var coal = resolve("coal", List.of(), List.of(), List.of("minecraft:stone"), List.of(), "minecraft:stone");
        assertEquals(Source.FALLBACK, coal.source());
        assertEquals(List.of("minecraft:stone"), coal.blocks());
        assertTrue(coal.rejected().isEmpty());

        var coalVanilla = resolve("coal", List.of(), List.of(), List.of(), List.of(), "minecraft:stone");
        assertEquals(Source.VANILLA_FALLBACK, coalVanilla.source());

        var coalDeepslate = resolve("coal", List.of("minecraft:deepslate"), List.of(), List.of(), List.of(), null);
        assertEquals(Source.NONE, coalDeepslate.source());

        var filler = resolve("granite", List.of(), List.of(), List.of(), List.of(), "minecraft:stone");
        assertEquals(Source.NONE, filler.source());
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
