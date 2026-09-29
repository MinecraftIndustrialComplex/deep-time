// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.rock;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Optional;
import java.util.function.Function;
import java.util.function.Predicate;

/**
 * The block-resolution rules of plan §7.5, free of Minecraft types so JUnit can test them
 * ({@code B} is a block state in the game, a string in tests). {@link RockPalette} gathers the
 * candidates from the tags, the {@code rock_states} registry and the class's
 * {@code vanilla_fallback}, then calls {@link #resolve}.
 *
 * <ol>
 *   <li><b>Members:</b> the blocks of {@code #deeptime:rock/<c>} in tag order, then the
 *       {@code rock_states} {@code member} entries (absent {@code required: false} blocks already
 *       dropped). Several members are kept; one is chosen per horizon or body ({@link Result#pick}).</li>
 *   <li>Else the <b>first installed fallback</b>: the entries of {@code #deeptime:rock_fallback/<c>}
 *       in tag order, then the {@code rock_states} {@code fallback} entries; the first one wins.</li>
 *   <li>Else the class's {@code vanilla_fallback}.</li>
 *   <li>Else nothing: the class stays unclassified crust.</li>
 * </ol>
 *
 * <p>Filler: {@code minecraft:stone} and {@code minecraft:deepslate} are the unclassified crust
 * itself and are never a class's block, with one exception, {@code coal}, which may fall back to
 * stone (the decisions file: Born in Chaos black argillite, then stone). A candidate that is air or
 * filler is rejected wherever it appears and reported. (Solidity is not required: vanilla snow and
 * Project Atmosphere's dust at {@code layers=8}, the adopted {@code rock_states} example, are
 * {@code forceSolidOff} blocks.)
 */
public final class PaletteRules {

    /** The one class allowed to resolve to filler stone. */
    public static final String COAL = "coal";

    public enum Source {
        MEMBER, FALLBACK, VANILLA_FALLBACK, NONE;

        public String id() {
            return name().toLowerCase(Locale.ROOT);
        }
    }

    /** What a block is, for the filler rule. */
    public interface Kinds<B> {
        boolean isStone(B b);

        boolean isFiller(B b);   // stone, deepslate

        boolean isRejected(B b); // air
    }

    /** A class's candidates, each list in its resolution order. */
    public record Candidates<B>(List<B> memberTag, List<B> memberStates, List<B> fallbackTag, List<B> fallbackStates,
                                Optional<B> vanillaFallback) {}

    /**
     * A class's resolution: its source, its blocks (several only for members; the first is the
     * primary member) and the candidates it rejected.
     */
    public record Result<B>(String rockClass, Source source, List<B> blocks, List<String> rejected) {

        public Optional<B> primary() {
            return blocks.isEmpty() ? Optional.empty() : Optional.of(blocks.get(0));
        }

        /**
         * The member for the horizon or body with this key: a hash of the key picks one member, so
         * one horizon is one block type, not noise. Null when the class resolves to nothing.
         */
        public B pick(long key) {
            int n = blocks.size();
            if (n == 0)
                return null;
            if (n == 1)
                return blocks.get(0);
            return blocks.get((int) Math.floorMod(mix(key), (long) n));
        }
    }

    private PaletteRules() {}

    public static <B> Result<B> resolve(String rockClass, Candidates<B> c, Kinds<B> kinds, Function<B, String> describe) {
        List<String> rejected = new ArrayList<>();
        Predicate<B> ok = b -> {
            String why = null;
            if (kinds.isRejected(b))
                why = "air";
            else if (kinds.isFiller(b) && !(COAL.equals(rockClass) && kinds.isStone(b)))
                why = "filler (stone and deepslate are unclassified crust; only coal may fall back to stone)";
            if (why != null)
                rejected.add(describe.apply(b) + ": " + why);
            return why == null;
        };

        LinkedHashSet<B> members = new LinkedHashSet<>();
        for (B b : c.memberTag())
            if (ok.test(b))
                members.add(b);
        for (B b : c.memberStates())
            if (ok.test(b))
                members.add(b);
        if (!members.isEmpty())
            return new Result<>(rockClass, Source.MEMBER, List.copyOf(members), List.copyOf(rejected));

        List<B> fallbacks = new ArrayList<>(c.fallbackTag());
        fallbacks.addAll(c.fallbackStates());
        for (B b : fallbacks)
            if (ok.test(b))
                return new Result<>(rockClass, Source.FALLBACK, List.of(b), List.copyOf(rejected));

        if (c.vanillaFallback().isPresent() && ok.test(c.vanillaFallback().get()))
            return new Result<>(rockClass, Source.VANILLA_FALLBACK, List.of(c.vanillaFallback().get()), List.copyOf(rejected));

        return new Result<>(rockClass, Source.NONE, List.of(), List.copyOf(rejected));
    }

    /** SplitMix64's finaliser: spreads consecutive keys over the members. */
    public static long mix(long z) {
        z = (z ^ (z >>> 30)) * 0xBF58476D1CE4E5B9L;
        z = (z ^ (z >>> 27)) * 0x94D049BB133111EBL;
        return z ^ (z >>> 31);
    }
}
