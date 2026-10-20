// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.screen.model.CustomizeTopics;
import com.minecraftindustrialcomplex.deeptime.screen.model.CustomizeTopics.Topic;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorModel;
import com.minecraftindustrialcomplex.deeptime.screen.model.Estimate;
import com.minecraftindustrialcomplex.deeptime.screen.model.Formats;
import com.minecraftindustrialcomplex.deeptime.screen.model.Phrase;
import com.minecraftindustrialcomplex.deeptime.screen.model.Problem;
import net.minecraft.ChatFormatting;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.Font;
import net.minecraft.client.gui.GuiGraphics;
import net.minecraft.client.gui.components.AbstractWidget;
import net.minecraft.client.gui.components.Button;
import net.minecraft.client.gui.components.ContainerObjectSelectionList;
import net.minecraft.client.gui.components.CycleButton;
import net.minecraft.client.gui.components.EditBox;
import net.minecraft.client.gui.components.Tooltip;
import net.minecraft.client.gui.components.events.GuiEventListener;
import net.minecraft.client.gui.narration.NarratableEntry;
import net.minecraft.network.chat.CommonComponents;
import net.minecraft.network.chat.Component;
import net.minecraft.network.chat.MutableComponent;

import java.util.ArrayList;
import java.util.EnumMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.BooleanSupplier;

/**
 * The Customize page (docs/design/create-world-ux.md, "Customize"): opt-in, for players who want to
 * shape the planet themselves. Six topic tabs over the {@link EditorFields} catalogue (the same
 * parameters as the browser workbench, sorted by {@link CustomizeTopics}); every setting is one row
 * with its name, what it does in one line, its range and unit, and a reset to the preset's value, and
 * a changed setting is marked. Above the list, the preset it starts from and how many changes were
 * made; below it, the live estimate (time and memory, with a warning when the run is slow or does not
 * fit the memory cap), share codes out and in, Back, and Generate. Advanced holds the memory cap, the
 * Phase 3 placeholders and a search over every setting, the only place the preset paths show.
 */
final class CustomizePage implements PlanetEditorScreen.Page {

    static final int ROW = 34, GAP = 8;
    static final int TEXT = 0xffe8e8e8, DIM = 0xff9aa3ad, FAINT = 0xff6f7883, ACCENT = 0xffffd060, ERROR = 0xffff7070,
            GOOD = 0xff80e080;

    /** Remembered across rebuilds: the open topic, each topic's scroll, the search. */
    private static Topic topic = Topic.PLANET;
    private static final Map<Topic, Double> SCROLL = new EnumMap<>(Topic.class);
    private static String query = "";

    /** The dev screenshot driver's switches. */
    static void setTopic(Topic t) {
        topic = t;
    }

    static void setQuery(String q) {
        query = q == null ? "" : q;
    }

    private PlanetEditorScreen screen;
    private EditorModel model;
    private Rows list;
    private Button generate, resetAll;
    private EditBox search;
    private Component status = Component.empty();
    private long statusUntil;
    private int listTop, footerY;

    @Override
    public Component title() {
        return Lang.tr("customize.title");
    }

