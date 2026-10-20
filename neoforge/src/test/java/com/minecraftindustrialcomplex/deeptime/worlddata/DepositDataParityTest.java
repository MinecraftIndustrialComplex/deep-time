// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * P1-15: the Java readers of sites, bodies and events return what the Rust writer wrote, and Java's
 * body evaluation is Rust's. The fixture is a synthetic deposit world
 * ({@code deeptime run --preset deposits_world_4k.json}) and {@code deeptime dump-deposits} over it:
 * every site as Rust sees it, index lookups at 500+ blocks, {@code Body::locate} and the zones that
 * apply at 401 blocks in and around every body (a quarter of them one planet east or west, across
 * the seam), and every forming event's cause chain.
 */
class DepositDataParityTest {

    private static final int MAX_REPORTED = 10;
    /** Rust and Java may differ in the last bits of sin/cos; d and v are compared within this. */
    private static final double TOL = 1e-9;

    static WorldData wd;
    static JsonObject dump;

    @BeforeAll
    static void load() {
        wd = WorldData.open(Fixtures.world("deposits4k"));
        dump = Fixtures.samples("deposits4k");
    }

    @Test
    void sitesMatchTheWriter() {
        Deposits d = wd.deposits();
        JsonArray sites = dump.getAsJsonArray("sites");
        assertEquals(sites.size(), d.sites().sites().size());
        assertTrue(sites.size() > 5, "only " + sites.size() + " sites");
        List<String> bad = new ArrayList<>();
        for (int i = 0; i < sites.size(); i++) {
            JsonObject r = sites.get(i).getAsJsonObject();
            SiteData.SiteRecord j = d.sites().sites().get(i);
            check(bad, j.id() == r.get("id").getAsInt(), "id", j.id());
            check(bad, j.kindName().equals(r.get("kind").getAsString()), "kind", j.id());
            check(bad, j.depositType().equals(r.get("deposit_type").getAsString()), "type", j.id());
            check(bad, Arrays.equals(j.center(), ints(r.getAsJsonArray("center"))), "center", j.id());
            JsonArray aabb = r.getAsJsonArray("aabb");
            check(bad, Arrays.equals(j.min(), ints(aabb.get(0).getAsJsonArray()))
                    && Arrays.equals(j.max(), ints(aabb.get(1).getAsJsonArray())), "aabb", j.id());
            check(bad, j.body() == (r.get("body").isJsonNull() ? SiteData.NONE : r.get("body").getAsInt()), "body", j.id());
            check(bad, j.parent() == (r.get("parent").isJsonNull() ? SiteData.NONE : r.get("parent").getAsInt()), "parent", j.id());
            check(bad, j.formingEvent() == r.get("forming_event").getAsLong(), "forming_event", j.id());
            JsonArray ll = r.getAsJsonArray("lat_lon");
            check(bad, j.lat() == ll.get(0).getAsFloat() && j.lon() == ll.get(1).getAsFloat(), "lat_lon", j.id());
            JsonArray prods = r.getAsJsonArray("products");
            check(bad, prods.size() == j.products().size(), "products", j.id());
            for (int p = 0; p < Math.min(prods.size(), j.products().size()); p++) {
                JsonObject rp = prods.get(p).getAsJsonObject();
                var jp = j.products().get(p);
                check(bad, jp.material().equals(rp.get("material").getAsString()) && jp.grade() == rp.get("grade").getAsFloat()
                        && jp.tonnageT() == rp.get("tonnage_t").getAsDouble() && jp.role().equals(rp.get("role").getAsString()),
                        "product " + p, j.id());
            }
            JsonObject ex = r.getAsJsonObject("extraction");
            check(bad, j.finite() == ex.get("finite").getAsBoolean() && j.amountBase() == ex.get("amount_base").getAsDouble()
                    && (ex.get("coe_vein").isJsonNull() ? j.coeVein() == null : ex.get("coe_vein").getAsString().equals(j.coeVein()))
                    && (ex.get("cdg_charge_mb").isJsonNull() ? j.cdgChargeMb() == SiteData.NONE_U64
                    : j.cdgChargeMb() == ex.get("cdg_charge_mb").getAsLong()), "extraction", j.id());
        }
        assertTrue(bad.isEmpty(), "site mismatches " + bad);
    }

