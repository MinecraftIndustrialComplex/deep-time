// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeLoader;
import com.minecraftindustrialcomplex.deeptime.nativebridge.SimSession;
import com.minecraftindustrialcomplex.deeptime.preset.PresetException;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.preset.ShareCode;
import com.minecraftindustrialcomplex.deeptime.server.Staging;
import com.minecraftindustrialcomplex.deeptime.worlddata.KeyframeInfo;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.Executor;
import java.util.function.BooleanSupplier;
import java.util.function.Supplier;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * The planet editor's state and rules (plan §7.12, §5.1), client-free so JUnit and GameTests drive
 * it without a screen. {@code PlanetEditorScreen} only draws it and forwards input.
 *
 * <h2>Setup page</h2>
 * <ul>
 *   <li>A <b>preset</b> is a built-in family ({@code earthlike_quick}, {@code earthlike_full}: the
 *       built-in names without their size suffix) at a <b>size</b> (16,384 or 32,768 blocks), or a
 *       preset imported from a share code (its size is fixed).</li>
 *   <li>The <b>timeline</b> — start configuration and length — has no default (decisions file,
 *       "Timeline"); Run stays disabled until both are chosen.</li>
 *   <li>The <b>parameters</b> of {@link EditorFields}: each is editable only when the preset carries
 *       its path, so the placeholder presets show them unavailable and the real presets need no
 *       screen change. Fantasy toggles are shown disabled (Phase 3).</li>
 *   <li>The <b>memory cap</b> (default 4 GiB, freely raisable; decisions file, "Memory") and the
 *       live <b>estimate</b> ({@link Estimate}) beside the machine's memory ({@link MachineInfo}).</li>
 *   <li><b>Share codes</b> export the effective preset (timeline included) and import any code.</li>
 * </ul>
 *
 * <h2>Run, review, accept</h2>
 * Run writes into a new staging folder ({@link Staging}); the {@link TimelapseModel} follows it.
 * Review ({@link ReviewModel}) shows the finished data; Accept returns what
 * {@code PlanetGenerators.withStaging} needs. A run replaced by another, discarded, cancelled or
 * failed has its folder deleted; the accepted one is kept until another is accepted.
 */
public final class EditorModel {

    public enum Page { SETUP, RUN, REVIEW }

    /** The native estimate ({@code SimSession.estimate}); throws on an invalid preset. */
    @FunctionalInterface
    public interface Estimator {
        String estimate(String presetJson);
    }

    /** Starts a run writing into {@code outDir} ({@link SessionRunHandle#launch} in the game). */
    @FunctionalInterface
    public interface Launcher {
        RunHandle launch(String presetJson, Path outDir, int threads, long capBytes);
    }

    /**
     * What the model works with.
     *
     * @param loader       decodes keyframes (a background thread in the game)
     * @param io           deletes folders and computes the review (a background thread in the game)
     * @param nativesLinked whether runs and estimates are possible
     * @param essentials   the essentials lists the loaded datapacks define (P1-15; none yet)
     */
    public record Deps(Estimator estimator, Launcher launcher, Path gameDir, MachineInfo machine, Executor loader,
                       Executor io, BooleanSupplier nativesLinked, Supplier<List<String>> essentials) {

        /** The game's: the natives through {@link SimSession}. */
        public static Deps game(Path gameDir, Executor loader, Executor io) {
            return game(gameDir, loader, io, () -> "");
        }

        /**
         * The game's, with the content bundle every estimate and run gets ({@code content}, asked at each call: the pack
         * placement tables of the biome mods the Create World screen's data packs register).
         */
        public static Deps game(Path gameDir, Executor loader, Executor io, Supplier<String> content) {
            return new Deps(json -> SimSession.estimate(json, content.get()),
                    (json, out, threads, cap) -> SessionRunHandle.launch(json, content.get(), out, threads, cap), gameDir,
                    MachineInfo.current(), loader, io, NativeLoader::isLinked, List::of);
        }
    }

