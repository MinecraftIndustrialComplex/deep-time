// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.screen.model;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.minecraftindustrialcomplex.deeptime.nativebridge.FailureClass;
import com.minecraftindustrialcomplex.deeptime.nativebridge.NativeStatus;
import com.minecraftindustrialcomplex.deeptime.worlddata.SiteRecords;
import java.nio.file.Path;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import org.junit.jupiter.api.Test;

/** Ramps, the globe projection, formats, province hints and the guidance text. */
class ModelHelpersTest {

    /** The same values Rust's {@code builtin_ramps_cover_every_layer} checks: one file, one colouring. */
    @Test
    void rampsMatchRust() {
        Ramps r = Ramps.builtin();
        assertEquals("elevation", r.rampName("elevation"));
        Ramps.Ramp e = r.forLayer("elevation");
        assertEquals(0x08142e, e.colour(-1000));
        assertEquals(0x3f7f3a, e.colour(63));
        assertEquals(0xffffff, e.colour(1000));
        assertEquals(0x76a852, e.colour(100), "a stop");
        // Half way between 63 (#3f7f3a) and 100 (#76a852): channels round half away from zero, as in Rust.
        assertEquals(0x5b9446, e.colour(81.5));
        assertEquals(0, e.abgr(Double.NaN), "nodata is transparent");
        assertEquals(0xff3a7f3f, e.abgr(63), "ABGR for NativeImage");
        assertEquals("grey", r.rampName("no/such/layer"));
        assertEquals("favourability", r.rampName("deposits/favourability/bif_fe"));
        assertEquals("relief", r.rampName("input/elevation_m"), "an exact entry beats a pattern");
        assertEquals("unit", r.rampName("input/coal_facies"));
        assertEquals(0xffd700, r.forLayer("deposits/sites").colour(3.0));
        assertEquals(0x0a0b0c, Ramps.hex("#0a0b0c"));
        assertEquals(0x0a0000, Ramps.hex("#0a"), "missing digits read as 0");
    }

    @Test
    void globeProjection() {
        GlobeProjection p = new GlobeProjection(20, 100);
        double[] ll = p.inverse(0, 0, new double[2]);
        assertEquals(20, ll[0], 1e-9);
        assertEquals(100, ll[1], 1e-9);
        assertNull(p.inverse(0.8, 0.8, new double[2]), "outside the disk");
        Random rnd = new Random(7);
        double[] uv = new double[2];
        for (int i = 0; i < 1000; i++) {
            double u = rnd.nextDouble() * 1.4 - 0.7, v = rnd.nextDouble() * 1.4 - 0.7;
            p.inverse(u, v, ll);
            assertTrue(ll[1] >= 0 && ll[1] < 360);
            assertNotNull(p.forward(ll[0], ll[1], uv));
            assertEquals(u, uv[0], 1e-9);
            assertEquals(v, uv[1], 1e-9);
        }
        assertNull(p.forward(-20, 280, uv), "the antipode is behind");
        assertNotNull(p.forward(20, 100 + 360, uv), "longitudes wrap");

        // Render: every source column has its own colour; the globe's centre shows the column at lon 100.
        int w = 360, h = 180;
        int[] src = new int[w * h];
        for (int y = 0; y < h; y++)
            for (int x = 0; x < w; x++)
                src[y * w + x] = 0xff000000 | x;
        int size = 65;
        int[] dst = new int[size * size];
        new GlobeProjection(0, 100).render(src, w, h, dst, size);
        assertEquals(100, dst[32 * size + 32] & 0xffff, "centre pixel is longitude 100");
        assertEquals(0, dst[0], "corners are outside the disk");
        assertEquals(0, GlobeProjection.shade(0x00123456, 0.5), "nodata stays transparent");

        double[] b = GlobeProjection.block(45, 90, 16_384, new double[2]);
        assertEquals(-4096, b[0], 1e-9, "lon 90 is a quarter east of the seam at x = −C/2");
        assertEquals(-16_384 / (2 * Math.PI) * Math.sin(Math.toRadians(45)), b[1], 1e-9, "north is −z; z = −R·sin φ (Lambert)");
        double[] back = GlobeProjection.latLon(b[0], b[1], 16_384, new double[2]);
        assertEquals(45, back[0], 1e-9);
        assertEquals(90, back[1], 1e-9);
        double[] mp = GlobeProjection.mapPixel(0, 0, 256, 128, new double[2]);
        assertEquals(0.5, mp[0], 1e-9, "pixel 0 is centred on the seam");
        assertEquals(64.5, mp[1], 1e-9);
    }

