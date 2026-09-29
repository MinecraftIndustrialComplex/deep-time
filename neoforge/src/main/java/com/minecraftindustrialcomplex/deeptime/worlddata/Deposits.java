// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.worlddata;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

/**
 * A world's sites, bodies and events together (plan §5.6, §5.7.4, §5.5), read once per world-data
 * folder ({@link WorldData#deposits()}). Chunk-local lookups go through the site index: the sites
 * whose footprint touches an area, then their bodies' AABBs. Bodies without a site (none yet;
 * intrusions later) are checked by AABB directly. Plain Java, immutable, thread-safe.
 */
public final class Deposits {

    private final int circumference;
    private final SiteData sites;
    private final BodyData bodies;
    private final EventData events;
    private final List<Body> siteless;

    public Deposits(int circumference, SiteData sites, BodyData bodies, EventData events) {
        this.circumference = circumference;
        this.sites = sites;
        this.bodies = bodies;
        this.events = events;
        List<Body> free = new ArrayList<>();
        for (Body b : bodies.bodies())
            if (b.site() == SiteData.NONE)
                free.add(b);
        this.siteless = List.copyOf(free);
    }

    public static Deposits read(Path folder, int circumference) {
        return new Deposits(circumference, SiteData.read(folder, circumference), BodyData.read(folder),
                EventData.read(folder));
    }

    public static Deposits empty(int circumference) {
        return new Deposits(circumference, SiteData.empty(circumference), BodyData.empty(), EventData.empty());
    }

    public int circumference() {
        return circumference;
    }

    public SiteData sites() {
        return sites;
    }

    public BodyData bodies() {
        return bodies;
    }

    public EventData events() {
        return events;
    }

    public boolean isEmpty() {
        return sites.sites().isEmpty() && bodies.bodies().isEmpty();
    }

    /**
     * Every body whose AABB overlaps the block box [x0, x1] × [y0, y1] × [z0, z1] (inclusive; x may
     * leave the canonical span), sorted by id.
     */
    public List<Body> bodiesIn(int x0, int y0, int z0, int x1, int y1, int z1) {
        List<Body> out = new ArrayList<>();
        for (SiteData.SiteRecord s : sites.touching(x0, z0, x1, z1))
            for (Body b : bodies.ofSite(s.id()))
                if (b.aabbOverlaps(circumference, x0, y0, z0, x1, y1, z1))
                    out.add(b);
        for (Body b : siteless)
            if (b.aabbOverlaps(circumference, x0, y0, z0, x1, y1, z1))
                out.add(b);
        out.sort((a, b) -> Integer.compareUnsigned(a.id(), b.id()));
        return out;
    }

    /** The bodies overlapping chunk (cx, cz) between y0 and y1. */
    public List<Body> bodiesInChunk(int cx, int cz, int y0, int y1) {
        int x0 = cx << 4, z0 = cz << 4;
        return bodiesIn(x0, y0, z0, x0 + 15, y1, z0 + 15);
    }
}