    @Override
    public void init(PlanetEditorScreen s) {
        screen = s;
        model = s.model();
        int w = s.width, h = s.height;

        // The topic tabs.
        Topic[] topics = Topic.values();
        int tabW = Math.min(110, (w - 2 * GAP - 2 * (topics.length - 1)) / topics.length);
        int tabsX = (w - (tabW * topics.length + 2 * (topics.length - 1))) / 2;
        for (int i = 0; i < topics.length; i++) {
            Topic t = topics[i];
            Button b = s.add(Button.builder(Lang.tr(t.key()), x -> {
                remember();
                topic = t;
                s.refresh();
            }).bounds(tabsX + i * (tabW + 2), 18, tabW, 20).build());
            b.setTooltip(Tooltip.create(Lang.tr(t.key() + ".hint")));
            b.active = t != topic;
        }

        // The starting preset and the changes.
        int barY = 42;
        List<String> presets = new ArrayList<>(model.families());
        if (model.isImported())
            presets.add("");
        int presetW = Math.min(200, w / 3);
        CycleButton<String> preset = s.add(CycleButton.<String>builder(f -> f.isEmpty()
                        ? Component.literal(model.importedName()) : Lang.family(f))
                .withValues(presets).withInitialValue(model.family().orElse(""))
                .create(GAP, barY, presetW, 20, Lang.tr("customize.start_from"), (b, v) -> {
                    if (!v.isEmpty()) {
                        model.selectFamily(v);
                        s.refresh();
                    }
                }));
        preset.setTooltip(Tooltip.create(Lang.tr("customize.start_from.hint")));
        int resetW = Math.min(90, w / 6);
        resetAll = s.add(Button.builder(Lang.tr("customize.reset_all"), b -> {
            model.resetAll();
            s.refresh();
        }).bounds(w - GAP - resetW, barY, resetW, 20).build());
        resetAll.setTooltip(Tooltip.create(Lang.tr("customize.reset_all.hint")));

        // Advanced: the search box above the list (outside it, so typing keeps the focus).
        listTop = barY + 24;
        search = null;
        if (topic == Topic.ADVANCED) {
            search = s.add(new EditBox(s.font(), GAP, listTop, w - 2 * GAP, 18, Lang.tr("customize.search")));
            search.setHint(Lang.tr("customize.search.hint", EditorFields.ALL.size()));
            search.setMaxLength(80);
            search.setValue(query);
            search.setResponder(q -> {
                query = q;
                fillList();
            });
            listTop += 22;
        }

        footerY = h - 50;
        list = s.add(new Rows(Minecraft.getInstance(), w, Math.max(ROW, footerY - 2 - listTop), listTop));
        fillList();
        list.setScrollAmount(SCROLL.getOrDefault(topic, 0.0));

        // The actions.
        int by = h - 24;
        int bw = Math.min(100, (w - 2 * GAP - 3 * 4) / 4);
        Button back = s.add(Button.builder(model.customizeFromRun() ? Lang.tr("customize.back.run") : CommonComponents.GUI_BACK, b -> back())
                .bounds(GAP, by, bw, 20).build());
        back.setTooltip(Tooltip.create(Lang.tr(model.customizeFromRun() ? "customize.back.run.hint" : "customize.back.hint")));
        Button copy = s.add(Button.builder(Lang.tr("customize.copy"), b -> {
            Minecraft.getInstance().keyboardHandler.setClipboard(model.exportShareCode());
            say(Lang.tr("customize.copied"));
        }).bounds(GAP + bw + 4, by, bw, 20).build());
        copy.setTooltip(Tooltip.create(Lang.tr("customize.copy.hint")));
        Button paste = s.add(Button.builder(Lang.tr("customize.paste"), b -> paste()).bounds(GAP + 2 * (bw + 4), by, bw, 20).build());
        paste.setTooltip(Tooltip.create(Lang.tr("customize.paste.hint")));
        int gw = Math.min(120, bw + 20);
        generate = s.add(Button.builder(Lang.tr("customize.generate").withStyle(ChatFormatting.BOLD), b -> {
            remember();
            model.generate();
            s.refresh();
        }).bounds(w - GAP - gw, by, gw, 20).build());
        tick();
    }

    private void back() {
        remember();
        if (model.customizeFromRun()) {
            model.showRun();
            screen.refresh();
        } else {
            screen.onClose();
        }
    }

    private void paste() {
        String code = Minecraft.getInstance().keyboardHandler.getClipboard().trim();
        Optional<Problem> p = code.isEmpty() ? Optional.of(Problem.of("customize.paste.empty")) : model.importShareCode(code);
        if (p.isPresent()) {
            say(Lang.problem(p.get()).withStyle(ChatFormatting.RED));
        } else {
            say(Lang.tr("customize.pasted"));
            remember();
            screen.refresh();
        }
    }

    private void say(Component c) {
        status = c;
        statusUntil = System.currentTimeMillis() + 6000;
    }

    private void remember() {
        if (list != null)
            SCROLL.put(topic, list.getScrollAmount());
    }

    // ------------------------------------------------------------------------------------------
    // The rows
    // ------------------------------------------------------------------------------------------

    private void fillList() {
        list.clear();
        Font font = screen.font();
        switch (topic) {
            case PLANET -> {
                list.heading(Lang.tr("customize.section.world"));
                worldRows(font);
                fieldRows(CustomizeTopics.fields(Topic.PLANET), font, false);
            }
            case ADVANCED -> advancedRows(font);
            default -> fieldRows(CustomizeTopics.fields(topic), font, false);
        }
    }

