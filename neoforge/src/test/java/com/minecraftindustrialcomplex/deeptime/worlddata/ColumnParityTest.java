// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Java↔Rust column parity (plan §9.2, P1-08): {@code deeptime parity} evaluated 500 columns of a
 * real baked planet ({@code tests/presets/planet_world_4k.json}, Gradle {@code testWorldPlanet4k})
 * with Rust's column evaluator and wrote {@code parity/columns.json}; {@link Columns#sample}, the
 * Java reader's evaluator, must give exactly the same for every one: the terrain height (the
 * elevation, the detail term and their sum, bit for bit), the top block, the water, the horizon
 * stack, the runs through bodies and the rock runs down to the floor.
 */
class ColumnParityTest {

    static Path world() {
        return Fixtures.world("planet4k");
    }

    static JsonObject fixture() throws IOException {
        return JsonParser.parseString(Files.readString(world().resolve("parity/columns.json"), StandardCharsets.UTF_8))
                .getAsJsonObject();
    }

    private static boolean same(double a, double b) {
        return a == b || (Double.isNaN(a) && Double.isNaN(b));
    }

    private static double number(JsonElement e) {
        return e == null || e.isJsonNull() ? Double.NaN : e.getAsDouble();
    }

    /** The differences between a Java sample and Rust's JSON, empty when they agree. */
    static List<String> differences(Columns.Sample s, JsonObject r) {
        List<String> out = new ArrayList<>();
        if (s.wx() != r.get("wx").getAsInt())
            out.add("wx " + s.wx() + " vs " + r.get("wx"));
        if (!same(s.elevation(), number(r.get("elevation"))))
            out.add("elevation " + s.elevation() + " vs " + r.get("elevation"));
        if (!same(s.detail(), number(r.get("detail"))))
            out.add("detail " + s.detail() + " vs " + r.get("detail"));
        if (!same(s.surface(), number(r.get("surface"))))
            out.add("surface " + s.surface() + " vs " + r.get("surface"));
        if (s.topY() != r.get("top_y").getAsInt())
            out.add("top_y " + s.topY() + " vs " + r.get("top_y"));
        if (!same(s.water(), number(r.get("water"))))
            out.add("water " + s.water() + " vs " + r.get("water"));
        JsonArray h = r.getAsJsonArray("horizons");
        if (h.size() != s.horizons().length)
            out.add("horizons " + s.horizons().length + " vs " + h.size());
        else
            for (int i = 0; i < h.size(); i++) {
                JsonArray e = h.get(i).getAsJsonArray();
                if (s.horizons()[i][0] != e.get(0).getAsInt() || s.horizons()[i][1] != e.get(1).getAsInt()
                        || s.horizons()[i][2] != e.get(2).getAsInt())
                    out.add("horizon " + i + " [" + s.horizons()[i][0] + ", " + s.horizons()[i][1] + "] vs " + e);
            }
        JsonArray b = r.getAsJsonArray("bodies");
        if (b.size() != s.bodies().size())
            out.add("body runs " + s.bodies() + " vs " + b);
        else
            for (int i = 0; i < b.size(); i++) {
                JsonArray e = b.get(i).getAsJsonArray();
                var j = s.bodies().get(i);
                if (j.id() != (int) e.get(0).getAsLong() || j.top() != e.get(1).getAsInt() || j.bottom() != e.get(2).getAsInt())
                    out.add("body run " + i + " " + j + " vs " + e);
            }
        out.addAll(hydroDifferences(s.hydro(), r.getAsJsonObject("hydro")));
        JsonArray k = r.getAsJsonArray("rock");
        if (k.size() != s.rock().size())
            out.add("rock runs " + s.rock().size() + " vs " + k.size() + ": " + s.rock() + " vs " + k);
        else
            for (int i = 0; i < k.size(); i++) {
                JsonArray e = k.get(i).getAsJsonArray();
                var j = s.rock().get(i);
                String cls = e.get(2).isJsonNull() ? null : e.get(2).getAsString();
                if (j.top() != e.get(0).getAsInt() || j.bottom() != e.get(1).getAsInt()
                        || !java.util.Objects.equals(j.rockClass(), cls))
                    out.add("rock run " + i + " " + j + " vs " + e);
            }
        return out;
    }

    private static Integer optInt(JsonElement e) {
        return e == null || e.isJsonNull() ? null : e.getAsInt();
    }