    @Test
    void indexMatchesTheWriter() {
        List<String> bad = new ArrayList<>();
        int checked = 0, nonEmpty = 0;
        for (JsonElement e : dump.getAsJsonObject("index").getAsJsonArray("samples")) {
            JsonArray a = e.getAsJsonArray();
            int x = a.get(0).getAsInt(), z = a.get(1).getAsInt();
            int[] want = ints(a.get(2).getAsJsonArray());
            int[] got = wd.deposits().sites().idsAt(x, z);
            if (!Arrays.equals(want, got) && bad.size() < MAX_REPORTED)
                bad.add("(" + x + ", " + z + "): rust " + Arrays.toString(want) + ", java " + Arrays.toString(got));
            checked++;
            if (want.length > 0)
                nonEmpty++;
        }
        assertTrue(bad.isEmpty(), "index mismatches " + bad);
        assertTrue(checked >= 500 && nonEmpty > 0, "checked " + checked + ", non-empty " + nonEmpty);
    }

    @Test
    void bodiesEvaluateLikeRust() {
        Deposits d = wd.deposits();
        int c = dump.get("circumference").getAsInt();
        JsonArray bodies = dump.getAsJsonArray("bodies");
        assertEquals(bodies.size(), d.bodies().bodies().size());
        List<String> bad = new ArrayList<>();
        int inside = 0, checked = 0, nearBoundary = 0;
        double worst = 0;
        for (int bi = 0; bi < bodies.size(); bi++) {
            JsonObject bo = bodies.get(bi).getAsJsonObject();
            JsonObject rb = bo.getAsJsonObject("body");
            Body jb = d.bodies().bodies().get(bi);
            assertEquals(rb.get("id").getAsInt(), jb.id());
            assertEquals(rb.get("shape").getAsString(), jb.shapeName());
            assertEquals(rb.get("role").getAsString(), jb.role());
            JsonElement age = rb.get("age_ma");
            assertEquals(age == null || age.isJsonNull() ? Float.NaN : age.getAsFloat(), jb.ageMa(),
                    "age of body " + jb.id());
            assertEquals(rb.getAsJsonArray("zones").size(), jb.zones().size(), "zones of body " + jb.id());
            JsonArray aabb = rb.getAsJsonArray("aabb");
            assertTrue(Arrays.equals(jb.min(), ints(aabb.get(0).getAsJsonArray()))
                    && Arrays.equals(jb.max(), ints(aabb.get(1).getAsJsonArray())), "aabb of body " + jb.id());
            for (JsonElement se : bo.getAsJsonArray("samples")) {
                JsonArray s = se.getAsJsonArray();
                int x = s.get(0).getAsInt(), y = s.get(1).getAsInt(), z = s.get(2).getAsInt();
                Body.Location at = jb.locate(c, x, y, z);
                checked++;
                if (s.get(3).isJsonNull()) {
                    if (at != null && at.d() < 1.0 - TOL && bad.size() < MAX_REPORTED)
                        bad.add("body " + jb.id() + " (" + x + "," + y + "," + z + "): rust outside, java d " + at.d());
                    if (at != null)
                        nearBoundary++;
                    continue;
                }
                double rd = s.get(3).getAsDouble(), rv = s.get(4).getAsDouble();
                if (at == null) {
                    if (rd < 1.0 - TOL && bad.size() < MAX_REPORTED)
                        bad.add("body " + jb.id() + " (" + x + "," + y + "," + z + "): rust d " + rd + ", java outside");
                    nearBoundary++;
                    continue;
                }
                inside++;
                worst = Math.max(worst, Math.max(Math.abs(at.d() - rd), Math.abs(at.v() - rv)));
                List<Integer> want = new ArrayList<>();
                s.get(5).getAsJsonArray().forEach(v -> want.add(v.getAsInt()));
                List<Integer> got = new ArrayList<>();
                for (int zi = 0; zi < jb.zones().size(); zi++)
                    if (jb.zones().get(zi).contains(at.d(), at.v()))
                        got.add(zi);
                if (!want.equals(got) && !nearZoneEdge(jb, at) && bad.size() < MAX_REPORTED)
                    bad.add("body " + jb.id() + " (" + x + "," + y + "," + z + "): zones rust " + want + ", java " + got);
            }
        }
        System.out.printf("[parity] bodies=%d samples=%d inside=%d boundary=%d worst|Δd|,|Δv|=%.3g%n",
                bodies.size(), checked, inside, nearBoundary, worst);
        assertTrue(bad.isEmpty(), "body mismatches " + bad);
        assertTrue(worst <= TOL, "d or v differ by " + worst);
        assertTrue(inside > bodies.size(), "only " + inside + " samples inside bodies");
    }