    /** Size and timeline: every preset has them, so they are not catalogue fields. */
    private void worldRows(Font font) {
        List<Integer> sizes = model.sizes().stream().filter(model::sizeSelectable).toList();
        CycleButton<Integer> size = CycleButton.<Integer>builder(c -> Lang.tr("editor.size." + (c == 16_384 ? "16k" : c == 32_768 ? "32k" : "other"), c))
                .withValues(sizes).withInitialValue(model.circumference()).displayOnlyValue()
                .create(0, 0, list.widgetW(), 20, Lang.tr("editor.size"), (b, v) -> {
                    model.selectSize(v);
                    refresh();
                });
        size.active = sizes.size() > 1;
        list.row(Lang.tr("editor.size"), size, () -> false, null, Lang.tr("customize.size.line"),
                Lang.tr("customize.range.choices", sizes.size()), Lang.tr("editor.size.tooltip"));

        List<String> starts = new ArrayList<>();
        starts.add("");
        starts.addAll(Presets.START_CONFIGS);
        CycleButton<String> start = CycleButton.<String>builder(v -> v.isEmpty()
                        ? Lang.tr("editor.timeline.start.none") : Lang.dynamic("deeptime.editor.timeline.start." + v, v))
                .withValues(starts).withInitialValue(model.start().orElse("")).displayOnlyValue()
                .create(0, 0, list.widgetW(), 20, Lang.tr("editor.timeline.start"), (b, v) -> model.setStart(v.isEmpty() ? null : v));
        list.row(Lang.tr("editor.timeline.start"), start, model::startEdited, () -> {
                    model.resetStart();
                    refresh();
                }, Lang.tr("customize.start.line"), Lang.tr("customize.range.choices", Presets.START_CONFIGS.size()),
                Lang.tr("editor.timeline.tooltip"));

        EditBox length = new EditBox(font, 0, 0, list.widgetW(), 20, Lang.tr("customize.length"));
        length.setValue(model.lengthText());
        length.setResponder(t -> {
            model.setLengthText(t);
            double v = Formats.parse(t);
            length.setTextColor(v >= Presets.MIN_LENGTH_MYR && v <= Presets.MAX_LENGTH_MYR ? 0xe0e0e0 : 0xff5555);
        });
        list.row(Lang.tr("customize.length"), length, model::lengthEdited, () -> {
                    model.resetLength();
                    refresh();
                }, Lang.tr("customize.length.line"),
                Lang.tr("customize.length.range", Formats.number(Presets.MIN_LENGTH_MYR), Formats.number(Presets.MAX_LENGTH_MYR)),
                Lang.tr("customize.length.hint", Formats.number(EditorModel.DEFAULT_LENGTH_MYR)));
    }

    /** Advanced: the memory cap, the Phase 3 placeholders, then the search results with their preset paths. */
    private void advancedRows(Font font) {
        List<EditorFields.Field> hits = CustomizeTopics.search(query, f -> label(f).getString());
        if (!query.isBlank()) {
            list.heading(hits.isEmpty() ? Lang.tr("customize.search.none")
                    : hits.size() > CustomizeTopics.MAX_RESULTS
                    ? Lang.tr("customize.search.more", CustomizeTopics.MAX_RESULTS, hits.size())
                    : Lang.tr("customize.search.count", hits.size()));
            fieldRows(hits.subList(0, Math.min(hits.size(), CustomizeTopics.MAX_RESULTS)), font, true);
            return;
        }
        list.heading(Lang.tr("editor.section.memory"));
        EditBox cap = new EditBox(font, 0, 0, list.widgetW(), 20, Lang.tr("customize.cap"));
        cap.setValue(model.capText());
        cap.setResponder(t -> {
            model.setCapText(t);
            cap.setTextColor(model.capValid() ? 0xe0e0e0 : 0xff5555);
        });
        list.row(Lang.tr("customize.cap"), cap, model::capEdited, () -> {
                    model.resetCap();
                    refresh();
                }, Lang.tr("customize.cap.line"), Lang.tr("customize.cap.range", Formats.number(EditorModel.MIN_CAP_GIB)),
                Lang.tr("editor.cap.tooltip"));
        fieldRows(CustomizeTopics.fields(Topic.ADVANCED), font, false);
        list.heading(Lang.tr("customize.search.title"));
        list.note(Lang.tr("customize.search.note", EditorFields.ALL.size()));
    }