    /** The differences in the {@code hydro} object (P1-22, {@link Water}). */
    static List<String> hydroDifferences(Water.Column h, JsonObject r) {
        List<String> out = new ArrayList<>();
        if (r == null) {
            out.add("hydro missing from the fixture");
            return out;
        }
        if (!h.kind().id.equals(r.get("kind").getAsString()))
            out.add("hydro.kind " + h.kind().id + " vs " + r.get("kind"));
        if (!same(h.surface(), number(r.get("surface"))))
            out.add("hydro.surface " + h.surface() + " vs " + r.get("surface"));
        if (h.ground() != r.get("ground").getAsInt())
            out.add("hydro.ground " + h.ground() + " vs " + r.get("ground"));
        if (!same(h.level() == null ? Double.NaN : h.level(), number(r.get("level"))))
            out.add("hydro.level " + h.level() + " vs " + r.get("level"));
        String[] keys = {"top", "bed", "min_ground", "max_ground", "seal_from"};
        Integer[] mine = {h.top(), h.bed(), h.minGround(), h.maxGround(), h.sealFrom()};
        for (int i = 0; i < keys.length; i++)
            if (!java.util.Objects.equals(mine[i], optInt(r.get(keys[i]))))
                out.add("hydro." + keys[i] + " " + mine[i] + " vs " + r.get(keys[i]));
        if (h.riverBiome() != r.get("river_biome").getAsBoolean())
            out.add("hydro.river_biome " + h.riverBiome() + " vs " + r.get("river_biome"));
        JsonElement rv = r.get("river");
        if ((h.river() == null) != (rv == null || rv.isJsonNull()))
            out.add("hydro.river " + h.river() + " vs " + rv);
        else if (h.river() != null) {
            JsonObject o = rv.getAsJsonObject();
            Rivers.Hit x = h.river();
            if (x.seg() != o.get("seg").getAsInt() || !same(x.t(), o.get("t").getAsDouble()) || !same(x.d(), o.get("d").getAsDouble())
                    || !same(x.w(), o.get("w").getAsDouble()) || !same(x.hw(), o.get("hw").getAsDouble())
                    || !same(x.depth(), o.get("depth").getAsDouble()))
                out.add("hydro.river " + x + " vs " + o);
        }
        return out;
    }

    /**
     * The fixture's world has folds and faults (P2-01), the Java reader reads them, and they change
     * exactly the columns Rust says they change: a good share of the fixture, so the deformed column
     * rule ({@link Structure#deformedStack}) is what {@link #everyColumnOfTheFixtureIsEqual} compares.
     */
    @Test
    void foldsAndFaultsChangeTheSameColumns() throws IOException {
        JsonObject doc = fixture();
        int events = doc.get("structure_events").getAsInt();
        assertTrue(events > 0, "the test planet has no structural events");
        WorldData wd = WorldData.open(world());
        {
            Structure st = wd.structure();
            assertTrue(st != null && st.size() == events, "Java reads " + (st == null ? 0 : st.size()) + " of " + events + " events");
            RasterSampler strata = wd.layer(WorldData.HORIZONS);
            Columns.Stack plain = new Columns.Stack(strata.stackDepth()), bent = new Columns.Stack(strata.stackDepth());
            Structure.Scratch scratch = new Structure.Scratch();
            int deformed = 0;
            JsonArray columns = doc.getAsJsonArray("columns");
            for (JsonElement e : columns) {
                JsonObject c = e.getAsJsonObject();
                int x = c.get("x").getAsInt(), z = c.get("z").getAsInt();
                Columns.stack(wd, strata, x, z, plain);
                Columns.deformedStack(wd, strata, x, z, bent, scratch);
                boolean same = plain.count == bent.count;
                for (int h = 0; same && h < plain.count; h++)
                    same = plain.tops[h] == bent.tops[h] && plain.classes[h] == bent.classes[h]
                            && plain.units[h] == bent.units[h];
                if (!same)
                    deformed++;
            }
            assertEquals(doc.get("deformed_columns").getAsInt(), deformed, "columns the events change");
            assertTrue(deformed * 10 >= columns.size(), deformed + " of " + columns.size() + " columns are deformed");
        }
    }