    /** An accepted run: what the world's generator is bound to. */
    public record Accepted(String stagingId, int circumference, String presetJson, String label) {}

    public static final double DEFAULT_CAP_GIB = 4.0;
    public static final double MIN_CAP_GIB = 0.25;
    public static final int DEFAULT_CIRCUMFERENCE = 16_384;

    private static final Pattern SIZE_SUFFIX = Pattern.compile("^(.*)_(\\d+)k$");

    private final Deps deps;
    /** family → (C → built-in name), in {@link Presets#BUILTIN}'s order. */
    private final Map<String, Map<Integer, String>> families = new LinkedHashMap<>();
    private final Map<String, JsonObject> builtinJson = new LinkedHashMap<>();

    private String family;
    private int circumference;
    private JsonObject imported;          // the imported preset (timeline removed), or null
    private String importedName;
    private String start;                 // null until chosen
    private String lengthText = "";
    private String capText = Formats.number(DEFAULT_CAP_GIB);
    private final Map<String, String> fieldTexts = new LinkedHashMap<>();
    private final Map<String, JsonElement> overrides = new LinkedHashMap<>();
    private boolean advancedOpen;         // the Advanced settings, collapsed until asked for
    private String effective;             // cached effective preset JSON

    private Page page = Page.SETUP;
    private TimelapseModel run;
    private ReviewModel review;
    private Accepted accepted;
    private String launchError;

    // The estimate (set from the estimator's thread).
    private String estimatedFor;
    private String estimateInFlight;
    private Estimate estimate;
    private String estimateError;

    public EditorModel(Deps deps) {
        this.deps = deps;
        for (String name : Presets.BUILTIN) {
            Optional<String> json = Presets.builtin(name);
            if (json.isEmpty())
                continue;
            JsonObject o = Presets.parseObject(json.get());
            o.remove("timeline");
            builtinJson.put(name, o);
            Presets.info(o).circumference().ifPresent(c -> families.computeIfAbsent(familyOf(name), k -> new LinkedHashMap<>()).put(c, name));
        }
        if (families.isEmpty())
            throw new IllegalStateException("no built-in presets in the jar");
        family = families.keySet().iterator().next();
        Map<Integer, String> sizes = families.get(family);
        circumference = sizes.containsKey(DEFAULT_CIRCUMFERENCE) ? DEFAULT_CIRCUMFERENCE : sizes.keySet().iterator().next();
    }

    static String familyOf(String builtinName) {
        Matcher m = SIZE_SUFFIX.matcher(builtinName);
        return m.matches() ? m.group(1) : builtinName;
    }

    public Deps deps() {
        return deps;
    }

    private void changed() {
        effective = null;
    }

    // ------------------------------------------------------------------------------------------
    // Preset and size
    // ------------------------------------------------------------------------------------------

    /** The built-in families, in the dropdown's order. */
    public List<String> families() {
        return List.copyOf(families.keySet());
    }

    /** The selected family, or empty for an imported preset. */
    public Optional<String> family() {
        return imported == null ? Optional.of(family) : Optional.empty();
    }

    public boolean isImported() {
        return imported != null;
    }

    public String importedName() {
        return importedName;
    }

    public void selectFamily(String f) {
        Map<Integer, String> sizes = families.get(f);
        if (sizes == null)
            throw new IllegalArgumentException("no preset family '" + f + "'");
        family = f;
        imported = null;
        importedName = null;
        if (!sizes.containsKey(circumference))
            circumference = sizes.containsKey(DEFAULT_CIRCUMFERENCE) ? DEFAULT_CIRCUMFERENCE : sizes.keySet().iterator().next();
        changed();
    }

    /** Every size a built-in preset exists for, ascending. */
    public List<Integer> sizes() {
        Set<Integer> all = new java.util.TreeSet<>();
        for (Map<Integer, String> m : families.values())
            all.addAll(m.keySet());
        if (imported != null)
            all.add(circumference);
        return List.copyOf(all);
    }

