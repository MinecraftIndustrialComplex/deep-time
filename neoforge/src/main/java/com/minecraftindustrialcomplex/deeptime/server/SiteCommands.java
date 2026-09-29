// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.api.DeepTimeSites;
import com.minecraftindustrialcomplex.deeptime.api.GeoEvent;
import com.minecraftindustrialcomplex.deeptime.api.Site;
import com.minecraftindustrialcomplex.deeptime.compat.SiteChunks;
import com.minecraftindustrialcomplex.deeptime.ore.ClaimOresBiomeModifier;
import com.minecraftindustrialcomplex.deeptime.ore.ClaimRules;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worldgen.DepositFeature;
import com.minecraftindustrialcomplex.deeptime.worldgen.GeologicChunkGenerator;
import com.minecraftindustrialcomplex.deeptime.worldgen.LevelSites;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.commands.arguments.blocks.BlockStateParser;
import net.minecraft.core.BlockPos;
import net.minecraft.network.chat.Component;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.phys.AABB;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * The site commands of plan §7.16 (decisions log Q13: "Why here?" through the site API,
 * {@code /deeptime why} and the viewer): {@code probe site <x> <y> <z>},
 * {@code sites near <x> <z> <radius>}, {@code why <siteId>}, plus {@code probe claims} (what
 * {@code deeptime:claim_ores} removes or would remove). Each prints one {@code [deeptime-probe]}
 * JSON line; {@code why} also prints the chain as readable lines.
 */
public final class SiteCommands {

    private SiteCommands() {}

    static JsonObject siteJson(Site s) {
        JsonObject o = new JsonObject();
        o.addProperty("id", s.id());
        o.addProperty("kind", s.kind().id());
        o.addProperty("type", s.depositType().toString());
        JsonArray c = new JsonArray();
        c.add(s.center().getX());
        c.add(s.center().getY());
        c.add(s.center().getZ());
        o.add("center", c);
        JsonArray bb = new JsonArray();
        var b = s.bounds();
        for (int v : new int[]{b.minX(), b.minY(), b.minZ(), b.maxX(), b.maxY(), b.maxZ()})
            bb.add(v);
        o.add("bounds", bb);
        JsonArray products = new JsonArray();
        s.products().forEach(p -> {
            JsonObject po = new JsonObject();
            po.addProperty("material", p.material().toString());
            po.addProperty("grade", p.grade());
            po.addProperty("tonnage_t", p.tonnageT());
            po.addProperty("role", p.role());
            products.add(po);
        });
        o.add("products", products);
        o.addProperty("forming_event", s.formingEvent());
        s.parent().ifPresent(p -> o.addProperty("parent", p));
        s.extraction().coeVein().ifPresent(v -> o.addProperty("coe_vein", v));
        s.extraction().cdgChargeMb().ifPresent(v -> o.addProperty("cdg_charge_mb", v));
        return o;
    }

    static JsonObject eventJson(GeoEvent e) {
        JsonObject o = new JsonObject();
        o.addProperty("id", e.id());
        o.addProperty("text", e.text());
        o.addProperty("kind", e.kind().toString());
        o.addProperty("source", e.source().toString());
        o.addProperty("t_start", e.tStart());
        o.addProperty("t_end", e.tEnd());
        o.addProperty("magnitude", e.magnitude());
        JsonObject place = new JsonObject();
        place.addProperty("type", e.place().type());
        if (e.place().type().equals("point")) {
            place.addProperty("lat", e.place().lat());
            place.addProperty("lon", e.place().lon());
        } else if (!e.place().type().equals("global")) {
            place.addProperty("ref", e.place().ref());
        }
        o.add("place", place);
        JsonArray causes = new JsonArray();
        e.causes().forEach(causes::add);
        o.add("causes", causes);
        JsonObject params = new JsonObject();
        e.params().forEach((k, v) -> {
            if (v instanceof Number n) params.addProperty(k, n);
            else if (v instanceof GeoEvent.IdRef id) params.addProperty(k, "id:" + id.id());
            else params.addProperty(k, String.valueOf(v));
        });
        o.add("params", params);
        return o;
    }

