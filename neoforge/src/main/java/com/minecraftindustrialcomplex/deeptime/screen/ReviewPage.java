// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.screen.model.Formats;
import com.minecraftindustrialcomplex.deeptime.screen.model.ProvinceHints;
import com.minecraftindustrialcomplex.deeptime.screen.model.Ramps;
import com.minecraftindustrialcomplex.deeptime.screen.model.ReviewModel;
import com.minecraftindustrialcomplex.deeptime.worlddata.WorldData;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.Button;
import net.minecraft.network.chat.Component;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * The review page (plan §7.12 page 3): the final map (the world data's elevation in the viewer's
 * colours), the spawn marker, ore-province hints (coarse cells, never exact deposits), a summary
 * and the elevation legend; Play binds the world to the run and creates it, as the generation page's
 * Play does. Opened from the generation page's More tab. The pack's biome names over the map (§7.2)
 * come with P1-09. On a cube world the map is the net as it is played, 4 : 3, with the border
 * ({@link com.minecraftindustrialcomplex.deeptime.screen.model.NetView}, cube plan G13); the page is
 * laid out again when the review finds a cube.
 */
final class ReviewPage implements PlanetEditorScreen.Page {

    private static boolean hints = true;

    private PlanetEditorScreen screen;
    private ReviewModel model;
    private final KeyframeTexture texture = new KeyframeTexture("review");
    private final KeyframeTexture hintsTexture = new KeyframeTexture("review_hints");
    private boolean uploaded;
    /** Whether the page was laid out for a cube world's net, and the ticks since the map was uploaded. */
    private boolean laidOutAsNet;
    private int shownTicks;
    /** The height kept under a stacked map for the legend and the summary. */
    private static final int STACK_TEXT_H = 24 + 100;

    private int mapX, mapY, mapW, mapH, sideX, sideW, textY;
    /** True when the map spans the width and the summary sits under it (a tall screen). */
    private boolean stacked;
    private Button accept;

    @Override
    public void init(PlanetEditorScreen s) {
        screen = s;
        model = s.model().review().orElseGet(() -> s.model().openReview());
        int top = 22, bottom = s.height - 30;
        int columnW = s.width >= 420 ? 150 : 110;
        int columnX = s.width - columnW - 6;
        var netView = model.ready() ? model.net().orElse(null) : null;
        laidOutAsNet = netView != null;
        int[] side = netView != null ? MapWidget.fit(columnX - 12, bottom - top - 12 - 24, 4, 3) : MapWidget.fit(columnX - 12, bottom - top - 12);
        int fullW = s.width - 12;
        int[] stack = netView != null ? MapWidget.fit(fullW, Math.max(20, bottom - top - STACK_TEXT_H), 4, 3)
                : MapWidget.fit(fullW, Math.max(20, bottom - top - STACK_TEXT_H));
        // As on the generation page: stacked when that makes the map much larger (2000x1500 at GUI
        // scale 3), else the summary beside it (1920x1080).
        stacked = stack[0] > side[0] * 5 / 4;
        int[] fit = stacked ? stack : side;
        mapX = stacked ? 6 + (fullW - fit[0]) / 2 : 6 + (columnX - 12 - fit[0]) / 2;
        mapY = top;
        mapW = fit[0];
        mapH = fit[1];
        sideX = stacked ? mapX : columnX;
        sideW = stacked ? (mapW - 12) / 2 : columnW;
        textY = stacked ? mapY + mapH + 4 + 20 + 6 : mapY;
        if (netView != null)
            s.add(new MapWidget(mapX, mapY, mapW, mapH, texture, this::spawnMarker, netView, hintsTexture, () -> hints));
        else
            s.add(new MapWidget(mapX, mapY, mapW, mapH, texture, this::spawnMarker, this::provinceBoxes));
        int by = s.height - 24;
        int bw = Math.min(96, (s.width - 32) / 3);
        s.add(Button.builder(Lang.tr("review.back"), b -> {
            s.model().showRun();
            s.refresh();
        }).bounds(8, by, bw, 20).build());
        s.add(Button.builder(hints ? Lang.tr("review.hints.on") : Lang.tr("review.hints.off"), b -> {
            hints = !hints;
            b.setMessage(hints ? Lang.tr("review.hints.on") : Lang.tr("review.hints.off"));
        }).bounds(8 + bw + 4, by, bw, 20).build());
        int pw = Math.min(120, bw + 24);
        accept = s.add(Button.builder(Lang.tr("gen.play").withStyle(net.minecraft.ChatFormatting.BOLD), b -> s.play())
                .bounds(s.width - 8 - pw, by, pw, 20).build());
        uploaded = false;
        shownTicks = 0;
        tick();
    }