    private static boolean nearZoneEdge(Body b, Body.Location at) {
        for (Body.Zone z : b.zones())
            for (double e : new double[]{z.d0(), z.d1()})
                if (Math.abs(at.d() - e) < TOL)
                    return true;
        return false;
    }

    @Test
    void eventsAndChainsMatchTheWriter() {
        EventData ev = wd.deposits().events();
        JsonObject e = dump.getAsJsonObject("events");
        assertEquals(e.get("count").getAsInt(), ev.size());
        List<String> bad = new ArrayList<>();
        for (JsonElement fe : e.getAsJsonArray("forming")) {
            JsonObject r = fe.getAsJsonObject();
            EventData.EventRecord j = ev.get(r.get("id").getAsLong());
            assertNotNull(j, "forming event " + r.get("id"));
            check(bad, j.kind().equals(r.get("kind").getAsString()) && j.source().equals(r.get("source").getAsString())
                    && j.tStart() == r.get("t_start").getAsDouble() && j.tEnd() == r.get("t_end").getAsDouble(), "event", (int) j.seq());
            check(bad, j.causes().length == r.getAsJsonArray("causes").size(), "causes", (int) j.seq());
            JsonObject place = r.getAsJsonObject("place");
            check(bad, j.placeType().equals(place.get("type").getAsString()), "place", (int) j.seq());
            JsonObject params = r.getAsJsonObject("params");
            check(bad, j.params().keySet().equals(params.keySet()), "params", (int) j.seq());
        }
        for (JsonElement ce : e.getAsJsonArray("chains")) {
            JsonObject r = ce.getAsJsonObject();
            long[] want = r.getAsJsonArray("chain").asList().stream().mapToLong(JsonElement::getAsLong).toArray();
            long[] got = ev.causeChain(r.get("event").getAsLong(), 3).stream().mapToLong(EventData.EventRecord::id).toArray();
            check(bad, Arrays.equals(want, got), "chain of site " + r.get("site"), r.get("site").getAsInt());
        }
        assertTrue(bad.isEmpty(), "event mismatches " + bad);
    }

    private static void check(List<String> bad, boolean ok, String what, int id) {
        if (!ok && bad.size() < MAX_REPORTED)
            bad.add(what + " of " + id);
    }

    private static int[] ints(JsonArray a) {
        int[] out = new int[a.size()];
        for (int i = 0; i < out.length; i++)
            out[i] = a.get(i).getAsInt();
        return out;
    }

    /** The fixture folder exists (a guard with a clear message when Gradle's task did not run). */
    @Test
    void fixtureIsTheDepositWorld() {
        Path p = Fixtures.world("deposits4k");
        assertTrue(p.resolve("sites/sites.dtsr").toFile().isFile(), p + " has no sites/");
        assertEquals(4096, wd.circumference());
        assertTrue(wd.has("elevation") && wd.has("strata/horizons"), "the deposit world has no terrain layers");
    }
}
