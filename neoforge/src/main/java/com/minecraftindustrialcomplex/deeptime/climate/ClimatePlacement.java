// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonParseException;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.server.packs.resources.ResourceManager;

import java.io.IOException;
import java.io.InputStream;
import java.util.Optional;

/**
 * The climate class placement table (plan §7.2, P1-09): which temperature and humidity band each
 * Köppen class maps to. The jar ships it as
 * {@code data/deeptime/deeptime/climate_mapping/placement.json} (the file the Rust core compiles in
 * as the bake's default, {@code crates/deeptime-core/data/climate_mapping/placement.json}), beside
 * the band edges of {@code default.json}, and a datapack may override it.
 *
 * <p>The bake is Rust, so the table reaches it through the content bundle: a dedicated server's
 * {@code FirstStartGenerator} adds the pack's copy under {@value PlacementBundle#KEY}
 * ({@link PlacementBundle#addTo}), and the core parses and checks it ({@code bake::placement}),
 * places the world's climate by it and records it in the world data
 * ({@code notes.bake.climate.placement}). Java never interprets the table. The Create World
 * editor's runs and the workbench pass no content bundle and so use the core's default.
 * FORMAT.md, "The climate mapping", is the schema.
 */
public final class ClimatePlacement {

    public static final ResourceLocation PACK_TABLE =
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, PlacementBundle.PATH);

    private ClimatePlacement() {}

    /**
     * The table of the highest-priority pack that has one (the jar's own when no datapack overrides
     * it); empty when there is none. Throws when the file is not a JSON object.
     */
    public static Optional<PlacementBundle.Found> read(ResourceManager resources) throws IOException {
        Optional<Resource> res = resources.getResource(PACK_TABLE);
        if (res.isEmpty())
            return Optional.empty();
        try (InputStream in = res.get().open()) {
            return Optional.of(new PlacementBundle.Found(PlacementBundle.parse(in), res.get().sourcePackId()));
        } catch (JsonParseException | IllegalStateException e) {
            throw new IOException(PACK_TABLE + " (pack " + res.get().sourcePackId() + ") is not a JSON object: " + e.getMessage(), e);
        }
    }
}