    @Test
    void formats() {
        assertEquals("512 B", Formats.bytes(512));
        assertEquals("3.0 MiB", Formats.bytes(3L << 20));
        assertEquals("1.5 GiB", Formats.bytes(3L << 29));
        assertEquals("4.0", Formats.gib(4L << 30));
        assertEquals(4L << 30, Formats.gibToBytes(4));
        assertEquals("5s", Formats.duration(5_400));
        assertEquals("2m05s", Formats.duration(125_000));
        assertEquals("1h02m", Formats.duration(3_720_000));
        assertEquals("<1s", Formats.seconds(0.2));
        assertEquals("312.0 Myr ago", Formats.time(-3.12e8));
        assertEquals("4.5 kyr ago", Formats.time(-4500));
        assertEquals("the present", Formats.time(0));
        assertEquals("+2.0 Myr", Formats.time(2e6));
        assertEquals("−312 Ma", Formats.ma(312_000_000));
        assertEquals("0 Ma", Formats.ma(0));
        assertEquals("37%", Formats.percent(375));
        assertEquals("9.81", Formats.number(9.81));
        assertEquals("300", Formats.number(300.0));
        assertEquals("0.001", Formats.number(0.001));
        assertEquals("0.0001", Formats.number(0.0001));
        // Under 1e-4 six decimals would round the value away (the erosion constant, 2e-9): scientific form.
        assertEquals("2e-9", Formats.number(2.0e-9));
        assertEquals("1.5e-7", Formats.number(1.5e-7));
        assertEquals("-5e-5", Formats.number(-5.0e-5));
        assertEquals(2.0e-9, Formats.parse(Formats.number(2.0e-9)));
        assertEquals(1.5e-7, Formats.parse("1,5e-7"));
        assertEquals(5.5, Formats.parse(" 5,5 "));
        assertTrue(Double.isNaN(Formats.parse("x")));
        assertTrue(Double.isNaN(Formats.parse("Infinity")));
        assertEquals("Rift onset", Formats.humanize("deeptime:rift_onset"));
        assertEquals("Bif fe", Formats.humanize("deposits/favourability/bif_fe"));
    }

    static SiteRecords.Site site(String type, double lat, double lon) {
        return new SiteRecords.Site(0, SiteRecords.Kind.DEPOSIT, type, 0, 0, 0, lat, lon);
    }

    /** Hints are coarse regions: a province shows where a type clusters, never a lone deposit's neighbourhood. */
    @Test
    void provinceHints() {
        assertTrue(ProvinceHints.compute(List.of()).isEmpty());
        List<ProvinceHints.Cell> one = ProvinceHints.compute(List.of(site("deeptime:placer_au", 10, 40)));
        assertEquals(1, one.size(), "a lone site marks only its own 30° cell");
        assertEquals(1, one.get(0).col());
        assertEquals(2, one.get(0).row());
        List<ProvinceHints.Cell> cluster = ProvinceHints.compute(List.of(
                site("deeptime:porphyry_cu_mo_au", -40, 359), site("deeptime:porphyry_cu_mo_au", -40, 1),
                site("deeptime:coal_seam", -40, 2)));
        // Two porphyries straddle the seam: cols 11 and 0; their neighbours reach 1.0 and show too.
        assertTrue(cluster.stream().anyMatch(c -> c.col() == 11 && c.row() == 4 && c.type().equals("deeptime:porphyry_cu_mo_au")));
        assertTrue(cluster.stream().anyMatch(c -> c.col() == 0 && c.row() == 4 && c.type().equals("deeptime:porphyry_cu_mo_au")),
                "porphyry (1.5) beats coal (1.0) in the seam's east cell");
        assertTrue(cluster.stream().anyMatch(c -> c.col() == 0 && c.row() == 3 && c.type().equals("deeptime:porphyry_cu_mo_au")),
                "two neighbouring sites blur into the next row (0.5 + 0.5)");
        assertFalse(cluster.stream().anyMatch(c -> c.col() == 1 && c.row() == 4), "one site's blur alone (0.5) shows nothing");
        assertFalse(cluster.stream().anyMatch(c -> c.row() == 0), "nothing near the pole");
        ProvinceHints.Cell c = cluster.get(0);
        assertEquals(c.lonEast() - c.lonWest(), 30, 1e-9);
        assertEquals(0xd7301f, ProvinceHints.colour("porphyry_cu_mo_au"));
        assertEquals(0xffb000, ProvinceHints.colour("deeptime:dummy"));
    }

