// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.ContainerObjectSelectionList;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.narration.NarratableEntry;
import net.minecraft.network.chat.Component;
import net.minecraft.util.FormattedCharSequence;

import java.util.List;
import java.util.function.BooleanSupplier;
import java.util.function.Supplier;

/**
 * A scrolling list of one-line rows on a dark panel, the generation page's History and Map tabs
 * (docs/design/create-world-ux.md): text, headings, colour swatches, a colour bar, the History's
 * milestone rows (a time column, a colour square, the words) and clickable choices and toggles.
 * Rows are one text line high, so a 270-pixel-high screen still shows a dozen of them; a row whose
 * words do not fit ends in an ellipsis and shows them whole on hover.
 */
final class PanelList extends ContainerObjectSelectionList<PanelList.Row> {

    static final int ROW = 12;
    private static final int BG = 0xff15181d, BORDER = 0xff2c313a;

    PanelList(Minecraft mc, int x, int y, int w, int h) {
        super(mc, w, h, y, ROW);
        setX(x);
    }

    /** Replaces the rows, keeping the scroll position where it still fits. */
    void set(List<Row> rows) {
        double scroll = getScrollAmount();
        replaceEntries(rows);
        setClampedScrollAmount(scroll);
    }

    int rows() {
        return getItemCount();
    }

    @Override
    public int getRowWidth() {
        return width - 12;
    }

    @Override
    public int getRowLeft() {
        return getX() + 4;
    }

    @Override
    protected int getScrollbarPosition() {
        return getX() + width - 6;
    }

    @Override
    protected void renderListBackground(GuiGraphics g) {
        g.fill(getX() - 1, getY() - 1, getRight() + 1, getBottom() + 1, BORDER);
        g.fill(getX(), getY(), getRight(), getBottom(), BG);
    }

    @Override
    protected void renderListSeparators(GuiGraphics g) {}

    // ------------------------------------------------------------------------------------------
    // Rows
    // ------------------------------------------------------------------------------------------

    abstract static class Row extends ContainerObjectSelectionList.Entry<Row> {

        @Override
        public List<? extends GuiEventListener> children() {
            return List.of();
        }

        @Override
        public List<? extends NarratableEntry> narratables() {
            return List.of();
        }

        static Font font() {
            return Minecraft.getInstance().font;
        }

        /** Draws {@code text} within {@code w}, ending in an ellipsis when cut; on hover a cut text shows whole. */
        static void fitted(GuiGraphics g, Component text, int x, int y, int w, int colour, boolean hovering) {
            Font f = font();
            if (f.width(text) <= w) {
                g.drawString(f, text, x, y, colour, false);
                return;
            }
            FormattedCharSequence cut = net.minecraft.locale.Language.getInstance()
                    .getVisualOrder(f.substrByWidth(text, Math.max(0, w - f.width("…"))));
            int end = g.drawString(f, cut, x, y, colour, false);
            g.drawString(f, "…", end, y, colour, false);
            if (hovering && Minecraft.getInstance().screen != null)
                Minecraft.getInstance().screen.setTooltipForNextRenderPass(text);
        }
    }

