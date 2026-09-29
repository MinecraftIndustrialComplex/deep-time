// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.preset.Presets;
import com.minecraftindustrialcomplex.deeptime.preset.ShareCode;
import com.minecraftindustrialcomplex.deeptime.server.Staging;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

/** The setup page's rules without a screen or natives (plan §7.12, §5.1). */
class EditorModelTest {

    @TempDir
    Path gameDir;

    static final long GIB = 1L << 30;

    /** A fake estimate: the JSON's length as peak bytes unless set. */
    long peak = 100_000_000L;
    /** The fake estimate's recorded history (spillable, P1-17); -1 leaves the fields out. */
    long history = -1;
    boolean natives = true;
    final List<String> estimated = new ArrayList<>();
    final List<FakeRunHandle> launched = new ArrayList<>();

    EditorModel model() {
        return new EditorModel(deps());
    }

    EditorModel.Deps deps() {
        return new EditorModel.Deps(json -> {
            estimated.add(json);
            if (json.contains("\"poison\""))
                throw new IllegalArgumentException("$.poison: unknown key");
            String spill = history < 0 ? ""
                    : ",\"base_bytes\":" + (peak - history) + ",\"history_bytes\":" + history;
            return "{\"peak_bytes\":" + peak + spill + ",\"stages\":[{\"bytes\":1,\"name\":\"dummy_drift\",\"seconds\":5.0},"
                    + "{\"bytes\":2,\"name\":\"dummy_erode\",\"seconds\":5.0}]}";
        }, (json, out, threads, cap) -> {
            FakeRunHandle h = new FakeRunHandle(out);
            launched.add(h);
            return h;
        }, gameDir, new MachineInfo(8 * GIB, 16 * GIB, 4 * GIB, 8), Runnable::run, Runnable::run, () -> natives, List::of);
    }

    static boolean has(List<Problem> ps, String key) {
        return ps.stream().anyMatch(p -> p.key().equals(key));
    }

    EditorModel ready() {
        EditorModel m = model();
        m.setStart("supercontinent");
        m.setLengthText("1000");
        m.refreshEstimate();
        assertTrue(m.canRun(), () -> m.problems().toString());
        return m;
    }

    @Test
    void builtInFamiliesAndSizes() {
        EditorModel m = model();
        assertEquals(List.of("earthlike_quick", "earthlike_full"), m.families());
        assertEquals(Optional.of("earthlike_quick"), m.family());
        assertEquals(16_384, m.circumference(), "singleplayer default");
        assertEquals("earthlike_quick_16k", m.presetName());
        assertEquals(List.of(16_384, 32_768), m.sizes());
        m.selectSize(32_768);
        assertEquals("earthlike_quick_32k", m.presetName());
        m.selectFamily("earthlike_full");
        assertEquals("earthlike_full_32k", m.presetName(), "the size is kept across families");
        assertFalse(m.info().placeholder(), "the built-ins are simulated planets since P1-08");
        assertFalse(has(m.problems(), "editor.warning.placeholder"));
    }

    /** No default story: Run is disabled until both parts of the timeline are chosen. */
    @Test
    void runDisabledWithoutATimeline() {
        EditorModel m = model();
        m.refreshEstimate();
        assertFalse(m.canRun());
        assertTrue(has(m.problems(), "editor.problem.start"));
        assertTrue(has(m.problems(), "editor.problem.length_missing"));
        assertFalse(m.effectivePresetJson().contains("\"timeline\""), "no timeline is invented");
        m.setStart("random");
        assertFalse(m.canRun());
        m.setLengthText("0.5");
        assertTrue(has(m.problems(), "editor.problem.length_range"));
        m.setLengthText("abc");
        assertTrue(has(m.problems(), "editor.problem.length_range"));
        m.setLengthText("4601");
        assertFalse(m.canRun());
        m.setLengthText("4600");
        assertFalse(m.canRun(), "the estimate of the new preset is not in yet");
        assertTrue(has(m.problems(), "editor.problem.estimating"));
        m.refreshEstimate();
        assertTrue(m.canRun(), () -> m.problems().toString());
        assertTrue(m.effectivePresetJson().contains("\"timeline\":{\"length_myr\":4600.0,\"start\":\"random\"}"),
                m.effectivePresetJson());
        m.setStart(null);
        assertFalse(m.canRun());
        assertThrows(IllegalArgumentException.class, () -> m.setStart("pangaea"));
    }

