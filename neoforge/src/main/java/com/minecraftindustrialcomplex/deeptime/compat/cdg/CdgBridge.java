// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.compat.cdg;

import com.minecraftindustrialcomplex.deeptime.api.Site;
import com.minecraftindustrialcomplex.deeptime.compat.SiteChunks;
import com.minecraftindustrialcomplex.deeptime.ore.OreData;
import com.minecraftindustrialcomplex.deeptime.worldgen.LevelSites;
import net.minecraft.core.RegistryAccess;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;

import java.util.Optional;
import java.util.OptionalInt;

/**
 * Create: Diesel Generators (MIT; plan §7.14, P1-16) as Deep Time decides it, in vanilla types only:
 * {@code mixin.compat.createdieselgenerators.OilChunksSavedDataMixin} (applied only with CDG) and
 * the probes call this; nothing here names a CDG class.
 *
 * <p>In a Deep Time level a chunk's base oil ({@code OilChunksSavedData.getBaseOilAmount}, what a
 * pumpjack or scanner finds before anything was pumped) is the share of a petroleum trap's charge
 * that lies under it, and 0 everywhere else: CDG's Perlin noise and biome tags play no part. The
 * trap is the site with a charge ({@code cdg_charge_mb}, the physical oil volume) whose ore body
 * fills the chunk most ({@link SiteChunks}); its chunk amount is
 * {@code round(cdg_charge_mb × cdg_scale × volume in the chunk / the body's volume)}, where
 * {@code cdg_scale} is the deposit type's (read from the registry at run time, so a pack can
 * rescale an existing world; 1 when the type is gone). As in CDG's own generator, an amount above
 * CDG's infinite threshold ({@code "Infinite oil chunk threshold"}, 10,000,000 mB by default) is
 * infinite ({@link Integer#MAX_VALUE}).
 */
public final class CdgBridge {

    private CdgBridge() {}

    /** Create: Diesel Generators' mod id (also the mixin package that CDG gates). */
    public static final String MOD_ID = "createdieselgenerators";

    /** CDG's default infinite threshold (mB), used when its config cannot be read. */
    public static final int DEFAULT_INFINITE_THRESHOLD = 10_000_000;

    /**
     * The trap under one chunk.
     *
     * @param site      the reservoir site
     * @param chargeMb  its physical charge (mB)
     * @param scale     the type's {@code cdg_scale}
     * @param share     the chunk's fraction of the ore body's volume
     * @param amountMb  {@code round(chargeMb × scale × share)}, before the infinite threshold
     */
    public record Charge(Site site, long chargeMb, double scale, double share, long amountMb) {

        /** What CDG sees: {@link Integer#MAX_VALUE} above {@code infiniteThreshold}, else the amount. */
        public int cdgAmount(int infiniteThreshold) {
            return amountMb > infiniteThreshold ? Integer.MAX_VALUE : (int) Math.max(0, amountMb);
        }
    }

    /** True for a site that feeds CDG: it has products and a charge, and its type (if loaded) is a reservoir type. */
    public static boolean feedsCdg(RegistryAccess access, Site site) {
        if (site.products().isEmpty() || site.extraction().cdgChargeMb().isEmpty())
            return false;
        return OreData.depositType(access, site.depositType()).map(t -> t.minecraft().cdgReservoir()).orElse(true);
    }

    /** The type's {@code cdg_scale}; 1 when the registry lacks the type. */
    public static double scaleOf(RegistryAccess access, Site site) {
        return OreData.depositType(access, site.depositType()).map(t -> t.minecraft().cdgScale()).orElse(1.0);
    }

    /** The trap under chunk (cx, cz); empty outside Deep Time levels and away from traps. */
    public static Optional<Charge> charge(ServerLevel level, int cx, int cz) {
        LevelSites sites = LevelSites.of(level);
        if (!sites.isDeepTimeLevel())
            return Optional.empty();
        RegistryAccess access = level.registryAccess();
        return SiteChunks.covering(sites, cx, cz, s -> feedsCdg(access, s)).map(s -> {
            int ccx = SiteChunks.canonicalChunkX(sites.deposits().circumference(), cx);
            long charge = s.extraction().cdgChargeMb().orElse(0);
            double scale = scaleOf(access, s);
            double share = SiteChunks.footprint(sites, s).share(ccx, cz);
            double mb = (double) charge * scale * share;
            return new Charge(s, charge, scale, share, mb >= Long.MAX_VALUE ? Long.MAX_VALUE : Math.round(mb));
        });
    }

    /**
     * What {@code getBaseOilAmount} returns for {@code pos}: in a Deep Time level the trap's amount
     * under the chunk, or 0; empty (CDG decides) in any other level.
     */
    public static OptionalInt baseOilAmount(ServerLevel level, ChunkPos pos, int infiniteThreshold) {
        if (!LevelSites.of(level).isDeepTimeLevel())
            return OptionalInt.empty();
        return OptionalInt.of(charge(level, pos.x, pos.z).map(c -> c.cdgAmount(infiniteThreshold)).orElse(0));
    }
}
