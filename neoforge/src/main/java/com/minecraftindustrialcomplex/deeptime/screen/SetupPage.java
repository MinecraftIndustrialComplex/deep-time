// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen;

import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorFields;
import com.minecraftindustrialcomplex.deeptime.screen.model.EditorModel;
import com.minecraftindustrialcomplex.deeptime.screen.model.Estimate;
import com.minecraftindustrialcomplex.deeptime.screen.model.Formats;
import com.minecraftindustrialcomplex.deeptime.screen.model.MachineInfo;
import com.minecraftindustrialcomplex.deeptime.screen.model.Problem;
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

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * The setup page (plan §7.12 page 1): a scrolling list of the preset, size, timeline, share code,
 * memory cap and the {@link EditorFields} parameters, with the live estimate, the machine's memory,
 * the first problem and Run in the footer. Every value goes straight into the {@link EditorModel}.
 */
final class SetupPage implements PlanetEditorScreen.Page {

    static final int ROW = 24, WIDGET_W = 150;

    private PlanetEditorScreen screen;
    private EditorModel model;
    private Options list;
    private Button run;
    private EditBox shareBox;
    private Component shareStatus = Component.empty();
    private static double scrollMemory;

    @Override
    public void init(PlanetEditorScreen s) {
        this.screen = s;
        this.model = s.model();
        Font font = s.font();
        int footer = 66;
        list = new Options(Minecraft.getInstance(), s.width, s.height - 20 - footer, 20);
        s.add(list);

        list.header(Lang.tr("editor.section.preset"));
        List<String> presets = new ArrayList<>(model.families());
        if (model.isImported())
            presets.add("");
        String current = model.family().orElse("");
        list.row(Lang.tr("editor.preset"), CycleButton.<String>builder(f -> f.isEmpty()
                        ? Component.literal(model.importedName()) : Lang.family(f))
                .withValues(presets).withInitialValue(current).displayOnlyValue()
                .create(0, 0, WIDGET_W, 20, Lang.tr("editor.preset"), (b, v) -> {
                    if (!v.isEmpty()) {
                        model.selectFamily(v);
                        s.refresh();
                    }
                }), model.info().description().orElse(null));
        List<Integer> sizes = model.sizes().stream().filter(model::sizeSelectable).toList();
        CycleButton<Integer> size = CycleButton.<Integer>builder(c -> Lang.tr("editor.size." + (c == 16_384 ? "16k" : c == 32_768 ? "32k" : "other"), c))
                .withValues(sizes).withInitialValue(model.circumference()).displayOnlyValue()
                .create(0, 0, WIDGET_W, 20, Lang.tr("editor.size"), (b, v) -> {
                    model.selectSize(v);
                    s.refresh();
                });
        size.active = sizes.size() > 1;
        list.row(Lang.tr("editor.size"), size, Lang.tr("editor.size.tooltip").getString());

        list.header(Lang.tr("editor.section.timeline"));
        List<String> starts = new ArrayList<>();
        starts.add("");
        starts.addAll(Presets.START_CONFIGS);
        list.row(Lang.tr("editor.timeline.start"), CycleButton.<String>builder(v -> v.isEmpty()
                        ? Lang.tr("editor.timeline.start.none") : Lang.dynamic("deeptime.editor.timeline.start." + v, v))
                .withValues(starts).withInitialValue(model.start().orElse("")).displayOnlyValue()
                .create(0, 0, WIDGET_W, 20, Lang.tr("editor.timeline.start"), (b, v) -> model.setStart(v.isEmpty() ? null : v)),
                Lang.tr("editor.timeline.tooltip").getString());
        EditBox length = new EditBox(font, 0, 0, WIDGET_W, 20, Lang.tr("editor.timeline.length"));
        length.setHint(Lang.tr("editor.timeline.length.hint", Formats.number(Presets.MIN_LENGTH_MYR), Formats.number(Presets.MAX_LENGTH_MYR)));
        length.setValue(model.lengthText());
        length.setResponder(t -> {
            model.setLengthText(t);
            double v = Formats.parse(t);
            length.setTextColor(t.isBlank() || (v >= Presets.MIN_LENGTH_MYR && v <= Presets.MAX_LENGTH_MYR) ? 0xe0e0e0 : 0xff5555);
        });
        list.row(Lang.tr("editor.timeline.length"), length, (String) null);

        list.header(Lang.tr("editor.section.share"));
        shareBox = new EditBox(font, 0, 0, WIDGET_W, 20, Lang.tr("editor.share"));
        shareBox.setMaxLength(1 << 20);
        shareBox.setHint(Lang.tr("editor.share.hint"));
        list.row(Lang.tr("editor.share"), shareBox, (String) null);
        Button importB = Button.builder(Lang.tr("editor.share.import"), b -> {
            Optional<Problem> p = model.importShareCode(shareBox.getValue());
            shareStatus = p.map(Lang::problem).map(c -> (Component) c.withStyle(net.minecraft.ChatFormatting.RED))
                    .orElse(Lang.tr("editor.share.imported"));
            if (p.isEmpty())
                s.refresh();
        }).width(WIDGET_W / 2 - 2).build();
        Button exportB = Button.builder(Lang.tr("editor.share.export"), b -> {
            String code = model.exportShareCode();
            shareBox.setValue(code);
            Minecraft.getInstance().keyboardHandler.setClipboard(code);
            shareStatus = Lang.tr("editor.share.copied");
        }).width(WIDGET_W / 2 - 2).build();
        list.pair(Component.empty(), importB, exportB);

        list.header(Lang.tr("editor.section.memory"));
        EditBox cap = new EditBox(font, 0, 0, WIDGET_W, 20, Lang.tr("editor.cap"));
        cap.setValue(model.capText());
        cap.setResponder(t -> {
            model.setCapText(t);
            cap.setTextColor(model.capValid() ? 0xe0e0e0 : 0xff5555);
        });
        list.row(Lang.tr("editor.cap"), cap, Lang.tr("editor.cap.tooltip").getString());

        EditorFields.Section section = null;
        for (EditorFields.Field f : EditorFields.ALL) {
            if (f.section() != section) {
                section = f.section();
                list.header(Lang.tr("editor.section." + sectionKey(section)));
            }
            list.row(fieldLabel(f), fieldWidget(f, font), tooltipOf(f));
        }
        list.setScrollAmount(scrollMemory);

        int by = s.height - 24;
        int bw = Math.min(100, (s.width - 40) / 3);
        run = s.add(Button.builder(Lang.tr("editor.run"), b -> {
            if (model.startRun().isPresent())
                s.refresh();
        }).bounds(s.width / 2 - bw - bw / 2 - 6, by, bw, 20).build());
        Button timelapse = s.add(Button.builder(Lang.tr("editor.timelapse"), b -> {
            model.showRun();
            s.refresh();
        }).bounds(s.width / 2 - bw / 2, by, bw, 20).build());
        timelapse.active = model.run().isPresent();
        s.add(Button.builder(CommonComponents.GUI_DONE, b -> s.onClose())
                .bounds(s.width / 2 + bw / 2 + 6, by, bw, 20).build());
        tick();
    }