    /** True once the final map has been on screen a few ticks in its own layout (the screenshot walk waits for it). */
    boolean settled() {
        return model.ready() && laidOutAsNet == model.net().isPresent() && (model.map().isEmpty() || uploaded && shownTicks >= 2);
    }

    private List<MapWidget.Marker> spawnMarker() {
        return model.spawn().map(p -> List.of(new MapWidget.Marker(p[0], p[1], 0xffff40ff, true))).orElse(List.of());
    }

    private List<MapWidget.Box> provinceBoxes() {
        if (!hints)
            return List.of();
        List<MapWidget.Box> out = new ArrayList<>();
        for (ProvinceHints.Cell c : model.provinces())
            out.add(new MapWidget.Box(c.lonWest(), c.lonEast(), c.latNorth(), c.latSouth(),
                    0x50000000 | ProvinceHints.colour(c.type()), ""));
        return out;
    }

    @Override
    public void tick() {
        if (model.ready() && model.net().isPresent() != laidOutAsNet) {
            // A cube world's net is 4 : 3, not the Lambert picture's π : 1.
            screen.refresh();
            return;
        }
        if (!uploaded && model.map().isPresent()) {
            ReviewModel.MapImage m = model.map().get();
            texture.upload(m.abgr(), m.w(), m.h());
            model.hintsOverlay().ifPresent(o -> hintsTexture.upload(o.abgr(), o.w(), o.h()));
            uploaded = true;
        } else if (uploaded) {
            shownTicks++;
        }
        accept.active = model.ready() && screen.model().canAccept();
        accept.setTooltip(net.minecraft.client.gui.components.Tooltip.create(Lang.tr("gen.play.hint")));
    }

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        Font font = screen.font();
        if (!model.ready()) {
            g.drawCenteredString(font, Lang.tr("review.computing"), mapX + mapW / 2, mapY + mapH / 2, 0xffa0a0a0);
            return;
        }
        // Legend: the elevation ramp, sea level marked.
        Ramps.Ramp ramp = Ramps.builtin().forLayer(WorldData.ELEVATION);
        int ly = mapY + mapH + 4, lw = Math.min(mapW, 200);
        for (int i = 0; i < lw; i++) {
            double v = ramp.min() + (ramp.max() - ramp.min()) * i / (lw - 1.0);
            g.fill(mapX + i, ly, mapX + i + 1, ly + 5, 0xff000000 | ramp.colour(v));
        }
        g.drawString(font, "y " + Formats.number(ramp.min()), mapX, ly + 7, 0xffa0a0a0, false);
        String hi = "y " + Formats.number(ramp.max());
        g.drawString(font, hi, mapX + lw - font.width(hi), ly + 7, 0xffa0a0a0, false);
        model.map().ifPresent(m -> g.drawString(font, Lang.tr("review.source." + ("keyframe".equals(m.source()) ? "keyframe" : "world_data")),
                mapX + lw + 6, ly, 0xffa0a0a0, false));
        if (laidOutAsNet) {
            // The net's key, beside the elevation ramp's numbers: the border's colour, and what the shape is.
            int ky = ly + 10;
            g.fill(mapX + lw + 6, ky + 3, mapX + lw + 16, ky + 5,
                    0xff000000 | GlobeWidget.abgrToArgb(com.minecraftindustrialcomplex.deeptime.screen.model.GlobeProjection.BORDER_ABGR));
            g.drawString(font, Lang.tr("review.net.border"), mapX + lw + 19, ky, 0xffa0a0a0, false);
        }

