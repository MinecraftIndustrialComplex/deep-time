// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.dh;

/**
 * Keeps Distant Horizons' world generator on while the player is on a Deep Time planet, against
 * the one rule of Distant Horizons' that switches it off behind the player's back. Plain Java: no
 * Minecraft and no Distant Horizons type appears here; {@link DhApiBinding#generationEnabled} reaches
 * the setting and {@code client/DhGenerationClient} feeds this the level the player is in.
 *
 * <p><b>Why.</b> Distant Horizons 3.3.1's client start ({@code AbstractModInitializer.
 * logIncompatibilityWarnings}, run from {@code onInitializeClient} only) puts
 * {@code generatorPlan = DISABLED} on the API override layer of every client that has Chunky
 * installed ({@code org.popcraft.chunky.api.ChunkyAPI} on the class path), whatever the player's
 * file says. A dedicated server never runs that check. The override then decides everything:
 * <ul>
 *   <li>multiplayer: the client sends its plan in its session config, and {@code SessionConfig}
 *       answers DISABLED when either side is DISABLED, so {@code DhClientLevel.shouldDoWorldGen}
 *       stays false, the request module never starts, and the client asks the server for nothing
 *       (STATUS.md build log 2026-10-20: Ben's client, "World Gen/Import Tasks: 0");</li>
 *   <li>singleplayer: the integrated server reads the same in-memory config, so it never
 *       generates either.</li>
 * </ul>
 * Deep Time's far terrain is that generator (a world-generator override, {@link DhLodGenerator}),
 * and Chunky is in the MIC pack for the server's pregeneration, so on a Deep Time planet this takes
 * the override off again. Distant Horizons' reason (Chunky writing chunks faster than it can turn
 * them into LODs) is about its chunk-save path, which this does not change.
 *
 * <p><b>What it touches.</b> Only the API override layer, and only a "generation off" value it
 * finds there with Chunky installed (Distant Horizons' rule), or one it put back itself. The
 * player's config file is never written: a player who sets the plan to DISABLED keeps it off. Off
 * the planet (title screen, the Nether, another server) it puts Distant Horizons' value back. If
 * something sets "off" again after it lifted it, that is left alone for the rest of the session.
 */
public final class DhGenerationGate {

    /** The class Distant Horizons looks for to decide Chunky is installed. */
    public static final String CHUNKY_API_CLASS = "org.popcraft.chunky.api.ChunkyAPI";

    private DhGenerationGate() {}

    /**
     * Distant Horizons' "distant generation enabled" setting ({@code IDhApiWorldGenerationConfig.
     * enableDistantWorldGeneration()}, the generator plan seen as a boolean: DISABLED is false).
     */
    public interface Slot {
        /** What the player's config file holds ({@code getTrueValue}); never the API layer. */
        boolean playerValue();

        /** The API override layer ({@code getApiValue}); null when no mod overrides it. */
        Boolean apiValue();

        /** Whether Distant Horizons lets mods override this setting. */
        boolean overridable();

        /** Sets the API override layer (in memory only). */
        boolean setApi(boolean enabled);

        /** Clears the API override layer, so the player's own value shows again. */
        boolean clearApi();
    }

    /** What {@link Controller#reconcile} did, for the log. */
    public enum Outcome {
        /** Nothing changed. */
        UNCHANGED,
        /** Distant Horizons' Chunky override was taken off: generation follows the player's file. */
        LIFTED,
        /** Off the planet: Distant Horizons' override was put back. */
        RESTORED,
        /** Generation is off by an override this does not lift (another mod's, or one set again after ours). */
        FOREIGN_OVERRIDE,
        /** Generation is off in the player's own config file; left alone. */
        PLAYER_OFF,
        /** Distant Horizons does not let mods override the setting. */
        NOT_OVERRIDABLE
    }

    /** One client's state: what it took off or put back. */
    public static final class Controller {
        private enum Held { NONE, LIFTED, RESTORED }

        private Held held = Held.NONE;
        private boolean yielded;

        /**
         * @param slot     Distant Horizons' setting
         * @param onPlanet the player is in a Deep Time planet level
         * @param chunky   Chunky is installed (Distant Horizons' own test, {@link #CHUNKY_API_CLASS})
         */
        public Outcome reconcile(Slot slot, boolean onPlanet, boolean chunky) {
            Boolean api = slot.apiValue();
            if (held == Held.LIFTED && api != null) {
                // Someone put a value on the layer after we cleared it: theirs from now on.
                held = Held.NONE;
                if (Boolean.FALSE.equals(api))
                    yielded = true;
            } else if (held == Held.RESTORED && !Boolean.FALSE.equals(api)) {
                held = Held.NONE; // our "off" was replaced or cleared: nothing of ours is there
            }
            if (onPlanet) {
                if (Boolean.FALSE.equals(api)) {
                    boolean ours = held == Held.RESTORED;
                    if (!ours && (yielded || !chunky))
                        return Outcome.FOREIGN_OVERRIDE;
                    if (!slot.overridable() || !slot.clearApi())
                        return Outcome.NOT_OVERRIDABLE;
                    held = Held.LIFTED;
                    return Outcome.LIFTED;
                }
                return api == null && !slot.playerValue() ? Outcome.PLAYER_OFF : Outcome.UNCHANGED;
            }
            if (held == Held.LIFTED) {
                held = Held.NONE;
                if (!slot.setApi(false))
                    return Outcome.NOT_OVERRIDABLE;
                held = Held.RESTORED;
                return Outcome.RESTORED;
            }
            return Outcome.UNCHANGED;
        }

        /** True while the override is off because of this controller. */
        public boolean lifted() {
            return held == Held.LIFTED;
        }
    }
}