    public int circumference() {
        return circumference;
    }

    public boolean sizeSelectable(int c) {
        return imported == null ? families.get(family).containsKey(c) : c == circumference;
    }

    public void selectSize(int c) {
        if (!sizeSelectable(c))
            throw new IllegalArgumentException("no " + (imported == null ? family : importedName) + " preset for C = " + c);
        circumference = c;
        changed();
    }

    /** The selected preset without timeline and edits. */
    JsonObject base() {
        return imported != null ? imported : builtinJson.get(families.get(family).get(circumference));
    }

    /** The built-in preset's name, or the imported preset's. */
    public String presetName() {
        return imported != null ? importedName : families.get(family).get(circumference);
    }

    public Presets.Info info() {
        return Presets.info(base());
    }

    // ------------------------------------------------------------------------------------------
    // Timeline
    // ------------------------------------------------------------------------------------------

    public Optional<String> start() {
        return Optional.ofNullable(start);
    }

    /** Chooses the start configuration ({@link Presets#START_CONFIGS}); null clears it. */
    public void setStart(String s) {
        if (s != null && !Presets.START_CONFIGS.contains(s))
            throw new IllegalArgumentException("unknown start configuration '" + s + "'");
        start = s;
        changed();
    }

    public String lengthText() {
        return lengthText;
    }

    public void setLengthText(String t) {
        lengthText = t == null ? "" : t;
        changed();
    }

    /** The timeline when both parts are valid. */
    public Optional<Presets.Timeline> timeline() {
        double len = Formats.parse(lengthText);
        Presets.Timeline t = new Presets.Timeline(start, len);
        return start != null && t.problems("timeline").isEmpty() ? Optional.of(t) : Optional.empty();
    }

    // ------------------------------------------------------------------------------------------
    // Parameters
    // ------------------------------------------------------------------------------------------

    /** True when the field can be edited in the selected preset. */
    public boolean available(EditorFields.Field f) {
        return f.availableIn(base());
    }

    /** The field's text: as typed, else the effective value. */
    public String fieldText(EditorFields.Field f) {
        String typed = fieldTexts.get(f.id());
        if (typed != null)
            return typed;
        Optional<JsonElement> v = value(f);
        if (v.isEmpty())
            return "";
        JsonElement e = v.get();
        return e.isJsonPrimitive() && e.getAsJsonPrimitive().isNumber() ? Formats.number(e.getAsDouble()) : e.isJsonPrimitive() ? e.getAsString() : e.toString();
    }

    /** The effective value: the player's edit, else the preset's. */
    public Optional<JsonElement> value(EditorFields.Field f) {
        JsonElement o = overrides.get(f.id());
        return o != null ? Optional.of(o) : f.get(base());
    }

    /** Sets a number field from typed text; an invalid text is kept (shown red) and blocks Run. */
    public void setFieldText(EditorFields.Field f, String text) {
        if (f.kind() != EditorFields.Kind.NUMBER || !available(f))
            return;
        fieldTexts.put(f.id(), text);
        double v = Formats.parse(text);
        if (f.inRange(v))
            overrides.put(f.id(), f.number(v));
        else
            overrides.remove(f.id());
        changed();
    }

    /** False for a number field whose typed text is not a number in range. */
    public boolean fieldValid(EditorFields.Field f) {
        String typed = fieldTexts.get(f.id());
        return typed == null || !available(f) || f.inRange(Formats.parse(typed));
    }

    public boolean toggle(EditorFields.Field f) {
        return value(f).filter(JsonElement::isJsonPrimitive).map(e -> e.getAsJsonPrimitive().isBoolean() && e.getAsBoolean()).orElse(false);
    }

    public void setToggle(EditorFields.Field f, boolean on) {
        if (f.kind() != EditorFields.Kind.TOGGLE || !available(f))
            return;
        overrides.put(f.id(), new JsonPrimitive(on));
        changed();
    }

