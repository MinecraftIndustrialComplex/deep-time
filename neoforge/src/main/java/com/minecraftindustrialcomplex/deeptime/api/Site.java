// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.api;

import net.minecraft.core.BlockPos;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.world.level.levelgen.structure.BoundingBox;

import java.util.List;
import java.util.OptionalInt;

/**
 * One site of a Deep Time world (plan §5.6): a deposit, reservoir, placer, supergene blanket or
 * fantasy site, fixed by generation and stable for the life of the world.
 *
 * @param id            stable id (the {@code siteId} of {@code /deeptime why})
 * @param kind          what the site is
 * @param depositType   the deposit type, e.g. {@code deeptime:porphyry_cu_mo_au}
 * @param center        block position; x in the canonical span [−C/2, C/2)
 * @param bounds        block bounds of every body of the site (x may leave the canonical span near
 *                      the seam; the level wraps it)
 * @param products      material, grade, tonnage and role of each product
 * @param formingEvent  the event that formed it ({@link DeepTimeSites#formingEvent})
 * @param parent        the parent site (a supergene blanket's porphyry), if any
 * @param extraction    how the site is mined (COE vein, CDG charge; P1-16)
 * @param latitude      degrees, north positive
 * @param longitude     degrees east of the seam, [0, 360)
 * @param firstBody     the id of the site's first body in the world data, or −1
 */
public record Site(int id, SiteKind kind, ResourceLocation depositType, BlockPos center, BoundingBox bounds,
                   List<Product> products, long formingEvent, OptionalInt parent, Extraction extraction,
                   float latitude, float longitude, int firstBody) {

    /** The primary product (the first with role {@code primary}, else the first), if any. */
    public java.util.Optional<Product> primary() {
        return products.stream().filter(p -> p.role().equals("primary")).findFirst()
                .or(() -> products.stream().findFirst());
    }

    /** True when one of the products is {@code material} (e.g. {@code c:iron}). */
    public boolean produces(ResourceLocation material) {
        return products.stream().anyMatch(p -> p.material().equals(material));
    }
}