    private void fieldRows(List<EditorFields.Field> fields, Font font, boolean paths) {
        String heading = null;
        for (EditorFields.Field f : fields) {
            String h = CustomizeTopics.heading(f);
            if (!paths && !h.equals(heading)) {
                heading = h;
                list.heading(Lang.tr(h));
            }
            boolean on = model.available(f);
            Component line = f.phase3() ? Lang.tr("customize.phase3")
                    : !on ? Lang.tr("customize.unavailable")
                    : paths ? Component.literal(f.path())
                    : Component.literal(f.hint());
            list.row(label(f), widget(f, font), () -> model.edited(f), on ? () -> {
                model.resetField(f);
                refresh();
            } : null, line, Lang.phrase(CustomizeTopics.range(f)), tooltip(f, paths));
        }
    }

    private static MutableComponent label(EditorFields.Field f) {
        return Component.translatable("deeptime." + f.labelKey());
    }

    /** The row's tooltip: what it does in full, its range, the preset's value (and, in Advanced, its path). */
    private Component tooltip(EditorFields.Field f, boolean paths) {
        MutableComponent t = Component.literal(f.hint().isEmpty() ? "" : f.hint());
        t.append("\n").append(Lang.phrase(CustomizeTopics.range(f)).withStyle(ChatFormatting.GRAY));
        model.presetText(f).ifPresent(v -> t.append("\n").append(Lang.tr("customize.preset_value", optionText(f, v)).withStyle(ChatFormatting.GRAY)));
        if (paths)
            t.append("\n").append(Component.literal(f.path()).withStyle(ChatFormatting.DARK_GRAY));
        return t;
    }

    /** A choice's option as the player reads it (a fixed option has a language key; others show as they are). */
    private static Component optionText(EditorFields.Field f, String value) {
        if (f.kind() == EditorFields.Kind.TOGGLE)
            return "true".equals(value) ? CommonComponents.OPTION_ON : CommonComponents.OPTION_OFF;
        return f.options().contains(value) ? Component.translatable("deeptime." + f.optionKey(value)) : Component.literal(value);
    }

    private AbstractWidget widget(EditorFields.Field f, Font font) {
        boolean on = model.available(f);
        int ww = list.widgetW();
        AbstractWidget w;
        switch (f.kind()) {
            case NUMBER -> {
                EditBox box = new EditBox(font, 0, 0, ww, 20, label(f));
                box.setMaxLength(40);
                box.setValue(model.fieldText(f));
                box.setTextColor(model.fieldValid(f) ? 0xe0e0e0 : 0xff5555);
                box.setResponder(t -> {
                    model.setFieldText(f, t);
                    box.setTextColor(model.fieldValid(f) ? 0xe0e0e0 : 0xff5555);
                });
                box.setEditable(on);
                w = box;
            }
            case TOGGLE -> w = CycleButton.onOffBuilder(model.toggle(f)).displayOnlyValue()
                    .create(0, 0, ww, 20, label(f), (b, v) -> {
                        model.setToggle(f, v);
                        refresh(); // a switch can enable or grey out another setting (steering)
                    });
            default -> {
                List<String> choices = model.choices(f);
                if (choices.isEmpty()) {
                    w = Button.builder(Lang.tr("editor.field.none"), b -> {}).width(ww).build();
                    on = false;
                } else {
                    String current = model.choice(f).filter(choices::contains).orElse(choices.get(0));
                    w = CycleButton.<String>builder(v -> optionText(f, v)).withValues(choices).withInitialValue(current)
                            .displayOnlyValue().create(0, 0, ww, 20, label(f), (b, v) -> {
                                model.setChoice(f, v);
                                refresh();
                            });
                }
            }
        }
        w.active = on;
        return w;
    }

    /** Rebuilds the page (a reset or a switch changed what other rows show), keeping the scroll. */
    private void refresh() {
        remember();
        screen.refresh();
    }

    // ------------------------------------------------------------------------------------------
    // Tick and footer
    // ------------------------------------------------------------------------------------------