    /**
     * The choices of a choice field: its fixed options (the catalogue's {@code options}), else the
     * essentials lists; the current value is added when it is none of them.
     */
    public List<String> choices(EditorFields.Field f) {
        Set<String> out = new LinkedHashSet<>(f.options());
        if ("essentials".equals(f.id()))
            out.addAll(deps.essentials().get());
        value(f).filter(JsonElement::isJsonPrimitive).map(JsonElement::getAsString).ifPresent(out::add);
        return List.copyOf(out);
    }

    /** The current choice of a choice field (its preset value unless the player picked another). */
    public Optional<String> choice(EditorFields.Field f) {
        return value(f).filter(JsonElement::isJsonPrimitive).map(JsonElement::getAsString);
    }

    /** Picks a choice; a value outside a fixed option list is ignored. */
    public void setChoice(EditorFields.Field f, String id) {
        if (f.kind() != EditorFields.Kind.CHOICE || !available(f))
            return;
        if (!f.options().isEmpty() && !f.options().contains(id))
            return;
        overrides.put(f.id(), new JsonPrimitive(id));
        changed();
    }

    // ------------------------------------------------------------------------------------------
    // The Advanced settings (settings audit)
    // ------------------------------------------------------------------------------------------

    /** Whether the setup page shows the advanced sections (collapsed by default). */
    public boolean advancedOpen() {
        return advancedOpen;
    }

    public void setAdvancedOpen(boolean open) {
        advancedOpen = open;
    }

    /** How many advanced settings the selected preset carries (the others show as unavailable). */
    public int advancedAvailable() {
        int n = 0;
        for (EditorFields.Field f : EditorFields.advancedFields())
            if (available(f))
                n++;
        return n;
    }

    /** How many advanced settings the player has edited (a valid value typed or picked). */
    public int advancedEdited() {
        int n = 0;
        for (EditorFields.Field f : EditorFields.advancedFields())
            if (overrides.containsKey(f.id()))
                n++;
        return n;
    }

    /** Drops every edit of the parameters (not the timeline or the cap). */
    public void resetFields() {
        overrides.clear();
        fieldTexts.clear();
        changed();
    }

    // ------------------------------------------------------------------------------------------
    // Memory cap
    // ------------------------------------------------------------------------------------------

    public String capText() {
        return capText;
    }

    public void setCapText(String t) {
        capText = t == null ? "" : t;
    }

    public boolean capValid() {
        double v = Formats.parse(capText);
        return v >= MIN_CAP_GIB;
    }

    /** The cap in bytes (the default when the text is invalid; Run is blocked then anyway). */
    public long capBytes() {
        return Formats.gibToBytes(capValid() ? Formats.parse(capText) : DEFAULT_CAP_GIB);
    }

    // ------------------------------------------------------------------------------------------
    // The effective preset and share codes
    // ------------------------------------------------------------------------------------------

    /**
     * The preset a run would use, as canonical JSON: the selected preset, the player's edits of
     * available fields, and the timeline once it is valid.
     */
    public String effectivePresetJson() {
        String e = effective;
        if (e == null) {
            JsonObject p = base().deepCopy();
            for (EditorFields.Field f : EditorFields.ALL) {
                JsonElement v = overrides.get(f.id());
                if (v != null && f.availableIn(base()))
                    EditorFields.set(p, f.path(), v);
            }
            Optional<Presets.Timeline> t = timeline();
            e = t.isPresent() ? Presets.withTimeline(p, t.get()) : Presets.canonical(p);
            effective = e;
        }
        return e;
    }

    /** The share code of the effective preset ({@code DT1.…}). */
    public String exportShareCode() {
        return ShareCode.encode(effectivePresetJson());
    }

