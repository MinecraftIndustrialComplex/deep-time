// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.server;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import com.minecraftindustrialcomplex.deeptime.compat.SiteChunks;
import com.minecraftindustrialcomplex.deeptime.compat.cdg.CdgAccess;
import com.minecraftindustrialcomplex.deeptime.compat.cdg.CdgBridge;
import com.minecraftindustrialcomplex.deeptime.compat.coe.CoeAccess;
import com.minecraftindustrialcomplex.deeptime.compat.coe.CoeBridge;
import com.minecraftindustrialcomplex.deeptime.worldgen.LevelSites;
import net.minecraft.commands.CommandSourceStack;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;

/**
 * The COE and CDG probes of plan §7.16 (P1-16): {@code /deeptime probe coe <chunkX> <chunkZ>} and
 * {@code /deeptime probe cdg <chunkX> <chunkZ>}. Each prints one {@code [deeptime-probe]} JSON line
 * with what Deep Time decides for the chunk ({@link CoeBridge}, {@link CdgBridge}) and, when the mod
 * is installed, what the mod itself then holds: COE's stored vein (read like a drill reads it, which
 * loads or generates the chunk and populates its vein on first access) and CDG's base and current
 * oil. {@code agrees} is true when the two match.
 */
public final class CompatCommands {

    private CompatCommands() {}

    public static int probeCoe(CommandSourceStack source, int cx, int cz) {
        ServerLevel level = source.getLevel();
        JsonObject o = header("coe", level, cx, cz);
        o.addProperty("coe_installed", CoeAccess.installed());
        LevelSites sites = LevelSites.of(level);
        var siteVein = CoeBridge.siteVein(level, cx, cz);
        if (sites.isDeepTimeLevel()) {
            JsonArray claimed = new JsonArray();
            CoeBridge.claimedVeins(level).forEach(v -> claimed.add(v.toString()));
            o.add("claimed_veins", claimed);
            siteVein.ifPresent(v -> {
                JsonObject so = new JsonObject();
                so.addProperty("site", v.site().id());
                so.addProperty("type", v.site().depositType().toString());
                so.addProperty("vein", v.vein().toString());
                so.addProperty("ore_blocks_in_chunk", Math.round(v.volume()));
                var fp = SiteChunks.footprint(sites, v.site());
                so.addProperty("ore_blocks_total", Math.round(fp.total()));
                so.addProperty("chunks", fp.byChunk().size());
                so.addProperty("tonnage_t", v.site().products().isEmpty() ? 0 : v.site().products().get(0).tonnageT());
                so.addProperty("quantile", v.randomMul());
                o.add("site_vein", so);
            });
        }
        CoeAccess.load().ifPresent(coe -> {
            var state = coe.chunkState(level.getChunk(cx, cz));
            JsonObject co = new JsonObject();
            co.addProperty("vein", state.vein().map(ResourceLocation::toString).orElse(null));
            co.addProperty("random_mul", state.randomMul());
            co.addProperty("amount", state.amount());
            co.addProperty("remaining", state.remaining());
            co.addProperty("infinite", state.infinite());
            o.add("coe", co);
            o.addProperty("coe_config", coe.config());
            if (sites.isDeepTimeLevel()) {
                boolean agrees = siteVein.map(v -> state.vein().equals(java.util.Optional.of(v.vein()))
                                && state.randomMul() == v.randomMul())
                        .orElse(state.vein().map(id -> !CoeBridge.claimedVeins(level).contains(id)).orElse(true));
                o.addProperty("agrees", agrees);
            }
        });
        return DeepTimeCommands.emit(source, o);
    }

    public static int probeCdg(CommandSourceStack source, int cx, int cz) {
        ServerLevel level = source.getLevel();
        JsonObject o = header("cdg", level, cx, cz);
        o.addProperty("cdg_installed", CdgAccess.installed());
        var cdg = CdgAccess.load();
        int threshold = cdg.map(CdgAccess::infiniteThreshold).orElse(CdgBridge.DEFAULT_INFINITE_THRESHOLD);
        var charge = CdgBridge.charge(level, cx, cz);
        charge.ifPresent(c -> {
            JsonObject so = new JsonObject();
            so.addProperty("site", c.site().id());
            so.addProperty("type", c.site().depositType().toString());
            so.addProperty("charge_mb", c.chargeMb());
            so.addProperty("cdg_scale", c.scale());
            so.addProperty("share", c.share());
            so.addProperty("amount_mb", c.amountMb());
            o.add("trap", so);
        });
        var expected = CdgBridge.baseOilAmount(level, new ChunkPos(cx, cz), threshold);
        if (expected.isPresent())
            o.addProperty("deep_time_amount", expected.getAsInt());
        cdg.ifPresent(a -> {
            JsonObject co = new JsonObject();
            int base = a.base(level, new ChunkPos(cx, cz));
            co.addProperty("base", base);
            co.addProperty("amount", a.amount(level, new ChunkPos(cx, cz)));
            co.addProperty("infinite_threshold", threshold);
            o.add("cdg", co);
            o.addProperty("cdg_config", a.config());
            if (expected.isPresent())
                o.addProperty("agrees", base == expected.getAsInt());
        });
        return DeepTimeCommands.emit(source, o);
    }

    private static JsonObject header(String probe, ServerLevel level, int cx, int cz) {
        JsonObject o = new JsonObject();
        o.addProperty("probe", probe);
        o.addProperty("level", level.dimension().location().toString());
        JsonArray chunk = new JsonArray();
        chunk.add(cx);
        chunk.add(cz);
        o.add("chunk", chunk);
        o.addProperty("deepTimeLevel", LevelSites.of(level).isDeepTimeLevel());
        return o;
    }
}