    @Test
    void everyColumnOfTheFixtureIsEqual() throws IOException {
        JsonObject doc = fixture();
        WorldData wd = WorldData.open(world());
        assertEquals(FormatVersion.CURRENT, doc.get("format_version").getAsString());
        assertEquals(wd.circumference(), doc.get("circumference").getAsInt());
        assertTrue(wd.detailNoise() != null, "the real planet has a detail term");
        JsonArray cols = doc.getAsJsonArray("columns");
        assertEquals(500, cols.size(), "plan §9.2: 500 columns");
        int equal = 0, bodies = 0, water = 0, detailed = 0, rivers = 0, lakes = 0, banks = 0;
        List<String> bad = new ArrayList<>();
        for (JsonElement e : cols) {
            JsonObject r = e.getAsJsonObject();
            int x = r.get("x").getAsInt(), z = r.get("z").getAsInt();
            Columns.Sample s = Columns.sample(wd, x, z);
            List<String> diff = differences(s, r);
            if (diff.isEmpty())
                equal++;
            else if (bad.size() < 10)
                bad.add("(" + x + ", " + z + "): " + String.join("; ", diff));
            if (!s.bodies().isEmpty())
                bodies++;
            if (!Double.isNaN(s.water()))
                water++;
            if (s.detail() != 0.0)
                detailed++;
            switch (s.hydro().kind()) {
                case RIVER -> rivers++;
                case LAKE -> lakes++;
                case BANK -> banks++;
                default -> {}
            }
        }
        System.out.println("[junit] ColumnParityTest columns=" + cols.size() + " equal=" + equal
                + " throughBodies=" + bodies + " withWater=" + water + " withDetail=" + detailed
                + " river=" + rivers + " lake=" + lakes + " bank=" + banks);
        assertTrue(bad.isEmpty(), (cols.size() - equal) + " columns differ:\n" + String.join("\n", bad));
        assertTrue(bodies > 0, "some columns pass through bodies");
        assertTrue(detailed > 0, "some columns have a detail term");
        assertTrue(rivers > 0 && banks > 0, "the fixture reaches rivers and their banks (P1-22)");
    }

    /**
     * P1-22 on the synthetic lake world ({@code lake1024}, the GameTest lake's data): its columns,
     * lake water and shores included, are Rust's too.
     */
    @Test
    void theLakeWorldIsEqual() throws IOException {
        Path w = Fixtures.world("lake1024");
        JsonObject doc = JsonParser.parseString(Files.readString(w.resolve("parity/columns.json"), StandardCharsets.UTF_8))
                .getAsJsonObject();
        WorldData wd = WorldData.open(w);
        int equal = 0, lakes = 0, banks = 0;
        List<String> bad = new ArrayList<>();
        JsonArray cols = doc.getAsJsonArray("columns");
        for (JsonElement e : cols) {
            JsonObject r = e.getAsJsonObject();
            int x = r.get("x").getAsInt(), z = r.get("z").getAsInt();
            Columns.Sample s = Columns.sample(wd, x, z);
            List<String> diff = differences(s, r);
            if (diff.isEmpty())
                equal++;
            else if (bad.size() < 10)
                bad.add("(" + x + ", " + z + "): " + String.join("; ", diff));
            if (s.hydro().kind() == Water.Kind.LAKE)
                lakes++;
            if (s.hydro().kind() == Water.Kind.BANK)
                banks++;
        }
        System.out.println("[junit] ColumnParityTest lake1024 columns=" + cols.size() + " equal=" + equal + " lake=" + lakes + " bank=" + banks);
        assertTrue(bad.isEmpty(), (cols.size() - equal) + " columns differ:\n" + String.join("\n", bad));
        assertTrue(lakes > 0, "the fixture reaches the lake");
    }

    /** The terrain height the generator reads is the evaluator's surface. */
    @Test
    void theGeneratorsHeightIsTheSurface() throws IOException {
        JsonObject doc = fixture();
        WorldData wd = WorldData.open(world());
        int c = wd.circumference();
        for (JsonElement e : doc.getAsJsonArray("columns")) {
            JsonObject r = e.getAsJsonObject();
            int x = r.get("x").getAsInt(), z = r.get("z").getAsInt();
            int wx = Math.floorMod(x + c / 2, c) - c / 2;
            assertTrue(same(wd.height(wx, z), number(r.get("surface"))), "(" + x + ", " + z + ")");
        }
    }
}