    private static String sectionKey(EditorFields.Section s) {
        return switch (s) {
            case PLANET -> "planet";
            case SCALE -> "scale";
            case MULTIPLIERS -> "multipliers";
            case BIOMES -> "biomes";
            case FANTASY -> "fantasy";
            case PLAYABILITY -> "playability";
        };
    }

    private static Component fieldLabel(EditorFields.Field f) {
        Component label = Component.translatable("deeptime." + f.labelKey());
        return f.unit().isEmpty() ? label : label.copy().append(" (" + f.unit() + ")");
    }

    private String tooltipOf(EditorFields.Field f) {
        if (f.phase3())
            return Lang.tr("editor.field.phase3").getString();
        if (!model.available(f))
            return Lang.tr("editor.field.unavailable", f.path()).getString();
        return f.kind() == EditorFields.Kind.NUMBER
                ? Lang.tr("editor.field.range", Formats.number(f.min()), Formats.number(f.max()), f.path()).getString() : f.path();
    }

    private AbstractWidget fieldWidget(EditorFields.Field f, Font font) {
        boolean on = model.available(f);
        AbstractWidget w;
        switch (f.kind()) {
            case NUMBER -> {
                EditBox box = new EditBox(font, 0, 0, WIDGET_W, 20, fieldLabel(f));
                box.setValue(model.fieldText(f));
                box.setResponder(t -> {
                    model.setFieldText(f, t);
                    box.setTextColor(model.fieldValid(f) ? 0xe0e0e0 : 0xff5555);
                });
                box.setEditable(on);
                w = box;
            }
            case TOGGLE -> w = CycleButton.onOffBuilder(model.toggle(f)).displayOnlyValue()
                    .create(0, 0, WIDGET_W, 20, fieldLabel(f), (b, v) -> model.setToggle(f, v));
            default -> {
                List<String> choices = model.choices(f);
                if (choices.isEmpty()) {
                    w = Button.builder(Lang.tr("editor.field.none"), b -> {}).width(WIDGET_W).build();
                    on = false;
                } else {
                    w = CycleButton.<String>builder(Component::literal).withValues(choices).withInitialValue(choices.get(0))
                            .displayOnlyValue().create(0, 0, WIDGET_W, 20, fieldLabel(f), (b, v) -> model.setChoice(f, v));
                }
            }
        }
        w.active = on;
        return w;
    }