    /**
     * Every real (non-placeholder) built-in preset carries every parameter path the editor edits,
     * so no field of a real preset is shown unavailable. Vacuous while the built-ins are P1-13's
     * placeholders; it bites when P1-08/P1-17 replace them.
     */
    @Test
    void realPresetsCarryEveryField() throws java.io.IOException {
        // Fields the catalogue marks planned (their stage or key does not exist yet) are exempt.
        java.util.Set<String> planned = new java.util.HashSet<>();
        try (var in = EditorFields.class.getResourceAsStream(EditorFields.RESOURCE)) {
            var cat = com.google.gson.JsonParser.parseReader(
                    new java.io.InputStreamReader(in, java.nio.charset.StandardCharsets.UTF_8)).getAsJsonObject();
            for (var e : cat.getAsJsonArray("fields")) {
                var o = e.getAsJsonObject();
                if (o.has("planned") && o.get("planned").getAsBoolean())
                    planned.add(o.get("id").getAsString());
            }
        }
        int real = 0;
        for (String name : com.minecraftindustrialcomplex.deeptime.preset.Presets.BUILTIN) {
            var o = com.minecraftindustrialcomplex.deeptime.preset.Presets.parseObject(
                    com.minecraftindustrialcomplex.deeptime.preset.Presets.builtin(name).orElseThrow());
            if (com.minecraftindustrialcomplex.deeptime.preset.Presets.info(o).placeholder())
                continue;
            real++;
            for (EditorFields.Field f : EditorFields.ALL)
                if (!f.phase3() && !planned.contains(f.id()))
                    assertTrue(f.availableIn(o), name + " has no " + f.path() + " (EditorFields." + f.id() + ")");
        }
        System.out.println("[junit] realPresetsCarryEveryField: " + real + " real built-in presets checked, "
                + planned.size() + " planned fields exempt");
        assertEquals(4, real, "every built-in is a simulated planet since P1-08");
        java.util.Set<String> ids = new java.util.HashSet<>();
        for (EditorFields.Field f : EditorFields.ALL)
            assertTrue(ids.add(f.id()), "duplicate field id " + f.id());
    }

    @Test
    void memAvailableFromMeminfo(@org.junit.jupiter.api.io.TempDir Path dir) throws Exception {
        Path f = dir.resolve("meminfo");
        java.nio.file.Files.writeString(f, "MemTotal:       16163264 kB\nMemFree:          734812 kB\nMemAvailable:    7340032 kB\n");
        assertEquals(7340032L * 1024, MachineInfo.memAvailable(f).orElseThrow());
        assertTrue(MachineInfo.memAvailable(dir.resolve("absent")).isEmpty());
        java.nio.file.Files.writeString(f, "MemTotal: 1 kB\n");
        assertTrue(MachineInfo.memAvailable(f).isEmpty());
        MachineInfo m = MachineInfo.current();
        assertTrue(m.processors() >= 1 && m.clientThreads() >= 1 && m.jvmMaxBytes() > 0);
    }