    /**
     * Imports a share code: its preset replaces the selection (a code of a built-in preset selects
     * that preset), its timeline fills the timeline, and edits are dropped. Returns the problem
     * when the code cannot be used; the selection is then unchanged.
     */
    public Optional<Problem> importShareCode(String code) {
        JsonObject o;
        try {
            o = Presets.parseObject(ShareCode.decode(code));
        } catch (PresetException e) {
            return Optional.of(Problem.of("editor.share.invalid", e.getMessage()));
        }
        Presets.Info info = Presets.info(o);
        if (info.circumference().isEmpty())
            return Optional.of(Problem.of("editor.share.no_size"));
        JsonObject base = o.deepCopy();
        base.remove("timeline");
        overrides.clear();
        fieldTexts.clear();
        if (info.timeline().isPresent()) {
            Presets.Timeline t = info.timeline().get();
            start = Presets.START_CONFIGS.contains(t.start()) ? t.start() : null;
            lengthText = Double.isNaN(t.lengthMyr()) ? "" : Formats.number(t.lengthMyr());
        }
        String canon = Presets.canonical(base);
        for (Map.Entry<String, JsonObject> b : builtinJson.entrySet()) {
            if (Presets.canonical(b.getValue()).equals(canon)) {
                family = familyOf(b.getKey());
                circumference = info.circumference().getAsInt();
                imported = null;
                importedName = null;
                changed();
                return Optional.empty();
            }
        }
        imported = base;
        importedName = info.name().orElse("imported");
        circumference = info.circumference().getAsInt();
        changed();
        return Optional.empty();
    }

    // ------------------------------------------------------------------------------------------
    // The estimate
    // ------------------------------------------------------------------------------------------

    /**
     * The preset to estimate next, if the current estimate is for another one and none is being
     * computed for it; marks it in flight. The caller runs the estimator (off the render thread) and
     * reports with {@link #acceptEstimate}.
     */
    public synchronized Optional<String> estimateRequest() {
        String json = effectivePresetJson();
        if (json.equals(estimatedFor) || json.equals(estimateInFlight) || !deps.nativesLinked().getAsBoolean())
            return Optional.empty();
        estimateInFlight = json;
        return Optional.of(json);
    }

    /**
     * Stores the estimator's answer for {@code forJson}: its JSON, or {@code error}. Safe from the
     * estimator's thread: it touches only the estimate fields; an answer for a preset that is no
     * longer the current one is never shown ({@link #estimate()} compares) and is asked again.
     */
    public synchronized void acceptEstimate(String forJson, String resultJson, String error) {
        if (forJson.equals(estimateInFlight))
            estimateInFlight = null;
        estimatedFor = forJson;
        if (error != null) {
            estimate = null;
            estimateError = error;
            return;
        }
        try {
            estimate = Estimate.parse(resultJson);
            estimateError = null;
        } catch (RuntimeException e) {
            estimate = null;
            estimateError = "unreadable estimate: " + e.getMessage();
        }
    }

    /** Runs the estimator now, on this thread (tests and the GameTests). */
    public void refreshEstimate() {
        Optional<String> req = estimateRequest();
        if (req.isEmpty())
            return;
        try {
            acceptEstimate(req.get(), deps.estimator().estimate(req.get()), null);
        } catch (RuntimeException e) {
            acceptEstimate(req.get(), null, e.getMessage() == null ? e.toString() : e.getMessage());
        }
    }

    /** The estimate of the current preset, if known. */
    public synchronized Optional<Estimate> estimate() {
        return effectivePresetJson().equals(estimatedFor) ? Optional.ofNullable(estimate) : Optional.empty();
    }

    public synchronized Optional<String> estimateError() {
        return effectivePresetJson().equals(estimatedFor) ? Optional.ofNullable(estimateError) : Optional.empty();
    }

    // ------------------------------------------------------------------------------------------
    // Validation
    // ------------------------------------------------------------------------------------------

