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
 * come with P1-09.
 */
final class ReviewPage implements PlanetEditorScreen.Page {

    private static boolean hints = true;

    private PlanetEditorScreen screen;
    private ReviewModel model;
    private final KeyframeTexture texture = new KeyframeTexture("review");
    private boolean uploaded;
    private int mapX, mapY, mapW, mapH, sideX, sideW;
    private Button accept;

    @Override
    public void init(PlanetEditorScreen s) {
        screen = s;
        model = s.model().review().orElseGet(() -> s.model().openReview());
        int top = 22, bottom = s.height - 30;
        sideW = s.width >= 420 ? 150 : 110;
        sideX = s.width - sideW - 6;
        int[] fit = MapWidget.fit(sideX - 12, bottom - top - 12);
        mapX = 6 + (sideX - 12 - fit[0]) / 2;
        mapY = top;
        mapW = fit[0];
        mapH = fit[1];
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
        tick();
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
        if (!uploaded && model.map().isPresent()) {
            ReviewModel.MapImage m = model.map().get();
            texture.upload(m.abgr(), m.w(), m.h());
            uploaded = true;
        }
        accept.active = model.ready() && screen.model().canAccept();
        accept.setTooltip(net.minecraft.client.gui.components.Tooltip.create(Lang.tr("gen.play.hint")));
    }

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        Font font = screen.font();
        int y = mapY;
        int x = sideX;
        if (!model.ready()) {
            g.drawCenteredString(font, Lang.tr("review.computing"), mapX + mapW / 2, mapY + mapH / 2, 0xffa0a0a0);
            return;
        }
        model.error().ifPresent(e -> g.drawString(font, font.plainSubstrByWidth(e, sideW), sideX, mapY + mapH + 4, 0xffff7070, true));
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

        // Summary.
        List<Component> lines = new ArrayList<>();
        model.summary().ifPresent(sm -> {
            lines.add(Lang.tr("review.size", sm.circumference()));
            lines.add(Lang.tr("review.counts", sm.events(), sm.sites(), sm.keyframes()));
            lines.add(Lang.tr("review.source", sm.source()));
        });
        lines.add(model.spawn().map(p -> (Component) Lang.tr("review.spawn", String.format(java.util.Locale.ROOT, "%.1f", p[0]),
                String.format(java.util.Locale.ROOT, "%.1f", p[1]))).orElse(Lang.tr("review.spawn.none")));
        lines.add(Component.empty());
        Map<String, Integer> byType = new LinkedHashMap<>();
        for (ProvinceHints.Cell c : model.provinces())
            byType.merge(c.type(), 1, Integer::sum);
        lines.add(byType.isEmpty() ? Lang.tr("review.hints.none") : Lang.tr("review.hints.title"));
        for (var e : byType.entrySet())
            lines.add(Component.literal("■ ").withColor(ProvinceHints.colour(e.getKey()))
                    .append(Component.literal(Formats.humanize(e.getKey()) + " (" + e.getValue() + ")").withColor(0xe0e0e0)));
        for (Component c : lines) {
            for (var seq : font.split(c, sideW)) {
                g.drawString(font, seq, x, y, 0xffe0e0e0, false);
                y += 10;
            }
        }
        if (screen.model().accepted().map(a -> a.stagingId().equals(screen.model().run().map(r -> r.stagingId()).orElse(""))).orElse(false))
            g.drawString(font, Lang.tr("review.already_accepted"), x, y + 4, 0xff80ff80, false);
    }

    @Override
    public Component title() {
        return Lang.tr("review.title");
    }

    @Override
    public void close() {
        texture.close();
    }
}