    @Override
    public void tick() {
        if (list != null)
            SCROLL.put(topic, list.getScrollAmount());
        if (resetAll != null)
            resetAll.active = model.editCount() > 0 || model.startEdited() || model.lengthEdited() || model.capEdited();
        if (generate != null) {
            Optional<Problem> blocking = blocking();
            generate.active = blocking.isEmpty();
            generate.setTooltip(Tooltip.create(blocking.map(p -> (Component) Lang.problem(p))
                    .orElse(Lang.tr(model.runIsCurrent() ? "customize.generate.current" : "customize.generate.hint"))));
        }
    }

    /** The first problem that keeps Generate off (waiting for the estimate does not: Generate waits for it). */
    private Optional<Problem> blocking() {
        return model.problems().stream().filter(Problem::blocking).filter(p -> !p.key().equals("editor.problem.estimating")).findFirst();
    }

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        Font font = screen.font();
        int w = screen.width;
        // The changes, beside the preset button.
        int presetRight = GAP + Math.min(200, w / 3) + 8;
        int resetX = w - GAP - Math.min(90, w / 6) - 8;
        int n = model.editCount();
        Component changes = n == 0 ? Lang.tr("customize.changes.none") : n == 1 ? Lang.tr("customize.changes.one") : Lang.tr("customize.changes", n);
        g.drawString(font, fit(font, changes, resetX - presetRight), presetRight, 48, n == 0 ? DIM : ACCENT, false);
        // Under the active tab, a bar.
        Topic[] topics = Topic.values();
        int tabW = Math.min(110, (w - 2 * GAP - 2 * (topics.length - 1)) / topics.length);
        int tabsX = (w - (tabW * topics.length + 2 * (topics.length - 1))) / 2;
        int tx = tabsX + topic.ordinal() * (tabW + 2);
        g.fill(tx + 2, 38, tx + tabW - 2, 40, ACCENT);

