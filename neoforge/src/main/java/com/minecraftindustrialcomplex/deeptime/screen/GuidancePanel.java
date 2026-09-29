// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.screen.model.GuidanceModel;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.Button;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import net.minecraft.util.FormattedCharSequence;

import java.util.ArrayList;
import java.util.List;

/**
 * Shown instead of the setup page when the natives did not load (plan §6.6): the detected platform,
 * the failure class, the library that was tried and what to do. Create stays disabled for Deep Time
 * ({@code CreateWorldGuard}); existing worlds still play.
 */
final class GuidancePanel implements PlanetEditorScreen.Page {

    private PlanetEditorScreen screen;
    private GuidanceModel guidance;
    private List<FormattedCharSequence> lines = List.of();

    @Override
    public void init(PlanetEditorScreen s) {
        screen = s;
        guidance = GuidanceModel.of(NativeLoader.status());
        Font font = s.font();
        int w = Math.min(s.width - 40, 360);
        List<FormattedCharSequence> out = new ArrayList<>();
        out.addAll(font.split(Lang.tr("guidance.headline"), w));
        out.add(FormattedCharSequence.EMPTY);
        out.addAll(font.split(Lang.tr("guidance.class", guidance.failureClass()), w));
        out.addAll(font.split(Lang.tr("guidance.platform", guidance.platform()), w));
        if (!guidance.library().isEmpty())
            out.addAll(font.split(Lang.tr("guidance.library", guidance.library()), w));
        out.add(FormattedCharSequence.EMPTY);
        out.addAll(font.split(Component.literal(guidance.guidance()), w));
        out.add(FormattedCharSequence.EMPTY);
        out.addAll(font.split(Lang.tr("guidance.create_disabled"), w));
        lines = out;
        int by = s.height - 28;
        s.add(Button.builder(Lang.tr("guidance.copy"), b -> {
            Minecraft.getInstance().keyboardHandler.setClipboard(guidance.details());
            b.setMessage(Lang.tr("guidance.copied"));
        }).bounds(s.width / 2 - 104, by, 100, 20).build());
        s.add(Button.builder(CommonComponents.GUI_BACK, b -> s.onClose()).bounds(s.width / 2 + 4, by, 100, 20).build());
    }

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        Font font = screen.font();
        int w = Math.min(screen.width - 40, 360);
        int x = (screen.width - w) / 2, y = 26;
        g.fill(x - 6, y - 4, x + w + 6, y + lines.size() * 10 + 4, 0xa0000000);
        for (int i = 0; i < lines.size(); i++)
            g.drawString(font, lines.get(i), x, y + i * 10, i == 0 ? 0xffff8060 : 0xffe0e0e0, false);
    }
}
