// SPDX-License-Identifier: LGPL-3.0-or-later
package com.minecraftindustrialcomplex.deeptime.climate;

import com.google.gson.JsonParseException;
import com.minecraftindustrialcomplex.deeptime.DeepTime;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceLocation;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.server.packs.resources.ResourceManager;
import net.minecraft.world.level.biome.Biome;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
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
 * editor's runs pass only the pack placement tables of the jar ({@link #jarPacks}) and the workbench
 * passes no content bundle; both use the core's default as the base table.
 * FORMAT.md, "The climate mapping", is the schema.
 *
 * <p>Pack placement tables ({@link PlacementPacks}) go the same way: {@link #packs} finds them in the
 * resources, {@link #chosen} keeps the ones whose biome mod is installed.
 */
public final class ClimatePlacement {

    public static final ResourceLocation PACK_TABLE =
            ResourceLocation.fromNamespaceAndPath(DeepTime.MODID, PlacementBundle.PATH);

    private ClimatePlacement() {}

    /**
     * Every pack placement table in the resources ({@code data/<namespace>/deeptime/climate_mapping/packs/*.json};
     * {@link PlacementPacks}), in location order. Throws when one cannot be read or is not a pack table.
     */
    public static List<PlacementPacks.Pack> packs(ResourceManager resources) throws IOException {
        List<PlacementPacks.Pack> out = new ArrayList<>();
        var found = new java.util.TreeMap<ResourceLocation, Resource>(resources.listResources(PlacementPacks.DIR,
                id -> id.getPath().endsWith(".json")));
        for (var e : found.entrySet()) {
            String source = e.getKey() + " (pack " + e.getValue().sourcePackId() + ")";
            try (InputStream in = e.getValue().open()) {
                out.add(PlacementPacks.parse(new String(in.readAllBytes(), StandardCharsets.UTF_8), source));
            } catch (IllegalArgumentException ex) {
                throw new IOException(ex.getMessage(), ex);
            }
        }
        return out;
    }

    /**
     * The pack placement tables in Deep Time's own jar (the Create World screen has no server resources to read: a
     * singleplayer world bakes with the jar's tables, a dedicated server with its datapacks').
     */
    public static List<PlacementPacks.Pack> jarPacks() throws IOException {
        List<PlacementPacks.Pack> out = new ArrayList<>();
        var file = net.neoforged.fml.ModList.get().getModFileById(DeepTime.MODID);
        if (file == null)
            return out;
        Path dir = file.getFile().findResource("data", DeepTime.MODID, "deeptime", "climate_mapping", "packs");
        if (!Files.isDirectory(dir))
            return out;
        try (var files = Files.list(dir)) {
            for (Path p : files.filter(f -> f.getFileName().toString().endsWith(".json")).sorted().toList()) {
                try {
                    out.add(PlacementPacks.parse(Files.readString(p, StandardCharsets.UTF_8), DeepTime.MODID + " jar " + p.getFileName()));
                } catch (IllegalArgumentException ex) {
                    throw new IOException(ex.getMessage(), ex);
                }
            }
        }
        return out;
    }

    /**
     * The pack tables a server's first start uses: those of its resources whose required biomes its registry holds.
     * Kept here so callers name only {@code MinecraftServer} (FirstStartGenerator is loaded by unit tests without
     * Minecraft's classes; verifying a {@code RegistryAccess} argument there would load them).
     */
    public static List<PlacementPacks.Pack> chosenFor(net.minecraft.server.MinecraftServer server) throws IOException {
        return chosen(packs(server.getResourceManager()), server.registryAccess());
    }

    /** The pack tables of {@code all} whose required biomes the biome registry of {@code registries} holds. */
    public static List<PlacementPacks.Pack> chosen(List<PlacementPacks.Pack> all, RegistryAccess registries) {
        Optional<? extends Registry<Biome>> biomes = registries.registry(Registries.BIOME);
        if (biomes.isEmpty())
            return List.of();
        return PlacementPacks.select(all, id -> {
            ResourceLocation rl = ResourceLocation.tryParse(id);
            return rl != null && biomes.get().containsKey(rl);
        });
    }

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
