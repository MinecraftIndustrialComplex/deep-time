// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worldgen;

import com.minecraftindustrialcomplex.deeptime.api.DeepTimeSites;
import com.minecraftindustrialcomplex.deeptime.api.Extraction;
import com.minecraftindustrialcomplex.deeptime.api.GeoEvent;
import com.minecraftindustrialcomplex.deeptime.api.Product;
import com.minecraftindustrialcomplex.deeptime.api.Site;
import com.minecraftindustrialcomplex.deeptime.api.SiteKind;
import com.minecraftindustrialcomplex.deeptime.worlddata.Body;
import com.minecraftindustrialcomplex.deeptime.worlddata.Deposits;
import com.minecraftindustrialcomplex.deeptime.worlddata.EventData;
import com.minecraftindustrialcomplex.deeptime.worlddata.SiteData;
import net.minecraft.core.BlockPos;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.phys.AABB;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import java.util.WeakHashMap;

/**
 * {@link DeepTimeSites} over a level's world data (P1-15). The plain-Java records of
 * {@link Deposits} are converted to the API's records once per world data and kept.
 */
public final class LevelSites implements DeepTimeSites {

    // CONVERTED first: NONE's constructor uses it.
    private static final Map<Deposits, List<Site>> CONVERTED = Collections.synchronizedMap(new WeakHashMap<>());
    private static final LevelSites NONE = new LevelSites(null, Deposits.empty(1024));

    /** The level's circumference, or null outside Deep Time levels. */
    private final Integer circumference;
    private final Deposits deposits;
    private final List<Site> sites;

    private LevelSites(Integer circumference, Deposits deposits) {
        this.circumference = circumference;
        this.deposits = deposits;
        this.sites = CONVERTED.computeIfAbsent(deposits, LevelSites::convert);
    }

    public static LevelSites of(ServerLevel level) {
        if (!(level.getChunkSource().getGenerator() instanceof GeologicChunkGenerator g))
            return NONE;
        return of(g);
    }

    /** The registry of a Deep Time generator: its world data's sites, or none on a test terrain. */
    public static LevelSites of(GeologicChunkGenerator g) {
        int c = g.dataRef().circumferenceBlocks();
        if (g.terrain() instanceof WorldDataTerrain w)
            return new LevelSites(c, w.data().deposits());
        return new LevelSites(c, Deposits.empty(c));
    }

    public Deposits deposits() {
        return deposits;
    }

    private static List<Site> convert(Deposits d) {
        List<Site> out = new ArrayList<>();
        for (SiteData.SiteRecord r : d.sites().sites())
            out.add(site(r));
        return List.copyOf(out);
    }

    static Site site(SiteData.SiteRecord r) {
        List<Product> products = r.products().stream()
                .map(p -> new Product(ResourceLocation.parse(p.material()), p.grade(), p.tonnageT(), p.role()))
                .toList();
        return new Site(r.id(), SiteKind.ofCode(r.kind()), ResourceLocation.parse(r.depositType()),
                new BlockPos(r.center()[0], r.center()[1], r.center()[2]),
                new BoundingBox(r.min()[0], r.min()[1], r.min()[2], r.max()[0], r.max()[1], r.max()[2]),
                products, r.formingEvent(), r.parent() == SiteData.NONE ? OptionalInt.empty() : OptionalInt.of(r.parent()),
                new Extraction(r.finite(), r.amountBase(), Optional.ofNullable(r.coeVein()),
                        r.cdgChargeMb() == SiteData.NONE_U64 ? OptionalLong.empty() : OptionalLong.of(r.cdgChargeMb())),
                r.lat(), r.lon(), r.body());
    }

    static GeoEvent event(EventData.EventRecord e) {
        List<Long> causes = new ArrayList<>(e.causes().length);
        for (long c : e.causes())
            causes.add(c);
        Map<String, Object> params = new java.util.LinkedHashMap<>();
        e.params().forEach((k, v) -> params.put(k, v instanceof EventData.IdParam id ? new GeoEvent.IdRef(id.id()) : v));
        GeoEvent.Place place = switch (e.placeTag()) {
            case 1 -> new GeoEvent.Place("point", e.lat(), e.lon(), 0);
            default -> new GeoEvent.Place(e.placeType(), 0f, 0f, Integer.toUnsignedLong(e.placeA()));
        };
        return new GeoEvent(e.id(), e.tStart(), e.tEnd(), ResourceLocation.parse(e.kind()), ResourceLocation.parse(e.source()),
                e.magnitude(), place, List.copyOf(causes), Collections.unmodifiableMap(params));
    }

    @Override
    public boolean isDeepTimeLevel() {
        return circumference != null;
    }

    @Override
    public List<Site> all() {
        return sites;
    }

    @Override
    public Optional<Site> at(BlockPos pos) {
        if (!isDeepTimeLevel())
            return Optional.empty();
        int c = circumference;
        for (SiteData.SiteRecord r : deposits.sites().touching(pos.getX(), pos.getZ(), pos.getX(), pos.getZ())) {
            for (Body b : deposits.bodies().ofSite(r.id()))
                if (b.aabbContains(c, pos.getX(), pos.getY(), pos.getZ())
                        && b.locate(c, pos.getX(), pos.getY(), pos.getZ()) != null)
                    return byId(r.id());
        }
        return Optional.empty();
    }

    @Override
    public List<Site> inChunk(ChunkPos pos) {
        return touching(pos.getMinBlockX(), Integer.MIN_VALUE, pos.getMinBlockZ(), pos.getMaxBlockX(), Integer.MAX_VALUE,
                pos.getMaxBlockZ());
    }

    @Override
    public List<Site> within(AABB box) {
        return touching((int) Math.floor(box.minX), (int) Math.floor(box.minY), (int) Math.floor(box.minZ),
                (int) Math.floor(box.maxX), (int) Math.floor(box.maxY), (int) Math.floor(box.maxZ));
    }

    private List<Site> touching(int x0, int y0, int z0, int x1, int y1, int z1) {
        if (!isDeepTimeLevel())
            return List.of();
        List<Site> out = new ArrayList<>();
        for (SiteData.SiteRecord r : deposits.sites().touching(x0, z0, x1, z1))
            if (r.max()[1] >= y0 && r.min()[1] <= y1)
                byId(r.id()).ifPresent(out::add);
        return out;
    }

    @Override
    public Optional<Site> byId(int id) {
        int lo = 0, hi = sites.size() - 1;
        while (lo <= hi) {
            int mid = (lo + hi) >>> 1;
            int cmp = Integer.compareUnsigned(sites.get(mid).id(), id);
            if (cmp == 0) return Optional.of(sites.get(mid));
            if (cmp < 0) lo = mid + 1;
            else hi = mid - 1;
        }
        return Optional.empty();
    }

    @Override
    public Optional<GeoEvent> formingEvent(Site site) {
        return event(site.formingEvent());
    }

    @Override
    public List<GeoEvent> causeChain(GeoEvent event, int maxDepth) {
        return deposits.events().causeChain(event.id(), maxDepth).stream().map(LevelSites::event).toList();
    }

    @Override
    public Optional<GeoEvent> event(long id) {
        return Optional.ofNullable(deposits.events().get(id)).map(LevelSites::event);
    }
}
