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

/**
 * The Create World screen's Create button for a Deep Time world (plan §6.6, §7.12; the one-click
 * flow of 2026-10-19, docs/design/create-world-ux.md). A Deep Time world is created from world data,
 * a finished run of the planet's history; without it Create opens the generation screen
 * ({@link CreateWorldFlow#interceptCreate}), so the button stays enabled and its tooltip says what
 * it will do. Only with the natives missing is it disabled, for Deep Time only (Enter is held back
 * by the same interception). The rule itself is {@link PlanetGenerators#createBlock}, which the
 * GameTests check. Also starts the background memory reading for the editor's Customize page
 * ({@link MachineInfo#prime()}).
 */
final class CreateWorldGuard {

    private CreateWorldGuard() {}

    static void init() {
        NeoForge.EVENT_BUS.addListener(ScreenEvent.Init.Post.class, CreateWorldGuard::onInit);
    }

    static PlanetGenerators.CreateBlock block(WorldCreationUiState ui) {
        return PlanetGenerators.createBlock(ui.getSettings().selectedDimensions(), FMLPaths.GAMEDIR.get(), NativeLoader.isLinked());
    }

    private static void onInit(ScreenEvent.Init.Post event) {
        if (!(event.getScreen() instanceof CreateWorldScreen screen))
            return;
        // The Customize page's "free memory" is read in the background (macOS: a vm_stat process); starting
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
        create.active = why != PlanetGenerators.CreateBlock.NATIVES;
        create.setTooltip(switch (why) {
            case NONE -> null;
            case NATIVES -> Tooltip.create(Lang.tr("create.blocked.natives"));
            case NOT_SIMULATED -> Tooltip.create(Lang.tr("create.blocked.not_simulated"));
            case STAGING_MISSING -> Tooltip.create(Lang.tr("create.blocked.staging_missing"));
        });
    }
}