    @Test
    void estimateIsForTheCurrentPresetOnly() {
        EditorModel m = model();
        Optional<String> first = m.estimateRequest();
        assertTrue(first.isPresent());
        assertTrue(m.estimateRequest().isEmpty(), "in flight");
        m.setStart("supercontinent");
        m.setLengthText("100");
        Optional<String> second = m.estimateRequest();
        assertTrue(second.isPresent());
        m.acceptEstimate(first.get(), "{\"peak_bytes\":1,\"stages\":[]}", null);
        assertTrue(m.estimate().isEmpty(), "a stale answer is dropped");
        m.acceptEstimate(second.get(), "{\"peak_bytes\":1234,\"stages\":[{\"bytes\":1,\"name\":\"a\",\"seconds\":2.5}]}", null);
        assertEquals(1234, m.estimate().orElseThrow().peakBytes());
        assertEquals(2.5, m.estimate().get().seconds());
        assertTrue(m.estimateRequest().isEmpty(), "up to date");
        m.acceptEstimate(second.get(), null, "$.dummy: boom");
        assertEquals(Optional.of("$.dummy: boom"), m.estimateError());
        assertTrue(has(m.problems(), "editor.problem.invalid"));
    }

    @Test
    void capBlocksAndIsFreelyRaisable() {
        peak = 5 * GIB;
        EditorModel m = model();
        m.setStart("supercontinent");
        m.setLengthText("1000");
        m.refreshEstimate();
        assertEquals(4 * GIB, m.capBytes(), "4 GiB default");
        assertFalse(m.canRun());
        assertTrue(has(m.problems(), "editor.problem.over_cap"));
        m.setCapText("64");
        assertTrue(m.canRun(), () -> m.problems().toString());
        assertEquals(64 * GIB, m.capBytes());
        assertTrue(has(m.problems(), "editor.warning.cap_above_ram"), "a warning, not a block");
        m.setCapText("0.1");
        assertTrue(has(m.problems(), "editor.problem.cap"));
        assertFalse(m.canRun());
        m.setCapText("5,5");
        assertTrue(m.capValid(), "a comma works as the decimal point");
    }

    /**
     * The recorded history spills to disk over the cap (P1-17): only the base must fit, a run
     * whose history does not is allowed with a warning, and one whose base does not is blocked
     * naming the base.
     */
    @Test
    void onlyTheBaseMustFitTheCap() {
        peak = 7 * GIB;
        history = 3 * GIB;
        EditorModel m = model();
        m.setStart("supercontinent");
        m.setLengthText("1000");
        m.refreshEstimate();
        Estimate e = m.estimate().orElseThrow();
        assertEquals(4 * GIB, e.baseBytes());
        assertEquals(4 * GIB, m.capBytes());
        assertTrue(m.canRun(), () -> m.problems().toString());
        assertTrue(has(m.problems(), "editor.warning.spill"));
        assertEquals(4 * GIB, e.peakUnder(m.capBytes()));
        assertEquals(3 * GIB, e.spillUnder(m.capBytes()));
        m.setCapText("5.5");
        assertEquals(5.5 * GIB, e.peakUnder(m.capBytes()), 1.0);
        assertEquals(1.5 * GIB, e.spillUnder(m.capBytes()), 1.0);
        m.setCapText("16");
        assertFalse(has(m.problems(), "editor.warning.spill"), "it all fits");
        assertEquals(7 * GIB, e.peakUnder(m.capBytes()));
        assertEquals(0, e.spillUnder(m.capBytes()));
        m.setCapText("3");
        assertFalse(m.canRun());
        assertTrue(has(m.problems(), "editor.problem.over_cap"));
        // An estimate without the fields (a library from before the spill): the peak is all base.
        Estimate old = Estimate.parse("{\"peak_bytes\":5,\"stages\":[]}");
        assertEquals(5, old.baseBytes());
        assertEquals(0, old.historyBytes());
        assertTrue(old.exceeds(4));
    }

    @Test
    void nativesMissingBlocksRunAndEstimates() {
        natives = false;
        EditorModel m = model();
        m.setStart("supercontinent");
        m.setLengthText("1000");
        assertTrue(m.estimateRequest().isEmpty());
        assertTrue(has(m.problems(), "editor.problem.natives"));
        assertFalse(m.canRun());
    }

