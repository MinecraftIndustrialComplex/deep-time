// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.screen.model.MachineInfo;
import com.minecraftindustrialcomplex.deeptime.worldgen.PlanetGenerators;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.screens.worldselection.CreateWorldScreen;
import net.minecraft.client.gui.screens.worldselection.WorldCreationUiState;
import net.minecraft.network.chat.contents.TranslatableContents;
import net.neoforged.fml.loading.FMLPaths;
import net.neoforged.neoforge.client.event.ScreenEvent;
import net.neoforged.neoforge.common.NeoForge;
import org.lwjgl.glfw.GLFW;

/**
 * Keeps the Create World screen's Create button disabled for a Deep Time world until it has world
 * data to be created from (plan §6.6, §7.12): a finished, accepted run in the planet editor. There
 * is no default story (decisions file, "Timeline"), so a Deep Time world cannot be created without
 * a run; with the natives missing, Create stays disabled for Deep Time only. Enter, which also
 * creates the world, is held back the same way. The rule itself is
 * {@link PlanetGenerators#createBlock}, which the GameTests check. Also starts the background
 * memory reading for the editor's setup page ({@link MachineInfo#prime()}).
 */
final class CreateWorldGuard {

    private CreateWorldGuard() {}

    static void init() {
        NeoForge.EVENT_BUS.addListener(ScreenEvent.Init.Post.class, CreateWorldGuard::onInit);
        NeoForge.EVENT_BUS.addListener(ScreenEvent.KeyPressed.Pre.class, CreateWorldGuard::onKey);
    }

    static PlanetGenerators.CreateBlock block(WorldCreationUiState ui) {
        return PlanetGenerators.createBlock(ui.getSettings().selectedDimensions(), FMLPaths.GAMEDIR.get(), NativeLoader.isLinked());
    }

    private static void onInit(ScreenEvent.Init.Post event) {
        if (!(event.getScreen() instanceof CreateWorldScreen screen))
            return;
        // The setup page's "free memory" is read in the background (macOS: a vm_stat process); starting
        // it here has the figure ready by the time the editor opens. Rate-limited, returns at once.
        MachineInfo.prime();
        for (GuiEventListener l : event.getListenersList()) {
            if (l instanceof Button b && b.getMessage().getContents() instanceof TranslatableContents t
                    && "selectWorld.create".equals(t.getKey())) {
                WorldCreationUiState ui = screen.getUiState();
                ui.addListener(state -> update(b, state));
                update(b, ui);
            }
        }
    }

    private static void update(Button create, WorldCreationUiState ui) {
        PlanetGenerators.CreateBlock why = block(ui);
        create.active = why == PlanetGenerators.CreateBlock.NONE;
        create.setTooltip(switch (why) {
            case NONE -> null;
            case NATIVES -> Tooltip.create(Lang.tr("create.blocked.natives"));
            case NOT_SIMULATED -> Tooltip.create(Lang.tr("create.blocked.not_simulated"));
            case STAGING_MISSING -> Tooltip.create(Lang.tr("create.blocked.staging_missing"));
        });
    }

    private static void onKey(ScreenEvent.KeyPressed.Pre event) {
        if (event.getScreen() instanceof CreateWorldScreen screen
                && (event.getKeyCode() == GLFW.GLFW_KEY_ENTER || event.getKeyCode() == GLFW.GLFW_KEY_KP_ENTER)
                && block(screen.getUiState()) != PlanetGenerators.CreateBlock.NONE)
            event.setCanceled(true);
    }
}
