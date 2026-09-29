// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.coe;

import com.minecraftindustrialcomplex.deeptime.api.Site;
import com.minecraftindustrialcomplex.deeptime.compat.SiteChunks;
import com.minecraftindustrialcomplex.deeptime.ore.OreData;
import com.minecraftindustrialcomplex.deeptime.worldgen.LevelSites;
import net.minecraft.core.RegistryAccess;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;

import java.util.Collections;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.TreeSet;
import java.util.WeakHashMap;

/**
 * Create Ore Excavation (MIT; plan §7.14, P1-16) as Deep Time decides it, in vanilla types only: the
 * mixins in {@code mixin.compat.createoreexcavation} (applied only with COE) and the probes call
 * this; nothing here names a COE class.
 *
 * <ul>
 *   <li><b>A site is its vein.</b> In a Deep Time level, a chunk that a site's ore body reaches
 *       ({@link SiteChunks}; the site whose ore fills the chunk most when several do) is the vein
 *       of the site's deposit type ({@code minecraft.coe_vein}), whatever COE's noise grid says.
 *       The deposit type is read from the {@code deeptime:deposit_type} registry at run time, so a
 *       pack that overrides the type in a datapack remaps the vein; a type the registry lacks falls
 *       back to the vein the site record was written with. A site without products (all of its ore
 *       leached or eroded) has no vein.</li>
 *   <li><b>Claimed veins come only from sites.</b> The veins Deep Time owns are the loaded deposit
 *       types' {@code coe_vein}s plus the veins the level's site records name; COE's own noise
 *       ({@code OreVeinGenerator.pick} and the vein finder's search grid) does not hand them out in
 *       a Deep Time level. Other veins (diamond, lapis, redstone, …) and other levels are COE's.</li>
 *   <li><b>The site's size sets the amount.</b> {@link #quantile}: the site's ore tonnage (after
 *       erosion and leaching) as a quantile of its type's tonnage lognormal, which becomes COE's
 *       {@code randomMul}, so a vein holds {@code round(((max − min)·q + min)·finiteAmountBase)}
 *       when finite.</li>
 * </ul>
 */
public final class CoeBridge {

    private CoeBridge() {}

    /** Create Ore Excavation's mod id (also the mixin package that COE gates). */
    public static final String MOD_ID = "createoreexcavation";

    /**
     * What Deep Time says about one chunk.
     *
     * @param site      the site whose ore fills the chunk most among the sites with a vein
     * @param vein      its vein recipe id
     * @param volume    its ore-body blocks in the chunk
     * @param randomMul COE's {@code randomMul} for the chunk ({@link #quantile})
     */
    public record SiteVein(Site site, ResourceLocation vein, double volume, float randomMul) {}

    private static final Map<RegistryAccess, Set<ResourceLocation>> TYPE_VEINS = Collections.synchronizedMap(new WeakHashMap<>());

    /** The vein {@code site} feeds: its type's {@code coe_vein} (registry first), none without products. */
    public static Optional<ResourceLocation> veinOf(RegistryAccess access, Site site) {
        if (site.products().isEmpty())
            return Optional.empty();
        Optional<String> id = OreData.depositType(access, site.depositType())
                .map(t -> t.minecraft().coeVein())
                .orElse(site.extraction().coeVein());
        return id.map(ResourceLocation::tryParse);
    }

    /** The site vein of chunk (cx, cz); empty outside Deep Time levels and where no site with a vein reaches. */
    public static Optional<SiteVein> siteVein(ServerLevel level, int cx, int cz) {
        LevelSites sites = LevelSites.of(level);
        if (!sites.isDeepTimeLevel())
            return Optional.empty();
        RegistryAccess access = level.registryAccess();
        return SiteChunks.covering(sites, cx, cz, s -> veinOf(access, s).isPresent()).map(s -> {
            int ccx = SiteChunks.canonicalChunkX(sites.deposits().circumference(), cx);
            return new SiteVein(s, veinOf(access, s).orElseThrow(), SiteChunks.footprint(sites, s).in(ccx, cz),
                    quantile(access, s));
        });
    }

    /** The veins COE's noise must not hand out in {@code level}, sorted: none outside Deep Time levels. */
    public static Set<ResourceLocation> claimedVeins(ServerLevel level) {
        LevelSites sites = LevelSites.of(level);
        if (!sites.isDeepTimeLevel())
            return Set.of();
        Set<ResourceLocation> types = TYPE_VEINS.computeIfAbsent(level.registryAccess(), CoeBridge::typeVeins);
        Set<ResourceLocation> records = RECORD_VEINS.computeIfAbsent(sites.deposits(), d -> recordVeins(sites));
        if (records.isEmpty() || types.containsAll(records))
            return types;
        Set<ResourceLocation> out = new TreeSet<>(types);
        out.addAll(records);
        return Collections.unmodifiableSet(out);
    }

    private static final Map<com.minecraftindustrialcomplex.deeptime.worlddata.Deposits, Set<ResourceLocation>> RECORD_VEINS =
            Collections.synchronizedMap(new WeakHashMap<>());

    private static Set<ResourceLocation> typeVeins(RegistryAccess access) {
        Set<ResourceLocation> out = new TreeSet<>();
        for (var e : OreData.entries(access, OreData.DEPOSIT_TYPE))
            e.getValue().minecraft().coeVein().map(ResourceLocation::tryParse).ifPresent(out::add);
        return Collections.unmodifiableSet(out);
    }

    private static Set<ResourceLocation> recordVeins(LevelSites sites) {
        Set<ResourceLocation> out = new TreeSet<>();
        for (Site s : sites.all())
            s.extraction().coeVein().map(ResourceLocation::tryParse).ifPresent(out::add);
        return Collections.unmodifiableSet(out);
    }

    private static final Set<ResourceLocation> MISSING = java.util.concurrent.ConcurrentHashMap.newKeySet();

    /** Logs once per vein id that a site's vein has no COE recipe (COE's own choice then applies there). */
    public static void missingRecipe(SiteVein v) {
        if (MISSING.add(v.vein()))
            com.minecraftindustrialcomplex.deeptime.DeepTime.LOGGER.warn(
                    "Create Ore Excavation has no vein recipe {} (site {}, {}); COE's own vein applies in its chunks",
                    v.vein(), v.site().id(), v.site().depositType());
    }

    /**
     * True when COE's noise must not put vein {@code id} into chunk (cx, cz) of {@code level}: a Deep
     * Time level, a claimed vein, and not the vein of the site that covers the chunk.
     */
    public static boolean dropNoiseVein(ServerLevel level, int cx, int cz, ResourceLocation id) {
        if (id == null || !claimedVeins(level).contains(id))
            return false;
        return siteVein(level, cx, cz).map(v -> !v.vein().equals(id)).orElse(true);
    }

    /**
     * The site's tonnage quantile within its type's tonnage lognormal (the core's split lognormal
     * through the type's {@code sizing.tonnage_mt} P90/P50/P10), as COE's {@code randomMul} in
     * [0, 1]. The tonnage is the site's present ore tonnage (FORMAT.md: after truncation and
     * leaching), so an eroded deposit yields less than it formed with. 0.5 when the type or its
     * percentiles are unknown.
     */
    public static float quantile(RegistryAccess access, Site site) {
        double tonnageMt = site.products().isEmpty() ? 0 : site.products().get(0).tonnageT() / 1e6;
        double q = OreData.depositType(access, site.depositType())
                .flatMap(t -> t.sizing().tonnageMt())
                .map(p -> p.cdf(tonnageMt))
                .orElse(0.5);
        return (float) Math.min(1, Math.max(0, q));
    }
}