        // Summary (beside the map, or under it in two columns: the run, then the provinces).
        List<Component> lines = new ArrayList<>();
        model.error().ifPresent(e -> lines.add(Component.literal(e).withColor(0xff7070)));
        model.summary().ifPresent(sm -> {
            lines.add(Lang.tr("review.size", sm.circumference()));
            lines.add(Lang.tr("review.counts", sm.events(), sm.sites(), sm.keyframes()));
            lines.add(Lang.tr("review.source", sm.source()));
        });
        if (laidOutAsNet)
            lines.add(Lang.tr("review.net.shape"));
        lines.add(model.spawn().map(p -> (Component) Lang.tr("review.spawn", String.format(java.util.Locale.ROOT, "%.1f", p[0]),
                String.format(java.util.Locale.ROOT, "%.1f", p[1]))).orElse(Lang.tr("review.spawn.none")));
        if (screen.model().accepted().map(a -> a.stagingId().equals(screen.model().run().map(r -> r.stagingId()).orElse(""))).orElse(false))
            lines.add(Lang.tr("review.already_accepted").withColor(0x80ff80));
        List<Component> provinces = new ArrayList<>();
        Map<String, Integer> byType = new LinkedHashMap<>();
        for (ProvinceHints.Cell c : model.provinces())
            byType.merge(c.type(), 1, Integer::sum);
        provinces.add(byType.isEmpty() ? Lang.tr("review.hints.none") : Lang.tr("review.hints.title"));
        for (var e : byType.entrySet())
            provinces.add(Component.literal("■ ").withColor(ProvinceHints.colour(e.getKey()))
                    .append(Component.empty().append(Lang.dynamic(ProvinceHints.nameKey(e.getKey()), e.getKey()))
                            .append(" (" + e.getValue() + ")").withColor(0xe0e0e0)));
        if (stacked) {
            drawLines(g, font, lines, sideX, textY);
            drawLines(g, font, provinces, sideX + sideW + 12, textY);
        } else {
            // Beside the map (a wide window) the map is bound by its width and leaves room under the
            // legend: the provinces go there, side by side, when they fit, else under the summary.
            int underY = ly + 7 + 10 + 8;
            if (!flowLines(g, font, provinces, mapX, underY, mapW, screen.height - 30 - underY)) {
                lines.add(Component.empty());
                lines.addAll(provinces);
            }
            drawLines(g, font, lines, sideX, textY);
        }
    }

    /**
     * The first line (the heading) on a line of its own, the rest side by side in rows of {@code w};
     * nothing is drawn and false is returned when that takes more than {@code h}.
     */
    private static boolean flowLines(GuiGraphics g, Font font, List<Component> items, int x, int y, int w, int h) {
        int n = items.size();
        int[] ox = new int[n], oy = new int[n];
        int line = 0, cx = 0;
        for (int i = 1; i < n; i++) {
            int tw = font.width(items.get(i));
            if (tw > w)
                return false;
            if (i == 1 || cx + tw > w) {
                line++;
                cx = 0;
            }
            ox[i] = cx;
            oy[i] = line * 10;
            cx += tw + 10;
        }
        if (n == 0 || font.width(items.get(0)) > w || (line + 1) * 10 > h)
            return false;
        g.drawString(font, items.get(0), x, y, 0xffe0e0e0, false);
        for (int i = 1; i < n; i++)
            g.drawString(font, items.get(i), x + ox[i], y + oy[i], 0xffe0e0e0, false);
        return true;
    }

    private void drawLines(GuiGraphics g, Font font, List<Component> lines, int x, int y) {
        for (Component c : lines)
            for (var seq : font.split(c, sideW)) {
                g.drawString(font, seq, x, y, 0xffe0e0e0, false);
                y += 10;
            }
    }

    @Override
    public Component title() {
        return Lang.tr("review.title");
    }

    @Override
    public void close() {
        texture.close();
        hintsTexture.close();
    }
}