    /** Everything that keeps Run disabled, then the warnings. */
    public List<Problem> problems() {
        List<Problem> out = new ArrayList<>();
        if (!deps.nativesLinked().getAsBoolean())
            out.add(Problem.of("editor.problem.natives"));
        if (start == null)
            out.add(Problem.of("editor.problem.start"));
        double len = Formats.parse(lengthText);
        if (lengthText.isBlank())
            out.add(Problem.of("editor.problem.length_missing"));
        else if (!(len >= Presets.MIN_LENGTH_MYR && len <= Presets.MAX_LENGTH_MYR))
            out.add(Problem.of("editor.problem.length_range", Formats.number(Presets.MIN_LENGTH_MYR), Formats.number(Presets.MAX_LENGTH_MYR)));
        int bad = 0;
        for (EditorFields.Field f : EditorFields.ALL)
            if (!fieldValid(f))
                bad++;
        if (bad > 0)
            out.add(Problem.of("editor.problem.fields", bad));
        if (!capValid())
            out.add(Problem.of("editor.problem.cap", Formats.number(MIN_CAP_GIB)));
        if (deps.nativesLinked().getAsBoolean()) {
            Optional<String> err = estimateError();
            Optional<Estimate> est = estimate();
            if (err.isPresent())
                out.add(Problem.of("editor.problem.invalid", err.get()));
            else if (est.isEmpty())
                out.add(Problem.of("editor.problem.estimating"));
            else if (capValid() && est.get().exceeds(capBytes()))
                out.add(Problem.of("editor.problem.over_cap", Formats.bytes(est.get().baseBytes()), Formats.gib(capBytes())));
            else if (capValid() && est.get().spillUnder(capBytes()) > 0)
                out.add(Problem.warning("editor.warning.spill", Formats.bytes(est.get().spillUnder(capBytes()))));
            if (est.isPresent() && deps.machine().freeBytes() >= 0
                    && est.get().peakUnder(capBytes()) > deps.machine().freeBytes())
                out.add(Problem.warning("editor.warning.low_memory", Formats.bytes(deps.machine().freeBytes())));
        }
        if (launchError != null)
            out.add(Problem.warning("editor.warning.launch_failed", launchError));
        if (info().placeholder())
            out.add(Problem.warning("editor.warning.placeholder"));
        if (capValid() && deps.machine().totalBytes() > 0 && capBytes() > deps.machine().totalBytes())
            out.add(Problem.warning("editor.warning.cap_above_ram", Formats.gib(deps.machine().totalBytes())));
        return out;
    }

    public boolean canRun() {
        for (Problem p : problems())
            if (p.blocking())
                return false;
        return run == null || run.state() != TimelapseModel.State.RUNNING;
    }

    // ------------------------------------------------------------------------------------------
    // Pages, run, review, accept
    // ------------------------------------------------------------------------------------------

    public Page page() {
        return page;
    }

    /** The timelapse of the current run (to view it again from the setup page). */
    public void showRun() {
        if (run != null)
            page = Page.RUN;
    }

    public void showSetup() {
        page = Page.SETUP;
    }

    public Optional<TimelapseModel> run() {
        return Optional.ofNullable(run);
    }

    public Optional<ReviewModel> review() {
        return Optional.ofNullable(review);
    }

    public Optional<Accepted> accepted() {
        return Optional.ofNullable(accepted);
    }

    /**
     * Starts a run of the effective preset into a new staging folder and shows its timelapse. A
     * previous run that was not accepted is cancelled or deleted. Returns empty (and records a
     * warning) when the run could not be launched.
     */
    public Optional<TimelapseModel> startRun() {
        if (!canRun())
            throw new IllegalStateException("Run is disabled: " + problems());
        String json = effectivePresetJson();
        List<String> stages = new ArrayList<>();
        estimate().ifPresent(e -> e.stages().forEach(s -> stages.add(s.name())));
        dropRun();
        String id = Staging.newId();
        Path dir = Staging.dir(deps.gameDir(), id);
        RunHandle h;
        try {
            Files.createDirectories(dir);
            h = deps.launcher().launch(json, dir, deps.machine().clientThreads(), capBytes());
        } catch (IOException | RuntimeException e) {
            launchError = e.getMessage() == null ? e.toString() : e.getMessage();
            deleteLater(dir);
            return Optional.empty();
        }
        launchError = null;
        run = new TimelapseModel(h, id, json, circumference, stages, deps.loader(), deps.io());
        estimate().ifPresent(run::setEstimate);
        page = Page.RUN;
        return Optional.of(run);
    }