    @Test
    void guidanceText() {
        NativeStatus s = NativeStatus.failure(FailureClass.UNSUPPORTED_PLATFORM, "linux-riscv64-gnu", Path.of("/x/lib.so"), null,
                FailureClass.UNSUPPORTED_PLATFORM.guidance("linux-riscv64-gnu", null, null));
        GuidanceModel g = GuidanceModel.of(Optional.of(s));
        assertEquals("UNSUPPORTED_PLATFORM", g.failureClass());
        assertEquals("linux-riscv64-gnu", g.platform());
        assertEquals("/x/lib.so", g.library());
        assertTrue(g.guidance().contains("deeptime run"), "the CLI route (plan §6.6)");
        assertTrue(g.details().contains("UNSUPPORTED_PLATFORM"));
        GuidanceModel none = GuidanceModel.of(Optional.empty());
        assertEquals("NOT_LOADED", none.failureClass());
        assertFalse(none.platform().isEmpty());
    }
    /**
     * Time left weights a coupled group by all of its stages' seconds (the session reports the group
     * under its lead's index) and scales by the run's own pace. The 2026-10-04 client walk: 250 Myr
     * of earthlike_quick_16k, estimate 296 s, plates/climate/surface coupled for 211 s of it; at
     * "plates 36 %" after 79 s the session said ~32 min left and the run ended 214 s later.
     */
    @Test
    void timeLeftWeightsCoupledGroupsAndThePace() {
        Estimate e = Estimate.parse("{\"peak_bytes\":1,\"stages\":["
                + "{\"name\":\"plates\",\"bytes\":1,\"seconds\":45.0,\"coupled_to\":null},"
                + "{\"name\":\"climate\",\"bytes\":1,\"seconds\":101.0,\"coupled_to\":\"plates\"},"
                + "{\"name\":\"surface\",\"bytes\":1,\"seconds\":65.0,\"coupled_to\":\"plates\"},"
                + "{\"name\":\"deposits\",\"bytes\":1,\"seconds\":85.0}]}");
        assertEquals("plates", e.stages().get(1).coupledTo());
        assertNull(e.stages().get(3).coupledTo());
        // Unloaded pace (elapsed as estimated): what is left of the estimate.
        long fresh = e.timeLeftMs(0, 0, 0);
        assertEquals(296_000, fresh);
        // The group 36 % done = 76 s of 211; 79 s elapsed is about the estimated pace: ~220 s left.
        long mid = e.timeLeftMs(0, 360, 79_000);
        assertTrue(mid > 190_000 && mid < 250_000, "time left " + mid);
        // Twice as slow as estimated: twice the time left.
        long slow = e.timeLeftMs(0, 360, 2 * 75_960);
        assertEquals(2 * (296_000 - 75_960), slow, 1_000);
        // After the group: the deposits stage alone remains.
        assertEquals(42_500, e.timeLeftMs(3, 500, 253_500), 1_000);
        assertEquals(-1, e.timeLeftMs(4, 0, 1));
        // Stages without coupling count on their own.
        Estimate plain = new Estimate(1, List.of(new Estimate.Stage("a", 1, 10), new Estimate.Stage("b", 1, 30)));
        assertEquals(35_000, plain.timeLeftMs(0, 500, 0));
    }

    /** The bake after the stages counts in the run's seconds, as a share of amplify's, at the stages' pace. */
    @Test
    void theBakeCountsAsAShareOfAmplify() {
        Estimate e = new Estimate(1, List.of(new Estimate.Stage("plates", 1, 100), new Estimate.Stage("amplify", 1, 50)));
        assertEquals(150, e.stageSeconds(), 1e-9);
        assertEquals(50 * Estimate.BAKE_PER_AMPLIFY, e.bakeSeconds(), 1e-9);
        assertEquals(150 + e.bakeSeconds(), e.seconds(), 1e-9);
        assertEquals(100 / e.seconds(), e.fraction(1, 0), 1e-9);
        assertEquals(150 / e.seconds(), e.fraction(1, 1000), 1e-9, "the stages done, the bake not");
        assertEquals(-1, e.fraction(2, 0));
        assertEquals(Math.round(e.bakeSeconds() * 1000), e.timeLeftMs(1, 1000, 0));
        // Stages that ran twice as slow as estimated: so is the bake.
        assertEquals(Math.round(2 * e.bakeSeconds() * 1000), e.bakeMs(300_000));
        assertEquals(Math.round(e.bakeSeconds() * 1000), e.bakeMs(0), "the estimate's own pace when the stages' time is unknown");
        assertEquals(0, e.bakeLeftMs(300_000, 999_000));
        assertEquals(150 / e.seconds(), e.bakeFraction(300_000, 0), 1e-9);
        assertEquals((150 + Estimate.BAKE_HOLD * e.bakeSeconds()) / e.seconds(), e.bakeFraction(300_000, 999_000), 1e-9);
        assertEquals(0, new Estimate(1, List.of(new Estimate.Stage("plates", 1, 100))).bakeSeconds(), "no amplify: no bake share");
    }
}