    /**
     * A field is editable exactly when the preset carries its path: a placeholder preset (the dummy
     * pipeline, P1-13's stand-in) carries none, so every field is unavailable and edits change
     * nothing; the built-ins carry the planet's.
     */
    @Test
    void fieldsFollowThePreset() {
        EditorModel m = model();
        assertTrue(m.importShareCode(ShareCode.encode(
                "{\"name\":\"stand_in\",\"placeholder\":true,\"scale\":{\"circumference_blocks\":16384},"
                        + "\"dummy\":{\"steps\":1,\"sleep_ms\":0}}")).isEmpty());
        assertTrue(m.info().placeholder());
        assertTrue(has(m.problems(), "editor.warning.placeholder"));
        String before = m.effectivePresetJson();
        for (EditorFields.Field f : EditorFields.ALL) {
            assertFalse(m.available(f), f.id());
            m.setFieldText(f, "1");
        }
        assertEquals(before, m.effectivePresetJson());
        assertTrue(model().available(EditorFields.byId("gravity")), "a built-in carries the planet's paths");

        // A preset that carries planet.gravity_m_s2 (imported through a share code).
        JsonObject p = Presets.parseObject(Presets.builtin("earthlike_quick_16k").orElseThrow());
        JsonObject planet = new JsonObject();
        planet.addProperty("gravity_m_s2", 9.81);
        p.add("planet", planet);
        p.addProperty("name", "custom");
        assertTrue(m.importShareCode(ShareCode.encode(p.toString())).isEmpty());
        assertTrue(m.isImported());
        EditorFields.Field g = EditorFields.byId("gravity");
        assertTrue(m.available(g));
        assertEquals("9.81", m.fieldText(g));
        m.setFieldText(g, "3.71");
        assertTrue(m.effectivePresetJson().contains("\"gravity_m_s2\":3.71"), m.effectivePresetJson());
        m.setFieldText(g, "1000");
        assertFalse(m.fieldValid(g));
        assertTrue(has(m.problems(), "editor.problem.fields"));
        assertTrue(m.effectivePresetJson().contains("\"gravity_m_s2\":9.81"), "an invalid value is not used");
        assertFalse(m.available(EditorFields.byId("sky_islands")), "fantasy toggles stay disabled (Phase 3)");
        m.resetFields();
        assertTrue(m.fieldValid(g));
    }

    /**
     * The catalogue is the shared deeptime/editor_fields.json (the browser workbench reads the same
     * file, P1-20); the seed is a whole number and lands in the preset as a JSON integer (a u64 in
     * the core), the other numbers as doubles.
     */
    @Test
    void catalogueIsTheSharedFileAndTheSeedIsAnInteger() {
        assertEquals("seed", EditorFields.ALL.get(0).id());
        assertTrue(EditorFields.byId("seed").integer());
        assertFalse(EditorFields.byId("gravity").integer());
        assertEquals(EditorFields.Section.PLANET, EditorFields.byId("seed").section());
        assertTrue(EditorFields.byId("fantasy_fields").phase3());
        EditorModel m = model();
        JsonObject p = Presets.parseObject(Presets.builtin("earthlike_quick_16k").orElseThrow());
        p.addProperty("seed", 1);
        p.addProperty("name", "seeded");
        assertTrue(m.importShareCode(ShareCode.encode(p.toString())).isEmpty());
        EditorFields.Field seed = EditorFields.byId("seed");
        assertTrue(m.available(seed));
        assertEquals("1", m.fieldText(seed));
        m.setFieldText(seed, "42");
        assertTrue(m.effectivePresetJson().matches(".*\"seed\":42[,}].*"), m.effectivePresetJson());
        m.setFieldText(seed, "4.5");
        assertFalse(m.fieldValid(seed), "a seed is whole");
        assertTrue(m.effectivePresetJson().matches(".*\"seed\":1[,}].*"), "an invalid seed is not used");
    }

    @Test
    void shareCodeRoundTrip() {
        EditorModel a = ready();
        a.selectSize(32_768);
        a.setLengthText("750");
        String code = a.exportShareCode();
        assertTrue(ShareCode.isShareCode(code), code);

        EditorModel b = model();
        assertTrue(b.importShareCode(code).isEmpty());
        assertFalse(b.isImported(), "a built-in's code selects the built-in");
        assertEquals("earthlike_quick_32k", b.presetName());
        assertEquals(Optional.of("supercontinent"), b.start());
        assertEquals("750", b.lengthText());
        assertEquals(a.effectivePresetJson(), b.effectivePresetJson());
        assertEquals(code, b.exportShareCode(), "one preset, one code");

        // A changed preset imports as a custom one with its size fixed.
        JsonObject p = Presets.parseObject(ShareCode.decode(code));
        p.addProperty("description", "tweaked");
        EditorModel c = model();
        assertTrue(c.importShareCode(ShareCode.encode(p.toString())).isEmpty());
        assertTrue(c.isImported());
        assertEquals(32_768, c.circumference());
        assertFalse(c.sizeSelectable(16_384));
        assertEquals(List.of(16_384, 32_768), c.sizes());

        // Bad codes change nothing.
        String before = b.effectivePresetJson();
        Optional<Problem> bad = b.importShareCode("DT1.!!!");
        assertTrue(bad.isPresent());
        assertEquals("editor.share.invalid", bad.get().key());
        assertEquals("editor.share.invalid", b.importShareCode("hello").orElseThrow().key());
        JsonObject noSize = new JsonObject();
        noSize.addProperty("name", "x");
        assertEquals("editor.share.no_size", b.importShareCode(ShareCode.encode(noSize.toString())).orElseThrow().key());
        assertEquals(before, b.effectivePresetJson());
    }