    /** Cancels the running run and returns to the setup page (its folder goes once it stopped). */
    public void cancelRun() {
        if (run != null && run.state() == TimelapseModel.State.RUNNING)
            run.cancel();
        page = Page.SETUP;
    }

    /** Opens the review page of a finished run. */
    public ReviewModel openReview() {
        if (run == null || run.state() != TimelapseModel.State.DONE)
            throw new IllegalStateException("no finished run to review");
        if (review == null || !review.dir().equals(run.run().outDir())) {
            Optional<KeyframeInfo> last = run.frames().isEmpty() ? Optional.empty()
                    : Optional.of(run.frames().get(run.frames().size() - 1));
            RunHandle h = run.run();
            review = new ReviewModel(h.outDir(), run.circumference(), last, h::pixels, Ramps.builtin(), deps.io());
        }
        page = Page.REVIEW;
        return review;
    }

    /** True when the current run finished and its staging folder is complete. */
    public boolean canAccept() {
        return run != null && run.state() == TimelapseModel.State.DONE && Staging.complete(run.run().outDir());
    }

    /**
     * Accepts the finished run: the world will be created from its staging folder. A previously
     * accepted run's folder is deleted. Returns to the setup page.
     */
    public Accepted accept() {
        if (!canAccept())
            throw new IllegalStateException("no finished run to accept");
        Presets.Info i = Presets.info(Presets.parseObject(run.presetJson()));
        String label = i.name().orElse(presetName()) + " · " + run.circumference() + " blocks"
                + i.timeline().map(t -> " · " + t).orElse("");
        Accepted a = new Accepted(run.stagingId(), run.circumference(), run.presetJson(), label);
        if (accepted != null && !accepted.stagingId().equals(a.stagingId()))
            deleteLater(Staging.dir(deps.gameDir(), accepted.stagingId()));
        accepted = a;
        page = Page.SETUP;
        return a;
    }

    /**
     * The Create World screen this model belongs to is gone (back to the title or the world list,
     * or the world is being created): a run that is not the accepted one is cancelled, or deleted
     * when it finished, so nothing keeps simulating (CPU, native memory) for a screen nobody can
     * reach again. The accepted run's folder stays for the server to move.
     */
    public void abandon() {
        if (run != null && (accepted == null || !accepted.stagingId().equals(run.stagingId()))) {
            dropRun();
            run = null;
            review = null;
        }
        page = Page.SETUP;
    }

    /** Discards the current (finished or failed) run: its folder is deleted unless it is the accepted one. */
    public void discardRun() {
        dropRun();
        run = null;
        review = null;
        page = Page.SETUP;
    }

    private void dropRun() {
        if (run == null)
            return;
        if (run.state() == TimelapseModel.State.RUNNING) {
            run.cancel(); // its folder goes when it stops
        } else if (accepted == null || !accepted.stagingId().equals(run.stagingId())) {
            deleteLater(run.run().outDir());
        }
        review = null;
    }

    private void deleteLater(Path dir) {
        deps.io().execute(() -> {
            try {
                Staging.deleteTree(dir);
            } catch (IOException ignored) {
                // Staging.cleanupAbandoned retries
            }
        });
    }

    /** The staging ids this editor still needs (never cleaned up while it lives). */
    public Set<String> heldStagingIds() {
        Set<String> out = new LinkedHashSet<>();
        if (accepted != null)
            out.add(accepted.stagingId());
        if (run != null && run.state() != TimelapseModel.State.CANCELLED && run.state() != TimelapseModel.State.FAILED)
            out.add(run.stagingId());
        return out;
    }
}
