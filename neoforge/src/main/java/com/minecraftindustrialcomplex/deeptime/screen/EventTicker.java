// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.Formats;
import com.minecraftindustrialcomplex.deeptime.screen.model.GeoEvent;
import com.minecraftindustrialcomplex.deeptime.screen.model.TimelapseModel;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.narration.NarrationElementOutput;
import net.minecraft.network.chat.Component;

import java.util.List;

/**
 * The event ticker (plan §7.12): the newest events up to the scrubbed time, newest first; events
 * since the previous frame are highlighted, as in the viewer's ticker. Scrolls with the wheel.
 */
class EventTicker extends AbstractWidget {

    static final int LINE = 10;

    private final TimelapseModel model;
    private int scroll;

    EventTicker(int x, int y, int w, int h, TimelapseModel model) {
        super(x, y, w, h, Component.empty());
        this.model = model;
    }

    @Override
    protected void renderWidget(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        Font font = Minecraft.getInstance().font;
        int x = getX(), y = getY(), w = getWidth(), h = getHeight();
        g.fill(x, y, x + w, y + h, 0x90000000);
        int rows = Math.max(1, (h - 4) / LINE);
        List<GeoEvent> shown = model.visibleEvents(rows + scroll);
        if (shown.isEmpty()) {
            g.drawString(font, Lang.tr("run.no_events"), x + 3, y + 3, 0xffa0a0a0, false);
            return;
        }
        int line = 0;
        for (int i = Math.min(scroll, shown.size()); i < shown.size() && line < rows; i++, line++) {
            GeoEvent e = shown.get(i);
            boolean fresh = model.fresh(e);
            int ly = y + 2 + line * LINE;
            if (fresh)
                g.fill(x + 1, ly - 1, x + w - 1, ly + LINE - 1, 0x40ffffff);
            String t = Formats.time(e.tStart());
            g.fill(x + 3, ly + 2, x + 7, ly + 6, com.minecraftindustrialcomplex.deeptime.screen.model.EventStyle.colour(e.kind()));
            g.drawString(font, font.plainSubstrByWidth(t, w / 2 - 12), x + 10, ly, fresh ? 0xffffffff : 0xffb0b0b0, false);
            Component kind = Lang.eventKind(e.kind());
            int kx = x + w / 2;
            g.drawString(font, font.substrByWidth(kind, w - (kx - x) - 3).getString(), kx, ly, fresh ? 0xffffe080 : 0xffd0d0d0, false);
        }
    }

    @Override
    public boolean mouseScrolled(double mouseX, double mouseY, double scrollX, double scrollY) {
        if (!isMouseOver(mouseX, mouseY))
            return false;
        scroll = Math.max(0, Math.min(Math.max(0, model.events().size() - 1), scroll - (int) Math.signum(scrollY) * 3));
        return true;
    }

    @Override
    protected void updateWidgetNarration(NarrationElementOutput out) {
        defaultButtonNarrationText(out);
    }

    @Override
    public void playDownSound(net.minecraft.client.sounds.SoundManager handler) {
        // not a button
    }
}