    /** {@code /deeptime probe site <x> <y> <z>}: the site and body at a block, the zones there, the block. */
    public static int probeSite(CommandSourceStack source, int x, int y, int z) {
        ServerLevel level = source.getLevel();
        JsonObject o = new JsonObject();
        o.addProperty("probe", "site");
        o.addProperty("level", level.dimension().location().toString());
        o.addProperty("x", x);
        o.addProperty("y", y);
        o.addProperty("z", z);
        DeepTimeSites sites = DeepTimeSites.of(level);
        o.addProperty("deepTimeLevel", sites.isDeepTimeLevel());
        BlockPos pos = new BlockPos(x, y, z);
        var chunk = level.getChunkSource().getChunkNow(x >> 4, z >> 4);
        if (chunk != null)
            o.addProperty("block", BlockStateParser.serialize(chunk.getBlockState(pos)));
        sites.at(pos).ifPresent(s -> o.add("site", siteJson(s)));
        if (sites instanceof LevelSites ls && level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g) {
            int c = g.dataRef().circumferenceBlocks();
            JsonArray bodies = new JsonArray();
            for (Body b : ls.deposits().bodiesIn(x, y, z, x, y, z)) {
                var at = b.locate(c, x, y, z);
                if (at == null)
                    continue;
                JsonObject bo = new JsonObject();
                bo.addProperty("body", b.id());
                bo.addProperty("site", b.site());
                bo.addProperty("role", b.role());
                bo.addProperty("shape", b.shapeName());
                bo.addProperty("d", at.d());
                bo.addProperty("v", at.v());
                if (b.rockClass() != null)
                    bo.addProperty("rock_class", b.rockClass());
                JsonArray zones = new JsonArray();
                for (Body.Zone zn : b.zones())
                    if (zn.contains(at.d(), at.v()))
                        zones.add(zn.name() + (zn.material() != null ? " (" + zn.material() + ", p " + zn.density() + ")" : ""));
                bo.add("zones", zones);
                String m = DepositFeature.materialAt(b, c, x, y, z);
                if (m != null)
                    bo.addProperty("ore_here", m);
                bodies.add(bo);
            }
            o.add("bodies", bodies);
        }
        return DeepTimeCommands.emit(source, o);
    }

    /** {@code /deeptime sites near <x> <z> <radius>}: the sites whose centre lies within the radius, nearest first. */
    public static int sitesNear(CommandSourceStack source, int x, int z, int radius) {
        ServerLevel level = source.getLevel();
        DeepTimeSites sites = DeepTimeSites.of(level);
        JsonObject o = new JsonObject();
        o.addProperty("probe", "sites-near");
        o.addProperty("level", level.dimension().location().toString());
        o.addProperty("x", x);
        o.addProperty("z", z);
        o.addProperty("radius", radius);
        o.addProperty("deepTimeLevel", sites.isDeepTimeLevel());
        int c = level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g ? g.dataRef().circumferenceBlocks() : 0;
        List<Map.Entry<Double, Site>> found = new ArrayList<>();
        for (Site s : sites.within(new AABB(x - radius, -1e6, z - radius, x + radius, 1e6, z + radius))) {
            double d = distance(c, x, z, s.center().getX(), s.center().getZ());
            if (d <= radius)
                found.add(Map.entry(d, s));
        }
        found.sort(Comparator.comparingDouble(Map.Entry::getKey));
        JsonArray arr = new JsonArray();
        for (var e : found) {
            JsonObject so = siteJson(e.getValue());
            so.addProperty("distance", Math.round(e.getKey() * 10) / 10.0);
            // P1-16: the chunk its ore body fills most (the chunk to probe with `probe coe` / `probe cdg`).
            if (sites instanceof LevelSites ls) {
                var best = SiteChunks.footprint(ls, e.getValue()).best();
                if (best != null) {
                    JsonArray bc = new JsonArray();
                    bc.add(best.x);
                    bc.add(best.z);
                    so.add("ore_chunk", bc);
                }
            }
            arr.add(so);
        }
        o.addProperty("count", found.size());
        o.add("sites", arr);
        return DeepTimeCommands.emit(source, o);
    }

