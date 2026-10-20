// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.client;

import com.minecraftindustrialcomplex.deeptime.DeepTime;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhApiBinding;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhFarTerrain;
import com.minecraftindustrialcomplex.deeptime.compat.dh.DhGenerationGate;
import com.minecraftindustrialcomplex.deeptime.net.ClientPlanetState;
import net.minecraft.client.Minecraft;
import net.neoforged.neoforge.client.event.ClientTickEvent;

import java.util.Optional;

/**
 * Keeps Distant Horizons' world generator on while the client is on a Deep Time planet, so the
 * client asks the server for far terrain (multiplayer) and the integrated server generates it
 * (singleplayer). The rule, and why Distant Horizons switches it off when Chunky is installed, is
 * {@link DhGenerationGate}.
 *
 * <p>Registered only when Distant Horizons is installed, bound by name ({@link DhApiBinding}), and
 * checked once a second on the client tick like {@link DhCurvatureClient}: Distant Horizons'
 * API starts after the game does, and the planet's info arrives after the player joins. A change
 * on Distant Horizons' API layer makes its client send its session config again, so lifting the
 * override after the join is enough.
 *
 * <p>Nothing here can be checked by the headless tests (plan §9.8); the rule is tested in
 * {@code DhGenerationGateTest} and the binding's surface in {@code DhApiBindingTest}.
 */
public final class DhGenerationClient {

    private static final int POLL_TICKS = 20;
    private static final DhGenerationGate.Controller CONTROLLER = new DhGenerationGate.Controller();
    private static boolean chunky;
    private static int ticks;
    private static boolean broken;
    private static DhGenerationGate.Outcome logged = DhGenerationGate.Outcome.UNCHANGED;

    private DhGenerationClient() {}

    /**
     * Called from {@link DeepTimeClient#init}; does nothing without Distant Horizons, or with
     * {@code -Ddeeptime.dhGenerationGate=false} (the control in {@code tools/client/dh-client-check.sh}).
     */
    static void init() {
        if (!DhFarTerrain.installed())
            return;
        chunky = present(DhGenerationGate.CHUNKY_API_CLASS);
        if ("false".equalsIgnoreCase(System.getProperty("deeptime.dhGenerationGate", "true"))) {
            DeepTime.LOGGER.info("[deeptime] Distant Horizons' world generator gate is off (-Ddeeptime.dhGenerationGate=false; Chunky {})",
                    chunky ? "installed" : "not installed");
            return;
        }
        net.neoforged.neoforge.common.NeoForge.EVENT_BUS.addListener(ClientTickEvent.Pre.class, e -> onClientTick());
    }

    private static boolean present(String className) {
        try {
            Class.forName(className, false, DhGenerationClient.class.getClassLoader());
            return true;
        } catch (ClassNotFoundException | LinkageError e) {
            return false;
        }
    }

    private static void onClientTick() {
        if (broken || ++ticks % POLL_TICKS != 0)
            return;
        Minecraft mc = Minecraft.getInstance();
        apply(mc.level != null && ClientPlanetState.circumference(mc.level) > 0);
    }

    private static void apply(boolean onPlanet) {
        Optional<DhApiBinding> api = DhFarTerrain.api();
        if (api.isEmpty()) {
            broken = true;
            return;
        }
        try {
            DhGenerationGate.Slot slot = api.get().generationEnabled();
            if (slot == null)
                return; // Distant Horizons has not started its API yet
            DhGenerationGate.Outcome outcome = CONTROLLER.reconcile(slot, onPlanet, chunky);
            if (outcome == DhGenerationGate.Outcome.UNCHANGED)
                return;
            boolean repeat = outcome == logged && outcome != DhGenerationGate.Outcome.LIFTED && outcome != DhGenerationGate.Outcome.RESTORED;
            logged = outcome;
            if (repeat)
                return;
            switch (outcome) {
                case LIFTED -> DeepTime.LOGGER.info("[deeptime] Distant Horizons' world generator is back on for this planet: "
                        + "Distant Horizons switches it off when Chunky is installed, and Deep Time's far terrain is that generator "
                        + "(API layer only; your Distant Horizons config still decides)");
                case RESTORED -> DeepTime.LOGGER.info("[deeptime] Distant Horizons' world generator setting handed back (not on a Deep Time planet)");
                case FOREIGN_OVERRIDE -> DeepTime.LOGGER.warn("[deeptime] Distant Horizons' world generator is switched off by another "
                        + "mod's override: no far terrain reaches this client on a Deep Time planet");
                case PLAYER_OFF -> DeepTime.LOGGER.warn("[deeptime] Distant Horizons' world generator is off in your Distant Horizons "
                        + "config (generatorPlan DISABLED): no far terrain reaches this client on a Deep Time planet");
                case NOT_OVERRIDABLE -> DeepTime.LOGGER.warn("[deeptime] Distant Horizons does not let mods change its world generator "
                        + "setting; if it is off, no far terrain reaches this client");
                default -> {
                }
            }
        } catch (ReflectiveOperationException | RuntimeException | LinkageError e) {
            broken = true;
            DeepTime.LOGGER.error("[deeptime] Distant Horizons world generator setting could not be reached; left alone from now on", e);
        }
    }
}
