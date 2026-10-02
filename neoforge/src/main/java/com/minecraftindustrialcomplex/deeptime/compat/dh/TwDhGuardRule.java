// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

/**
 * The rule {@link TwDhGuard} applies before it lets Deep Time's far-terrain generator through
 * Toroidal World's Distant Horizons guard, on plain values (no Minecraft, no Toroidal World type),
 * so it is unit-tested ({@code TwDhGuardRuleTest}).
 */
public final class TwDhGuardRule {

    private TwDhGuardRule() {}

    /**
     * Whether the generator may pass TW's guard, and why. {@code guarded}: TW's guard applies to the
     * level at all (it shapes it); when it does not, TW lets every generator through itself.
     */
    public record Verdict(boolean pass, boolean guarded, String reason) {}

    /**
     * The rule, on plain values.
     *
     * @param generatorLevel the level the generator was made for
     * @param level          the level DH is binding it to
     * @param shaped         TW shapes the level (without a shape TW's guard lets everything through)
     * @param loopsX         the shape loops east-west
     * @param widthX         its period east-west, in blocks
     * @param loopsZ         the shape loops north-south
     * @param circumference  the world data's circumference C in blocks, or -1 when the level does not
     *                       generate from world data
     */
    // ledger: BAK-041 (game-choice, Ben 2026-10-02: "Mixin around TW's guard")
    public static Verdict verdict(String generatorLevel, String level, boolean shaped, boolean loopsX, long widthX, boolean loopsZ,
                                  int circumference) {
        if (!shaped)
            return new Verdict(false, false, "the level has no Toroidal World shape, so the guard does not apply");
        if (!generatorLevel.equals(level))
            return new Verdict(false, true, "the generator was made for " + generatorLevel + ", not " + level);
        if (circumference <= 0)
            return new Verdict(false, true, "the level does not generate from Deep Time's world data");
        if (!loopsX || widthX != circumference)
            return new Verdict(false, true, "the level loops east-west " + (loopsX ? "every " + widthX + " blocks" : "not at all")
                    + " but the world data every " + circumference + ": far terrain would not meet the chunks at the seam");
        if (loopsZ)
            return new Verdict(false, true, "the level also loops north-south, which the world data does not: far terrain would not "
                    + "meet the chunks at that seam");
        return new Verdict(true, true, "it reads the world data, which wraps every " + circumference
                + " blocks east-west as the level does; Toroidal World's seam handling is unchanged");
    }

}