    @Override
    public void tick() {
        if (list != null)
            scrollMemory = list.getScrollAmount();
        if (run != null) {
            run.active = model.canRun();
            List<Problem> ps = model.problems();
            run.setTooltip(ps.stream().filter(Problem::blocking).findFirst()
                    .map(p -> Tooltip.create(Lang.problem(p))).orElse(null));
        }
    }

    @Override
    public void render(GuiGraphics g, int mouseX, int mouseY, float partialTick) {
        Font font = screen.font();
        int y = screen.height - 64;
        int x = 8, w = screen.width - 16;
        // The estimate and the machine.
        Optional<Estimate> est = model.estimate();
        MachineInfo m = model.deps().machine();
        Component estimate = est.map(e -> (Component) Lang.tr("editor.estimate", Formats.bytes(e.peakBytes()),
                        Formats.gib(model.capBytes()), Formats.seconds(e.seconds()), e.stages().size()))
                .orElse(model.estimateError().map(err -> (Component) Lang.tr("editor.estimate.error"))
                        .orElse(Lang.tr("editor.estimate.pending")));
        g.drawString(font, font.substrByWidth(estimate, w).getString(), x, y, 0xffe0e0e0, true);
        Component machine = Lang.tr("editor.machine", m.freeBytes() < 0 ? "?" : Formats.gib(m.freeBytes()),
                m.totalBytes() < 0 ? "?" : Formats.gib(m.totalBytes()), Formats.gib(m.jvmMaxBytes()), m.clientThreads());
        g.drawString(font, font.substrByWidth(machine, w).getString(), x, y + 10, 0xffa0a0a0, true);
        // The first problem (else the first warning), or the accepted run, or the share status.
        List<Problem> ps = model.problems();
        Optional<Problem> first = ps.stream().filter(Problem::blocking).findFirst().or(() -> ps.stream().findFirst());
        Component line;
        int colour;
        if (first.isPresent()) {
            line = Lang.problem(first.get());
            colour = first.get().blocking() ? 0xffff7070 : 0xffffd060;
        } else {
            line = Lang.tr("editor.ready");
            colour = 0xff80ff80;
        }
        g.drawString(font, font.substrByWidth(line, w).getString(), x, y + 20, colour, true);
        Component status = model.accepted().map(a -> (Component) Lang.tr("editor.accepted", a.label())).orElse(shareStatus);
        g.drawString(font, font.substrByWidth(status, w).getString(), x, y + 30, 0xff80d0ff, true);
    }

    // ------------------------------------------------------------------------------------------
    // The list
    // ------------------------------------------------------------------------------------------

    /** A scrolling list of label + widget rows and section headers (the game-rules screen's pattern). */
    static final class Options extends ContainerObjectSelectionList<Options.Entry> {

        Options(Minecraft mc, int width, int height, int y) {
            super(mc, width, height, y, ROW);
        }

        @Override
        public int getRowWidth() {
            return Math.min(360, width - 20);
        }

        void header(Component label) {
            addEntry(new Header(label));
        }

        void row(Component label, AbstractWidget widget, String tooltip) {
            if (tooltip != null && !tooltip.isEmpty())
                widget.setTooltip(Tooltip.create(Component.literal(tooltip)));
            addEntry(new Row(label, List.of(widget)));
        }

        void pair(Component label, AbstractWidget a, AbstractWidget b) {
            addEntry(new Row(label, List.of(a, b)));
        }

        abstract static class Entry extends ContainerObjectSelectionList.Entry<Entry> {}

        static final class Header extends Entry {
            private final Component label;

            Header(Component label) {
                this.label = label;
            }

            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                g.drawCenteredString(Minecraft.getInstance().font, label, left + width / 2, top + 7, 0xffffd060);
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

        static final class Row extends Entry {
            private final Component label;
            private final List<AbstractWidget> widgets;

            Row(Component label, List<AbstractWidget> widgets) {
                this.label = label;
                this.widgets = widgets;
            }

            @Override
            public void render(GuiGraphics g, int index, int top, int left, int width, int height, int mouseX, int mouseY,
                    boolean hovering, float partialTick) {
                Font font = Minecraft.getInstance().font;
                int ww = WIDGET_W;
                g.drawString(font, font.substrByWidth(label, width - ww - 8).getString(), left, top + 6, 0xffffffff, false);
                int x = left + width - ww;
                for (AbstractWidget w : widgets) {
                    w.setX(x);
                    w.setY(top);
                    w.render(g, mouseX, mouseY, partialTick);
                    x += w.getWidth() + 4;
                }
            }

            @Override
            public List<? extends GuiEventListener> children() {
                return widgets;
            }

            @Override
            public List<? extends NarratableEntry> narratables() {
                return widgets;
            }
        }
    }
}