    /** A line of text. */
    static Row text(Component text, int colour) {
        return new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                fitted(g, text, left, top + 1, width, colour, hovering);
            }
        };
    }

    /** Wrapped text: one row per line. */
    static List<Row> wrapped(Component text, int width, int colour) {
        return Row.font().split(text, width).stream().map(line -> (Row) new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int w, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                g.drawString(Row.font(), line, left, top + 1, colour, false);
            }
        }).toList();
    }

    /** A section heading. */
    static Row heading(Component text) {
        return text(text.copy().withStyle(net.minecraft.ChatFormatting.BOLD), 0xffffd060);
    }

    /** A colour and what it stands for. */
    static Row swatch(int rgb, Component text) {
        return new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                g.fill(left, top, left + 9, top + 9, 0xff000000);
                g.fill(left + 1, top + 1, left + 8, top + 8, 0xff000000 | rgb);
                fitted(g, text, left + 13, top + 1, width - 13, 0xffd8d8d8, hovering);
            }
        };
    }

    /** A colour bar with its two ends' words. */
    static Row bar(int[] rgb, Component low, Component high) {
        return new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                Font f = font();
                int lw = f.width(low), hw = f.width(high);
                g.drawString(f, low, left, top + 1, 0xffd8d8d8, false);
                g.drawString(f, high, left + width - hw, top + 1, 0xffd8d8d8, false);
                int x0 = left + lw + 4, x1 = left + width - hw - 4;
                if (x1 - x0 < 8)
                    return;
                g.fill(x0 - 1, top, x1 + 1, top + 9, 0xff000000);
                for (int i = 0; i < x1 - x0; i++) {
                    int c = rgb[Math.min(rgb.length - 1, i * rgb.length / (x1 - x0))];
                    g.fill(x0 + i, top + 1, x0 + i + 1, top + 8, 0xff000000 | c);
                }
            }
        };
    }

    /**
     * A History row: the time in the column headed "Million years ago", the kind's colour (the map
     * markers' colour), the words; highlighted when {@code fresh}.
     */
    static Row milestone(String time, int timeWidth, int colour, Component text, boolean fresh) {
        return new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                Font f = font();
                if (fresh)
                    g.fill(left - 2, top - 1, left + width, top + 10, 0xff252b35);
                g.drawString(f, time, left + timeWidth - f.width(time), top + 1, 0xff9aa3ad, false);
                int sx = left + timeWidth + 4;
                g.fill(sx, top + 1, sx + 7, top + 8, 0xff000000 | colour);
                fitted(g, text, sx + 11, top + 1, width - (sx + 11 - left), fresh ? 0xffffffff : 0xffd8d8d8, hovering);
            }
        };
    }

    /** A column heading over the milestone rows' time column and words. */
    static Row columns(Component time, int timeWidth, Component what) {
        return new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                Font f = font();
                fitted(g, time, left, top + 1, timeWidth + 6, 0xff7d8590, hovering);
                fitted(g, what, left + timeWidth + 15, top + 1, width - timeWidth - 15, 0xff7d8590, hovering);
            }
        };
    }

    /**
     * A choice (a layer): drawn highlighted when {@code selected}, dimmed when {@code dim}; a click
     * runs {@code onClick}.
     */
    static Row choice(Component text, BooleanSupplier selected, BooleanSupplier dim, Supplier<Component> tooltip, Runnable onClick) {
        return new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                boolean on = selected.getAsBoolean();
                if (on)
                    g.fill(left - 2, top - 1, left + width, top + 10, 0xff3a5f8f);
                else if (hovering)
                    g.fill(left - 2, top - 1, left + width, top + 10, 0xff262c36);
                int colour = on ? 0xffffffff : dim.getAsBoolean() ? 0xff6c727a : 0xffd8d8d8;
                fitted(g, Component.literal(on ? "● " : "○ ").append(text), left, top + 1, width, colour, hovering);
                Component tip = tooltip.get();
                if (hovering && tip != null && Minecraft.getInstance().screen != null)
                    Minecraft.getInstance().screen.setTooltipForNextRenderPass(tip);
            }

            @Override
            public boolean mouseClicked(double mouseX, double mouseY, int button) {
                if (button != 0)
                    return false;
                Minecraft.getInstance().getSoundManager().play(net.minecraft.client.resources.sounds.SimpleSoundInstance
                        .forUI(net.minecraft.sounds.SoundEvents.UI_BUTTON_CLICK, 1.0F));
                onClick.run();
                return true;
            }
        };
    }

    /** A check box line: ☑ when {@code on}. */
    static Row toggle(Component text, BooleanSupplier on, Runnable onClick) {
        return new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                if (hovering)
                    g.fill(left - 2, top - 1, left + width, top + 10, 0xff262c36);
                fitted(g, Component.literal(on.getAsBoolean() ? "☑ " : "☐ ").append(text), left, top + 1, width, 0xffd8d8d8, hovering);
            }

            @Override
            public boolean mouseClicked(double mouseX, double mouseY, int button) {
                if (button != 0)
                    return false;
                Minecraft.getInstance().getSoundManager().play(net.minecraft.client.resources.sounds.SimpleSoundInstance
                        .forUI(net.minecraft.sounds.SoundEvents.UI_BUTTON_CLICK, 1.0F));
                onClick.run();
                return true;
            }
        };
    }

    /** A group selector: ◂ name ▸, the arrows step through the groups. */
    static Row stepper(Supplier<Component> text, Runnable previous, Runnable next) {
        return new Row() {
            private int left, width;

            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                this.left = left;
                this.width = width;
                Font f = font();
                g.fill(left - 2, top - 1, left + width, top + 10, 0xff222831);
                boolean overPrev = hovering && mouseX < left + 14, overNext = hovering && mouseX > left + width - 14;
                g.drawString(f, "◀", left + 2, top + 1, overPrev ? 0xffffffff : 0xffa0a8b0, false);
                g.drawString(f, "▶", left + width - 2 - f.width("▶"), top + 1, overNext ? 0xffffffff : 0xffa0a8b0, false);
                Component t = text.get().copy().withStyle(net.minecraft.ChatFormatting.BOLD);
                g.drawCenteredString(f, t, left + width / 2, top + 1, 0xffffd060);
            }

            @Override
            public boolean mouseClicked(double mouseX, double mouseY, int button) {
                if (button != 0)
                    return false;
                Minecraft.getInstance().getSoundManager().play(net.minecraft.client.resources.sounds.SimpleSoundInstance
                        .forUI(net.minecraft.sounds.SoundEvents.UI_BUTTON_CLICK, 1.0F));
                if (mouseX < left + width / 2.0)
                    previous.run();
                else
                    next.run();
                return true;
            }
        };
    }

    /** An empty row (spacing). */
    static Row gap() {
        return new Row() {
            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {}
        };
    }
}