        // The estimate, then the first thing to know.
        int x = GAP, lw = w - 2 * GAP, y = footerY + 2;
        Optional<Estimate> est = model.estimate();
        Component estimate = est.map(e -> (Component) Lang.tr("customize.estimate", Lang.phrase(Phrase.duration(e.seconds())),
                        Formats.bytes(e.peakUnder(model.capBytes())), Formats.gib(model.capBytes())))
                .orElse(model.estimateError().isPresent() ? Lang.tr("editor.estimate.error") : Lang.tr("editor.estimate.pending"));
        g.drawString(font, fit(font, estimate, lw), x, y, TEXT, false);
        List<Problem> ps = model.problems();
        Optional<Problem> stop = blocking();
        Optional<Problem> warn = ps.stream().filter(p -> !p.blocking()).findFirst();
        Component line;
        int colour;
        if (System.currentTimeMillis() < statusUntil) {
            line = status;
            colour = 0xff80d0ff;
        } else if (stop.isPresent()) {
            line = Lang.problem(stop.get());
            colour = ERROR;
        } else if (est.isPresent() && CustomizeTopics.slow(est.get())) {
            line = Lang.tr("customize.slow", Lang.phrase(Phrase.duration(est.get().seconds())));
            colour = ACCENT;
        } else if (warn.isPresent()) {
            line = Lang.problem(warn.get());
            colour = ACCENT;
        } else if (model.runIsCurrent()) {
            line = Lang.tr("customize.current");
            colour = GOOD;
        } else {
            line = Lang.tr("customize.ready");
            colour = GOOD;
        }
        g.drawString(font, fit(font, line, lw), x, y + 11, colour, false);
    }

    /** {@code c} cut to {@code width} pixels, with an ellipsis when it was cut. */
    static String fit(Font font, Component c, int width) {
        String s = c.getString();
        if (width <= 0)
            return "";
        if (font.width(s) <= width)
            return s;
        return font.plainSubstrByWidth(s, Math.max(0, width - font.width("…"))) + "…";
    }

    // ------------------------------------------------------------------------------------------
    // The list
    // ------------------------------------------------------------------------------------------

    /** The settings list: headings, notes and setting rows of two lines each. */
    static final class Rows extends ContainerObjectSelectionList<Rows.Entry> {

        Rows(Minecraft mc, int width, int height, int y) {
            super(mc, width, height, y, ROW);
        }

        @Override
        public int getRowWidth() {
            return Math.min(560, width - 28);
        }

        /** The width of a setting's control. */
        int widgetW() {
            return Math.max(80, Math.min(130, getRowWidth() / 4));
        }

        void clear() {
            clearEntries();
            setScrollAmount(0);
        }

        void heading(Component text) {
            addEntry(new Heading(text));
        }

        void note(Component text) {
            addEntry(new Note(text));
        }

        void row(Component label, AbstractWidget widget, BooleanSupplier edited, Runnable reset, Component line, Component range,
                Component tooltip) {
            Button r = Button.builder(Component.literal("↺"), b -> {
                if (reset != null)
                    reset.run();
            }).width(20).build();
            r.setTooltip(Tooltip.create(Lang.tr("customize.reset")));
            if (tooltip != null && !tooltip.getString().isBlank())
                widget.setTooltip(Tooltip.create(tooltip));
            addEntry(new Setting(label, widget, r, reset != null ? edited : () -> false, line, range, tooltip, widgetW()));
        }

        abstract static class Entry extends ContainerObjectSelectionList.Entry<Entry> {}

        static final class Heading extends Entry {
            private final Component text;

            Heading(Component text) {
                this.text = text;
            }

            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                Font font = Minecraft.getInstance().font;
                int y = top + height - 14;
                g.drawString(font, fit(font, text, width), left, y, ACCENT, false);
                g.fill(left, y + 10, left + width, y + 11, 0x40ffffff);
            }

            @Override
            public List<? extends GuiEventListener> children() {
                return List.of();
            }

            @Override
            public List<? extends NarratableEntry> narratables() {
                return List.of();
            }
        }

        static final class Note extends Entry {
            private final Component text;

            Note(Component text) {
                this.text = text;
            }

            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                Font font = Minecraft.getInstance().font;
                int y = top + 3;
                for (var l : font.split(text, width).stream().limit(3).toList()) {
                    g.drawString(font, l, left, y, DIM, false);
                    y += 10;
                }
            }

            @Override
            public List<? extends GuiEventListener> children() {
                return List.of();
            }

            @Override
            public List<? extends NarratableEntry> narratables() {
                return List.of();
            }
        }

        /** One setting: name and control on the first line, what it does and its range on the second. */
        static final class Setting extends Entry {
            private final Component label, line, range, tooltip;
            private final AbstractWidget widget;
            private final Button reset;
            private final BooleanSupplier edited;
            private final int widgetW;

            Setting(Component label, AbstractWidget widget, Button reset, BooleanSupplier edited, Component line, Component range,
                    Component tooltip, int widgetW) {
                this.label = label;
                this.widget = widget;
                this.reset = reset;
                this.edited = edited;
                this.line = line;
                this.range = range;
                this.tooltip = tooltip;
                this.widgetW = widgetW;
            }

            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                Font font = Minecraft.getInstance().font;
                boolean ed = edited.getAsBoolean();
                int rx = left + width - 20;
                int wx = rx - 4 - widgetW;
                int textW = wx - left - 12;
                if (ed)
                    g.fill(left, top + 1, left + 2, top + height - 4, ACCENT);
                g.drawString(font, fit(font, label, textW), left + 6, top + 6, ed ? ACCENT : TEXT, false);
                g.drawString(font, fit(font, line, textW), left + 6, top + 20, DIM, false);
                String r = fit(font, range, widgetW + 24);
                g.drawString(font, r, left + width - font.width(r), top + 20, FAINT, false);
                widget.setX(wx);
                widget.setY(top);
                widget.render(g, mouseX, mouseY, partialTick);
                reset.setX(rx);
                reset.setY(top);
                reset.active = ed;
                reset.render(g, mouseX, mouseY, partialTick);
                if (hovering && mouseX < wx && tooltip != null && !tooltip.getString().isBlank()
                        && Minecraft.getInstance().screen != null)
                    Minecraft.getInstance().screen.setTooltipForNextRenderPass(font.split(tooltip, Math.max(160, width / 2)));
            }

            @Override
            public List<? extends GuiEventListener> children() {
                return List.of(widget, reset);
            }

            @Override
            public List<? extends NarratableEntry> narratables() {
                return List.of(widget, reset);
            }
        }
    }
}
