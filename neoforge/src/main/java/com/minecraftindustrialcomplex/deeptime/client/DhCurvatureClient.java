// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.client;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhApiBinding;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhCurvature;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhFarTerrain;
import com.minecraftindustrialcomplex.deeptime.net.ClientPlanetState;
import net.minecraft.client.Minecraft;
import net.neoforged.neoforge.client.event.ClientTickEvent;

import java.util.Optional;

/**
 * Sets Distant Horizons' world curvature to the planet's while the client is in a Deep Time level,
 * and puts it back when it is not (decisions log 2026-10-03). The planet's circumference is what
 * the server's {@code WorldInfoPayload} already tells the client ({@link ClientPlanetState}); the
 * rule is {@link DhCurvature}.
 *
 * <p><b>Optional.</b> Registered only when Distant Horizons is installed, and bound by name like
 * the rest of the compat ({@link DhApiBinding}); without it, or with an API this cannot bind,
 * nothing happens. The value goes on Distant Horizons' API override layer, never into the player's
 * config file. Checked once a second on the client tick: Distant Horizons' API is not ready when
 * the game starts, a Deep Time level's info arrives after the player joins, and the player may
 * change their own value at any time.
 *
 * <p>Nothing here can be checked by the headless tests (plan §9.8); the rule and the binding are
 * tested in {@code DhCurvatureTest} and {@code DhApiBindingTest}.
 */
public final class DhCurvatureClient {

    private static final int POLL_TICKS = 20;
    private static final DhCurvature.Controller CONTROLLER = new DhCurvature.Controller();
    private static int ticks;
    private static boolean broken;
    private static DhCurvature.Outcome logged = DhCurvature.Outcome.UNCHANGED;

    private DhCurvatureClient() {}

    /** Called from {@link DeepTimeClient#init}; does nothing without Distant Horizons. */
    static void init() {
        if (!DhFarTerrain.installed())
            return;
        net.neoforged.neoforge.common.NeoForge.EVENT_BUS.addListener(ClientTickEvent.Pre.class, e -> onClientTick());
        DeepTime.LOGGER.info("[deeptime] Distant Horizons is installed: its world curvature follows the planet "
                + "(client config dhAutoCurvature, default on)");
    }

    private static void onClientTick() {
        if (broken || ++ticks % POLL_TICKS != 0)
            return;
        Minecraft mc = Minecraft.getInstance();
        int circumference = mc.level == null ? 0 : ClientPlanetState.circumference(mc.level);
        apply(circumference, ClientConfig.dhAutoCurvature());
    }

    /** The player left the server: take our value off now rather than at the next poll. */
    static void release() {
        apply(0, false);
    }

    private static void apply(int circumference, boolean enabled) {
        Optional<DhApiBinding> api = DhFarTerrain.api();
        if (api.isEmpty()) {
            broken = true;
            return;
        }
        try {
            DhCurvature.Slot slot = api.get().earthCurvature();
            if (slot == null)
                return; // Distant Horizons has not started its API yet
            DhCurvature.Outcome outcome = CONTROLLER.reconcile(slot, circumference, enabled);
            if (outcome != DhCurvature.Outcome.UNCHANGED && (outcome != logged || outcome == DhCurvature.Outcome.SET)) {
                Integer ours = CONTROLLER.ours();
                switch (outcome) {
                    case SET -> DeepTime.LOGGER.info("[deeptime] Distant Horizons curvature set to the planet's: ratio {} "
                            + "(sphere of {} blocks; circumference {})", ours, Math.round(DhCurvature.radiusOfRatio(ours)), circumference);
                    case CLEARED -> DeepTime.LOGGER.info("[deeptime] Distant Horizons curvature handed back (not in a Deep Time level, or switched off)");
                    case PLAYER_VALUE -> DeepTime.LOGGER.info("[deeptime] Distant Horizons curvature left as the player set it");
                    case FOREIGN_OVERRIDE -> DeepTime.LOGGER.info("[deeptime] Distant Horizons curvature is overridden by another mod; left alone");
                    case NOT_OVERRIDABLE -> DeepTime.LOGGER.warn("[deeptime] Distant Horizons does not let mods override its curvature; left alone");
                    default -> {
                    }
                }
            }
            if (outcome != DhCurvature.Outcome.UNCHANGED)
                logged = outcome;
        } catch (ReflectiveOperationException | RuntimeException | LinkageError e) {
            broken = true;
            DeepTime.LOGGER.error("[deeptime] Distant Horizons curvature: the setting could not be reached; "
                    + "its curvature is left alone from now on", e);
        }
    }
}
