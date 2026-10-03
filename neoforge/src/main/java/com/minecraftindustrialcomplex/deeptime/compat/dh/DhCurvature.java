// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

/**
 * The Distant Horizons world curvature that matches a Deep Time planet (Ben, decisions log
 * 2026-10-03: "make it so those default to values that make sense for the given world a player is
 * in"). Plain Java: no Minecraft and no Distant Horizons type appears here; {@link DhApiBinding}
 * reaches the setting and {@code client/DhCurvatureClient} feeds this the level the player is in.
 *
 * <p><b>The setting</b> (Distant Horizons 3.3.1, client config {@code Config.Client.Advanced.Graphics.
 * Experimental.earthCurveRatio}; API {@code IDhApiGraphicsConfig.earthCurvatureRatio()}, an
 * {@code Integer}): the ratio of Earth's size to the curved world's. 0 is flat (the default); the
 * terrain shader draws a sphere of radius {@code 6,371,000 / ratio} blocks around the camera. DH
 * accepts 50 to 5,000 (or the same negated, a bowl) and rewrites 1 to 49 to 50 <i>in the player's
 * config file</i>, so this class never offers a value in that gap. Only Distant Horizons' own level
 * of detail meshes are bent; Minecraft's chunks are drawn flat.
 *
 * <p>docs/science/world-scale-and-bake.md, "Distant Horizons world curvature", has the arithmetic,
 * the table and the sources.
 */
public final class DhCurvature {

    /** Distant Horizons' constant: its shader's earth radius is this over the ratio, in blocks (Earth's mean radius in metres, one block a metre). */
    public static final double EARTH_RADIUS_BLOCKS = 6_371_000.0; // ledger: BAK-043
    /** The smallest ratio Distant Horizons accepts: a sphere of 127,420 blocks. */
    public static final int MIN_RATIO = 50; // ledger: BAK-046
    /** The largest ratio Distant Horizons accepts: a sphere of 1,274 blocks. */
    public static final int MAX_RATIO = 5_000; // ledger: BAK-046

    private DhCurvature() {}

    /** The planet's radius in blocks: its circumference over 2 pi. */
    public static double radiusBlocks(long circumferenceBlocks) {
        return circumferenceBlocks / (2.0 * Math.PI); // ledger: BAK-044
    }

    /** The sphere radius, in blocks, that Distant Horizons draws for {@code ratio}; 0 for the flat world (ratio 0). */
    public static double radiusOfRatio(int ratio) {
        return ratio == 0 ? 0 : EARTH_RADIUS_BLOCKS / ratio;
    }

    /**
     * How far the ground {@code distance} blocks away falls below the tangent plane on a sphere of
     * {@code radius} blocks, as Distant Horizons' terrain shader moves it:
     * {@code R (1 - cos(d / R))}, for ground at the sea-level radius.
     */
    public static double dropBlocks(double radius, double distance) {
        return radius * (1.0 - Math.cos(distance / radius)); // ledger: BAK-044
    }

    /**
     * The ratio to give Distant Horizons for a planet of {@code circumferenceBlocks}, or 0 when
     * there is nothing to set: no planet, or one so large (radius over 127,420 blocks, a circumference
     * over 800,600) that the sphere is flatter than Distant Horizons can draw, so flat is closer to it
     * than the weakest curve is. A planet smaller than Distant Horizons' strongest curve (radius under
     * 1,274 blocks, a circumference under 8,006) gets that strongest curve.
     */
    public static int ratioFor(long circumferenceBlocks) {
        if (circumferenceBlocks <= 0)
            return 0;
        double ratio = EARTH_RADIUS_BLOCKS / radiusBlocks(circumferenceBlocks);
        if (ratio < MIN_RATIO) // ledger: BAK-046
            return 0;
        return (int) Math.min(MAX_RATIO, Math.round(ratio));
    }

    /**
     * Distant Horizons' side of the setting: its value layers. {@code DhApiBinding} implements it
     * over {@code IDhApiConfigValue}; the tests implement it with a fake.
     */
    public interface Slot {
        /** What the player's config file holds ({@code getTrueValue}); never the API layer. */
        int playerValue();

        /** Distant Horizons' default for the setting (0, flat). */
        int defaultValue();

        /** The API override layer ({@code getApiValue}); null when no mod overrides it. */
        Integer apiValue();

        /** Whether Distant Horizons lets mods override this setting. */
        boolean overridable();

        /** Sets the API override layer (in memory; Distant Horizons never writes it to the config file). */
        boolean setApi(int ratio);

        /** Clears the API override layer, so the player's own value shows again. */
        boolean clearApi();
    }

    /** What {@link Controller#reconcile} did, for the log. */
    public enum Outcome {
        /** Nothing changed. */
        UNCHANGED,
        /** The planet's ratio was set on the API layer. */
        SET,
        /** Our value was taken off the API layer. */
        CLEARED,
        /** The player has a value of their own; it is left alone. */
        PLAYER_VALUE,
        /** Another mod's API override is in place; it is left alone. */
        FOREIGN_OVERRIDE,
        /** Distant Horizons does not let mods override this setting. */
        NOT_OVERRIDABLE
    }

    /**
     * Keeps the API layer of one Distant Horizons setting in step with the level the player is in.
     * It owns at most one value, the last it set, and takes only that one off: a value another mod
     * put there, and the player's config file, are never touched.
     */
    public static final class Controller {
        private Integer ours;

        /**
         * @param slot          Distant Horizons' setting
         * @param circumference the planet's circumference in blocks, 0 when the level is not a Deep Time planet
         * @param enabled       Deep Time's switch ("auto curvature"); false behaves as a level with no planet
         */
        public Outcome reconcile(Slot slot, long circumference, boolean enabled) {
            // The player's own value wins over everything of ours; looked at on every call, so a value
            // typed into Distant Horizons' screen later releases ours too.
            boolean player = slot.playerValue() != slot.defaultValue();
            int planetRatio = ratioFor(circumference);
            int wanted = enabled && !player ? planetRatio : 0;
            Integer api = slot.apiValue();
            if (ours != null && !ours.equals(api))
                ours = null; // someone else replaced or cleared our value: it is no longer ours to take off
            if (wanted > 0) {
                if (!slot.overridable())
                    return Outcome.NOT_OVERRIDABLE;
                if (api != null && ours == null)
                    return Outcome.FOREIGN_OVERRIDE;
                if (ours != null && ours == wanted)
                    return Outcome.UNCHANGED;
                if (!slot.setApi(wanted))
                    return Outcome.NOT_OVERRIDABLE;
                ours = wanted;
                return Outcome.SET;
            }
            if (ours != null) {
                slot.clearApi();
                ours = null;
                return Outcome.CLEARED;
            }
            return enabled && player && planetRatio > 0 ? Outcome.PLAYER_VALUE : Outcome.UNCHANGED;
        }

        /** Takes our value off, if it is still ours (the player leaves the game). */
        public Outcome release(Slot slot) {
            return reconcile(slot, 0, false);
        }

        /** The ratio this controller has on the API layer, or null. */
        public Integer ours() {
            return ours;
        }
    }
}