    @Test
    void runAcceptAndDiscardManageStagingFolders() throws Exception {
        EditorModel m = ready();
        TimelapseModel first = m.startRun().orElseThrow();
        assertEquals(EditorModel.Page.RUN, m.page());
        Path dir1 = first.run().outDir();
        assertEquals(Staging.dir(gameDir, first.stagingId()), dir1);
        assertTrue(Files.isDirectory(dir1));
        assertFalse(m.canRun(), "one run at a time");
        assertThrows(IllegalStateException.class, m::accept);

        // Finish it: a manifest makes the folder complete.
        Files.writeString(dir1.resolve("manifest.json"), "{}");
        launched.get(0).completion.complete("{}");
        first.poll();
        assertEquals(TimelapseModel.State.DONE, first.state());
        assertTrue(m.canAccept());
        EditorModel.Accepted a = m.accept();
        assertEquals(first.stagingId(), a.stagingId());
        assertEquals(16_384, a.circumference());
        assertTrue(a.label().contains("supercontinent"), a.label());
        assertEquals(EditorModel.Page.SETUP, m.page());
        assertTrue(m.heldStagingIds().contains(a.stagingId()));

        // A second run keeps the accepted folder until it is itself accepted.
        TimelapseModel second = m.startRun().orElseThrow();
        Path dir2 = second.run().outDir();
        assertTrue(Files.isDirectory(dir1), "the accepted run survives a new run");
        Files.writeString(dir2.resolve("manifest.json"), "{}");
        launched.get(1).completion.complete("{}");
        second.poll();
        m.accept();
        assertFalse(Files.exists(dir1), "the replaced accepted run is deleted");
        assertTrue(Files.isDirectory(dir2));

        // A third run discarded: its folder goes, the accepted one stays.
        TimelapseModel third = m.startRun().orElseThrow();
        Path dir3 = third.run().outDir();
        Files.writeString(dir3.resolve("manifest.json"), "{}");
        launched.get(2).completion.complete("{}");
        third.poll();
        m.discardRun();
        assertFalse(Files.exists(dir3));
        assertTrue(Files.isDirectory(dir2));
        assertEquals(Optional.empty(), m.run());

        // A cancelled run deletes its folder once it stopped.
        TimelapseModel fourth = m.startRun().orElseThrow();
        Path dir4 = fourth.run().outDir();
        m.cancelRun();
        assertTrue(launched.get(3).cancelled);
        assertFalse(Files.exists(dir4));
        fourth.poll();
        assertEquals(TimelapseModel.State.CANCELLED, fourth.state());
        assertEquals(EditorModel.Page.SETUP, m.page());
        assertTrue(Files.isDirectory(dir2));
    }

    @Test
    void launchFailureIsAWarning() {
        EditorModel.Deps d = deps();
        EditorModel m = new EditorModel(new EditorModel.Deps(d.estimator(), (json, out, t, cap) -> {
            throw new IllegalStateException("natives vanished");
        }, d.gameDir(), d.machine(), d.loader(), d.io(), d.nativesLinked(), d.essentials()));
        m.setStart("supercontinent");
        m.setLengthText("10");
        m.refreshEstimate();
        assertTrue(m.startRun().isEmpty());
        assertEquals(EditorModel.Page.SETUP, m.page());
        assertTrue(has(m.problems(), "editor.warning.launch_failed"));
    }

    @Test
    void runUsesTheEffectivePresetAndCap() {
        AtomicReference<String> seen = new AtomicReference<>();
        AtomicReference<Long> cap = new AtomicReference<>();
        EditorModel.Deps d = deps();
        EditorModel m = new EditorModel(new EditorModel.Deps(d.estimator(), (json, out, t, c) -> {
            seen.set(json);
            cap.set(c);
            return new FakeRunHandle(out);
        }, d.gameDir(), d.machine(), d.loader(), d.io(), d.nativesLinked(), d.essentials()));
        m.setStart("scattered_continents");
        m.setLengthText("300");
        m.setCapText("6");
        m.refreshEstimate();
        m.startRun().orElseThrow();
        assertEquals(m.effectivePresetJson(), seen.get());
        assertEquals(Presets.info(Presets.parseObject(seen.get())).timeline().orElseThrow(),
                new Presets.Timeline("scattered_continents", 300.0));
        assertEquals(6 * GIB, cap.get());
        assertNotEquals(0, estimated.size());
    }
}