    /** Horizontal distance on a planet of circumference c (x wraps; c = 0: no wrap). */
    public static double distance(int c, double x0, double z0, double x1, double z1) {
        double dx = x1 - x0;
        if (c > 0) {
            dx = Math.floorMod((long) Math.round(dx), (long) c);
            dx = Math.min(dx, c - dx);
        }
        return Math.hypot(dx, z1 - z0);
    }

    /** {@code /deeptime why <siteId>}: the site, its forming event and the cause chain (4 links). */
    public static int why(CommandSourceStack source, int siteId) {
        ServerLevel level = source.getLevel();
        DeepTimeSites sites = DeepTimeSites.of(level);
        JsonObject o = new JsonObject();
        o.addProperty("probe", "why");
        o.addProperty("level", level.dimension().location().toString());
        o.addProperty("site", siteId);
        var site = sites.byId(siteId);
        if (site.isEmpty()) {
            o.addProperty("error", sites.isDeepTimeLevel() ? "no site " + siteId + " in this level" : "not a Deep Time level");
            return DeepTimeCommands.emit(source, o);
        }
        o.add("detail", siteJson(site.get()));
        var formed = sites.formingEvent(site.get());
        List<String> lines = new ArrayList<>();
        lines.add(String.format(Locale.ROOT, "Site %d: %s (%s) at %d %d %d", siteId, site.get().depositType(),
                site.get().kind().id(), site.get().center().getX(), site.get().center().getY(), site.get().center().getZ()));
        if (formed.isPresent()) {
            o.add("forming_event", eventJson(formed.get()));
            lines.add("  formed: " + describe(formed.get()));
            JsonArray chain = new JsonArray();
            for (GeoEvent e : sites.causeChain(formed.get(), 4)) {
                chain.add(eventJson(e));
                lines.add("  <- " + describe(e));
            }
            o.add("chain", chain);
        } else {
            o.addProperty("error", "the forming event " + site.get().formingEvent() + " is not in the event log");
        }
        for (String l : lines)
            source.sendSuccess(() -> Component.literal(l), false);
        return DeepTimeCommands.emit(source, o);
    }

    static String describe(GeoEvent e) {
        String when = e.tStart() == e.tEnd() ? ma(e.tStart()) : ma(e.tStart()) + " to " + ma(e.tEnd());
        String why = "";
        if (e.params().containsKey("limiting"))
            why = ", limiting " + e.params().get("limiting");
        if (e.params().containsKey("feature"))
            why += ", " + e.params().get("feature");
        return e.kind().getPath().replace('_', ' ') + " " + when + " [" + e.text() + "]" + why;
    }

    static String ma(double years) {
        return String.format(Locale.ROOT, "%.1f Ma", -years / 1e6);
    }

    /**
     * {@code /deeptime probe claims}: whether this world claims ores, the claimed materials, and the
     * placed features removed (a Deep Time world) or that would be removed (a dry run elsewhere).
     */
    public static int probeClaims(CommandSourceStack source) {
        var server = source.getServer();
        JsonObject o = new JsonObject();
        o.addProperty("probe", "claims");
        boolean on = ClaimOresBiomeModifier.deepTimeWorld(server);
        o.addProperty("active", on);
        var rules = ClaimRules.dryRun(server);
        JsonArray materials = new JsonArray();
        rules.materials().forEach(m -> materials.add(m.toString()));
        o.add("materials", materials);
        var plan = ClaimRules.byVerdict(rules.planAll(server.registryAccess()));
        for (var e : plan.entrySet()) {
            if (e.getKey() == ClaimRules.Verdict.KEEP)
                continue;
            JsonArray a = new JsonArray();
            e.getValue().forEach(id -> a.add(id.toString()));
            o.add("would_remove_" + e.getKey().name().toLowerCase(Locale.ROOT), a);
        }
        JsonArray removed = new JsonArray();
        synchronized (ClaimRules.REMOVED) {
            ClaimRules.REMOVED.forEach((id, v) -> removed.add(id + " " + v.name().toLowerCase(Locale.ROOT)));
        }
        o.add("removed", removed);
        return DeepTimeCommands.emit(source, o);
    }
}
